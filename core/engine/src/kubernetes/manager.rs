use std::collections::HashMap;
use kube::{Client, config::{Kubeconfig, KubeConfigOptions, Config}};
use crate::kubernetes::models::{ClusterInfo, UserProfileInfo};
use crate::config::OrbitConfig;


pub struct KubeManager {
    pub kubeconfig: Option<Kubeconfig>,
    pub active_context: Option<String>,
    pub active_client: Option<Client>,
    pub watch_cancel: Option<tokio::sync::watch::Sender<bool>>,
    pub active_context_healthy: bool,
    pub log_cancel: Vec<tokio::sync::oneshot::Sender<()>>,
    pub port_forward_cancel: HashMap<String, (tokio::sync::oneshot::Sender<()>, tokio::task::JoinHandle<()>)>,
    pub config: OrbitConfig,
}

impl KubeManager {
    pub async fn new() -> Self {
        let config = OrbitConfig::load();

        let mut manager = Self {
            kubeconfig: None,
            active_context: None,
            active_client: None,
            watch_cancel: None,
            active_context_healthy: false,
            log_cancel: Vec::new(),
            port_forward_cancel: HashMap::new(),
            config,
        };
        
        // Try reading default kubeconfig
        if let Ok(config) = Kubeconfig::read() {
            manager.kubeconfig = Some(config);
            // Intentionally do not connect — user must explicitly select a cluster
        }
        
        // Reload persisted custom kubeconfig files
        for path in manager.config.custom_kubeconfig_paths.clone() {
            if let Err(e) = manager.merge_kubeconfig_from_path(&path) {
                log::warn!("Failed to load persisted kubeconfig from {}: {}", path, e);
            }
        }

        manager
    }

    pub fn get_clusters(&self) -> Vec<ClusterInfo> {
        let mut clusters = Vec::new();
        if let Some(ref config) = self.kubeconfig {
            for ctx in &config.contexts {
                let status = if Some(&ctx.name) == self.active_context.as_ref() {
                    if self.active_context_healthy {
                        "healthy".to_string()
                    } else {
                        "offline".to_string()
                    }
                } else {
                    "offline".to_string()
                };
                clusters.push(ClusterInfo {
                    id: ctx.name.clone(),
                    name: ctx.name.clone(),
                    status,
                });
            }
        }
        clusters
    }

    pub fn get_user_profile(&self) -> UserProfileInfo {
        let active_context = self.active_context.clone();
        let mut user_name = None;
        let mut cluster_name = None;
        let mut auth_type = "None".to_string();
        let mut server_url = None;

        if let (Some(config), Some(ctx_name)) = (&self.kubeconfig, &active_context) {
            let named_ctx = config.contexts.iter().find(|c| &c.name == ctx_name);
            let ctx = named_ctx.and_then(|c| c.context.as_ref());

            if let Some(ctx) = ctx {
                user_name = ctx.user.clone();
                cluster_name = Some(ctx.cluster.clone());

                if let Some(u_name) = &user_name {
                    let named_auth = config.auth_infos.iter().find(|a| &a.name == u_name);
                    let auth = named_auth.and_then(|a| a.auth_info.as_ref());
                    if let Some(auth) = auth {
                        if auth.client_certificate.is_some() || auth.client_certificate_data.is_some() {
                            auth_type = "Certificate".to_string();
                        } else if auth.token.is_some() || auth.token_file.is_some() {
                            auth_type = "Token".to_string();
                        } else if auth.exec.is_some() {
                            auth_type = "Exec Plugin".to_string();
                        } else if auth.auth_provider.is_some() {
                            auth_type = "OIDC".to_string();
                        } else if auth.username.is_some() {
                            auth_type = "Basic Auth".to_string();
                        } else {
                            auth_type = "Configured".to_string();
                        }
                    }
                }

                if let Some(c_name) = &cluster_name {
                    let named_cl = config.clusters.iter().find(|cl| &cl.name == c_name);
                    let cl = named_cl.and_then(|cl| cl.cluster.as_ref());
                    if let Some(cl) = cl {
                        server_url = cl.server.clone();
                    }
                }
            }
        }

        let mut kubeconfig_paths = Vec::new();
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            let mut default_path = std::path::PathBuf::from(home);
            default_path.push(".kube");
            default_path.push("config");
            if default_path.exists() {
                let norm = OrbitConfig::normalize_path(&default_path.to_string_lossy());
                kubeconfig_paths.push(norm);
            }
        }
        for path in &self.config.custom_kubeconfig_paths {
            let norm = OrbitConfig::normalize_path(path);
            let is_duplicate = if cfg!(windows) {
                kubeconfig_paths.iter().any(|p| p.eq_ignore_ascii_case(&norm))
            } else {
                kubeconfig_paths.iter().any(|p| p == &norm)
            };
            if !is_duplicate {
                kubeconfig_paths.push(norm);
            }
        }

        let k8s_version = if self.active_context.is_some() {
            if self.active_context_healthy {
                Some("Connected".to_string())
            } else {
                Some("Offline".to_string())
            }
        } else {
            None
        };

