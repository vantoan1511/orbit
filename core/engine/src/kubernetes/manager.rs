use kube::{Client, config::{Kubeconfig, KubeConfigOptions, Config}};
use crate::kubernetes::models::ClusterInfo;

pub struct KubeManager {
    pub kubeconfig: Option<Kubeconfig>,
    pub active_context: Option<String>,
    pub active_client: Option<Client>,
    pub watch_cancel: Option<tokio::sync::watch::Sender<bool>>,
}

impl KubeManager {
    pub async fn new() -> Self {
        let mut manager = Self {
            kubeconfig: None,
            active_context: None,
            active_client: None,
            watch_cancel: None,
        };
        
        // Try reading default kubeconfig
        if let Ok(config) = Kubeconfig::read() {
            let active_ctx = config.current_context.clone();
            manager.kubeconfig = Some(config);
            if let Some(ref ctx) = active_ctx {
                let _ = manager.switch_context(ctx).await;
            }
        }
        
        manager
    }

    pub fn get_clusters(&self) -> Vec<ClusterInfo> {
        let mut clusters = Vec::new();
        if let Some(ref config) = self.kubeconfig {
            for ctx in &config.contexts {
                let status = if Some(&ctx.name) == self.active_context.as_ref() {
                    "healthy".to_string()
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
            
        self.active_context = Some(context_name.to_string());
        self.active_client = Some(client);
        
        Ok(())
    }

    pub async fn add_kubeconfig_file(&mut self, file_path: &str) -> Result<(), String> {
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
