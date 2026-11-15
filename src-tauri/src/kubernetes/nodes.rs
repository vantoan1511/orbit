use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    Client,
};
use k8s_openapi::api::core::v1::{Node, Pod};
pub use orbit_core::kubernetes::nodes::{
    list_nodes, map_node, parse_cpu_quantity, parse_memory_quantity,
};

/// Builds the merge patch JSON for changing a node's schedulable (cordoned) status.
pub fn build_node_schedulable_patch(unschedulable: bool) -> serde_json::Value {
    serde_json::json!({
        "spec": {
            "unschedulable": unschedulable
        }
    })
}

/// Updates the `unschedulable` field of a Kubernetes node via Merge Patch.
pub async fn set_node_schedulable(client: &Client, name: &str, unschedulable: bool) -> Result<(), kube::Error> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let patch = build_node_schedulable_patch(unschedulable);
    nodes_api
        .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

/// Determines whether a Pod should be evicted during a Node drain operation.
/// Excludes DaemonSet-managed pods, mirror pods, and pods in Succeeded/Failed phases.
pub fn is_pod_evictable(pod: &Pod) -> bool {
    // 1. Exclude terminal pods (Succeeded or Failed)
    if let Some(status) = &pod.status {
        if let Some(phase) = &status.phase {
            if phase == "Succeeded" || phase == "Failed" {
                return false;
            }
        }
    }

    // 2. Exclude mirror pods with kubernetes.io/config.mirror annotation
    if let Some(annotations) = &pod.metadata.annotations {
        if annotations.contains_key("kubernetes.io/config.mirror") {
            return false;
        }
    }

    // 3. Exclude DaemonSet-owned pods
    if let Some(owners) = &pod.metadata.owner_references {
        for owner in owners {
            if owner.kind == "DaemonSet" {
                return false;
            }
        }
    }

    true
}

