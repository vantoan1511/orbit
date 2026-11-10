use std::collections::HashMap;
use orbit_core::events::{TypedResourceBatch, WatchAction};
use orbit_core::kubernetes::metrics::PodMetricInfo;
use orbit_core::kubernetes::models::*;

pub const MAX_EVENTS_RETAINED: usize = 500;

#[derive(Debug, Clone, Default)]
pub struct ClusterStore {
    pub clusters: Vec<ClusterInfo>,
    pub active_cluster_id: Option<String>,
    pub user_profile: Option<UserProfileInfo>,

    pub namespaces: Vec<NamespaceInfo>,
    pub pods: Vec<PodInfo>,
    pub deployments: Vec<DeploymentInfo>,
    pub stateful_sets: Vec<StatefulSetInfo>,
    pub daemon_sets: Vec<DaemonSetInfo>,
    pub replica_sets: Vec<ReplicaSetInfo>,
    pub jobs: Vec<JobInfo>,
    pub cron_jobs: Vec<CronJobInfo>,
    pub hpas: Vec<HpaInfo>,
    pub nodes: Vec<NodeInfo>,
    pub services: Vec<ServiceInfo>,
    pub ingresses: Vec<IngressInfo>,
    pub config_maps: Vec<ConfigMapInfo>,
    pub secrets: Vec<SecretInfo>,
    pub events: Vec<EventInfo>,
    pub persistent_volumes: Vec<PersistentVolumeInfo>,
    pub persistent_volume_claims: Vec<PersistentVolumeClaimInfo>,
    pub storage_classes: Vec<StorageClassInfo>,
    pub policies: Vec<PolicyInfo>,

    pub pod_metrics: HashMap<String, (String, String)>,
}

