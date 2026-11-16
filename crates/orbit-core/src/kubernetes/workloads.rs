use std::time::Duration;

use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ApiResource, DynamicObject, GroupVersionKind, ListParams},
    Client,
};

use crate::kubernetes::models;
use crate::kubernetes::namespaces::format_age;

pub const DEPLOYMENT_FETCH_TIMEOUT: Duration = Duration::from_secs(10);
pub const POD_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub fn map_pod(p: &Pod) -> models::PodInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let namespace_name = p.metadata.namespace.clone().unwrap_or_default();

    let status = p
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let age = format_age(&p.metadata.creation_timestamp);

    let node = p.spec.as_ref().and_then(|s| s.node_name.clone());
    let ip = p.status.as_ref().and_then(|s| s.pod_ip.clone());
    let node_ip = p.status.as_ref().and_then(|s| s.host_ip.clone());
    let qos_class = p.status.as_ref().and_then(|s| s.qos_class.clone());

    let controlled_by = p
        .metadata
        .owner_references
        .as_ref()
        .and_then(|owners| owners.first())
        .map(|o| format!("{}/{}", o.kind, o.name));

    let labels = p.metadata.labels.clone().unwrap_or_default();
    let annotations = p.metadata.annotations.clone().unwrap_or_default();

    let mut containers = Vec::new();
    let mut images = Vec::new();
    let mut total_restarts = 0;

    let container_statuses = p.status.as_ref().and_then(|s| s.container_statuses.as_ref());

    if let Some(spec) = p.spec.as_ref() {
        for c in &spec.containers {
            let c_name = c.name.clone();
            let c_image = c.image.clone().unwrap_or_default();
            if !c_image.is_empty() && !images.contains(&c_image) {
                images.push(c_image.clone());
            }

            let mut status_str = "Waiting".to_string();
            let mut ready_str = "false".to_string();
            let mut restarts = 0;

            if let Some(statuses) = container_statuses
                && let Some(cs) = statuses.iter().find(|s| s.name == c_name)
            {
                ready_str = cs.ready.to_string();
                restarts = cs.restart_count;
                total_restarts += restarts;

                if let Some(state) = &cs.state {
                    if state.running.is_some() {
                        status_str = "Running".to_string();
                    } else if let Some(term) = &state.terminated {
                        let reason = term.reason.as_deref().unwrap_or("Terminated");
                        status_str = format!("Terminated ({})", reason);
                    } else if let Some(waiting) = &state.waiting {
                        status_str = waiting
                            .reason
                            .clone()
                            .unwrap_or_else(|| "Waiting".to_string());
                    }
                }
            }

            let mut ports_str = None;
            if let Some(ports) = &c.ports {
                let port_list: Vec<String> = ports
                    .iter()
                    .map(|port| {
                        format!(
                            "{}/{}",
                            port.container_port,
                            port.protocol.as_deref().unwrap_or("TCP")
                        )
                    })
                    .collect();
                if !port_list.is_empty() {
                    ports_str = Some(port_list.join(", "));
                }
            }

            containers.push(models::PodContainer {
                name: c_name,
                image: c_image,
                status: status_str,
                ready: ready_str,
                restarts,
                ports: ports_str,
            });
        }
    }

    models::PodInfo {
        name,
        namespace: namespace_name,
        status,
        age,
        cpu: None,
        memory: None,
        cpu_cores: None,
        memory_bytes: None,
        cpu_pct: None,
        memory_pct: None,
        node,
        restarts: total_restarts,
        images,
        labels,
        annotations,
        ip,
        node_ip,
        controlled_by,
        qos_class,
        containers,
    }
}

