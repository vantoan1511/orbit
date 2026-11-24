use std::collections::HashMap;
use http::Uri;
use kube::{Client, config::{Kubeconfig, KubeConfigOptions, Config}};
use serde::{Deserialize, Serialize};
use crate::kubernetes::kubeconfig_import::{
    AddClusterOutcome, AddClusterRequest, AddClusterSource, filter_kubeconfig_yaml,
    parse_kubeconfig_contexts,
};
use crate::kubernetes::models::{ClusterInfo, UserProfileInfo};
use crate::config::OrbitConfig;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSwitchOutcome {
    pub context_name: String,
    pub healthy: bool,
    pub k8s_version: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterDisconnectOutcome {
    pub previous_context: Option<String>,
}

pub struct KubeManager {
    pub kubeconfig: Option<Kubeconfig>,
    pub active_context: Option<String>,
    pub active_client: Option<Client>,
    pub watch_cancel: Option<tokio::sync::watch::Sender<bool>>,
    pub active_context_healthy: bool,
    pub log_cancel: Vec<tokio::sync::oneshot::Sender<()>>,
    pub port_forward_cancel: HashMap<String, (tokio::sync::oneshot::Sender<()>, tokio::task::JoinHandle<()>)>,
    pub config: OrbitConfig,
    pub kubeconfig_error: Option<String>,
}

impl KubeManager {
    pub fn new_empty(config: OrbitConfig) -> Self {
        Self::new_from_kubeconfig(None, config)
    }

    pub fn new_from_kubeconfig(kubeconfig: Option<Kubeconfig>, config: OrbitConfig) -> Self {
        Self {
            kubeconfig,
            active_context: None,
            active_client: None,
            watch_cancel: None,
            active_context_healthy: false,
            log_cancel: Vec::new(),
            port_forward_cancel: HashMap::new(),
            config,
            kubeconfig_error: None,
        }
    }

    pub fn kubeconfig_load_error(&self) -> Option<&str> {
        self.kubeconfig_error.as_deref()
    }

    pub async fn new() -> Self {
        let config = OrbitConfig::load();

        let mut manager = Self::new_empty(config);
        
        // Try reading default kubeconfig
        match Kubeconfig::read() {
            Ok(config) => {
                manager.kubeconfig = Some(config);
                // Intentionally do not connect — user must explicitly select a cluster
            }
            Err(err) => {
                // Missing default ~/.kube/config is expected on unconfigured machines;
                // only surface read/parse errors when a kubeconfig file actually exists.
                let default_exists = std::env::var_os("KUBECONFIG")
                    .filter(|v| !v.is_empty())
                    .is_some()
                    || std::env::var_os("HOME")
                        .or_else(|| std::env::var_os("USERPROFILE"))
                        .map(|home| {
                            std::path::PathBuf::from(home)
                                .join(".kube")
                                .join("config")
                                .exists()
                        })
                        .unwrap_or(false);
                if default_exists {
                    manager.kubeconfig_error = Some(format!("Failed to load kubeconfig: {err}"));
                }
            }
        }
        
        // Reload persisted custom kubeconfig files
        for path in manager.config.custom_kubeconfig_paths() {
            if let Err(e) = manager.merge_kubeconfig_from_path(&path) {
                tracing::warn!(path = %path, error = %e, "Failed to load persisted kubeconfig");
                if manager.kubeconfig_error.is_none() {
                    manager.kubeconfig_error = Some(format!("Failed to load custom kubeconfig '{path}': {e}"));
                }
            }
        }

        manager
    }

    pub fn get_clusters(&self) -> Vec<ClusterInfo> {
        let mut clusters = Vec::new();
        let customizations = self.config.cluster_customizations();

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

                let server_url = ctx
                    .context
                    .as_ref()
                    .and_then(|c| config.clusters.iter().find(|cl| cl.name == c.cluster))
                    .and_then(|cl| cl.cluster.as_ref())
                    .and_then(|cl| cl.server.clone());

                let custom = customizations.get(&ctx.name);

                clusters.push(ClusterInfo {
                    id: ctx.name.clone(),
                    name: ctx.name.clone(),
                    status,
                    alias: custom.and_then(|c| c.alias.clone()),
                    tags: custom.map(|c| c.tags.clone()).unwrap_or_default(),
                    color: custom.and_then(|c| c.color.clone()),
                    description: custom.and_then(|c| c.description.clone()),
                    server_url,
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
                    auth_type = crate::kubernetes::kubeconfig_import::detect_auth_type(auth).to_string();
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
        for path in &self.config.custom_kubeconfig_paths() {
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

    /// Resolves outbound proxy URI from given proxy settings configuration.
    pub fn resolve_proxy_uri(proxy_settings: &serde_json::Value) -> Option<Uri> {
        if !proxy_settings.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false) {
            return None;
        }

        let proxy_str = proxy_settings
            .get("httpsProxy")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                proxy_settings
                    .get("httpProxy")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
            })?;

        proxy_str.parse::<Uri>().ok()
    }

    /// Checks whether a given host should bypass the proxy according to the `no_proxy` rules.
    pub fn should_bypass_proxy(no_proxy: &str, target_host: &str) -> bool {
        let trimmed_host = target_host.trim().to_lowercase();
        if trimmed_host.is_empty() {
            return false;
        }

        for item in no_proxy.split(',') {
            let item = item.trim().to_lowercase();
            if item.is_empty() {
                continue;
            }
            if item == "*" {
                return true;
            }
            let item_clean = item.strip_prefix('.').unwrap_or(&item);
            if trimmed_host == item_clean || trimmed_host.ends_with(&format!(".{}", item_clean)) {
                return true;
            }
        }

        false
    }

    /// Resolves outbound proxy URI from proxy settings, checking if the cluster URL host is in the bypass list.
    pub fn resolve_proxy_uri_for_cluster(
        proxy_settings: &serde_json::Value,
        cluster_url: &Uri,
    ) -> Option<Uri> {
        if !proxy_settings.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false) {
            return None;
        }

        if let Some(host) = cluster_url.host()
            && let Some(no_proxy) = proxy_settings.get("noProxy").and_then(|v| v.as_str())
            && Self::should_bypass_proxy(no_proxy, host)
        {
            return None;
        }

        Self::resolve_proxy_uri(proxy_settings)
    }

    fn format_kube_connection_error(err: &kube::Error) -> String {
        format!("Failed to reach cluster API server: {err}")
    }

    pub async fn switch_context_with_status(
        &mut self,
        context_name: &str,
    ) -> Result<ClusterSwitchOutcome, String> {
        let config = self
            .kubeconfig
            .as_ref()
            .ok_or_else(|| "No kubeconfig loaded".to_string())?;

        if !config.contexts.iter().any(|c| c.name == context_name) {
            return Err(format!("Context '{context_name}' not found in kubeconfig"));
        }

        let options = KubeConfigOptions {
            context: Some(context_name.to_string()),
            ..Default::default()
        };

        let mut client_config = Config::from_custom_kubeconfig(config.clone(), &options)
            .await
            .map_err(|e| format!("Failed to create client config: {e}"))?;

        if let Some(proxy_uri) = Self::resolve_proxy_uri_for_cluster(
            &self.config.proxy_settings(),
            &client_config.cluster_url,
        ) {
            client_config.proxy_url = Some(proxy_uri);
        }

        let client = Client::try_from(client_config)
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let (healthy, k8s_version, error_message) = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.apiserver_version(),
        )
        .await
        {
            Ok(Ok(info)) => (true, Some(info.git_version), None),
            Ok(Err(err)) => (false, None, Some(Self::format_kube_connection_error(&err))),
            Err(_elapsed) => (
                false,
                None,
                Some("Connection timed out after 10s while reaching cluster API server".to_string()),
            ),
        };

        self.cancel_active_tasks();
        self.active_context = Some(context_name.to_string());
        self.active_client = Some(client);
        self.active_context_healthy = healthy;

        Ok(ClusterSwitchOutcome {
            context_name: context_name.to_string(),
            healthy,
            k8s_version,
            error_message,
        })
    }

    pub fn cancel_active_tasks(&mut self) {
        if let Some(tx) = self.watch_cancel.take() {
            let _ = tx.send(true);
        }
        for tx in self.log_cancel.drain(..) {
            let _ = tx.send(());
        }
        for (_, (tx, handle)) in self.port_forward_cancel.drain() {
            let _ = tx.send(());
            handle.abort();
        }
    }

    pub fn disconnect_active_cluster(&mut self) -> ClusterDisconnectOutcome {
        self.cancel_active_tasks();
        let previous_context = self.active_context.take();
        self.active_client = None;
        self.active_context_healthy = false;
        ClusterDisconnectOutcome { previous_context }
    }

    pub async fn reconnect_active_cluster(&mut self) -> Result<ClusterSwitchOutcome, String> {
        let context_name = self
            .active_context
            .clone()
            .ok_or_else(|| "No active cluster selected".to_string())?;
        self.switch_context_with_status(&context_name).await
    }

    pub async fn switch_context(&mut self, context_name: &str) -> Result<(), String> {
        self.switch_context_with_status(context_name)
            .await
            .map(|_| ())
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

    pub fn add_kubeconfig_file_without_switch(&mut self, file_path: &str) -> Result<(), String> {
        self.merge_kubeconfig_from_path(file_path)?;

        if let Err(e) = self.config.add_kubeconfig_path(file_path) {
            tracing::warn!(path = %file_path, error = %e, "Failed to persist kubeconfig path");
        }

        Ok(())
    }

    pub async fn add_kubeconfig_file(&mut self, file_path: &str) -> Result<(), String> {
        self.add_kubeconfig_file_without_switch(file_path)?;

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

    /// Validates raw kubeconfig YAML content, persists it to ~/.orbit/kubeconfigs/,
    /// merges its contexts into the active manager, and registers it in persistent config.
    pub fn active_client_for_context(&self, cluster_id: &str) -> Option<Client> {
        if self.active_context.as_deref() == Some(cluster_id) && self.active_context_healthy {
            self.active_client.clone()
        } else {
            None
        }
    }

    pub fn add_kubeconfig_content_without_switch(&mut self, content: &str) -> Result<String, String> {
        // Validate YAML syntax and Kubeconfig schema
        Kubeconfig::from_yaml(content)
            .map_err(|e| format!("Invalid kubeconfig content: {}", e))?;

        let config_dir = OrbitConfig::config_dir()
            .ok_or_else(|| "Could not determine user config directory".to_string())?;
        let kubeconfigs_dir = config_dir.join("kubeconfigs");
        if !kubeconfigs_dir.exists() {
            std::fs::create_dir_all(&kubeconfigs_dir)
                .map_err(|e| format!("Failed to create kubeconfigs directory: {}", e))?;
        }

        let file_name = format!("orbit_cluster_{}.yaml", uuid::Uuid::new_v4());
        let file_path = kubeconfigs_dir.join(file_name);
        let file_path_str = file_path.to_string_lossy().to_string();

        std::fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write kubeconfig file: {}", e))?;

        self.add_kubeconfig_file_without_switch(&file_path_str)?;

        Ok(file_path_str)
    }

    pub async fn add_kubeconfig_content(&mut self, content: &str) -> Result<String, String> {
        let file_path_str = self.add_kubeconfig_content_without_switch(content)?;

        if self.active_context.is_none() {
            let next_ctx = self.kubeconfig.as_ref().and_then(|config| {
                config
                    .current_context
                    .clone()
                    .or_else(|| config.contexts.first().map(|c| c.name.clone()))
            });
            if let Some(ctx) = next_ctx {
                let _ = self.switch_context(&ctx).await;
            }
        }

        Ok(file_path_str)
    }

    pub async fn register_kubeconfig(
        &mut self,
        request: AddClusterRequest,
    ) -> Result<AddClusterOutcome, String> {
        let mut selected = Vec::new();
        for ctx in request.selected_contexts {
            let trimmed = ctx.trim().to_string();
            if !trimmed.is_empty() && !selected.contains(&trimmed) {
                selected.push(trimmed);
            }
        }
        if selected.is_empty() {
            return Err("At least one context must be selected".to_string());
        }

        let raw_yaml = match &request.source {
            AddClusterSource::FilePath(path) => std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read kubeconfig file '{path}': {e}"))?,
            AddClusterSource::YamlContent(yaml) => yaml.clone(),
        };

        let discovered = parse_kubeconfig_contexts(&raw_yaml)?;
        for sel in &selected {
            if !discovered.iter().any(|d| d.name == *sel) {
                return Err(format!("Context '{sel}' not found in kubeconfig"));
            }
        }

        if let Some(ref existing) = self.kubeconfig {
            for sel in &selected {
                if existing.contexts.iter().any(|c| c.name == *sel) {
                    return Err(format!("Context '{sel}' already exists in Orbit"));
                }
            }
        }

        match &request.source {
            AddClusterSource::FilePath(path) if selected.len() == discovered.len() => {
                self.add_kubeconfig_file_without_switch(path)?;
            }
            _ => {
                let filtered_yaml = filter_kubeconfig_yaml(&raw_yaml, &selected)?;
                self.add_kubeconfig_content_without_switch(&filtered_yaml)?;
            }
        }

        if let Some(customization) = request.customization {
            self.config
                .set_cluster_customization(&selected[0], customization);
            if let Err(e) = self.config.save() {
                tracing::warn!(error = %e, "Failed to save cluster customization");
            }
        }

        let (switch_outcome, profile) = if request.switch_context {
            let outcome = match self.switch_context_with_status(&selected[0]).await {
                Ok(outcome) => outcome,
                Err(err) => ClusterSwitchOutcome {
                    context_name: selected[0].clone(),
                    healthy: false,
                    k8s_version: None,
                    error_message: Some(err),
                },
            };
            let profile = Some(self.get_user_profile());
            (Some(outcome), profile)
        } else {
            (None, None)
        };

        Ok(AddClusterOutcome {
            added_contexts: selected,
            clusters: self.get_clusters(),
            switch_outcome,
            profile,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_kubeconfig_nonexistent_file() {
        let mut manager = KubeManager::new_empty(OrbitConfig::default());

        let result = manager.merge_kubeconfig_from_path("/nonexistent/path/to/kubeconfig");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Provided kubeconfig path does not exist");
    }

    #[test]
    fn test_merge_kubeconfig_from_temp_file() {
        let mut manager = KubeManager::new_empty(OrbitConfig::default());

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
        assert_eq!(clusters[0].server_url, Some("https://127.0.0.1:6443".to_string()));

        // Verify custom metadata enrichment
        manager.config.set_cluster_customization("test-context", crate::kubernetes::models::ClusterCustomization {
            alias: Some("My Friendly Cluster".to_string()),
            tags: vec!["production".to_string()],
            color: Some("#ef4444".to_string()),
            description: Some("Production node".to_string()),
        });
        let enriched = manager.get_clusters();
        assert_eq!(enriched[0].alias, Some("My Friendly Cluster".to_string()));
        assert_eq!(enriched[0].tags, vec!["production".to_string()]);
        assert_eq!(enriched[0].color, Some("#ef4444".to_string()));
        assert_eq!(enriched[0].description, Some("Production node".to_string()));
        assert_eq!(enriched[0].server_url, Some("https://127.0.0.1:6443".to_string()));

        let _ = std::fs::remove_file(file_path);
    }

    #[tokio::test]
    async fn test_add_kubeconfig_content_creates_managed_file_and_merges() {
        let _guard = crate::config::TestConfigDirGuard::new("managed_kubeconfig");
        let mut manager = KubeManager::new_empty(OrbitConfig::load());

        let sample_yaml = r#"
apiVersion: v1
clusters:
- cluster:
    server: https://10.0.0.1:6443
  name: pasted-cluster
contexts:
- context:
    cluster: pasted-cluster
    user: pasted-user
  name: pasted-context
current-context: pasted-context
kind: Config
preferences: {}
users:
- name: pasted-user
  user: {}
"#;

        let result = manager.add_kubeconfig_content(sample_yaml).await;
        assert!(result.is_ok(), "add_kubeconfig_content should succeed: {:?}", result.err());
        let saved_path = result.unwrap();
        assert!(std::path::Path::new(&saved_path).exists());
        assert!(saved_path.contains("kubeconfigs"));

        let clusters = manager.get_clusters();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].name, "pasted-context");
        assert_eq!(clusters[0].server_url, Some("https://10.0.0.1:6443".to_string()));

        // Check it was added to customKubeconfigPaths
        let persisted_paths = manager.config.custom_kubeconfig_paths();
        assert!(persisted_paths.iter().any(|p| p.contains("orbit_cluster_")));
    }

    #[tokio::test]
    async fn test_register_kubeconfig_yaml_persists_customization_and_respects_switch_flag() {
        let _guard = crate::config::TestConfigDirGuard::new("register_yaml_no_switch");
        let mut manager = KubeManager::new_empty(OrbitConfig::load());

        let yaml = r#"
apiVersion: v1
kind: Config
current-context: ctx-a
clusters:
  - name: cl-a
    cluster:
      server: https://10.0.0.10:6443
  - name: cl-b
    cluster:
      server: https://10.0.0.11:6443
contexts:
  - name: ctx-a
    context:
      cluster: cl-a
      user: u-a
  - name: ctx-b
    context:
      cluster: cl-b
      user: u-b
users:
  - name: u-a
    user:
      token: tok-a
  - name: u-b
    user:
      token: tok-b
"#;

        let req = AddClusterRequest {
            source: AddClusterSource::YamlContent(yaml.to_string()),
            selected_contexts: vec!["ctx-b".to_string()],
            customization: Some(crate::kubernetes::models::ClusterCustomization {
                alias: Some("Staging B".to_string()),
                tags: vec!["staging".to_string()],
                color: Some("#3b82f6".to_string()),
                description: None,
            }),
            switch_context: false,
        };

        let outcome = manager
            .register_kubeconfig(req)
            .await
            .expect("register_kubeconfig should succeed");

        assert_eq!(outcome.added_contexts, vec!["ctx-b".to_string()]);
        assert!(outcome.switch_outcome.is_none());
        assert!(manager.active_context.is_none(), "must not switch when switch_context == false");
        assert_eq!(outcome.clusters.len(), 1);
        assert_eq!(outcome.clusters[0].name, "ctx-b");
        assert_eq!(outcome.clusters[0].alias.as_deref(), Some("Staging B"));
        assert_eq!(outcome.clusters[0].tags, vec!["staging".to_string()]);
        assert_eq!(outcome.clusters[0].color.as_deref(), Some("#3b82f6"));
        assert!(
            manager
                .config
                .custom_kubeconfig_paths()
                .iter()
                .any(|p| p.contains("orbit_cluster_"))
        );
        assert!(manager.config.cluster_customizations().contains_key("ctx-b"));
    }

    #[tokio::test]
    async fn test_register_kubeconfig_rejects_duplicate_context_without_mutating_state() {
        let _guard = crate::config::TestConfigDirGuard::new("register_duplicate");
        let mut manager = KubeManager::new_empty(OrbitConfig::load());

        let yaml = r#"
apiVersion: v1
kind: Config
clusters:
  - name: cl-dup
    cluster:
      server: https://10.0.0.20:6443
contexts:
  - name: dup-ctx
    context:
      cluster: cl-dup
      user: u-dup
users:
  - name: u-dup
    user:
      token: tok
"#;

        manager
            .register_kubeconfig(AddClusterRequest {
                source: AddClusterSource::YamlContent(yaml.to_string()),
                selected_contexts: vec!["dup-ctx".to_string()],
                customization: None,
                switch_context: false,
            })
            .await
            .expect("first registration should succeed");

        let count_before = manager.get_clusters().len();
        let paths_before = manager.config.custom_kubeconfig_paths().len();

        let err = manager
            .register_kubeconfig(AddClusterRequest {
                source: AddClusterSource::YamlContent(yaml.to_string()),
                selected_contexts: vec!["dup-ctx".to_string()],
                customization: None,
                switch_context: false,
            })
            .await
            .unwrap_err();

        assert!(err.contains("already exists"), "got: {err}");
        assert_eq!(manager.get_clusters().len(), count_before);
        assert_eq!(manager.config.custom_kubeconfig_paths().len(), paths_before);
    }

    #[tokio::test]
    async fn test_register_kubeconfig_missing_file_returns_clear_error() {
        let _guard = crate::config::TestConfigDirGuard::new("register_missing_file");
        let mut manager = KubeManager::new_empty(OrbitConfig::load());

        let err = manager
            .register_kubeconfig(AddClusterRequest {
                source: AddClusterSource::FilePath("/nonexistent/orbit-missing-kubeconfig.yaml".to_string()),
                selected_contexts: vec!["ctx".to_string()],
                customization: None,
                switch_context: false,
            })
            .await
            .unwrap_err();

        assert!(err.contains("Failed to read"), "got: {err}");
    }

    #[tokio::test]
    async fn test_register_kubeconfig_unreachable_endpoint_registers_cluster_and_surfaces_unhealthy_outcome() {
        let _guard = crate::config::TestConfigDirGuard::new("register_unreachable");
        let mut manager = KubeManager::new_empty(OrbitConfig::load());

        let yaml = r#"
apiVersion: v1
kind: Config
clusters:
  - name: unreachable-cl
    cluster:
      server: http://127.0.0.1:1
contexts:
  - name: unreachable-ctx
    context:
      cluster: unreachable-cl
      user: u
users:
  - name: u
    user:
      token: tok
"#;

        let outcome = manager
            .register_kubeconfig(AddClusterRequest {
                source: AddClusterSource::YamlContent(yaml.to_string()),
                selected_contexts: vec!["unreachable-ctx".to_string()],
                customization: None,
                switch_context: true,
            })
            .await
            .expect("registration should succeed even when endpoint is unreachable");

        assert_eq!(outcome.added_contexts, vec!["unreachable-ctx".to_string()]);
        let switch = outcome.switch_outcome.expect("switch_outcome should be present");
        assert!(!switch.healthy);
        assert!(switch.error_message.as_deref().is_some_and(|m| !m.is_empty()));
    }

    #[test]
    fn test_get_user_profile_default() {
        let manager = KubeManager::new_empty(OrbitConfig::default());

        let profile = manager.get_user_profile();
        assert_eq!(profile.active_context, None);
        assert_eq!(profile.user_name, None);
        assert_eq!(profile.auth_type, "None");
        assert_eq!(profile.cluster_name, None);
        assert_eq!(profile.server_url, None);
    }

    #[test]
    fn test_resolve_proxy_uri() {
        let disabled_cfg = serde_json::json!({
            "enabled": false,
            "httpProxy": "http://127.0.0.1:8080",
            "httpsProxy": "http://127.0.0.1:8443",
            "noProxy": ""
        });
        assert_eq!(KubeManager::resolve_proxy_uri(&disabled_cfg), None);

        let enabled_https = serde_json::json!({
            "enabled": true,
            "httpProxy": "http://127.0.0.1:8080",
            "httpsProxy": "http://proxy.corp.internal:8443",
            "noProxy": ""
        });
        let uri = KubeManager::resolve_proxy_uri(&enabled_https).expect("Should resolve URI");
        assert_eq!(uri.to_string(), "http://proxy.corp.internal:8443/");

        let enabled_http_fallback = serde_json::json!({
            "enabled": true,
            "httpProxy": "http://proxy.corp.internal:8080",
            "httpsProxy": "",
            "noProxy": ""
        });
        let uri = KubeManager::resolve_proxy_uri(&enabled_http_fallback).expect("Should resolve HTTP fallback");
        assert_eq!(uri.to_string(), "http://proxy.corp.internal:8080/");

        let invalid_uri = serde_json::json!({
            "enabled": true,
            "httpProxy": "not a uri",
            "httpsProxy": "",
            "noProxy": ""
        });
        assert_eq!(KubeManager::resolve_proxy_uri(&invalid_uri), None);
    }

    #[test]
    fn test_should_bypass_proxy() {
        let no_proxy = "localhost, 127.0.0.1, .cluster.local, internal";
        assert!(KubeManager::should_bypass_proxy(no_proxy, "localhost"));
        assert!(KubeManager::should_bypass_proxy(no_proxy, "127.0.0.1"));
        assert!(KubeManager::should_bypass_proxy(no_proxy, "cluster.local"));
        assert!(KubeManager::should_bypass_proxy(no_proxy, "minikube.cluster.local"));
        assert!(KubeManager::should_bypass_proxy(no_proxy, "api.internal"));
        assert!(!KubeManager::should_bypass_proxy(no_proxy, "example.com"));
        assert!(!KubeManager::should_bypass_proxy(no_proxy, "10.0.0.1"));

        // Wildcard support
        assert!(KubeManager::should_bypass_proxy("*", "anyhost.org"));
        assert!(!KubeManager::should_bypass_proxy("", "localhost"));
    }

    #[test]
    fn test_resolve_proxy_uri_for_cluster() {
        let proxy_cfg = serde_json::json!({
            "enabled": true,
            "httpProxy": "http://proxy.corp.internal:8080",
            "httpsProxy": "http://proxy.corp.internal:8443",
            "noProxy": "localhost,127.0.0.1,.cluster.local"
        });

        let local_cluster: Uri = "https://127.0.0.1:6443".parse().expect("valid uri");
        assert_eq!(KubeManager::resolve_proxy_uri_for_cluster(&proxy_cfg, &local_cluster), None);

        let k8s_local: Uri = "https://minikube.cluster.local:8443".parse().expect("valid uri");
        assert_eq!(KubeManager::resolve_proxy_uri_for_cluster(&proxy_cfg, &k8s_local), None);

        let remote_cluster: Uri = "https://k8s.cloud.provider.com:6443".parse().expect("valid uri");
        let resolved = KubeManager::resolve_proxy_uri_for_cluster(&proxy_cfg, &remote_cluster)
            .expect("should route remote cluster through proxy");
        assert_eq!(resolved.to_string(), "http://proxy.corp.internal:8443/");
    }

    fn sample_kubeconfig_with_contexts(contexts: &[(&str, &str)]) -> Kubeconfig {
        let mut yaml = String::from("apiVersion: v1\nkind: Config\nclusters:\n");
        for (idx, (_, server)) in contexts.iter().enumerate() {
            yaml.push_str(&format!(
                "- name: cluster-{idx}\n  cluster:\n    server: {server}\n"
            ));
        }
        yaml.push_str("contexts:\n");
        for (idx, (ctx_name, _)) in contexts.iter().enumerate() {
            yaml.push_str(&format!(
                "- name: {ctx_name}\n  context:\n    cluster: cluster-{idx}\n    user: user-{idx}\n"
            ));
        }
        yaml.push_str("users:\n");
        for (idx, _) in contexts.iter().enumerate() {
            yaml.push_str(&format!("- name: user-{idx}\n  user: {{}}\n"));
        }
        Kubeconfig::from_yaml(&yaml).expect("valid sample kubeconfig")
    }

    #[test]
    fn test_new_from_kubeconfig_loads_contexts_disconnected() {
        let kc = sample_kubeconfig_with_contexts(&[
            ("dev-ctx", "https://127.0.0.1:6443"),
            ("prod-ctx", "https://10.0.0.10:6443"),
        ]);
        let manager = KubeManager::new_from_kubeconfig(Some(kc), OrbitConfig::default());

        let clusters = manager.get_clusters();
        assert_eq!(clusters.len(), 2);
        assert!(manager.active_context.is_none());
        assert!(!manager.active_context_healthy);
        assert!(clusters.iter().all(|c| c.status == "offline"));
        assert_eq!(clusters[0].name, "dev-ctx");
        assert_eq!(clusters[1].name, "prod-ctx");
    }

    #[tokio::test]
    async fn test_switch_context_with_status_missing_context_returns_clear_error() {
        let kc = sample_kubeconfig_with_contexts(&[("dev-ctx", "http://127.0.0.1:1")]);
        let mut manager = KubeManager::new_from_kubeconfig(Some(kc), OrbitConfig::default());

        let result = manager.switch_context_with_status("nonexistent").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_lowercase().contains("not found"),
            "Expected 'not found' in error: {err}"
        );
        assert!(manager.active_context.is_none());
    }

    #[tokio::test]
    async fn test_switch_context_with_status_unreachable_cluster_preserves_error_message() {
        let kc = sample_kubeconfig_with_contexts(&[("unreachable-ctx", "http://127.0.0.1:1")]);
        let mut manager = KubeManager::new_from_kubeconfig(Some(kc), OrbitConfig::default());

        let outcome = manager
            .switch_context_with_status("unreachable-ctx")
            .await
            .expect("switch_context_with_status should return Ok(outcome) for unreachable cluster");

        assert_eq!(outcome.context_name, "unreachable-ctx");
        assert!(!outcome.healthy);
        assert!(outcome.k8s_version.is_none());
        let msg = outcome
            .error_message
            .as_deref()
            .expect("Expected diagnostic error_message");
        assert!(!msg.trim().is_empty());
        assert_eq!(manager.active_context.as_deref(), Some("unreachable-ctx"));
        assert!(!manager.active_context_healthy);
    }

    #[tokio::test]
    async fn test_reconnect_active_cluster_requires_active_context() {
        let mut manager = KubeManager::new_from_kubeconfig(None, OrbitConfig::default());
        let result = manager.reconnect_active_cluster().await;
        assert_eq!(result, Err("No active cluster selected".to_string()));
    }

    #[tokio::test]
    async fn test_disconnect_active_cluster_cancels_tasks_clears_state_and_allows_reconnect() {
        let kc = sample_kubeconfig_with_contexts(&[("dev-ctx", "http://127.0.0.1:1")]);
        let mut manager = KubeManager::new_from_kubeconfig(Some(kc), OrbitConfig::default());

        let switch_outcome = manager
            .switch_context_with_status("dev-ctx")
            .await
            .expect("switch should return Ok");
        assert_eq!(switch_outcome.context_name, "dev-ctx");
        assert_eq!(manager.active_context.as_deref(), Some("dev-ctx"));
        assert!(manager.active_client.is_some());

        let (watch_tx, watch_rx) = tokio::sync::watch::channel(false);
        manager.watch_cancel = Some(watch_tx);

        let (log_tx, log_rx) = tokio::sync::oneshot::channel();
        manager.log_cancel.push(log_tx);

        let (pf_tx, pf_rx) = tokio::sync::oneshot::channel();
        let pf_handle = tokio::spawn(async move {
            let _ = pf_rx.await;
            std::future::pending::<()>().await;
        });
        manager
            .port_forward_cancel
            .insert("pf-1".to_string(), (pf_tx, pf_handle));

        let outcome = manager.disconnect_active_cluster();
        assert_eq!(
            outcome,
            ClusterDisconnectOutcome {
                previous_context: Some("dev-ctx".to_string()),
            }
        );

        assert!(*watch_rx.borrow(), "watch_cancel should signal true");
        assert!(manager.watch_cancel.is_none());
        assert!(log_rx.await.is_ok(), "log_cancel oneshot should fire");
        assert!(manager.log_cancel.is_empty());
        assert!(manager.port_forward_cancel.is_empty());
        assert!(manager.active_context.is_none());
        assert!(manager.active_client.is_none());
        assert!(!manager.active_context_healthy);
        assert!(manager.get_clusters().iter().all(|c| c.status == "offline"));

        let reconnect = manager
            .switch_context_with_status("dev-ctx")
            .await
            .expect("re-selecting context after disconnect should succeed");
        assert_eq!(reconnect.context_name, "dev-ctx");
        assert_eq!(manager.active_context.as_deref(), Some("dev-ctx"));
    }

    #[test]
    fn test_disconnect_active_cluster_is_idempotent_when_already_disconnected() {
        let kc = sample_kubeconfig_with_contexts(&[("dev-ctx", "http://127.0.0.1:1")]);
        let mut manager = KubeManager::new_from_kubeconfig(Some(kc), OrbitConfig::default());

        let first = manager.disconnect_active_cluster();
        assert_eq!(
            first,
            ClusterDisconnectOutcome {
                previous_context: None,
            }
        );
        let second = manager.disconnect_active_cluster();
        assert_eq!(
            second,
            ClusterDisconnectOutcome {
                previous_context: None,
            }
        );
        assert!(manager.active_context.is_none());
        assert!(manager.active_client.is_none());
        assert!(!manager.active_context_healthy);
    }

    #[test]
    fn test_active_client_for_context_returns_none_when_disconnected_or_mismatched() {
        let manager = KubeManager::new_empty(OrbitConfig::default());
        assert!(manager.active_client_for_context("any-ctx").is_none());
    }
}


