use std::time::Duration;

use k8s_openapi::api::batch::v1::{CronJob, Job};
use kube::{
    Client,
    api::{Api, ListParams, PostParams},
};

use crate::kubernetes::models;
use crate::kubernetes::namespaces::format_age;

pub const JOB_FETCH_TIMEOUT: Duration = Duration::from_secs(10);
pub const CRONJOB_FETCH_TIMEOUT: Duration = Duration::from_secs(10);
pub const CRONJOB_INSTANTIATE_ANNOTATION: &str = "cronjob.kubernetes.io/instantiate";
pub const CRONJOB_MUTATION_TIMEOUT: Duration = Duration::from_secs(15);

pub fn format_manual_job_name(cronjob_name: &str, suffix: &str) -> String {
    let suffix_len = suffix.len() + "-manual-".len();
    let max_base_len = 63usize.saturating_sub(suffix_len);
    let mut base = cronjob_name;
    if base.len() > max_base_len {
        base = &base[..max_base_len];
    }
    let trimmed = base.trim_end_matches('-');
    format!("{}-manual-{}", trimmed, suffix)
}

pub fn build_job_from_cronjob(cronjob: &CronJob, job_name: String) -> Result<Job, String> {
    let spec = cronjob
        .spec
        .as_ref()
        .ok_or_else(|| "CronJob has no spec".to_string())?;

    let job_template = &spec.job_template;
    let job_spec = job_template
        .spec
        .clone()
        .ok_or_else(|| "CronJob jobTemplate has no spec".to_string())?;

    let mut annotations = job_template
        .metadata
        .as_ref()
        .and_then(|m| m.annotations.clone())
        .unwrap_or_default();
    annotations.insert(
        CRONJOB_INSTANTIATE_ANNOTATION.to_string(),
        "manual".to_string(),
    );

    let labels = job_template
        .metadata
        .as_ref()
        .and_then(|m| m.labels.clone());

    let cronjob_name = cronjob.metadata.name.clone().unwrap_or_default();
    let cronjob_uid = cronjob.metadata.uid.clone().unwrap_or_default();

    let owner_references = if !cronjob_name.is_empty() && !cronjob_uid.is_empty() {
        Some(vec![
            k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference {
                api_version: "batch/v1".to_string(),
                kind: "CronJob".to_string(),
                name: cronjob_name,
                uid: cronjob_uid,
                controller: Some(true),
                block_owner_deletion: Some(true),
            },
        ])
    } else {
        None
    };

    let namespace = cronjob.metadata.namespace.clone();

    Ok(Job {
        metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            name: Some(job_name),
            namespace,
            labels,
            annotations: Some(annotations),
            owner_references,
            ..Default::default()
        },
        spec: Some(job_spec),
        status: None,
    })
}

pub async fn trigger_cronjob(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<String, kube::Error> {
    tracing::info!(namespace = %namespace, name = %name, "Triggering CronJob manually");
    let cj_api: Api<CronJob> = Api::namespaced(client.clone(), namespace);
    let cronjob = cj_api.get(name).await?;

    let random_suffix = uuid::Uuid::new_v4().to_string().replace('-', "")[..5].to_lowercase();
    let job_name = format_manual_job_name(name, &random_suffix);

    let job = build_job_from_cronjob(&cronjob, job_name.clone()).map_err(|e| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: e,
            reason: "Invalid".to_string(),
            code: 400,
        })
    })?;

    let job_api: Api<Job> = Api::namespaced(client.clone(), namespace);
    let post_params = PostParams::default();
    job_api.create(&post_params, &job).await?;
    tracing::info!(
        namespace = %namespace,
        name = %name,
        job_name = %job_name,
        "Successfully created manual Job from CronJob"
    );

    Ok(job_name)
}