        UserProfileInfo {
            active_context,
            user_name,
            auth_type,
            cluster_name,
            server_url,
            kubeconfig_paths,
            k8s_version,
        }
    }

    pub async fn refresh_active_cluster_health(&mut self) {
        if let Some(ref client) = self.active_client {
            self.active_context_healthy = client.apiserver_version().await.is_ok();
        } else {
            self.active_context_healthy = false;
        }
    }

    pub async fn switch_context(&mut self, context_name: &str) -> Result<(), String> {
        let config = self.kubeconfig.as_ref().ok_or_else(|| "No kubeconfig loaded".to_string())?;
        
        let options = KubeConfigOptions {
            context: Some(context_name.to_string()),
            ..Default::default()
        };
        
        let config = Config::from_custom_kubeconfig(config.clone(), &options)
            .await
            .map_err(|e| format!("Failed to create client config: {}", e))?;
            
        let client = Client::try_from(config)
            .map_err(|e| format!("Failed to build client: {}", e))?;
            
        // Test connection
        let healthy = client.apiserver_version().await.is_ok();
        
        self.active_context = Some(context_name.to_string());
        self.active_client = Some(client);
        self.active_context_healthy = healthy;
        
        Ok(())
    }

    /// Read a kubeconfig from `file_path` and merge its contexts, clusters, and auth_infos into `self.kubeconfig`.
    /// Does not auto-switch context and does not modify persistent configuration.
    pub fn merge_kubeconfig_from_path(&mut self, file_path: &str) -> Result<(), String> {
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return Err("Provided kubeconfig path does not exist".to_string());
        }
        if !path.is_file() {
            return Err("Provided kubeconfig path is not a file".to_string());
        }

        let new_config = Kubeconfig::read_from(file_path)
            .map_err(|e| format!("Failed to read custom kubeconfig: {}", e))?;
            
        if let Some(ref mut current) = self.kubeconfig {
            // Merge clusters, contexts, and auth infos (users)
            for ctx in new_config.contexts {
                if !current.contexts.iter().any(|c| c.name == ctx.name) {
                    current.contexts.push(ctx);
                }
            }
            for cl in new_config.clusters {
                if !current.clusters.iter().any(|c| c.name == cl.name) {
                    current.clusters.push(cl);
                }
            }
            for user in new_config.auth_infos {
                if !current.auth_infos.iter().any(|u| u.name == user.name) {
                    current.auth_infos.push(user);
                }
            }
        } else {
            self.kubeconfig = Some(new_config);
        }

        Ok(())
    }

    pub async fn add_kubeconfig_file(&mut self, file_path: &str) -> Result<(), String> {
        self.merge_kubeconfig_from_path(file_path)?;

        if let Err(e) = self.config.add_kubeconfig_path(file_path) {
            log::warn!("Failed to persist kubeconfig path {}: {}", file_path, e);
        }
        
        // Try to switch to the new config's current context if none is active
        if self.active_context.is_none() {
            let next_ctx = if let Some(ref config) = self.kubeconfig {
                config.current_context.clone().or_else(|| config.contexts.first().map(|c| c.name.clone()))
            } else {
                None
            };
            if let Some(ctx) = next_ctx {
                let _ = self.switch_context(&ctx).await;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_kubeconfig_nonexistent_file() {
        let mut manager = KubeManager {
            kubeconfig: None,
            active_context: None,
            active_client: None,
            watch_cancel: None,
            active_context_healthy: false,
            log_cancel: Vec::new(),
            port_forward_cancel: HashMap::new(),
            config: OrbitConfig::default(),
        };

        let result = manager.merge_kubeconfig_from_path("/nonexistent/path/to/kubeconfig");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Provided kubeconfig path does not exist");
    }

    #[test]
    fn test_merge_kubeconfig_from_temp_file() {
        let mut manager = KubeManager {
            kubeconfig: None,
            active_context: None,
            active_client: None,
            watch_cancel: None,
            active_context_healthy: false,
            log_cancel: Vec::new(),
            port_forward_cancel: HashMap::new(),
            config: OrbitConfig::default(),
        };

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("orbit_test_kubeconfig.yaml");
        let sample_yaml = r#"
apiVersion: v1
clusters:
- cluster:
    server: https://127.0.0.1:6443
  name: test-cluster
contexts:
- context:
    cluster: test-cluster
    user: test-user
  name: test-context
current-context: test-context
kind: Config
preferences: {}
users:
- name: test-user
  user: {}
"#;
        std::fs::write(&file_path, sample_yaml).unwrap();

        let result = manager.merge_kubeconfig_from_path(file_path.to_str().unwrap());
        assert!(result.is_ok());

        let clusters = manager.get_clusters();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].name, "test-context");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_get_user_profile_default() {
        let manager = KubeManager {
            kubeconfig: None,
            active_context: None,
            active_client: None,
            watch_cancel: None,
            active_context_healthy: false,
            log_cancel: Vec::new(),
            port_forward_cancel: HashMap::new(),
            config: OrbitConfig::default(),
        };

        let profile = manager.get_user_profile();
        assert_eq!(profile.active_context, None);
        assert_eq!(profile.user_name, None);
        assert_eq!(profile.auth_type, "None");
        assert_eq!(profile.cluster_name, None);
        assert_eq!(profile.server_url, None);
    }
}