pub async fn list_pods(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::PodInfo>, kube::Error> {
    let pods: Api<Pod> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let ar = ApiResource::from_gvk(&gvk);
    let metrics_api: Api<DynamicObject> = if let Some(ref ns) = namespace {
        Api::namespaced_with(client.clone(), ns, &ar)
    } else {
        Api::all_with(client.clone(), &ar)
    };

    let lp = ListParams::default();
    let (pods_res, metrics_res) = tokio::join!(pods.list(&lp), metrics_api.list(&lp));

    let pod_items = pods_res?;

    let metrics_map = match metrics_res {
        Ok(metric_list) => crate::kubernetes::metrics::parse_pod_metrics_map(&metric_list.items),
        Err(e) => {
            tracing::debug!(
                error = ?e,
                "Could not fetch pod metrics on list (Metrics Server not installed?)"
            );
            std::collections::HashMap::new()
        }
    };

    let mut pod_list = Vec::new();
    for p in pod_items {
        let mut info = map_pod(&p);
        if let Some(metric) = metrics_map.get(&(info.name.clone(), info.namespace.clone())) {
            info.cpu = Some(metric.cpu.clone());
            info.memory = Some(metric.memory.clone());
            info.cpu_cores = metric.cpu_cores;
            info.memory_bytes = metric.memory_bytes;
        }
        pod_list.push(info);
    }

    Ok(pod_list)
}

pub async fn fetch_pods(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::PodInfo>, String> {
    match tokio::time::timeout(POD_FETCH_TIMEOUT, list_pods(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching pods after 10s".to_string()),
    }
}

pub fn map_deployment(d: &Deployment) -> models::DeploymentInfo {
    let name = d.metadata.name.clone().unwrap_or_default();
    let namespace_name = d.metadata.namespace.clone().unwrap_or_default();

    let age = format_age(&d.metadata.creation_timestamp);

    let desired = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
    let status_replicas = d.status.as_ref();
    let current = status_replicas.and_then(|s| s.replicas).unwrap_or(0);
    let available = status_replicas.and_then(|s| s.available_replicas).unwrap_or(0);
    let up_to_date = status_replicas.and_then(|s| s.updated_replicas).unwrap_or(0);

    let replicas = models::Replicas { current, desired };

    let mut status = "Progressing".to_string();
    if (desired == 0 && current == 0) || available == desired {
        status = "Running".to_string();
    } else if let Some(conds) = status_replicas.and_then(|st| st.conditions.as_ref()) {
        for c in conds {
            if c.type_ == "ReplicaFailure" && c.status == "True" {
                status = "Failed".to_string();
            }
        }
    }

    let conditions = status_replicas
        .and_then(|st| st.conditions.as_ref())
        .cloned();

    let mut images = Vec::new();
    let mut containers = Vec::new();
    let mut ports = Vec::new();
    if let Some(spec) = d.spec.as_ref() {
        for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
            if let Some(img) = &c.image {
                images.push(img.clone());
            }
            containers.push(models::ContainerImageInfo {
                name: c.name.clone(),
                image: c.image.clone().unwrap_or_default(),
            });
            if let Some(c_ports) = &c.ports {
                for cp in c_ports {
                    if cp.container_port > 0 && !ports.contains(&cp.container_port) {
                        ports.push(cp.container_port);
                    }
                }
            }
        }
    }

    let strategy = d
        .spec
        .as_ref()
        .and_then(|s| s.strategy.as_ref())
        .and_then(|strt| strt.type_.clone());

    let min_ready_seconds = d.spec.as_ref().and_then(|s| s.min_ready_seconds).unwrap_or(0);
    let revision_history = d.spec.as_ref().and_then(|s| s.revision_history_limit);

    let labels = d.metadata.labels.clone().unwrap_or_default();
    let annotations = d.metadata.annotations.clone().unwrap_or_default();

    models::DeploymentInfo {
        kind: "Deployment".to_string(),
        name,
        namespace: namespace_name,
        status,
        conditions,
        replicas,
        available,
        up_to_date,
        age,
        images,
        containers,
        strategy,
        min_ready_seconds,
        revision_history,
        labels,
        annotations,
        ports,
    }
}

pub async fn list_deployments(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::DeploymentInfo>, kube::Error> {
    let api: Api<Deployment> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for d in api.list(&ListParams::default()).await? {
        list.push(map_deployment(&d));
    }

    Ok(list)
}

pub async fn fetch_deployments(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::DeploymentInfo>, String> {
    match tokio::time::timeout(DEPLOYMENT_FETCH_TIMEOUT, list_deployments(client, namespace)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching deployments after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::apps::v1::{
        DeploymentCondition, DeploymentSpec, DeploymentStatus, DeploymentStrategy,
    };
    use k8s_openapi::api::core::v1::{Container, ContainerPort, PodSpec, PodTemplateSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_map_deployment_status_and_replicas() {
        let running = Deployment {
            metadata: ObjectMeta {
                name: Some("api".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(3),
                strategy: Some(DeploymentStrategy {
                    type_: Some("RollingUpdate".to_string()),
                    ..Default::default()
                }),
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "api".to_string(),
                            image: Some("ghcr.io/org/api:v1.2.3".to_string()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                replicas: Some(3),
                available_replicas: Some(3),
                updated_replicas: Some(3),
                ..Default::default()
            }),
        };

        let mapped = map_deployment(&running);
        assert_eq!(mapped.kind, "Deployment");
        assert_eq!(mapped.name, "api");
        assert_eq!(mapped.namespace, "prod");
        assert_eq!(mapped.status, "Running");
        assert_eq!(mapped.replicas.current, 3);
        assert_eq!(mapped.replicas.desired, 3);
        assert_eq!(mapped.available, 3);
        assert_eq!(mapped.up_to_date, 3);
        assert_eq!(mapped.strategy.as_deref(), Some("RollingUpdate"));
        assert_eq!(mapped.images, vec!["ghcr.io/org/api:v1.2.3"]);

        let scaled_to_zero = Deployment {
            metadata: ObjectMeta {
                name: Some("worker".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(0),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                replicas: Some(0),
                available_replicas: Some(0),
                updated_replicas: Some(0),
                ..Default::default()
            }),
        };
        assert_eq!(map_deployment(&scaled_to_zero).status, "Running");

        let progressing = Deployment {
            metadata: ObjectMeta {
                name: Some("web".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(2),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                replicas: Some(2),
                available_replicas: Some(1),
                updated_replicas: Some(1),
                ..Default::default()
            }),
        };
        assert_eq!(map_deployment(&progressing).status, "Progressing");

        let failed = Deployment {
            metadata: ObjectMeta {
                name: Some("broken".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(2),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                replicas: Some(1),
                available_replicas: Some(0),
                conditions: Some(vec![DeploymentCondition {
                    type_: "ReplicaFailure".to_string(),
                    status: "True".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };
        assert_eq!(map_deployment(&failed).status, "Failed");
    }

    #[test]
    fn test_map_deployment_conditions() {
        let dep = Deployment {
            metadata: ObjectMeta {
                name: Some("test-deploy".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(DeploymentStatus {
                conditions: Some(vec![
                    DeploymentCondition {
                        type_: "Available".to_string(),
                        status: "True".to_string(),
                        reason: Some("MinimumReplicasAvailable".to_string()),
                        message: Some("Deployment has minimum availability.".to_string()),
                        last_transition_time: None,
                        last_update_time: None,
                    },
                    DeploymentCondition {
                        type_: "Progressing".to_string(),
                        status: "True".to_string(),
                        reason: Some("NewReplicaSetAvailable".to_string()),
                        message: Some("ReplicaSet is progressing.".to_string()),
                        last_transition_time: None,
                        last_update_time: None,
                    },
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mapped = map_deployment(&dep);
        assert_eq!(mapped.name, "test-deploy");
        assert_eq!(mapped.kind, "Deployment");
        let conditions = mapped.conditions.as_ref().expect("conditions should be present");
        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0].type_, "Available");
        assert_eq!(conditions[0].status, "True");
        assert_eq!(conditions[0].reason.as_deref(), Some("MinimumReplicasAvailable"));
        assert_eq!(conditions[1].type_, "Progressing");
    }

    #[test]
    fn test_map_deployment_ports() {
        let dep = Deployment {
            metadata: ObjectMeta {
                name: Some("web-app".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                template: PodTemplateSpec {
                    spec: Some(PodSpec {
                        containers: vec![
                            Container {
                                name: "nginx".to_string(),
                                ports: Some(vec![
                                    ContainerPort {
                                        container_port: 80,
                                        protocol: Some("TCP".to_string()),
                                        ..Default::default()
                                    },
                                    ContainerPort {
                                        container_port: 443,
                                        protocol: Some("TCP".to_string()),
                                        ..Default::default()
                                    },
                                ]),
                                ..Default::default()
                            },
                            Container {
                                name: "sidecar".to_string(),
                                ports: Some(vec![
                                    ContainerPort {
                                        container_port: 80,
                                        protocol: Some("TCP".to_string()),
                                        ..Default::default()
                                    },
                                    ContainerPort {
                                        container_port: 9090,
                                        protocol: Some("TCP".to_string()),
                                        ..Default::default()
                                    },
                                ]),
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
        };

        let mapped = map_deployment(&dep);
        assert_eq!(mapped.name, "web-app");
        assert_eq!(mapped.kind, "Deployment");
        assert_eq!(mapped.ports, vec![80, 443, 9090]);
    }

    #[test]
    fn test_map_deployment_no_ports() {
        let dep = Deployment {
            metadata: ObjectMeta {
                name: Some("no-ports".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let mapped = map_deployment(&dep);
        assert_eq!(mapped.ports, Vec::<i32>::new());
    }

    #[test]
    fn test_map_pod_status_containers_and_metadata() {
        use k8s_openapi::api::core::v1::{
            ContainerState, ContainerStateRunning, ContainerStateWaiting, ContainerStatus, PodSpec,
            PodStatus,
        };
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
        use std::collections::BTreeMap;

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "web".to_string());
        let mut annotations = BTreeMap::new();
        annotations.insert("owner".to_string(), "platform".to_string());

        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("web-abc-123".to_string()),
                namespace: Some("prod".to_string()),
                labels: Some(labels),
                annotations: Some(annotations),
                owner_references: Some(vec![OwnerReference {
                    kind: "ReplicaSet".to_string(),
                    name: "web-abc".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            spec: Some(PodSpec {
                node_name: Some("node-1".to_string()),
                containers: vec![
                    Container {
                        name: "app".to_string(),
                        image: Some("nginx:1.25".to_string()),
                        ports: Some(vec![ContainerPort {
                            container_port: 8080,
                            protocol: Some("TCP".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    Container {
                        name: "sidecar".to_string(),
                        image: Some("envoy:1.29".to_string()),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                pod_ip: Some("10.0.0.42".to_string()),
                host_ip: Some("192.168.1.10".to_string()),
                qos_class: Some("Burstable".to_string()),
                container_statuses: Some(vec![
                    ContainerStatus {
                        name: "app".to_string(),
                        ready: true,
                        restart_count: 0,
                        state: Some(ContainerState {
                            running: Some(ContainerStateRunning::default()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    ContainerStatus {
                        name: "sidecar".to_string(),
                        ready: false,
                        restart_count: 3,
                        state: Some(ContainerState {
                            waiting: Some(ContainerStateWaiting {
                                reason: Some("CrashLoopBackOff".to_string()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
        };

        let mapped = map_pod(&pod);
        assert_eq!(mapped.name, "web-abc-123");
        assert_eq!(mapped.namespace, "prod");
        assert_eq!(mapped.status, "Running");
        assert_eq!(mapped.ip.as_deref(), Some("10.0.0.42"));
        assert_eq!(mapped.node_ip.as_deref(), Some("192.168.1.10"));
        assert_eq!(mapped.node.as_deref(), Some("node-1"));
        assert_eq!(mapped.controlled_by.as_deref(), Some("ReplicaSet/web-abc"));
        assert_eq!(mapped.qos_class.as_deref(), Some("Burstable"));
        assert_eq!(mapped.restarts, 3);
        assert_eq!(mapped.images, vec!["nginx:1.25", "envoy:1.29"]);
        assert_eq!(mapped.labels.get("app").map(String::as_str), Some("web"));
        assert_eq!(
            mapped.annotations.get("owner").map(String::as_str),
            Some("platform")
        );
        assert_eq!(mapped.containers.len(), 2);
        assert_eq!(mapped.containers[0].ready, "true");
        assert_eq!(mapped.containers[0].status, "Running");
        assert_eq!(mapped.containers[0].ports.as_deref(), Some("8080/TCP"));
        assert_eq!(mapped.containers[1].ready, "false");
        assert_eq!(mapped.containers[1].status, "CrashLoopBackOff");
        assert_eq!(mapped.containers[1].restarts, 3);
        assert_eq!(mapped.containers[1].ports, None);
    }

    #[test]
    fn test_map_pod_defaults_missing_fields() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("bare-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let mapped = map_pod(&pod);
        assert_eq!(mapped.name, "bare-pod");
        assert_eq!(mapped.namespace, "default");
        assert_eq!(mapped.status, "Unknown");
        assert_eq!(mapped.restarts, 0);
        assert_eq!(mapped.node, None);
        assert_eq!(mapped.ip, None);
        assert!(mapped.containers.is_empty());
    }
}
