use std::time::Duration;

use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{ContainerStatus, Pod};
use kube::api::ListParams;
use kube::{Api, Client, ResourceExt};
use serde::{Deserialize, Serialize};

pub const WORKLOADS_OVERVIEW_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkloadControllerKind {
    Pods,
    Deployments,
    StatefulSets,
    DaemonSets,
    ReplicaSets,
    Jobs,
    CronJobs,
    Hpas,
}

impl WorkloadControllerKind {
    pub const ALL: [WorkloadControllerKind; 8] = [
        Self::Pods,
        Self::Deployments,
        Self::StatefulSets,
        Self::DaemonSets,
        Self::ReplicaSets,
        Self::Jobs,
        Self::CronJobs,
        Self::Hpas,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkloadHealthStatus {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadHealthItem {
    pub namespace: String,
    pub status: WorkloadHealthStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorkloadKindSnapshot {
    pub items: Vec<WorkloadHealthItem>,
    pub error: Option<String>,
}

impl WorkloadKindSnapshot {
    pub fn counts_for_namespaces(&self, selected: &[String]) -> WorkloadHealthCounts {
        let mut counts = WorkloadHealthCounts::default();
        for item in &self.items {
            if !selected.is_empty() && !selected.iter().any(|ns| ns == &item.namespace) {
                continue;
            }
            counts.total += 1;
            match item.status {
                WorkloadHealthStatus::Healthy => counts.healthy += 1,
                WorkloadHealthStatus::Degraded => counts.degraded += 1,
                WorkloadHealthStatus::Failed => counts.failed += 1,
            }
        }
        counts
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorkloadsOverviewSnapshot {
    pub pods: WorkloadKindSnapshot,
    pub deployments: WorkloadKindSnapshot,
    pub statefulsets: WorkloadKindSnapshot,
    pub daemonsets: WorkloadKindSnapshot,
    pub replicasets: WorkloadKindSnapshot,
    pub jobs: WorkloadKindSnapshot,
    pub cronjobs: WorkloadKindSnapshot,
    pub hpas: WorkloadKindSnapshot,
}

impl WorkloadsOverviewSnapshot {
    pub fn kind_snapshot(&self, kind: WorkloadControllerKind) -> &WorkloadKindSnapshot {
        match kind {
            WorkloadControllerKind::Pods => &self.pods,
            WorkloadControllerKind::Deployments => &self.deployments,
            WorkloadControllerKind::StatefulSets => &self.statefulsets,
            WorkloadControllerKind::DaemonSets => &self.daemonsets,
            WorkloadControllerKind::ReplicaSets => &self.replicasets,
            WorkloadControllerKind::Jobs => &self.jobs,
            WorkloadControllerKind::CronJobs => &self.cronjobs,
            WorkloadControllerKind::Hpas => &self.hpas,
        }
    }

    pub fn aggregate_counts(&self, selected: &[String]) -> WorkloadHealthCounts {
        let mut total = WorkloadHealthCounts::default();
        for kind in WorkloadControllerKind::ALL {
            let counts = self.kind_snapshot(kind).counts_for_namespaces(selected);
            total.total += counts.total;
            total.healthy += counts.healthy;
            total.degraded += counts.degraded;
            total.failed += counts.failed;
        }
        total
    }

    pub fn has_any_error(&self) -> bool {
        WorkloadControllerKind::ALL
            .iter()
            .any(|kind| self.kind_snapshot(*kind).error.is_some())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorkloadHealthCounts {
    pub total: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub failed: usize,
}

const FAILED_WAITING_REASONS: [&str; 6] = [
    "CrashLoopBackOff",
    "ImagePullBackOff",
    "ErrImagePull",
    "CreateContainerConfigError",
    "CreateContainerError",
    "InvalidImageName",
];

const FAILED_TERMINATED_REASONS: [&str; 2] = ["OOMKilled", "Error"];

fn container_has_failure_state(cs: &ContainerStatus) -> bool {
    let Some(state) = cs.state.as_ref() else {
        return false;
    };
    let waiting_failed = state
        .waiting
        .as_ref()
        .and_then(|w| w.reason.as_deref())
        .is_some_and(|reason| FAILED_WAITING_REASONS.contains(&reason));
    if waiting_failed {
        return true;
    }
    state
        .terminated
        .as_ref()
        .and_then(|t| t.reason.as_deref())
        .is_some_and(|reason| FAILED_TERMINATED_REASONS.contains(&reason))
}

pub fn classify_pod_health(pod: &Pod) -> WorkloadHealthStatus {
    let status = pod.status.as_ref();
    let phase = status.and_then(|s| s.phase.as_deref());

    let has_failed_container = status.is_some_and(|s| {
        s.container_statuses
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .any(container_has_failure_state)
            || s.init_container_statuses
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(container_has_failure_state)
    });

    if phase == Some("Failed") || has_failed_container {
        WorkloadHealthStatus::Failed
    } else if matches!(phase, Some("Running" | "Succeeded")) {
        WorkloadHealthStatus::Healthy
    } else {
        WorkloadHealthStatus::Degraded
    }
}

pub fn classify_deployment_health(deployment: &Deployment) -> WorkloadHealthStatus {
    let desired = deployment
        .spec
        .as_ref()
        .and_then(|s| s.replicas)
        .unwrap_or(1)
        .max(0);
    let current = deployment
        .status
        .as_ref()
        .and_then(|s| s.replicas)
        .unwrap_or(0)
        .max(0);
    let available = deployment
        .status
        .as_ref()
        .and_then(|s| s.available_replicas)
        .unwrap_or(0)
        .max(0);
    let conditions = deployment
        .status
        .as_ref()
        .and_then(|s| s.conditions.as_deref())
        .unwrap_or(&[]);

    let has_replica_failure = conditions
        .iter()
        .any(|c| c.type_ == "ReplicaFailure" && c.status == "True");
    if has_replica_failure {
        return WorkloadHealthStatus::Failed;
    }

    let is_available = conditions
        .iter()
        .any(|c| c.type_ == "Available" && c.status == "True");
    if is_available || (desired == 0 && current == 0) || (desired > 0 && available >= desired) {
        WorkloadHealthStatus::Healthy
    } else {
        WorkloadHealthStatus::Degraded
    }
}

pub fn classify_statefulset_health(statefulset: &StatefulSet) -> WorkloadHealthStatus {
    let desired = statefulset
        .spec
        .as_ref()
        .and_then(|s| s.replicas)
        .unwrap_or(1)
        .max(0);
    let ready = statefulset
        .status
        .as_ref()
        .and_then(|s| s.ready_replicas)
        .unwrap_or(0)
        .max(0);

    if ready >= desired {
        WorkloadHealthStatus::Healthy
    } else {
        WorkloadHealthStatus::Degraded
    }
}

pub fn classify_daemonset_health(daemonset: &DaemonSet) -> WorkloadHealthStatus {
    let Some(status) = daemonset.status.as_ref() else {
        return WorkloadHealthStatus::Degraded;
    };
    let desired = status.desired_number_scheduled.max(0);
    let ready = status.number_ready.max(0);
    let unavailable = status.number_unavailable.unwrap_or(0).max(0);
    let misscheduled = status.number_misscheduled.max(0);

    if desired > 0 && ready == 0 && unavailable > 0 {
        WorkloadHealthStatus::Failed
    } else if ready >= desired && misscheduled == 0 {
        WorkloadHealthStatus::Healthy
    } else {
        WorkloadHealthStatus::Degraded
    }
}

pub fn classify_replicaset_health(replicaset: &ReplicaSet) -> WorkloadHealthStatus {
    let desired = replicaset
        .spec
        .as_ref()
        .and_then(|s| s.replicas)
        .unwrap_or(1)
        .max(0);
    let ready = replicaset
        .status
        .as_ref()
        .and_then(|s| s.ready_replicas)
        .unwrap_or(0)
        .max(0);
    let conditions = replicaset
        .status
        .as_ref()
        .and_then(|s| s.conditions.as_deref())
        .unwrap_or(&[]);

    if conditions
        .iter()
        .any(|c| c.type_ == "ReplicaFailure" && c.status == "True")
    {
        WorkloadHealthStatus::Failed
    } else if ready >= desired {
        WorkloadHealthStatus::Healthy
    } else {
        WorkloadHealthStatus::Degraded
    }
}

pub fn classify_job_health(job: &Job) -> WorkloadHealthStatus {
    let desired_completions = job
        .spec
        .as_ref()
        .and_then(|s| s.completions)
        .unwrap_or(1)
        .max(0);
    let status = job.status.as_ref();
    let active = status.and_then(|s| s.active).unwrap_or(0).max(0);
    let succeeded = status.and_then(|s| s.succeeded).unwrap_or(0).max(0);
    let failed = status.and_then(|s| s.failed).unwrap_or(0).max(0);
    let conditions = status
        .and_then(|s| s.conditions.as_deref())
        .unwrap_or(&[]);

    let cond_failed = conditions
        .iter()
        .any(|c| c.type_ == "Failed" && c.status == "True");
    let cond_complete = conditions
        .iter()
        .any(|c| c.type_ == "Complete" && c.status == "True");

    if cond_failed || (active == 0 && succeeded < desired_completions && failed > 0) {
        WorkloadHealthStatus::Failed
    } else if cond_complete || succeeded >= desired_completions || (active > 0 && failed == 0) {
        WorkloadHealthStatus::Healthy
    } else {
        WorkloadHealthStatus::Degraded
    }
}

pub fn classify_cronjob_health(cronjob: &CronJob) -> WorkloadHealthStatus {
    if cronjob.spec.as_ref().and_then(|s| s.suspend) == Some(true) {
        WorkloadHealthStatus::Degraded
    } else {
        WorkloadHealthStatus::Healthy
    }
}

pub fn classify_hpa_health(hpa: &HorizontalPodAutoscaler) -> WorkloadHealthStatus {
    let status = hpa.status.as_ref();
    let current = status.and_then(|s| s.current_replicas).unwrap_or(0).max(0);
    let desired = status.map(|s| s.desired_replicas).unwrap_or(0).max(0);
    let conditions = status
        .and_then(|s| s.conditions.as_deref())
        .unwrap_or(&[]);

    let is_failed = conditions.iter().any(|c| {
        (c.type_ == "ScalingActive" || c.type_ == "AbleToScale") && c.status == "False"
    });
    if is_failed {
        return WorkloadHealthStatus::Failed;
    }

    let is_limited = conditions
        .iter()
        .any(|c| c.type_ == "ScalingLimited" && c.status == "True");
    if is_limited || (current == 0 && desired == 0 && conditions.is_empty()) {
        WorkloadHealthStatus::Degraded
    } else {
        WorkloadHealthStatus::Healthy
    }
}

fn build_kind_snapshot<K, F>(result: Result<&[K], String>, classify: F) -> WorkloadKindSnapshot
where
    K: ResourceExt,
    F: Fn(&K) -> WorkloadHealthStatus,
{
    match result {
        Ok(list) => WorkloadKindSnapshot {
            items: list
                .iter()
                .map(|item| WorkloadHealthItem {
                    namespace: item.namespace().unwrap_or_default(),
                    status: classify(item),
                })
                .collect(),
            error: None,
        },
        Err(err) => WorkloadKindSnapshot {
            items: Vec::new(),
            error: Some(err),
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_workloads_overview_snapshot(
    pods: Result<&[Pod], String>,
    deployments: Result<&[Deployment], String>,
    statefulsets: Result<&[StatefulSet], String>,
    daemonsets: Result<&[DaemonSet], String>,
    replicasets: Result<&[ReplicaSet], String>,
    jobs: Result<&[Job], String>,
    cronjobs: Result<&[CronJob], String>,
    hpas: Result<&[HorizontalPodAutoscaler], String>,
) -> WorkloadsOverviewSnapshot {
    WorkloadsOverviewSnapshot {
        pods: build_kind_snapshot(pods, classify_pod_health),
        deployments: build_kind_snapshot(deployments, classify_deployment_health),
        statefulsets: build_kind_snapshot(statefulsets, classify_statefulset_health),
        daemonsets: build_kind_snapshot(daemonsets, classify_daemonset_health),
        replicasets: build_kind_snapshot(replicasets, classify_replicaset_health),
        jobs: build_kind_snapshot(jobs, classify_job_health),
        cronjobs: build_kind_snapshot(cronjobs, classify_cronjob_health),
        hpas: build_kind_snapshot(hpas, classify_hpa_health),
    }
}

pub async fn fetch_workloads_overview(client: &Client) -> WorkloadsOverviewSnapshot {
    let pods_api: Api<Pod> = Api::all(client.clone());
    let deployments_api: Api<Deployment> = Api::all(client.clone());
    let statefulsets_api: Api<StatefulSet> = Api::all(client.clone());
    let daemonsets_api: Api<DaemonSet> = Api::all(client.clone());
    let replicasets_api: Api<ReplicaSet> = Api::all(client.clone());
    let jobs_api: Api<Job> = Api::all(client.clone());
    let cronjobs_api: Api<CronJob> = Api::all(client.clone());
    let hpas_api: Api<HorizontalPodAutoscaler> = Api::all(client.clone());

    let (
        pods_res,
        deployments_res,
        statefulsets_res,
        daemonsets_res,
        replicasets_res,
        jobs_res,
        cronjobs_res,
        hpas_res,
    ) = tokio::join!(
        fetch_list(&pods_api, "pods"),
        fetch_list(&deployments_api, "deployments"),
        fetch_list(&statefulsets_api, "statefulsets"),
        fetch_list(&daemonsets_api, "daemonsets"),
        fetch_list(&replicasets_api, "replicasets"),
        fetch_list(&jobs_api, "jobs"),
        fetch_list(&cronjobs_api, "cronjobs"),
        fetch_list(&hpas_api, "horizontalpodautoscalers"),
    );

    build_workloads_overview_snapshot(
        pods_res.as_deref().map_err(Clone::clone),
        deployments_res.as_deref().map_err(Clone::clone),
        statefulsets_res.as_deref().map_err(Clone::clone),
        daemonsets_res.as_deref().map_err(Clone::clone),
        replicasets_res.as_deref().map_err(Clone::clone),
        jobs_res.as_deref().map_err(Clone::clone),
        cronjobs_res.as_deref().map_err(Clone::clone),
        hpas_res.as_deref().map_err(Clone::clone),
    )
}

async fn fetch_list<K>(api: &Api<K>, resource_name: &str) -> Result<Vec<K>, String>
where
    K: kube::Resource + Clone + serde::de::DeserializeOwned + std::fmt::Debug,
{
    match tokio::time::timeout(
        WORKLOADS_OVERVIEW_FETCH_TIMEOUT,
        api.list(&ListParams::default()),
    )
    .await
    {
        Ok(Ok(list)) => Ok(list.items),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err(format!(
            "Timed out fetching {resource_name} after {}s",
            WORKLOADS_OVERVIEW_FETCH_TIMEOUT.as_secs()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::apps::v1::{
        DaemonSetStatus, DeploymentCondition, DeploymentSpec, DeploymentStatus,
        ReplicaSetCondition, ReplicaSetSpec, ReplicaSetStatus, StatefulSetSpec, StatefulSetStatus,
    };
    use k8s_openapi::api::autoscaling::v2::{
        HorizontalPodAutoscalerCondition, HorizontalPodAutoscalerStatus,
    };
    use k8s_openapi::api::batch::v1::{CronJobSpec, JobCondition, JobSpec, JobStatus};
    use k8s_openapi::api::core::v1::{
        ContainerState, ContainerStateTerminated, ContainerStateWaiting, ContainerStatus, PodStatus,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    fn meta_in(ns: &str) -> ObjectMeta {
        ObjectMeta {
            namespace: Some(ns.to_string()),
            ..Default::default()
        }
    }

    fn make_pod(
        ns: &str,
        phase: Option<&str>,
        waiting_reason: Option<&str>,
        terminated_reason: Option<&str>,
    ) -> Pod {
        let state = if waiting_reason.is_some() || terminated_reason.is_some() {
            Some(ContainerState {
                waiting: waiting_reason.map(|r| ContainerStateWaiting {
                    reason: Some(r.to_string()),
                    ..Default::default()
                }),
                terminated: terminated_reason.map(|r| ContainerStateTerminated {
                    reason: Some(r.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
        } else {
            None
        };
        let container_statuses = state.map(|s| {
            vec![ContainerStatus {
                name: "app".to_string(),
                state: Some(s),
                ..Default::default()
            }]
        });
        Pod {
            metadata: meta_in(ns),
            status: Some(PodStatus {
                phase: phase.map(ToString::to_string),
                container_statuses,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_deployment(
        ns: &str,
        desired: i32,
        current: i32,
        available: i32,
        cond: Option<(&str, &str)>,
    ) -> Deployment {
        Deployment {
            metadata: meta_in(ns),
            spec: Some(DeploymentSpec {
                replicas: Some(desired),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                replicas: Some(current),
                available_replicas: Some(available),
                conditions: cond.map(|(t, s)| {
                    vec![DeploymentCondition {
                        type_: t.to_string(),
                        status: s.to_string(),
                        ..Default::default()
                    }]
                }),
                ..Default::default()
            }),
        }
    }

    fn make_statefulset(ns: &str, desired: i32, ready: i32) -> StatefulSet {
        StatefulSet {
            metadata: meta_in(ns),
            spec: Some(StatefulSetSpec {
                replicas: Some(desired),
                ..Default::default()
            }),
            status: Some(StatefulSetStatus {
                ready_replicas: Some(ready),
                ..Default::default()
            }),
        }
    }

    fn make_daemonset(
        ns: &str,
        desired: i32,
        ready: i32,
        unavailable: Option<i32>,
        misscheduled: i32,
    ) -> DaemonSet {
        DaemonSet {
            metadata: meta_in(ns),
            status: Some(DaemonSetStatus {
                desired_number_scheduled: desired,
                number_ready: ready,
                number_unavailable: unavailable,
                number_misscheduled: misscheduled,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_replicaset(
        ns: &str,
        desired: i32,
        ready: i32,
        cond: Option<(&str, &str)>,
    ) -> ReplicaSet {
        ReplicaSet {
            metadata: meta_in(ns),
            spec: Some(ReplicaSetSpec {
                replicas: Some(desired),
                ..Default::default()
            }),
            status: Some(ReplicaSetStatus {
                ready_replicas: Some(ready),
                conditions: cond.map(|(t, s)| {
                    vec![ReplicaSetCondition {
                        type_: t.to_string(),
                        status: s.to_string(),
                        ..Default::default()
                    }]
                }),
                ..Default::default()
            }),
        }
    }

    fn make_job(
        ns: &str,
        completions: Option<i32>,
        active: Option<i32>,
        succeeded: Option<i32>,
        failed: Option<i32>,
        cond: Option<(&str, &str)>,
    ) -> Job {
        Job {
            metadata: meta_in(ns),
            spec: Some(JobSpec {
                completions,
                ..Default::default()
            }),
            status: Some(JobStatus {
                active,
                succeeded,
                failed,
                conditions: cond.map(|(t, s)| {
                    vec![JobCondition {
                        type_: t.to_string(),
                        status: s.to_string(),
                        ..Default::default()
                    }]
                }),
                ..Default::default()
            }),
        }
    }

    fn make_cronjob(ns: &str, suspend: Option<bool>) -> CronJob {
        CronJob {
            metadata: meta_in(ns),
            spec: Some(CronJobSpec {
                suspend,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_hpa(
        ns: &str,
        current: i32,
        desired: i32,
        cond: Option<(&str, &str)>,
    ) -> HorizontalPodAutoscaler {
        HorizontalPodAutoscaler {
            metadata: meta_in(ns),
            status: Some(HorizontalPodAutoscalerStatus {
                current_replicas: Some(current),
                desired_replicas: desired,
                conditions: cond.map(|(t, s)| {
                    vec![HorizontalPodAutoscalerCondition {
                        type_: t.to_string(),
                        status: s.to_string(),
                        ..Default::default()
                    }]
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_classify_all_eight_workload_kinds_into_healthy_degraded_failed() {
        // Pods
        assert_eq!(
            classify_pod_health(&make_pod("default", Some("Running"), None, None)),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_pod_health(&make_pod("default", Some("Succeeded"), None, None)),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_pod_health(&make_pod("default", Some("Pending"), None, None)),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_pod_health(&make_pod("default", Some("Failed"), None, None)),
            WorkloadHealthStatus::Failed
        );
        assert_eq!(
            classify_pod_health(&make_pod(
                "default",
                Some("Running"),
                Some("CrashLoopBackOff"),
                None
            )),
            WorkloadHealthStatus::Failed
        );
        assert_eq!(
            classify_pod_health(&make_pod(
                "default",
                Some("Running"),
                None,
                Some("OOMKilled")
            )),
            WorkloadHealthStatus::Failed
        );

        // Deployments
        assert_eq!(
            classify_deployment_health(&make_deployment(
                "default",
                3,
                3,
                3,
                Some(("Available", "True"))
            )),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_deployment_health(&make_deployment("default", 0, 0, 0, None)),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_deployment_health(&make_deployment(
                "default",
                3,
                3,
                1,
                Some(("Progressing", "True"))
            )),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_deployment_health(&make_deployment(
                "default",
                3,
                0,
                0,
                Some(("ReplicaFailure", "True"))
            )),
            WorkloadHealthStatus::Failed
        );

        // StatefulSets
        assert_eq!(
            classify_statefulset_health(&make_statefulset("default", 3, 3)),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_statefulset_health(&make_statefulset("default", 3, 1)),
            WorkloadHealthStatus::Degraded
        );

        // DaemonSets
        assert_eq!(
            classify_daemonset_health(&make_daemonset("default", 4, 4, Some(0), 0)),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_daemonset_health(&make_daemonset("default", 4, 2, Some(2), 0)),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_daemonset_health(&make_daemonset("default", 4, 0, Some(4), 0)),
            WorkloadHealthStatus::Failed
        );

        // ReplicaSets
        assert_eq!(
            classify_replicaset_health(&make_replicaset("default", 2, 2, None)),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_replicaset_health(&make_replicaset("default", 2, 1, None)),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_replicaset_health(&make_replicaset(
                "default",
                2,
                0,
                Some(("ReplicaFailure", "True"))
            )),
            WorkloadHealthStatus::Failed
        );

        // Jobs
        assert_eq!(
            classify_job_health(&make_job(
                "default",
                Some(1),
                Some(0),
                Some(1),
                Some(0),
                Some(("Complete", "True"))
            )),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_job_health(&make_job(
                "default",
                Some(1),
                Some(1),
                Some(0),
                Some(0),
                None
            )),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_job_health(&make_job(
                "default",
                Some(1),
                Some(1),
                Some(0),
                Some(1),
                None
            )),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_job_health(&make_job(
                "default",
                Some(1),
                Some(0),
                Some(0),
                Some(2),
                Some(("Failed", "True"))
            )),
            WorkloadHealthStatus::Failed
        );

        // CronJobs
        assert_eq!(
            classify_cronjob_health(&make_cronjob("default", Some(false))),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_cronjob_health(&make_cronjob("default", Some(true))),
            WorkloadHealthStatus::Degraded
        );

        // HPAs
        assert_eq!(
            classify_hpa_health(&make_hpa("default", 2, 2, Some(("ScalingActive", "True")))),
            WorkloadHealthStatus::Healthy
        );
        assert_eq!(
            classify_hpa_health(&make_hpa("default", 5, 5, Some(("ScalingLimited", "True")))),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_hpa_health(&make_hpa("default", 0, 0, None)),
            WorkloadHealthStatus::Degraded
        );
        assert_eq!(
            classify_hpa_health(&make_hpa("default", 1, 1, Some(("ScalingActive", "False")))),
            WorkloadHealthStatus::Failed
        );
        assert_eq!(
            classify_hpa_health(&make_hpa("default", 1, 1, Some(("AbleToScale", "False")))),
            WorkloadHealthStatus::Failed
        );
    }

    #[test]
    fn test_snapshot_counts_filter_by_selected_namespaces_and_preserve_partial_rbac_errors() {
        let pods = vec![
            make_pod("default", Some("Running"), None, None),
            make_pod("prod", Some("Pending"), None, None),
            make_pod("staging", Some("Failed"), None, None),
        ];
        let deployments = vec![
            make_deployment("prod", 2, 2, 2, Some(("Available", "True"))),
            make_deployment("staging", 2, 2, 1, Some(("Progressing", "True"))),
        ];
        let statefulsets = vec![make_statefulset("prod", 3, 3)];
        let daemonsets = vec![make_daemonset("default", 2, 2, Some(0), 0)];
        let replicasets = vec![make_replicaset("prod", 2, 2, None)];
        let jobs = vec![make_job(
            "staging",
            Some(1),
            Some(0),
            Some(1),
            Some(0),
            Some(("Complete", "True")),
        )];
        let cronjobs = vec![make_cronjob("default", Some(true))];

        let snapshot = build_workloads_overview_snapshot(
            Ok(&pods),
            Ok(&deployments),
            Ok(&statefulsets),
            Ok(&daemonsets),
            Ok(&replicasets),
            Ok(&jobs),
            Ok(&cronjobs),
            Err("Forbidden".to_string()),
        );

        assert_eq!(snapshot.hpas.error.as_deref(), Some("Forbidden"));
        assert!(snapshot.has_any_error());

        // All namespaces
        let all_pods = snapshot.pods.counts_for_namespaces(&[]);
        assert_eq!(
            all_pods,
            WorkloadHealthCounts {
                total: 3,
                healthy: 1,
                degraded: 1,
                failed: 1,
            }
        );

        // Single namespace: prod
        let prod_ns = vec!["prod".to_string()];
        assert_eq!(
            snapshot.pods.counts_for_namespaces(&prod_ns),
            WorkloadHealthCounts {
                total: 1,
                healthy: 0,
                degraded: 1,
                failed: 0,
            }
        );
        assert_eq!(
            snapshot.deployments.counts_for_namespaces(&prod_ns),
            WorkloadHealthCounts {
                total: 1,
                healthy: 1,
                degraded: 0,
                failed: 0,
            }
        );

        // Multi-namespace: prod + staging
        let prod_staging = vec!["prod".to_string(), "staging".to_string()];
        let prod_staging_pods = snapshot.pods.counts_for_namespaces(&prod_staging);
        assert_eq!(
            prod_staging_pods,
            WorkloadHealthCounts {
                total: 2,
                healthy: 0,
                degraded: 1,
                failed: 1,
            }
        );

        // Aggregate counts across all 8 kinds
        let all_agg = snapshot.aggregate_counts(&[]);
        assert_eq!(all_agg.total, 10);
        assert_eq!(all_agg.healthy + all_agg.degraded + all_agg.failed, all_agg.total);
        assert_eq!(all_agg.healthy, 6);
        assert_eq!(all_agg.degraded, 3);
        assert_eq!(all_agg.failed, 1);

        let prod_agg = snapshot.aggregate_counts(&prod_ns);
        assert_eq!(prod_agg.total, 4);
        assert_eq!(prod_agg.healthy, 3);
        assert_eq!(prod_agg.degraded, 1);
        assert_eq!(prod_agg.failed, 0);
    }
}
