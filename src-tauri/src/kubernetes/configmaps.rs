use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{ConfigMap, Pod};
use std::collections::HashMap;
use crate::kubernetes::models::{ConfigMapInfo, UsedByPod};
use crate::kubernetes::{format_age, format_size};

/// Resolves ConfigMap usage across pods in all or specific namespaces.
pub fn resolve_configmap_pod_usage(pods: &[Pod]) -> HashMap<(String, String), Vec<UsedByPod>> {
    let mut usage_map: HashMap<(String, String), Vec<UsedByPod>> = HashMap::new();

    for pod in pods {
        let pod_name = match &pod.metadata.name {
            Some(n) if !n.is_empty() => n.clone(),
            _ => continue,
        };
        let pod_ns = pod.metadata.namespace.clone().unwrap_or_else(|| "default".to_string());
        let status = pod.status.as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut pod_cm_usage: HashMap<String, (bool, bool, bool)> = HashMap::new();

        if let Some(spec) = &pod.spec {
            // 1. Volumes & projected volumes
            if let Some(volumes) = &spec.volumes {
                for vol in volumes {
                    if let Some(cm) = &vol.config_map {
                        if !cm.name.is_empty() {
                            let entry = pod_cm_usage.entry(cm.name.clone()).or_insert((false, false, false));
                            entry.0 = true;
                        }
                    }
                    if let Some(proj) = &vol.projected {
                        if let Some(sources) = &proj.sources {
                            for source in sources {
                                if let Some(cm) = &source.config_map {
                                    if !cm.name.is_empty() {
                                        let entry = pod_cm_usage.entry(cm.name.clone()).or_insert((false, false, false));
                                        entry.0 = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Helper closure to scan container env & env_from
            let mut scan_container = |env: Option<&Vec<k8s_openapi::api::core::v1::EnvVar>>,
                                      env_from: Option<&Vec<k8s_openapi::api::core::v1::EnvFromSource>>| {
                if let Some(envs) = env {
                    for e in envs {
                        if let Some(vf) = &e.value_from {
                            if let Some(cm_ref) = &vf.config_map_key_ref {
                                if !cm_ref.name.is_empty() {
                                    let entry = pod_cm_usage.entry(cm_ref.name.clone()).or_insert((false, false, false));
                                    entry.1 = true;
                                }
                            }
                        }
                    }
                }
                if let Some(efs) = env_from {
                    for ef in efs {
                        if let Some(cm_ref) = &ef.config_map_ref {
                            if !cm_ref.name.is_empty() {
                                let entry = pod_cm_usage.entry(cm_ref.name.clone()).or_insert((false, false, false));
                                entry.2 = true;
                            }
                        }
                    }
                }
            };

            for c in &spec.containers {
                scan_container(c.env.as_ref(), c.env_from.as_ref());
            }
            if let Some(init_containers) = &spec.init_containers {
                for c in init_containers {
                    scan_container(c.env.as_ref(), c.env_from.as_ref());
                }
            }
            if let Some(ephemeral_containers) = &spec.ephemeral_containers {
                for c in ephemeral_containers {
                    scan_container(c.env.as_ref(), c.env_from.as_ref());
                }
            }
        }

        for (cm_name, (has_volume, has_env, has_env_from)) in pod_cm_usage {
            let mut parts = Vec::new();
            if has_volume {
                parts.push("Volume");
            }
            if has_env {
                parts.push("Env");
            }
            if has_env_from {
                parts.push("EnvFrom");
            }
            let mount_type = parts.join(", ");
            usage_map.entry((pod_ns.clone(), cm_name)).or_default().push(UsedByPod {
                name: pod_name.clone(),
                namespace: pod_ns.clone(),
                status: status.clone(),
                mount_type,
            });
        }
    }

    // Sort used_by for each configmap deterministically by pod name
    for list in usage_map.values_mut() {
        list.sort_by(|a, b| a.name.cmp(&b.name));
    }

    usage_map
}

/// Formats a Kubernetes `ConfigMap` into `ConfigMapInfo` with default empty usage.
pub fn format_configmap(c: &ConfigMap) -> ConfigMapInfo {
    let empty_map = HashMap::new();
    format_configmap_with_usage(c, &empty_map)
}

/// Formats a Kubernetes `ConfigMap` into `ConfigMapInfo` using resolved pod usage.
pub fn format_configmap_with_usage(c: &ConfigMap, usage_map: &HashMap<(String, String), Vec<UsedByPod>>) -> ConfigMapInfo {
    let name = c.metadata.name.clone().unwrap_or_default();
    let namespace = c.metadata.namespace.clone().unwrap_or_default();
    
    let age = format_age(&c.metadata.creation_timestamp);
    let created = c.metadata.creation_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();
        
    let resource_version = c.metadata.resource_version.clone().unwrap_or_default();
    let immutable = c.immutable.unwrap_or(false);
    
    let data = c.data.clone().unwrap_or_default();
    let keys_count = data.len() as i32;
    
    let mut total_bytes = 0;
    for (k, v) in &data {
        total_bytes += k.len() + v.len();
    }
    if let Some(binary) = &c.binary_data {
        for (k, v) in binary {
            total_bytes += k.len() + v.0.len();
        }
    }
    let size = format_size(total_bytes);
    
    let annotations = c.metadata.annotations.as_ref().map(|a| a.len() as i32).unwrap_or(0);
    let labels = c.metadata.labels.clone().unwrap_or_default();
    
    let used_by = usage_map.get(&(namespace.clone(), name.clone())).cloned().unwrap_or_default();
    let mounted_pods = used_by.len() as i32;
    
    ConfigMapInfo {
        name,
        namespace,
        labels,
        annotations,
        created,
        age,
        resource_version,
        immutable,
        keys_count,
        size,
        mounted_pods,
        used_by,
        data,
    }
}

/// Lists all ConfigMaps in the specified namespace or all namespaces, concurrently resolving pod usage.
pub async fn list_configmaps(client: &Client, namespace: Option<String>) -> Result<Vec<ConfigMapInfo>, kube::Error> {
    let cm_api: Api<ConfigMap> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let pod_api: Api<Pod> = if let Some(ref ns) = namespace {
        Api::namespaced(client.clone(), ns)
    } else {
        Api::all(client.clone())
    };

    let lp = ListParams::default();
    let (cm_res, pod_res) = tokio::join!(
        cm_api.list(&lp),
        pod_api.list(&lp)
    );

    let cms = cm_res?;
    let pods = pod_res.map(|l| l.items).unwrap_or_default();
    let usage_map = resolve_configmap_pod_usage(&pods);

    let mut list = Vec::new();
    for c in cms {
        list.push(format_configmap_with_usage(&c, &usage_map));
    }
    
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        ConfigMapEnvSource, ConfigMapKeySelector, ConfigMapProjection, ConfigMapVolumeSource,
        Container, EnvFromSource, EnvVar, EnvVarSource, PodSpec, PodStatus,
        ProjectedVolumeSource, Volume, VolumeProjection,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use k8s_openapi::ByteString;
    use std::collections::BTreeMap;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_configmap_basic() {
        let mut data = BTreeMap::new();
        data.insert("key1".to_string(), "val1".to_string());
        data.insert("key2".to_string(), "longer_value".to_string());

        let mut binary_data = BTreeMap::new();
        binary_data.insert("bin1".to_string(), ByteString(vec![0, 1, 2, 3]));

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "orbit".to_string());

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some("test-cm".to_string()),
                namespace: Some("default".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                resource_version: Some("42".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            data: Some(data),
            binary_data: Some(binary_data),
            immutable: Some(true),
        };

        let info = format_configmap(&cm);

        assert_eq!(info.name, "test-cm");
        assert_eq!(info.namespace, "default");
        assert_eq!(info.resource_version, "42");
        assert!(info.immutable);
        assert_eq!(info.keys_count, 2);
        assert_eq!(info.size, "32 B");
        assert_eq!(info.labels.get("app").map(|s| s.as_str()), Some("orbit"));
        assert_eq!(info.data.get("key1").map(|s| s.as_str()), Some("val1"));
    }

    #[test]
    fn test_used_by_pod_serialization() {
        let pod = UsedByPod {
            name: "test-pod".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            mount_type: "Volume, Env".to_string(),
        };
        let json = serde_json::to_string(&pod).unwrap();
        assert!(json.contains("\"mountType\":\"Volume, Env\""));
        assert!(json.contains("\"namespace\":\"default\""));
    }

    #[test]
    fn test_format_configmap_empty_pods() {
        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some("orphan-cm".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let info = format_configmap(&cm);
        assert_eq!(info.mounted_pods, 0);
        assert!(info.used_by.is_empty());
    }

    #[test]
    fn test_resolve_configmap_pod_usage_volume_and_env() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("app-pod".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            spec: Some(PodSpec {
                volumes: Some(vec![
                    Volume {
                        name: "vol-cm".to_string(),
                        config_map: Some(ConfigMapVolumeSource {
                            name: "my-cm".to_string(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    Volume {
                        name: "proj-vol".to_string(),
                        projected: Some(ProjectedVolumeSource {
                            sources: Some(vec![VolumeProjection {
                                config_map: Some(ConfigMapProjection {
                                    name: "proj-cm".to_string(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ]),
                containers: vec![
                    Container {
                        name: "main".to_string(),
                        env: Some(vec![EnvVar {
                            name: "CONFIG_KEY".to_string(),
                            value_from: Some(EnvVarSource {
                                config_map_key_ref: Some(ConfigMapKeySelector {
                                    name: "my-cm".to_string(),
                                    key: "key1".to_string(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        env_from: Some(vec![EnvFromSource {
                            config_map_ref: Some(ConfigMapEnvSource {
                                name: "envfrom-cm".to_string(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
        };

        let usage = resolve_configmap_pod_usage(&[pod]);
        
        // my-cm used via Volume and Env -> deduplicated to "Volume, Env"
        let my_cm_users = usage.get(&("default".to_string(), "my-cm".to_string())).expect("my-cm should be found");
        assert_eq!(my_cm_users.len(), 1);
        assert_eq!(my_cm_users[0].name, "app-pod");
        assert_eq!(my_cm_users[0].namespace, "default");
        assert_eq!(my_cm_users[0].status, "Running");
        assert_eq!(my_cm_users[0].mount_type, "Volume, Env");

        // proj-cm used via projected Volume
        let proj_users = usage.get(&("default".to_string(), "proj-cm".to_string())).expect("proj-cm should be found");
        assert_eq!(proj_users[0].mount_type, "Volume");

        // envfrom-cm used via EnvFrom
        let envfrom_users = usage.get(&("default".to_string(), "envfrom-cm".to_string())).expect("envfrom-cm should be found");
        assert_eq!(envfrom_users[0].mount_type, "EnvFrom");
    }
}
