use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Pod, Secret};
use std::collections::HashMap;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use crate::kubernetes::models::{SecretInfo, UsedByPod};
use crate::kubernetes::{format_age, format_size};

/// Resolves Secret usage across pods in all or specific namespaces.
pub fn resolve_secret_pod_usage(pods: &[Pod]) -> HashMap<(String, String), Vec<UsedByPod>> {
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

        let mut pod_sec_usage: HashMap<String, (bool, bool, bool, bool)> = HashMap::new();

        if let Some(spec) = &pod.spec {
            // 1. Volumes & projected volumes
            if let Some(volumes) = &spec.volumes {
                for vol in volumes {
                    if let Some(sec) = &vol.secret
                        && let Some(name) = &sec.secret_name
                        && !name.is_empty()
                    {
                        let entry = pod_sec_usage.entry(name.clone()).or_insert((false, false, false, false));
                        entry.0 = true;
                    }
                    if let Some(proj) = &vol.projected
                        && let Some(sources) = &proj.sources
                    {
                        for source in sources {
                            if let Some(sec) = &source.secret
                                && !sec.name.is_empty()
                            {
                                let entry = pod_sec_usage.entry(sec.name.clone()).or_insert((false, false, false, false));
                                entry.0 = true;
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
                        if let Some(vf) = &e.value_from
                            && let Some(sec_ref) = &vf.secret_key_ref
                            && !sec_ref.name.is_empty()
                        {
                            let entry = pod_sec_usage.entry(sec_ref.name.clone()).or_insert((false, false, false, false));
                            entry.1 = true;
                        }
                    }
                }
                if let Some(efs) = env_from {
                    for ef in efs {
                        if let Some(sec_ref) = &ef.secret_ref
                            && !sec_ref.name.is_empty()
                        {
                            let entry = pod_sec_usage.entry(sec_ref.name.clone()).or_insert((false, false, false, false));
                            entry.2 = true;
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

            // 3. Image pull secrets
            if let Some(image_pull_secrets) = &spec.image_pull_secrets {
                for ips in image_pull_secrets {
                    if !ips.name.is_empty() {
                        let entry = pod_sec_usage.entry(ips.name.clone()).or_insert((false, false, false, false));
                        entry.3 = true;
                    }
                }
            }
        }

        for (sec_name, (has_volume, has_env, has_env_from, has_image_pull_secret)) in pod_sec_usage {
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
            if has_image_pull_secret {
                parts.push("ImagePullSecret");
            }
            let mount_type = parts.join(", ");
            usage_map.entry((pod_ns.clone(), sec_name)).or_default().push(UsedByPod {
                name: pod_name.clone(),
                namespace: pod_ns.clone(),
                status: status.clone(),
                mount_type,
            });
        }
    }

    // Sort used_by for each secret deterministically by pod name
    for list in usage_map.values_mut() {
        list.sort_by(|a, b| a.name.cmp(&b.name));
    }

    usage_map
}

/// Formats a Kubernetes `Secret` into `SecretInfo` with default empty usage.
pub fn format_secret(s: &Secret) -> SecretInfo {
    let empty_map = HashMap::new();
    format_secret_with_usage(s, &empty_map)
}

/// Formats a Kubernetes `Secret` into `SecretInfo` using resolved pod usage.
pub fn format_secret_with_usage(s: &Secret, usage_map: &HashMap<(String, String), Vec<UsedByPod>>) -> SecretInfo {
    let name = s.metadata.name.clone().unwrap_or_default();
    let namespace = s.metadata.namespace.clone().unwrap_or_default();
    
    let age = format_age(&s.metadata.creation_timestamp);
    let created = s.metadata.creation_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();
        
    let resource_version = s.metadata.resource_version.clone().unwrap_or_default();
    let immutable = s.immutable.unwrap_or(false);
    let secret_type = s.type_.clone().unwrap_or_else(|| "Opaque".to_string());
    
    let raw_data = s.data.clone().unwrap_or_default();
    let keys_count = raw_data.len() as i32;
    
    let mut total_bytes = 0;
    let mut data = std::collections::BTreeMap::new();
    for (k, v) in &raw_data {
        total_bytes += k.len() + v.0.len();
        let base64_str = BASE64_STANDARD.encode(&v.0);
        data.insert(k.clone(), base64_str);
    }
    let size = format_size(total_bytes);
    
    let annotations = s.metadata.annotations.as_ref().map(|a| a.len() as i32).unwrap_or(0);
    let labels = s.metadata.labels.clone().unwrap_or_default();
    
    let used_by = usage_map.get(&(namespace.clone(), name.clone())).cloned().unwrap_or_default();
    let mounted_pods = used_by.len() as i32;
    
    SecretInfo {
        name,
        namespace,
        labels,
        annotations,
        r#type: secret_type,
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

/// Lists all Secrets in the specified namespace or all namespaces, concurrently resolving pod usage.
pub async fn list_secrets(client: &Client, namespace: Option<String>) -> Result<Vec<SecretInfo>, kube::Error> {
    let sec_api: Api<Secret> = if let Some(ref ns) = namespace {
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
    let (sec_res, pod_res) = tokio::join!(
        sec_api.list(&lp),
        pod_api.list(&lp)
    );

    let secrets = sec_res?;
    let pods = pod_res.map(|l| l.items).unwrap_or_default();
    let usage_map = resolve_secret_pod_usage(&pods);

    let mut list = Vec::new();
    for s in secrets {
        list.push(format_secret_with_usage(&s, &usage_map));
    }
    
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        Container, EnvFromSource, EnvVar, EnvVarSource, LocalObjectReference,
        PodSpec, PodStatus, SecretEnvSource, SecretKeySelector, SecretVolumeSource, Volume,
    };
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use k8s_openapi::ByteString;
    use std::collections::BTreeMap;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_secret_basic() {
        let mut raw_data = BTreeMap::new();
        raw_data.insert("username".to_string(), ByteString(b"admin".to_vec()));
        raw_data.insert("password".to_string(), ByteString(b"secret123".to_vec()));

        let mut labels = BTreeMap::new();
        labels.insert("type".to_string(), "credential".to_string());

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some("test-secret".to_string()),
                namespace: Some("default".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                resource_version: Some("101".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            type_: Some("kubernetes.io/basic-auth".to_string()),
            data: Some(raw_data),
            immutable: Some(false),
            ..Default::default()
        };

        let info = format_secret(&secret);

        assert_eq!(info.name, "test-secret");
        assert_eq!(info.namespace, "default");
        assert_eq!(info.r#type, "kubernetes.io/basic-auth");
        assert_eq!(info.resource_version, "101");
        assert!(!info.immutable);
        assert_eq!(info.keys_count, 2);
        assert_eq!(info.size, "30 B");
        assert_eq!(info.labels.get("type").map(|s| s.as_str()), Some("credential"));
        
        let expected_user_b64 = BASE64_STANDARD.encode(b"admin");
        assert_eq!(info.data.get("username"), Some(&expected_user_b64));
    }

    #[test]
    fn test_resolve_secret_pod_usage_image_pull_secrets() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("secret-pod".to_string()),
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
                        name: "sec-vol".to_string(),
                        secret: Some(SecretVolumeSource {
                            secret_name: Some("my-secret".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ]),
                containers: vec![
                    Container {
                        name: "worker".to_string(),
                        env: Some(vec![EnvVar {
                            name: "PASS".to_string(),
                            value_from: Some(EnvVarSource {
                                secret_key_ref: Some(SecretKeySelector {
                                    name: "my-secret".to_string(),
                                    key: "pass".to_string(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        env_from: Some(vec![EnvFromSource {
                            secret_ref: Some(SecretEnvSource {
                                name: "envfrom-sec".to_string(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                ],
                image_pull_secrets: Some(vec![
                    LocalObjectReference {
                        name: "reg-cred".to_string(),
                    },
                ]),
                ..Default::default()
            }),
        };

        let usage = resolve_secret_pod_usage(&[pod]);

        // my-secret used via Volume and Env -> "Volume, Env"
        let my_sec_users = usage.get(&("default".to_string(), "my-secret".to_string())).expect("my-secret should be found");
        assert_eq!(my_sec_users.len(), 1);
        assert_eq!(my_sec_users[0].name, "secret-pod");
        assert_eq!(my_sec_users[0].namespace, "default");
        assert_eq!(my_sec_users[0].status, "Running");
        assert_eq!(my_sec_users[0].mount_type, "Volume, Env");

        // envfrom-sec used via EnvFrom
        let envfrom_users = usage.get(&("default".to_string(), "envfrom-sec".to_string())).expect("envfrom-sec should be found");
        assert_eq!(envfrom_users[0].mount_type, "EnvFrom");

        // reg-cred used via ImagePullSecret
        let reg_users = usage.get(&("default".to_string(), "reg-cred".to_string())).expect("reg-cred should be found");
        assert_eq!(reg_users[0].mount_type, "ImagePullSecret");
    }
}
