use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use crate::kubernetes::{models, format_age};

pub async fn list_jobs(client: &Client, namespace: Option<String>) -> Result<Vec<models::JobInfo>, kube::Error> {
    let api: Api<Job> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for j in api.list(&ListParams::default()).await? {
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
            let end = status_j.and_then(|s| s.completion_time.clone())
                .or_else(|| {
                    status_j.and_then(|s| s.conditions.as_ref())
                        .and_then(|conds| conds.iter().find(|c| c.type_ == "Failed" && c.status == "True"))
                        .and_then(|c| c.last_transition_time.clone())
                });

            if let Some(et) = end {
                let dur = et.0.signed_duration_since(st.0);
                let secs = dur.num_seconds();
                if secs < 60 {
                    Some(format!("{}s", secs))
                } else if secs < 3600 {
                    Some(format!("{}m {}s", dur.num_minutes(), secs % 60))
                } else {
                    Some(format!("{}h {}m", dur.num_hours(), dur.num_minutes() % 60))
                }
            } else {
                let dur = chrono::Utc::now().signed_duration_since(st.0);
                Some(format!("{}s (running)", dur.num_seconds()))
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
        if let Some(spec) = j.spec.as_ref() {
            for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }

        let labels = j.metadata.labels.clone().unwrap_or_default();
        let annotations = j.metadata.annotations.clone().unwrap_or_default();

        list.push(models::JobInfo {
            name,
            namespace: namespace_name,
            status,
            completions,
            duration,
            age,
            images,
            labels,
            annotations,
        });
    }

    Ok(list)
}

pub async fn list_cronjobs(client: &Client, namespace: Option<String>) -> Result<Vec<models::CronJobInfo>, kube::Error> {
    let api: Api<CronJob> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for cj in api.list(&ListParams::default()).await? {
        let name = cj.metadata.name.clone().unwrap_or_default();
        let namespace_name = cj.metadata.namespace.clone().unwrap_or_default();
        let age = format_age(&cj.metadata.creation_timestamp);

        let spec = cj.spec.as_ref();
        let schedule = spec.map(|s| s.schedule.clone()).unwrap_or_default();
        let suspend = spec.and_then(|s| s.suspend).unwrap_or(false);

        let status_cj = cj.status.as_ref();
        let active = status_cj.and_then(|s| s.active.as_ref()).map(|a| a.len() as i32).unwrap_or(0);

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
        if let Some(job_spec) = cj.spec.as_ref().and_then(|spec| spec.job_template.spec.as_ref()) {
            for c in &job_spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }

        let labels = cj.metadata.labels.clone().unwrap_or_default();
        let annotations = cj.metadata.annotations.clone().unwrap_or_default();

        list.push(models::CronJobInfo {
            name,
            namespace: namespace_name,
            schedule,
            suspend,
            active,
            last_schedule,
            age,
            images,
            labels,
            annotations,
        });
    }

    Ok(list)
}
