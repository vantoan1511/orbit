use std::time::Duration;

use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Node, Pod, Service};
use kube::api::ListParams;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};

const OVERVIEW_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClusterOverallHealth {
    Healthy,
    Degraded,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NodeHealthSummary {
    pub total: usize,
    pub ready: usize,
    pub not_ready: usize,
    pub cordoned: usize,
    pub unknown: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PodHealthSummary {
    pub total: usize,
    pub running: usize,
    pub pending: usize,
    pub failed: usize,
    pub succeeded: usize,
    pub crash_loop: usize,
    pub other: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeploymentHealthSummary {
    pub total: usize,
    pub available: usize,
    pub progressing: usize,
    pub failed: usize,
    pub desired_replicas: i32,
    pub available_replicas: i32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServiceSummary {
    pub total: usize,
    pub cluster_ip: usize,
    pub node_port: usize,
    pub load_balancer: usize,
    pub external_name: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ClusterOverviewSummary {
    pub k8s_version: Option<String>,
    pub overall_health: ClusterOverallHealth,
    pub nodes: NodeHealthSummary,
    pub pods: PodHealthSummary,
    pub deployments: DeploymentHealthSummary,
    pub services: ServiceSummary,
}

pub fn summarize_nodes(nodes: &[Node]) -> NodeHealthSummary {
    let mut summary = NodeHealthSummary {
        total: nodes.len(),
        ..Default::default()
    };

    for node in nodes {
        if node.spec.as_ref().and_then(|s| s.unschedulable) == Some(true) {
            summary.cordoned += 1;
        }

        let ready_status = node
            .status
            .as_ref()
            .and_then(|s| s.conditions.as_ref())
            .and_then(|conditions| conditions.iter().find(|c| c.type_ == "Ready"))
            .map(|c| c.status.as_str());

        match ready_status {
            Some("True") => summary.ready += 1,
            Some("False") => summary.not_ready += 1,
            _ => summary.unknown += 1,
        }
    }

    summary
}

pub fn summarize_pods(pods: &[Pod]) -> PodHealthSummary {
    let mut summary = PodHealthSummary {
        total: pods.len(),
        ..Default::default()
    };

    for pod in pods {
        let status = pod.status.as_ref();
        let phase = status.and_then(|s| s.phase.as_deref());

        match phase {
            Some("Running") => summary.running += 1,
            Some("Pending") => summary.pending += 1,
            Some("Failed") => summary.failed += 1,
            Some("Succeeded") => summary.succeeded += 1,
            _ => summary.other += 1,
        }

        let has_crash_loop = status
            .map(|s| {
                let container_crash = s
                    .container_statuses
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .any(is_crash_loop_container);
                let init_crash = s
                    .init_container_statuses
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .any(is_crash_loop_container);
                container_crash || init_crash
            })
            .unwrap_or(false);

        if has_crash_loop {
            summary.crash_loop += 1;
        }
    }

    summary
}

fn is_crash_loop_container(cs: &k8s_openapi::api::core::v1::ContainerStatus) -> bool {
    cs.state
        .as_ref()
        .and_then(|state| state.waiting.as_ref())
        .and_then(|waiting| waiting.reason.as_deref())
        == Some("CrashLoopBackOff")
}

pub fn summarize_deployments(deployments: &[Deployment]) -> DeploymentHealthSummary {
    let mut summary = DeploymentHealthSummary {
        total: deployments.len(),
        ..Default::default()
    };

    for deployment in deployments {
        let desired = deployment
            .spec
            .as_ref()
            .and_then(|s| s.replicas)
            .unwrap_or(1)
            .max(0);
        let available = deployment
            .status
            .as_ref()
            .and_then(|s| s.available_replicas)
            .unwrap_or(0)
            .max(0);

        summary.desired_replicas += desired;
        summary.available_replicas += available;

        let conditions = deployment
            .status
            .as_ref()
            .and_then(|s| s.conditions.as_deref())
            .unwrap_or(&[]);

        let has_replica_failure = conditions
            .iter()
            .any(|c| c.type_ == "ReplicaFailure" && c.status == "True");
        let is_available = conditions
            .iter()
            .any(|c| c.type_ == "Available" && c.status == "True");
        let is_progressing = conditions
            .iter()
            .any(|c| c.type_ == "Progressing" && c.status == "True");

        if has_replica_failure {
            summary.failed += 1;
        } else if is_available || (desired > 0 && available >= desired) {
            summary.available += 1;
        } else if is_progressing || desired > available {
            summary.progressing += 1;
        } else {
            summary.available += 1;
        }
    }

    summary
}

pub fn summarize_services(services: &[Service]) -> ServiceSummary {
    let mut summary = ServiceSummary {
        total: services.len(),
        ..Default::default()
    };

    for service in services {
        let service_type = service
            .spec
            .as_ref()
            .and_then(|s| s.type_.as_deref())
            .unwrap_or("ClusterIP");

        match service_type {
            "NodePort" => summary.node_port += 1,
            "LoadBalancer" => summary.load_balancer += 1,
            "ExternalName" => summary.external_name += 1,
            _ => summary.cluster_ip += 1,
        }
    }

    summary
}

pub fn compute_overall_health(
    nodes: &NodeHealthSummary,
    pods: &PodHealthSummary,
    deployments: &DeploymentHealthSummary,
) -> ClusterOverallHealth {
    if nodes.not_ready > 0
        || pods.failed > 0
        || pods.crash_loop > 0
        || deployments.failed > 0
    {
        ClusterOverallHealth::Degraded
    } else if nodes.error.is_some() && pods.error.is_some() && deployments.error.is_some() {
        ClusterOverallHealth::Unknown
    } else {
        ClusterOverallHealth::Healthy
    }
}

pub fn build_cluster_overview_summary(
    k8s_version: Option<String>,
    nodes: Result<&[Node], String>,
    pods: Result<&[Pod], String>,
    deployments: Result<&[Deployment], String>,
    services: Result<&[Service], String>,
) -> ClusterOverviewSummary {
    let nodes_summary = match nodes {
        Ok(list) => summarize_nodes(list),
        Err(err) => NodeHealthSummary {
            error: Some(err),
            ..Default::default()
        },
    };
    let pods_summary = match pods {
        Ok(list) => summarize_pods(list),
        Err(err) => PodHealthSummary {
            error: Some(err),
            ..Default::default()
        },
    };
    let deployments_summary = match deployments {
        Ok(list) => summarize_deployments(list),
        Err(err) => DeploymentHealthSummary {
            error: Some(err),
            ..Default::default()
        },
    };
    let services_summary = match services {
        Ok(list) => summarize_services(list),
        Err(err) => ServiceSummary {
            error: Some(err),
            ..Default::default()
        },
    };

    let overall_health =
        compute_overall_health(&nodes_summary, &pods_summary, &deployments_summary);

    ClusterOverviewSummary {
        k8s_version,
        overall_health,
        nodes: nodes_summary,
        pods: pods_summary,
        deployments: deployments_summary,
        services: services_summary,
    }
}

pub async fn fetch_cluster_overview(
    client: &Client,
    k8s_version: Option<String>,
) -> ClusterOverviewSummary {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());
    let deployments_api: Api<Deployment> = Api::all(client.clone());
    let services_api: Api<Service> = Api::all(client.clone());

    let (nodes_res, pods_res, deployments_res, services_res) = tokio::join!(
        fetch_list(&nodes_api, "nodes"),
        fetch_list(&pods_api, "pods"),
        fetch_list(&deployments_api, "deployments"),
        fetch_list(&services_api, "services"),
    );

    build_cluster_overview_summary(
        k8s_version,
        nodes_res.as_deref().map_err(Clone::clone),
        pods_res.as_deref().map_err(Clone::clone),
        deployments_res.as_deref().map_err(Clone::clone),
        services_res.as_deref().map_err(Clone::clone),
    )
}

async fn fetch_list<K>(api: &Api<K>, resource_name: &str) -> Result<Vec<K>, String>
where
    K: kube::Resource + Clone + serde::de::DeserializeOwned + std::fmt::Debug,
{
    match tokio::time::timeout(OVERVIEW_FETCH_TIMEOUT, api.list(&ListParams::default())).await {
        Ok(Ok(list)) => Ok(list.items),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err(format!(
            "Timed out fetching {resource_name} after {}s",
            OVERVIEW_FETCH_TIMEOUT.as_secs()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::apps::v1::{DeploymentCondition, DeploymentSpec, DeploymentStatus};
    use k8s_openapi::api::core::v1::{
        ContainerState, ContainerStateWaiting, ContainerStatus, NodeCondition, NodeSpec,
        NodeStatus, PodSpec, PodStatus, ServiceSpec,
    };

    fn make_node(ready_status: Option<&str>, unschedulable: Option<bool>) -> Node {
        let conditions = ready_status.map(|status| {
            vec![NodeCondition {
                type_: "Ready".to_string(),
                status: status.to_string(),
                ..Default::default()
            }]
        });
        Node {
            spec: Some(NodeSpec {
                unschedulable,
                ..Default::default()
            }),
            status: Some(NodeStatus {
                conditions,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_pod(phase: &str, crash_loop: bool) -> Pod {
        let container_statuses = if crash_loop {
            Some(vec![ContainerStatus {
                name: "app".to_string(),
                state: Some(ContainerState {
                    waiting: Some(ContainerStateWaiting {
                        reason: Some("CrashLoopBackOff".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }])
        } else {
            None
        };
        Pod {
            spec: Some(PodSpec::default()),
            status: Some(PodStatus {
                phase: Some(phase.to_string()),
                container_statuses,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_deployment(
        desired: i32,
        available_replicas: i32,
        condition_type: &str,
        condition_status: &str,
    ) -> Deployment {
        Deployment {
            spec: Some(DeploymentSpec {
                replicas: Some(desired),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                available_replicas: Some(available_replicas),
                conditions: Some(vec![DeploymentCondition {
                    type_: condition_type.to_string(),
                    status: condition_status.to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_service(service_type: Option<&str>) -> Service {
        Service {
            spec: Some(ServiceSpec {
                type_: service_type.map(ToString::to_string),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_summarize_nodes_pods_deployments_services_and_overall_health() {
        let nodes = vec![
            make_node(Some("True"), None),
            make_node(Some("False"), None),
            make_node(Some("True"), Some(true)),
            make_node(None, None),
        ];
        let pods = vec![
            make_pod("Running", false),
            make_pod("Pending", false),
            make_pod("Failed", false),
            make_pod("Succeeded", false),
            make_pod("Running", true),
        ];
        let deployments = vec![
            make_deployment(2, 2, "Available", "True"),
            make_deployment(3, 1, "Progressing", "True"),
            make_deployment(2, 0, "ReplicaFailure", "True"),
        ];
        let services = vec![
            make_service(Some("ClusterIP")),
            make_service(Some("NodePort")),
            make_service(Some("LoadBalancer")),
            make_service(Some("ExternalName")),
        ];

        let summary = build_cluster_overview_summary(
            Some("v1.31.1".to_string()),
            Ok(&nodes),
            Ok(&pods),
            Ok(&deployments),
            Ok(&services),
        );

        assert_eq!(summary.k8s_version.as_deref(), Some("v1.31.1"));
        assert_eq!(summary.nodes.total, 4);
        assert_eq!(summary.nodes.ready, 2);
        assert_eq!(summary.nodes.not_ready, 1);
        assert_eq!(summary.nodes.cordoned, 1);
        assert_eq!(summary.nodes.unknown, 1);
        assert!(summary.nodes.error.is_none());

        assert_eq!(summary.pods.total, 5);
        assert_eq!(summary.pods.running, 2);
        assert_eq!(summary.pods.crash_loop, 1);
        assert_eq!(summary.pods.pending, 1);
        assert_eq!(summary.pods.failed, 1);
        assert_eq!(summary.pods.succeeded, 1);
        assert_eq!(summary.pods.other, 0);
        assert!(summary.pods.error.is_none());

        assert_eq!(summary.deployments.total, 3);
        assert_eq!(summary.deployments.available, 1);
        assert_eq!(summary.deployments.progressing, 1);
        assert_eq!(summary.deployments.failed, 1);
        assert_eq!(summary.deployments.desired_replicas, 7);
        assert_eq!(summary.deployments.available_replicas, 3);
        assert!(summary.deployments.error.is_none());

        assert_eq!(summary.services.total, 4);
        assert_eq!(summary.services.cluster_ip, 1);
        assert_eq!(summary.services.node_port, 1);
        assert_eq!(summary.services.load_balancer, 1);
        assert_eq!(summary.services.external_name, 1);
        assert_eq!(
            summary.services.cluster_ip
                + summary.services.node_port
                + summary.services.load_balancer
                + summary.services.external_name,
            4
        );
        assert_eq!(summary.overall_health, ClusterOverallHealth::Degraded);

        let healthy_nodes = vec![make_node(Some("True"), None)];
        let healthy_pods = vec![make_pod("Running", false)];
        let healthy_deployments = vec![make_deployment(1, 1, "Available", "True")];
        let healthy_services = vec![make_service(None)];
        let healthy_summary = build_cluster_overview_summary(
            Some("v1.31.1".to_string()),
            Ok(&healthy_nodes),
            Ok(&healthy_pods),
            Ok(&healthy_deployments),
            Ok(&healthy_services),
        );
        assert_eq!(
            healthy_summary.overall_health,
            ClusterOverallHealth::Healthy
        );
    }

    #[test]
    fn test_build_cluster_overview_summary_preserves_partial_results_on_rbac_error() {
        let ready_nodes = vec![make_node(Some("True"), None)];
        let deps = vec![make_deployment(1, 1, "Available", "True")];
        let svcs = vec![make_service(Some("ClusterIP"))];

        let summary = build_cluster_overview_summary(
            Some("v1.30.0".to_string()),
            Ok(&ready_nodes),
            Err("Forbidden: pods is forbidden".to_string()),
            Ok(&deps),
            Ok(&svcs),
        );

        assert_eq!(summary.nodes.total, 1);
        assert_eq!(
            summary.pods.error.as_deref(),
            Some("Forbidden: pods is forbidden")
        );
        assert_eq!(summary.pods.total, 0);
        assert_eq!(summary.deployments.total, 1);
        assert_eq!(summary.services.total, 1);
        assert_eq!(summary.overall_health, ClusterOverallHealth::Healthy);
    }
}
