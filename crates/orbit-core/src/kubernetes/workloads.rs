use std::time::Duration;

use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Api, ListParams},
    Client,
};

use crate::kubernetes::models;
use crate::kubernetes::namespaces::format_age;

pub const DEPLOYMENT_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

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
}