/// Cordons the target node and evicts all eligible scheduled pods.
pub async fn drain_node(client: &Client, name: &str) -> Result<usize, kube::Error> {
    // Step 1: Cordon node first to prevent new pods from scheduling
    set_node_schedulable(client, name, true).await?;

    // Step 2: List all pods currently bound to this node
    let pods_api: Api<Pod> = Api::all(client.clone());
    let list_params = ListParams::default().fields(&format!("spec.nodeName={}", name));
    let pods = pods_api.list(&list_params).await?;

    let mut evicted_count = 0;
    for pod in pods.items {
        if !is_pod_evictable(&pod) {
            continue;
        }

        let pod_name = pod.metadata.name.as_deref().unwrap_or_default();
        let namespace = pod.metadata.namespace.as_deref().unwrap_or("default");
        if pod_name.is_empty() {
            continue;
        }

        let namespaced_pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
        let eviction_payload = serde_json::json!({
            "apiVersion": "policy/v1",
            "kind": "Eviction",
            "metadata": {
                "name": pod_name,
                "namespace": namespace
            }
        });

        let eviction_bytes = match serde_json::to_vec(&eviction_payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to serialize eviction payload");
                continue;
            }
        };

        match namespaced_pods.create_subresource::<serde_json::Value>("eviction", pod_name, &kube::api::PostParams::default(), eviction_bytes).await {
            Ok(_) => {
                evicted_count += 1;
            }
            Err(e) => {
                tracing::warn!(pod = %pod_name, namespace = %namespace, error = %e, "Failed to evict pod during node drain");
            }
        }
    }

    Ok(evicted_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_quantity() {
        assert_eq!(parse_cpu_quantity("2"), 2.0);
        assert_eq!(parse_cpu_quantity("250m"), 0.25);
        assert_eq!(parse_cpu_quantity("500u"), 0.0005);
        assert_eq!(parse_cpu_quantity("3141592n"), 0.003141592);
        assert_eq!(parse_cpu_quantity("0"), 0.0);
        assert_eq!(parse_cpu_quantity(""), 0.0);
        assert_eq!(parse_cpu_quantity("invalid"), 0.0);
    }

    #[test]
    fn test_parse_memory_quantity() {
        assert_eq!(parse_memory_quantity("1Gi"), 1.0);
        assert_eq!(parse_memory_quantity("1024Mi"), 1.0);
        assert_eq!(parse_memory_quantity("1048576Ki"), 1.0);
        assert_eq!(parse_memory_quantity(""), 0.0);
    }

    #[test]
    fn test_map_node_extracts_status_role_and_cordoned_state() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        use k8s_openapi::api::core::v1::{NodeCondition, NodeSpec, NodeStatus};
        use std::collections::BTreeMap;

        let mut labels = BTreeMap::new();
        labels.insert("node-role.kubernetes.io/control-plane".to_string(), "".to_string());

        let node1 = Node {
            metadata: ObjectMeta {
                name: Some("node-1".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(NodeSpec {
                unschedulable: Some(true),
                ..Default::default()
            }),
            status: Some(NodeStatus {
                conditions: Some(vec![NodeCondition {
                    type_: "Ready".to_string(),
                    status: "True".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };

        let mapped1 = map_node(&node1);
        assert_eq!(mapped1.name, "node-1");
        assert_eq!(mapped1.status, "Ready");
        assert_eq!(mapped1.role, "control-plane");
        assert!(mapped1.is_cordoned);

        let node2 = Node {
            metadata: ObjectMeta {
                name: Some("node-2".to_string()),
                ..Default::default()
            },
            spec: Some(NodeSpec {
                unschedulable: None,
                ..Default::default()
            }),
            status: Some(NodeStatus {
                conditions: Some(vec![NodeCondition {
                    type_: "Ready".to_string(),
                    status: "False".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };

        let mapped2 = map_node(&node2);
        assert_eq!(mapped2.name, "node-2");
        assert_eq!(mapped2.status, "NotReady");
        assert_eq!(mapped2.role, "worker");
        assert!(!mapped2.is_cordoned);
    }

    #[test]
    fn test_build_node_schedulable_patch() {
        let patch_cordon = build_node_schedulable_patch(true);
        assert_eq!(patch_cordon, serde_json::json!({ "spec": { "unschedulable": true } }));

        let patch_uncordon = build_node_schedulable_patch(false);
        assert_eq!(patch_uncordon, serde_json::json!({ "spec": { "unschedulable": false } }));
    }

    #[test]
    fn test_is_pod_evictable_excludes_daemonset_mirror_and_terminal_pods() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
        use k8s_openapi::api::core::v1::{Pod, PodStatus};
        use std::collections::BTreeMap;

        // Normal running pod is evictable
        let normal_pod = Pod {
            metadata: ObjectMeta {
                name: Some("normal-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(is_pod_evictable(&normal_pod));

        // DaemonSet owned pod is NOT evictable
        let daemonset_pod = Pod {
            metadata: ObjectMeta {
                name: Some("ds-pod".to_string()),
                namespace: Some("kube-system".to_string()),
                owner_references: Some(vec![OwnerReference {
                    api_version: "apps/v1".to_string(),
                    kind: "DaemonSet".to_string(),
                    name: "kube-proxy".to_string(),
                    uid: "123".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&daemonset_pod));

        // Mirror pod (kubernetes.io/config.mirror) is NOT evictable
        let mut annotations = BTreeMap::new();
        annotations.insert("kubernetes.io/config.mirror".to_string(), "mirror-hash".to_string());
        let mirror_pod = Pod {
            metadata: ObjectMeta {
                name: Some("mirror-pod".to_string()),
                namespace: Some("kube-system".to_string()),
                annotations: Some(annotations),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&mirror_pod));

        // Succeeded pod is NOT evictable
        let succeeded_pod = Pod {
            metadata: ObjectMeta {
                name: Some("succeeded-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Succeeded".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&succeeded_pod));

        // Failed pod is NOT evictable
        let failed_pod = Pod {
            metadata: ObjectMeta {
                name: Some("failed-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Failed".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(!is_pod_evictable(&failed_pod));
    }
}
