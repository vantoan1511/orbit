use std::collections::HashSet;

use base64::Engine;
use http::Uri;
use kube::config::{
    AuthInfo, Cluster, Context, Kubeconfig, NamedAuthInfo, NamedCluster, NamedContext,
};
use serde::{Deserialize, Serialize};

use crate::kubernetes::manager::ClusterSwitchOutcome;
use crate::kubernetes::models::{ClusterCustomization, ClusterInfo, UserProfileInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ManualAuthType {
    #[default]
    Token,
    Certificate,
    Basic,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredContextInfo {
    pub name: String,
    pub cluster_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    pub auth_type: String,
    pub is_current_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualEndpointConfig {
    pub context_name: String,
    pub cluster_name: String,
    pub server_url: String,
    pub auth_type: ManualAuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub insecure_skip_tls_verify: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert_data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AddClusterSource {
    FilePath(String),
    YamlContent(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddClusterRequest {
    pub source: AddClusterSource,
    pub selected_contexts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customization: Option<ClusterCustomization>,
    pub switch_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddClusterOutcome {
    pub added_contexts: Vec<String>,
    pub clusters: Vec<ClusterInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_outcome: Option<ClusterSwitchOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileInfo>,
}

pub fn detect_auth_type(auth: Option<&AuthInfo>) -> &'static str {
    let Some(auth) = auth else {
        return "None";
    };
    if auth.client_certificate.is_some() || auth.client_certificate_data.is_some() {
        "Certificate"
    } else if auth.token.is_some() || auth.token_file.is_some() {
        "Token"
    } else if auth.exec.is_some() {
        "Exec Plugin"
    } else if auth.auth_provider.is_some() {
        "OIDC"
    } else if auth.username.is_some() {
        "Basic Auth"
    } else {
        "Configured"
    }
}

pub fn parse_kubeconfig_contexts(yaml: &str) -> Result<Vec<DiscoveredContextInfo>, String> {
    if yaml.trim().is_empty() {
        return Err("Kubeconfig YAML is empty".to_string());
    }
    let config = Kubeconfig::from_yaml(yaml)
        .map_err(|e| format!("Invalid kubeconfig YAML: {e}"))?;
    if config.contexts.is_empty() {
        return Err("No contexts found in kubeconfig".to_string());
    }

    let current = config.current_context.as_deref();
    let discovered = config
        .contexts
        .iter()
        .map(|named_ctx| {
            let ctx = named_ctx.context.as_ref();
            let cluster_name = ctx.map(|c| c.cluster.clone()).unwrap_or_default();
            let user_name = ctx.and_then(|c| c.user.clone());
            let server_url = config
                .clusters
                .iter()
                .find(|cl| cl.name == cluster_name)
                .and_then(|cl| cl.cluster.as_ref())
                .and_then(|cl| cl.server.clone());
            let auth = user_name.as_deref().and_then(|uname| {
                config
                    .auth_infos
                    .iter()
                    .find(|a| a.name == uname)
                    .and_then(|a| a.auth_info.as_ref())
            });
            let auth_type = detect_auth_type(auth).to_string();
            let is_current_context = current == Some(named_ctx.name.as_str());

            DiscoveredContextInfo {
                name: named_ctx.name.clone(),
                cluster_name,
                server_url,
                user_name,
                auth_type,
                is_current_context,
            }
        })
        .collect();

    Ok(discovered)
}

pub fn filter_kubeconfig_yaml(yaml: &str, selected_contexts: &[String]) -> Result<String, String> {
    if selected_contexts.is_empty() {
        return Err("At least one context must be selected".to_string());
    }
    if yaml.trim().is_empty() {
        return Err("Kubeconfig YAML is empty".to_string());
    }
    let mut config = Kubeconfig::from_yaml(yaml)
        .map_err(|e| format!("Invalid kubeconfig YAML: {e}"))?;

    let selected_set: HashSet<&str> = selected_contexts.iter().map(String::as_str).collect();
    for sel in &selected_set {
        if !config.contexts.iter().any(|c| c.name == *sel) {
            return Err(format!("Context '{sel}' not found in kubeconfig"));
        }
    }

    config
        .contexts
        .retain(|c| selected_set.contains(c.name.as_str()));

    let referenced_clusters: HashSet<String> = config
        .contexts
        .iter()
        .filter_map(|c| c.context.as_ref().map(|ctx| ctx.cluster.clone()))
        .collect();
    let referenced_users: HashSet<String> = config
        .contexts
        .iter()
        .filter_map(|c| c.context.as_ref().and_then(|ctx| ctx.user.clone()))
        .collect();

    config
        .clusters
        .retain(|cl| referenced_clusters.contains(&cl.name));
    config
        .auth_infos
        .retain(|u| referenced_users.contains(&u.name));

    let keep_current = config
        .current_context
        .as_deref()
        .is_some_and(|c| selected_set.contains(c));
    if !keep_current {
        config.current_context = selected_contexts.first().cloned();
    }

    serde_yaml_ng::to_string(&config)
        .map_err(|e| format!("Failed to serialize filtered kubeconfig: {e}"))
}

pub fn normalize_server_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("API server URL is required".to_string());
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    let uri: Uri = with_scheme
        .parse()
        .map_err(|e| format!("Invalid API server URL: {e}"))?;
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| "API server URL must include http:// or https://".to_string())?;
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "Unsupported API server URL scheme '{scheme}': expected http or https"
        ));
    }
    let host = uri
        .host()
        .ok_or_else(|| "API server URL must include a valid host".to_string())?;
    if host.is_empty() {
        return Err("API server URL must include a valid host".to_string());
    }

    Ok(with_scheme)
}

pub fn normalize_cert_data(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("-----BEGIN") {
        base64::engine::general_purpose::STANDARD.encode(trimmed.as_bytes())
    } else {
        trimmed.to_string()
    }
}

fn non_empty_trimmed(opt: Option<&str>) -> Option<String> {
    opt.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub fn build_kubeconfig_from_endpoint(config: &ManualEndpointConfig) -> Result<String, String> {
    let cluster_name = config.cluster_name.trim();
    if cluster_name.is_empty() {
        return Err("Cluster name is required".to_string());
    }
    let context_name = if config.context_name.trim().is_empty() {
        cluster_name.to_string()
    } else {
        config.context_name.trim().to_string()
    };
    let server_url = normalize_server_url(&config.server_url)?;

    let ca_cert_data = non_empty_trimmed(config.ca_cert_data.as_deref())
        .map(|raw| normalize_cert_data(&raw));

    let cluster = Cluster {
        server: Some(server_url),
        insecure_skip_tls_verify: if config.insecure_skip_tls_verify {
            Some(true)
        } else {
            None
        },
        certificate_authority_data: ca_cert_data,
        ..Default::default()
    };

    let user_name = format!("{context_name}-user");
    let mut auth_info = AuthInfo::default();
    let mut has_user = true;

    match config.auth_type {
        ManualAuthType::Token => {
            let token = non_empty_trimmed(config.token.as_deref())
                .ok_or_else(|| "Bearer token is required for Token authentication".to_string())?;
            auth_info.token = Some(token.into());
        }
        ManualAuthType::Certificate => {
            let cert = non_empty_trimmed(config.client_cert_data.as_deref())
                .ok_or_else(|| "Client certificate is required for Certificate authentication".to_string())?;
            let key = non_empty_trimmed(config.client_key_data.as_deref())
                .ok_or_else(|| "Client key is required for Certificate authentication".to_string())?;
            auth_info.client_certificate_data = Some(normalize_cert_data(&cert));
            auth_info.client_key_data = Some(normalize_cert_data(&key).into());
        }
        ManualAuthType::Basic => {
            let username = non_empty_trimmed(config.username.as_deref())
                .ok_or_else(|| "Username is required for Basic authentication".to_string())?;
            let password = non_empty_trimmed(config.password.as_deref())
                .ok_or_else(|| "Password is required for Basic authentication".to_string())?;
            auth_info.username = Some(username);
            auth_info.password = Some(password.into());
        }
        ManualAuthType::None => {
            has_user = false;
        }
    }

    let kubeconfig = Kubeconfig {
        api_version: Some("v1".to_string()),
        kind: Some("Config".to_string()),
        clusters: vec![NamedCluster {
            name: cluster_name.to_string(),
            cluster: Some(cluster),
        }],
        contexts: vec![NamedContext {
            name: context_name.clone(),
            context: Some(Context {
                cluster: cluster_name.to_string(),
                user: if has_user {
                    Some(user_name.clone())
                } else {
                    None
                },
                ..Default::default()
            }),
        }],
        auth_infos: if has_user {
            vec![NamedAuthInfo {
                name: user_name,
                auth_info: Some(auth_info),
            }]
        } else {
            Vec::new()
        },
        current_context: Some(context_name),
        ..Default::default()
    };

    serde_yaml_ng::to_string(&kubeconfig)
        .map_err(|e| format!("Failed to serialize kubeconfig: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MULTI_CONTEXT_YAML: &str = r#"
apiVersion: v1
kind: Config
current-context: prod-ctx
clusters:
  - name: prod-cluster
    cluster:
      server: https://prod.k8s.example.com:6443
  - name: dev-cluster
    cluster:
      server: https://dev.k8s.example.com:6443
  - name: exec-cluster
    cluster:
      server: https://exec.k8s.example.com
  - name: oidc-cluster
    cluster:
      server: https://oidc.k8s.example.com
  - name: basic-cluster
    cluster:
      server: https://basic.k8s.example.com
contexts:
  - name: prod-ctx
    context:
      cluster: prod-cluster
      user: prod-user
  - name: dev-ctx
    context:
      cluster: dev-cluster
      user: dev-user
  - name: exec-ctx
    context:
      cluster: exec-cluster
      user: exec-user
  - name: oidc-ctx
    context:
      cluster: oidc-cluster
      user: oidc-user
  - name: basic-ctx
    context:
      cluster: basic-cluster
      user: basic-user
users:
  - name: prod-user
    user:
      token: my-secret-token
  - name: dev-user
    user:
      client-certificate-data: Y2VydA==
      client-key-data: a2V5
  - name: exec-user
    user:
      exec:
        apiVersion: client.authentication.k8s.io/v1beta1
        command: aws
  - name: oidc-user
    user:
      auth-provider:
        name: oidc
  - name: basic-user
    user:
      username: admin
      password: secret
"#;

    #[test]
    fn test_parse_kubeconfig_contexts_extracts_metadata_and_auth_types() {
        let contexts = parse_kubeconfig_contexts(MULTI_CONTEXT_YAML)
            .expect("valid multi-context kubeconfig should parse");
        assert_eq!(contexts.len(), 5);

        let prod = &contexts[0];
        assert_eq!(prod.name, "prod-ctx");
        assert_eq!(prod.cluster_name, "prod-cluster");
        assert_eq!(
            prod.server_url.as_deref(),
            Some("https://prod.k8s.example.com:6443")
        );
        assert_eq!(prod.user_name.as_deref(), Some("prod-user"));
        assert_eq!(prod.auth_type, "Token");
        assert!(prod.is_current_context);

        assert_eq!(contexts[1].auth_type, "Certificate");
        assert!(!contexts[1].is_current_context);
        assert_eq!(contexts[2].auth_type, "Exec Plugin");
        assert_eq!(contexts[3].auth_type, "OIDC");
        assert_eq!(contexts[4].auth_type, "Basic Auth");
    }

    #[test]
    fn test_parse_kubeconfig_contexts_rejects_malformed_and_empty_yaml() {
        assert!(parse_kubeconfig_contexts("").is_err());
        assert!(parse_kubeconfig_contexts("   \n  ").is_err());
        assert!(parse_kubeconfig_contexts("not: [valid: yaml").is_err());
        let empty_contexts = "apiVersion: v1\nkind: Config\nclusters: []\ncontexts: []\nusers: []\n";
        let err = parse_kubeconfig_contexts(empty_contexts).unwrap_err();
        assert!(err.to_lowercase().contains("context"), "got: {err}");
    }

    #[test]
    fn test_filter_kubeconfig_yaml_prunes_unselected_contexts_clusters_and_users() {
        let filtered_yaml =
            filter_kubeconfig_yaml(MULTI_CONTEXT_YAML, &["dev-ctx".to_string()])
                .expect("filtering should succeed");
        let parsed = Kubeconfig::from_yaml(&filtered_yaml).expect("filtered YAML should be valid");

        assert_eq!(parsed.contexts.len(), 1);
        assert_eq!(parsed.contexts[0].name, "dev-ctx");
        assert_eq!(parsed.clusters.len(), 1);
        assert_eq!(parsed.clusters[0].name, "dev-cluster");
        assert_eq!(parsed.auth_infos.len(), 1);
        assert_eq!(parsed.auth_infos[0].name, "dev-user");
        assert_eq!(parsed.current_context.as_deref(), Some("dev-ctx"));
    }

    #[test]
    fn test_build_kubeconfig_from_endpoint_supports_token_cert_basic_and_validates_url() {
        // Missing scheme gets https:// prepended
        let token_cfg = ManualEndpointConfig {
            context_name: "staging-ctx".to_string(),
            cluster_name: "staging".to_string(),
            server_url: "api.staging.example.com:6443".to_string(),
            auth_type: ManualAuthType::Token,
            token: Some("tok-123".to_string()),
            ca_cert_data: Some("-----BEGIN CERTIFICATE-----\nABC\n-----END CERTIFICATE-----".to_string()),
            ..Default::default()
        };
        let token_yaml = build_kubeconfig_from_endpoint(&token_cfg).expect("token config should build");
        let parsed = Kubeconfig::from_yaml(&token_yaml).expect("generated yaml must parse");
        assert_eq!(
            parsed.clusters[0].cluster.as_ref().and_then(|c| c.server.as_deref()),
            Some("https://api.staging.example.com:6443")
        );
        let ca = parsed.clusters[0]
            .cluster
            .as_ref()
            .and_then(|c| c.certificate_authority_data.as_deref())
            .expect("CA should be set");
        assert!(!ca.starts_with("-----BEGIN"), "PEM should be base64 encoded");

        // Certificate auth with PEM normalization
        let cert_cfg = ManualEndpointConfig {
            context_name: "cert-ctx".to_string(),
            cluster_name: "cert-cluster".to_string(),
            server_url: "https://cert.example.com".to_string(),
            auth_type: ManualAuthType::Certificate,
            client_cert_data: Some("-----BEGIN CERTIFICATE-----\nCERT\n-----END CERTIFICATE-----".to_string()),
            client_key_data: Some("-----BEGIN TEST KEY-----\nKEY\n-----END TEST KEY-----".to_string()),
            insecure_skip_tls_verify: true,
            ..Default::default()
        };
        let cert_yaml = build_kubeconfig_from_endpoint(&cert_cfg).expect("cert config should build");
        let cert_parsed = Kubeconfig::from_yaml(&cert_yaml).expect("cert yaml must parse");
        assert_eq!(
            cert_parsed.clusters[0].cluster.as_ref().and_then(|c| c.insecure_skip_tls_verify),
            Some(true)
        );
        assert!(cert_parsed.auth_infos[0]
            .auth_info
            .as_ref()
            .and_then(|a| a.client_certificate_data.as_ref())
            .is_some());

        // Basic auth
        let basic_cfg = ManualEndpointConfig {
            context_name: "basic-ctx".to_string(),
            cluster_name: "basic-cluster".to_string(),
            server_url: "http://127.0.0.1:8080".to_string(),
            auth_type: ManualAuthType::Basic,
            username: Some("admin".to_string()),
            password: Some("pass".to_string()),
            ..Default::default()
        };
        let basic_yaml = build_kubeconfig_from_endpoint(&basic_cfg).expect("basic config should build");
        let basic_parsed = Kubeconfig::from_yaml(&basic_yaml).expect("basic yaml must parse");
        assert_eq!(
            basic_parsed.auth_infos[0].auth_info.as_ref().and_then(|a| a.username.as_deref()),
            Some("admin")
        );

        // Invalid / empty server URL rejected
        let bad_cfg = ManualEndpointConfig {
            context_name: "bad".to_string(),
            cluster_name: "bad".to_string(),
            server_url: "   ".to_string(),
            auth_type: ManualAuthType::None,
            ..Default::default()
        };
        assert!(build_kubeconfig_from_endpoint(&bad_cfg).is_err());
    }
}