impl ClusterStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_cluster_resources(&mut self) {
        self.namespaces.clear();
        self.pods.clear();
        self.deployments.clear();
        self.stateful_sets.clear();
        self.daemon_sets.clear();
        self.replica_sets.clear();
        self.jobs.clear();
        self.cron_jobs.clear();
        self.hpas.clear();
        self.nodes.clear();
        self.services.clear();
        self.ingresses.clear();
        self.config_maps.clear();
        self.secrets.clear();
        self.events.clear();
        self.persistent_volumes.clear();
        self.persistent_volume_claims.clear();
        self.storage_classes.clear();
        self.policies.clear();
        self.pod_metrics.clear();
    }

    pub fn apply_pod_metrics(&mut self, metrics: Vec<PodMetricInfo>) {
        for m in metrics {
            let key = format!("{}/{}", m.namespace, m.name);
            self.pod_metrics.insert(key.clone(), (m.cpu.clone(), m.memory.clone()));

            if let Some(pod) = self.pods.iter_mut().find(|p| p.namespace == m.namespace && p.name == m.name) {
                pod.cpu = Some(m.cpu);
                pod.memory = Some(m.memory);
            }
        }
    }

    pub fn apply_batch(&mut self, batch: TypedResourceBatch) {
        match batch {
            TypedResourceBatch::Pods(actions) => {
                for action in actions {
                    match action {
                        WatchAction::Applied(mut pod) => {
                            let key = format!("{}/{}", pod.namespace, pod.name);
                            if let Some((cpu, mem)) = self.pod_metrics.get(&key) {
                                pod.cpu = Some(cpu.clone());
                                pod.memory = Some(mem.clone());
                            }
                            if let Some(idx) = self.pods.iter().position(|p| p.namespace == pod.namespace && p.name == pod.name) {
                                self.pods[idx] = pod;
                            } else {
                                self.pods.push(pod);
                            }
                        }
                        WatchAction::Deleted(pod) => {
                            self.pods.retain(|p| !(p.namespace == pod.namespace && p.name == pod.name));
                        }
                    }
                }
            }
            TypedResourceBatch::Deployments(actions) => {
                apply_namespaced(&mut self.deployments, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::StatefulSets(actions) => {
                apply_namespaced(&mut self.stateful_sets, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::DaemonSets(actions) => {
                apply_namespaced(&mut self.daemon_sets, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::ReplicaSets(actions) => {
                apply_namespaced(&mut self.replica_sets, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::Jobs(actions) => {
                apply_namespaced(&mut self.jobs, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::CronJobs(actions) => {
                apply_namespaced(&mut self.cron_jobs, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::Hpas(actions) => {
                apply_namespaced(&mut self.hpas, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::Namespaces(actions) => {
                apply_namespaced(&mut self.namespaces, actions, |a, b| a.name == b.name);
            }
            TypedResourceBatch::Nodes(actions) => {
                apply_namespaced(&mut self.nodes, actions, |a, b| a.name == b.name);
            }
            TypedResourceBatch::Services(actions) => {
                apply_namespaced(&mut self.services, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::Ingresses(actions) => {
                apply_namespaced(&mut self.ingresses, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::ConfigMaps(actions) => {
                apply_namespaced(&mut self.config_maps, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::Secrets(actions) => {
                apply_namespaced(&mut self.secrets, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::Events(actions) => {
                for action in actions {
                    match action {
                        WatchAction::Applied(event) => {
                            if let Some(idx) = self.events.iter().position(|e| e.uid == event.uid) {
                                self.events[idx] = event;
                            } else {
                                self.events.insert(0, event);
                            }
                        }
                        WatchAction::Deleted(event) => {
                            self.events.retain(|e| e.uid != event.uid);
                        }
                    }
                }
                if self.events.len() > MAX_EVENTS_RETAINED {
                    self.events.truncate(MAX_EVENTS_RETAINED);
                }
            }
            TypedResourceBatch::PersistentVolumes(actions) => {
                apply_namespaced(&mut self.persistent_volumes, actions, |a, b| a.name == b.name);
            }
            TypedResourceBatch::PersistentVolumeClaims(actions) => {
                apply_namespaced(&mut self.persistent_volume_claims, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
            TypedResourceBatch::StorageClasses(actions) => {
                apply_namespaced(&mut self.storage_classes, actions, |a, b| a.name == b.name);
            }
            TypedResourceBatch::Policies(actions) => {
                apply_namespaced(&mut self.policies, actions, |a, b| a.namespace == b.namespace && a.name == b.name);
            }
        }
    }
}

fn apply_namespaced<T, F>(list: &mut Vec<T>, actions: Vec<WatchAction<T>>, match_fn: F)
where
    F: Fn(&T, &T) -> bool,
{
    for action in actions {
        match action {
            WatchAction::Applied(item) => {
                if let Some(idx) = list.iter().position(|existing| match_fn(existing, &item)) {
                    list[idx] = item;
                } else {
                    list.push(item);
                }
            }
            WatchAction::Deleted(item) => {
                list.retain(|existing| !match_fn(existing, &item));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_pod_batch_upserts_and_deletes_by_namespace_and_name() {
        let mut store = ClusterStore::new();

        let pod_v1 = PodInfo {
            name: "web-pod".to_string(),
            namespace: "prod".to_string(),
            status: "Pending".to_string(),
            ..Default::default()
        };

        let pod_v2 = PodInfo {
            name: "web-pod".to_string(),
            namespace: "prod".to_string(),
            status: "Running".to_string(),
            ..Default::default()
        };

        let other_pod = PodInfo {
            name: "db-pod".to_string(),
            namespace: "prod".to_string(),
            status: "Running".to_string(),
            ..Default::default()
        };

        // Insert pod_v1 and other_pod
        store.apply_batch(TypedResourceBatch::Pods(vec![
            WatchAction::Applied(pod_v1),
            WatchAction::Applied(other_pod.clone()),
        ]));
        assert_eq!(store.pods.len(), 2);
        assert_eq!(store.pods[0].status, "Pending");

        // Update pod_v1 with pod_v2
        store.apply_batch(TypedResourceBatch::Pods(vec![
            WatchAction::Applied(pod_v2.clone()),
        ]));
        assert_eq!(store.pods.len(), 2);
        let updated = store.pods.iter().find(|p| p.name == "web-pod").unwrap();
        assert_eq!(updated.status, "Running");

        // Delete pod_v2
        store.apply_batch(TypedResourceBatch::Pods(vec![
            WatchAction::Deleted(pod_v2),
        ]));
        assert_eq!(store.pods.len(), 1);
        assert_eq!(store.pods[0].name, "db-pod");
    }

    #[test]
    fn test_apply_pod_metrics_updates_cpu_and_memory_in_place() {
        let mut store = ClusterStore::new();
        let pod = PodInfo {
            name: "api-gateway".to_string(),
            namespace: "default".to_string(),
            cpu: Some("0m".to_string()),
            memory: Some("0Mi".to_string()),
            ..Default::default()
        };
        store.apply_batch(TypedResourceBatch::Pods(vec![WatchAction::Applied(pod)]));

        let metrics = vec![PodMetricInfo {
            name: "api-gateway".to_string(),
            namespace: "default".to_string(),
            cpu: "250m".to_string(),
            memory: "128Mi".to_string(),
            cpu_cores: Some(0.25),
            memory_bytes: Some(134217728.0),
        }];

        store.apply_pod_metrics(metrics);
        assert_eq!(store.pods[0].cpu.as_deref(), Some("250m"));
        assert_eq!(store.pods[0].memory.as_deref(), Some("128Mi"));
    }

    #[test]
    fn test_events_retention_capped_at_500_sorted_newest_first() {
        let mut store = ClusterStore::new();
        let events: Vec<WatchAction<EventInfo>> = (0..600)
            .map(|i| {
                WatchAction::Applied(EventInfo {
                    uid: format!("event-{}", i),
                    namespace: "default".to_string(),
                    message: format!("Test event {}", i),
                    ..Default::default()
                })
            })
            .collect();

        store.apply_batch(TypedResourceBatch::Events(events));
        assert_eq!(store.events.len(), MAX_EVENTS_RETAINED);
        assert_eq!(store.events[0].uid, "event-599");
    }
}