pub async fn execute_trigger_cronjob(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<String, String> {
    match tokio::time::timeout(
        CRONJOB_MUTATION_TIMEOUT,
        trigger_cronjob(client, namespace, name),
    )
    .await
    {
        Ok(Ok(job_name)) => Ok(job_name),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err(format!(
            "Timed out triggering CronJob '{}/{}' after 15s",
            namespace, name
        )),
    }
}

pub fn map_job(j: &Job) -> models::JobInfo {
    let name = j.metadata.name.clone().unwrap_or_default();
    let namespace_name = j.metadata.namespace.clone().unwrap_or_default();
    let age = format_age(&j.metadata.creation_timestamp);

    let spec_jobs = j.spec.as_ref();
    let desired_completions = spec_jobs.and_then(|s| s.completions).unwrap_or(1);

    let status_j = j.status.as_ref();
    let succeeded = status_j.and_then(|s| s.succeeded).unwrap_or(0);
    let active = status_j.and_then(|s| s.active).unwrap_or(0);
    let failed = status_j.and_then(|s| s.failed).unwrap_or(0);

    let completions = format!("{}/{}", succeeded, desired_completions);

    let duration = if let Some(st) = status_j.and_then(|s| s.start_time.clone()) {
        let end = status_j
            .and_then(|s| s.completion_time.clone())
            .or_else(|| {
                status_j
                    .and_then(|s| s.conditions.as_ref())
                    .and_then(|conds| {
                        conds
                            .iter()
                            .find(|c| c.type_ == "Failed" && c.status == "True")
                    })
                    .and_then(|c| c.last_transition_time.clone())
            });

        if let Some(et) = end {
            let secs = et.0.signed_duration_since(st.0).num_seconds().max(0);
            if secs < 60 {
                Some(format!("{}s", secs))
            } else if secs < 3600 {
                Some(format!("{}m {}s", secs / 60, secs % 60))
            } else {
                Some(format!("{}h {}m", secs / 3600, (secs % 3600) / 60))
            }
        } else {
            let secs = chrono::Utc::now()
                .signed_duration_since(st.0)
                .num_seconds()
                .max(0);
            Some(format!("{}s (running)", secs))
        }
    } else {
        None
    };

    let status = if active > 0 {
        "Active".to_string()
    } else if succeeded >= desired_completions {
        "Succeeded".to_string()
    } else if failed > 0 {
        "Failed".to_string()
    } else {
        "Unknown".to_string()
    };

    let mut images = Vec::new();
    let mut containers = Vec::new();
    if let Some(spec) = j.spec.as_ref()
        && let Some(pod_spec) = spec.template.spec.as_ref()
    {
        for c in &pod_spec.containers {
            if let Some(img) = &c.image {
                images.push(img.clone());
            }
            containers.push(models::ContainerImageInfo {
                name: c.name.clone(),
                image: c.image.clone().unwrap_or_default(),
            });
        }
    }

    let labels = j.metadata.labels.clone().unwrap_or_default();
    let annotations = j.metadata.annotations.clone().unwrap_or_default();

    models::JobInfo {
        kind: "Job".to_string(),
        name,
        namespace: namespace_name,
        status,
        completions,
        duration,
        age,
        images,
        containers,
        labels,
        annotations,
    }
}

pub async fn list_jobs(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::JobInfo>, kube::Error> {
    let api: Api<Job> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for j in api.list(&ListParams::default()).await? {
        list.push(map_job(&j));
    }

    Ok(list)
}

pub async fn fetch_jobs(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::JobInfo>, String> {
    match tokio::time::timeout(JOB_FETCH_TIMEOUT, list_jobs(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching jobs after 10s".to_string()),
    }
}

pub fn map_cronjob(cj: &CronJob) -> models::CronJobInfo {
    let name = cj.metadata.name.clone().unwrap_or_default();
    let namespace_name = cj.metadata.namespace.clone().unwrap_or_default();
    let age = format_age(&cj.metadata.creation_timestamp);

    let spec = cj.spec.as_ref();
    let schedule = spec.map(|s| s.schedule.clone()).unwrap_or_default();
    let suspend = spec.and_then(|s| s.suspend).unwrap_or(false);

    let status_cj = cj.status.as_ref();
    let active = status_cj
        .and_then(|s| s.active.as_ref())
        .map(|a| a.len() as i32)
        .unwrap_or(0);

    let last_schedule = if let Some(last) = status_cj.and_then(|s| s.last_schedule_time.clone()) {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last.0);
        let seconds = duration.num_seconds();
        if seconds < 0 {
            Some("0s".to_string())
        } else if seconds < 60 {
            Some(format!("{}s ago", seconds))
        } else if seconds < 3600 {
            Some(format!("{}m ago", duration.num_minutes()))
        } else if seconds < 86400 {
            Some(format!("{}h ago", duration.num_hours()))
        } else {
            Some(format!("{}d ago", duration.num_days()))
        }
    } else {
        None
    };

    let mut images = Vec::new();
    let mut containers = Vec::new();
    if let Some(job_spec) = cj
        .spec
        .as_ref()
        .and_then(|spec| spec.job_template.spec.as_ref())
        && let Some(pod_spec) = job_spec.template.spec.as_ref()
    {
        for c in &pod_spec.containers {
            if let Some(img) = &c.image {
                images.push(img.clone());
            }
            containers.push(models::ContainerImageInfo {
                name: c.name.clone(),
                image: c.image.clone().unwrap_or_default(),
            });
        }
    }

    let labels = cj.metadata.labels.clone().unwrap_or_default();
    let annotations = cj.metadata.annotations.clone().unwrap_or_default();

    models::CronJobInfo {
        kind: "CronJob".to_string(),
        name,
        namespace: namespace_name,
        schedule,
        suspend,
        active,
        last_schedule,
        age,
        images,
        containers,
        labels,
        annotations,
    }
}

pub async fn list_cronjobs(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::CronJobInfo>, kube::Error> {
    let api: Api<CronJob> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for cj in api.list(&ListParams::default()).await? {
        list.push(map_cronjob(&cj));
    }

    Ok(list)
}

pub async fn fetch_cronjobs(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::CronJobInfo>, String> {
    match tokio::time::timeout(CRONJOB_FETCH_TIMEOUT, list_cronjobs(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching cronjobs after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use k8s_openapi::api::batch::v1::{JobCondition, JobSpec, JobStatus};
    use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};

    #[test]
    fn test_map_job_status_completions_duration_and_containers() {
        let base = chrono::Utc::now() - chrono::Duration::hours(2);

        // Fixture 1: Succeeded (135s -> "2m 15s"), 2 containers, labels & annotations
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "db-migrate".to_string());
        let mut annotations = BTreeMap::new();
        annotations.insert("owner".to_string(), "platform".to_string());

        let succeeded_job = Job {
            metadata: ObjectMeta {
                name: Some("db-migrate".to_string()),
                namespace: Some("prod".to_string()),
                labels: Some(labels.clone()),
                annotations: Some(annotations.clone()),
                ..Default::default()
            },
            spec: Some(JobSpec {
                completions: Some(2),
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        containers: vec![
                            Container {
                                name: "migrate".to_string(),
                                image: Some("ghcr.io/orbit/migrate:v1".to_string()),
                                ..Default::default()
                            },
                            Container {
                                name: "sidecar".to_string(),
                                image: Some("busybox:1.36".to_string()),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            status: Some(JobStatus {
                succeeded: Some(2),
                start_time: Some(Time(base)),
                completion_time: Some(Time(base + chrono::Duration::seconds(135))),
                ..Default::default()
            }),
        };

        let mapped = map_job(&succeeded_job);
        assert_eq!(mapped.kind, "Job");
        assert_eq!(mapped.name, "db-migrate");
        assert_eq!(mapped.namespace, "prod");
        assert_eq!(mapped.status, "Succeeded");
        assert_eq!(mapped.completions, "2/2");
        assert_eq!(mapped.duration.as_deref(), Some("2m 15s"));
        assert_eq!(
            mapped.images,
            vec![
                "ghcr.io/orbit/migrate:v1".to_string(),
                "busybox:1.36".to_string()
            ]
        );
        assert_eq!(
            mapped.containers,
            vec![
                models::ContainerImageInfo {
                    name: "migrate".to_string(),
                    image: "ghcr.io/orbit/migrate:v1".to_string(),
                },
                models::ContainerImageInfo {
                    name: "sidecar".to_string(),
                    image: "busybox:1.36".to_string(),
                },
            ]
        );
        assert_eq!(mapped.labels, labels);
        assert_eq!(mapped.annotations, annotations);

        // Fixture 2: Active (running)
        let active_job = Job {
            metadata: ObjectMeta {
                name: Some("indexer".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(JobStatus {
                active: Some(1),
                start_time: Some(Time(chrono::Utc::now() - chrono::Duration::seconds(12))),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mapped_active = map_job(&active_job);
        assert_eq!(mapped_active.status, "Active");
        assert!(
            mapped_active
                .duration
                .as_deref()
                .is_some_and(|d| d.ends_with("(running)"))
        );

        // Fixture 3: Failed (45s via Failed condition last_transition_time)
        let failed_job = Job {
            metadata: ObjectMeta {
                name: Some("failing-sync".to_string()),
                namespace: Some("staging".to_string()),
                ..Default::default()
            },
            spec: Some(JobSpec {
                completions: Some(1),
                ..Default::default()
            }),
            status: Some(JobStatus {
                succeeded: Some(0),
                failed: Some(2),
                start_time: Some(Time(base)),
                conditions: Some(vec![JobCondition {
                    type_: "Failed".to_string(),
                    status: "True".to_string(),
                    last_transition_time: Some(Time(base + chrono::Duration::seconds(45))),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };
        let mapped_failed = map_job(&failed_job);
        assert_eq!(mapped_failed.status, "Failed");
        assert_eq!(mapped_failed.completions, "0/1");
        assert_eq!(mapped_failed.duration.as_deref(), Some("45s"));

        // Fixture 4: Hours duration (5400s -> "1h 30m")
        let long_job = Job {
            metadata: ObjectMeta {
                name: Some("etl".to_string()),
                namespace: Some("data".to_string()),
                ..Default::default()
            },
            status: Some(JobStatus {
                succeeded: Some(1),
                start_time: Some(Time(base)),
                completion_time: Some(Time(base + chrono::Duration::seconds(5400))),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mapped_long = map_job(&long_job);
        assert_eq!(mapped_long.duration.as_deref(), Some("1h 30m"));
    }

    #[test]
    fn test_map_job_defaults_missing_fields() {
        let bare = Job::default();
        let mapped = map_job(&bare);
        assert_eq!(mapped.kind, "Job");
        assert_eq!(mapped.completions, "0/1");
        assert_eq!(mapped.status, "Unknown");
        assert_eq!(mapped.duration, None);
        assert!(mapped.images.is_empty());
        assert!(mapped.containers.is_empty());
        assert!(mapped.labels.is_empty());
        assert!(mapped.annotations.is_empty());
    }

    #[test]
    fn test_map_cronjob_schedule_suspend_active_last_schedule_and_containers() {
        use k8s_openapi::api::batch::v1::{CronJobSpec, CronJobStatus, JobTemplateSpec};
        use k8s_openapi::api::core::v1::ObjectReference;

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "nightly-backup".to_string());
        let mut annotations = BTreeMap::new();
        annotations.insert("owner".to_string(), "sre".to_string());

        let cj = CronJob {
            metadata: ObjectMeta {
                name: Some("nightly-backup".to_string()),
                namespace: Some("prod".to_string()),
                labels: Some(labels.clone()),
                annotations: Some(annotations.clone()),
                ..Default::default()
            },
            spec: Some(CronJobSpec {
                schedule: "0 2 * * *".to_string(),
                suspend: Some(true),
                job_template: JobTemplateSpec {
                    spec: Some(JobSpec {
                        template: PodTemplateSpec {
                            spec: Some(PodSpec {
                                containers: vec![
                                    Container {
                                        name: "backup".to_string(),
                                        image: Some("ghcr.io/orbit/backup:v2".to_string()),
                                        ..Default::default()
                                    },
                                    Container {
                                        name: "metrics".to_string(),
                                        image: Some("busybox:1.36".to_string()),
                                        ..Default::default()
                                    },
                                ],
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            status: Some(CronJobStatus {
                active: Some(vec![ObjectReference::default(), ObjectReference::default()]),
                last_schedule_time: Some(Time(chrono::Utc::now() - chrono::Duration::seconds(120))),
                ..Default::default()
            }),
        };

        let mapped = map_cronjob(&cj);
        assert_eq!(mapped.kind, "CronJob");
        assert_eq!(mapped.name, "nightly-backup");
        assert_eq!(mapped.namespace, "prod");
        assert_eq!(mapped.schedule, "0 2 * * *");
        assert!(mapped.suspend);
        assert_eq!(mapped.active, 2);
        assert_eq!(mapped.last_schedule.as_deref(), Some("2m ago"));
        assert_eq!(
            mapped.images,
            vec![
                "ghcr.io/orbit/backup:v2".to_string(),
                "busybox:1.36".to_string()
            ]
        );
        assert_eq!(
            mapped.containers,
            vec![
                models::ContainerImageInfo {
                    name: "backup".to_string(),
                    image: "ghcr.io/orbit/backup:v2".to_string(),
                },
                models::ContainerImageInfo {
                    name: "metrics".to_string(),
                    image: "busybox:1.36".to_string(),
                },
            ]
        );
        assert_eq!(mapped.labels, labels);
        assert_eq!(mapped.annotations, annotations);

        // Verify seconds / hours / days / future last_schedule formatting branches
        let seconds_cj = CronJob {
            status: Some(CronJobStatus {
                last_schedule_time: Some(Time(chrono::Utc::now() - chrono::Duration::seconds(20))),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(
            map_cronjob(&seconds_cj)
                .last_schedule
                .as_deref()
                .is_some_and(|v| v.ends_with("s ago"))
        );

        let hours_cj = CronJob {
            status: Some(CronJobStatus {
                last_schedule_time: Some(Time(chrono::Utc::now() - chrono::Duration::hours(3))),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            map_cronjob(&hours_cj).last_schedule.as_deref(),
            Some("3h ago")
        );

        let days_cj = CronJob {
            status: Some(CronJobStatus {
                last_schedule_time: Some(Time(chrono::Utc::now() - chrono::Duration::days(4))),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(
            map_cronjob(&days_cj).last_schedule.as_deref(),
            Some("4d ago")
        );

        let future_cj = CronJob {
            status: Some(CronJobStatus {
                last_schedule_time: Some(Time(chrono::Utc::now() + chrono::Duration::seconds(30))),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(map_cronjob(&future_cj).last_schedule.as_deref(), Some("0s"));
    }

    #[test]
    fn test_map_cronjob_defaults_missing_fields() {
        let bare = CronJob::default();
        let mapped = map_cronjob(&bare);
        assert_eq!(mapped.kind, "CronJob");
        assert_eq!(mapped.schedule, "");
        assert!(!mapped.suspend);
        assert_eq!(mapped.active, 0);
        assert_eq!(mapped.last_schedule, None);
        assert!(mapped.images.is_empty());
        assert!(mapped.containers.is_empty());
        assert!(mapped.labels.is_empty());
        assert!(mapped.annotations.is_empty());
    }

    #[test]
    fn test_format_manual_job_name_truncates_to_63_chars_and_trims_trailing_hyphen() {
        let name = format_manual_job_name("nightly-db-backup", "abc12");
        assert_eq!(name, "nightly-db-backup-manual-abc12");

        let long_cj_name = format!("{}-{}", "a".repeat(49), "b".repeat(10));
        let long_job = format_manual_job_name(&long_cj_name, "abc12");
        assert!(long_job.len() <= 63);
        assert!(long_job.ends_with("-manual-abc12"));
        assert!(!long_job.contains("--"));
    }

    #[test]
    fn test_build_job_from_cronjob_sets_manual_annotation_and_owner_ref() {
        use k8s_openapi::api::batch::v1::{CronJobSpec, JobSpec, JobTemplateSpec};
        use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        use std::collections::BTreeMap;

        let mut job_labels = BTreeMap::new();
        job_labels.insert("tier".to_string(), "batch".to_string());

        let cronjob = CronJob {
            metadata: ObjectMeta {
                name: Some("nightly-backup".to_string()),
                namespace: Some("default".to_string()),
                uid: Some("cj-uid-1".to_string()),
                ..Default::default()
            },
            spec: Some(CronJobSpec {
                schedule: "0 0 * * *".to_string(),
                job_template: JobTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(job_labels.clone()),
                        ..Default::default()
                    }),
                    spec: Some(JobSpec {
                        template: PodTemplateSpec {
                            spec: Some(PodSpec {
                                containers: vec![Container {
                                    name: "worker".to_string(),
                                    image: Some("alpine:latest".to_string()),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            status: None,
        };

        let job = build_job_from_cronjob(&cronjob, "nightly-backup-manual-123".to_string())
            .expect("Should build job");
        assert_eq!(
            job.metadata.name.as_deref(),
            Some("nightly-backup-manual-123")
        );
        assert_eq!(job.metadata.namespace.as_deref(), Some("default"));
        let annotations = job.metadata.annotations.unwrap();
        assert_eq!(
            annotations
                .get(CRONJOB_INSTANTIATE_ANNOTATION)
                .map(|s| s.as_str()),
            Some("manual")
        );
        let owners = job.metadata.owner_references.unwrap();
        assert_eq!(owners.len(), 1);
        assert_eq!(owners[0].kind, "CronJob");
        assert_eq!(owners[0].name, "nightly-backup");
        assert_eq!(owners[0].uid, "cj-uid-1");
        assert_eq!(owners[0].controller, Some(true));
        assert_eq!(job.metadata.labels.unwrap(), job_labels);
        assert!(job.spec.is_some());

        let missing_spec = CronJob::default();
        assert!(build_job_from_cronjob(&missing_spec, "x".to_string()).is_err());
    }
}
