use std::collections::HashSet;

use orbit_core::kubernetes::kubeconfig_import::{
    AddClusterRequest, AddClusterSource, DiscoveredContextInfo, ManualAuthType,
    ManualEndpointConfig, build_kubeconfig_from_endpoint, parse_kubeconfig_contexts,
};
use orbit_core::kubernetes::models::{ClusterCustomization, ClusterInfo};

pub const CLUSTER_COLOR_PRESETS: &[(&str, &str)] = &[
    ("Blue", "#3b82f6"),
    ("Emerald", "#10b981"),
    ("Amber", "#f59e0b"),
    ("Rose", "#f43f5e"),
    ("Violet", "#8b5cf6"),
    ("Cyan", "#06b6d4"),
    ("Orange", "#f97316"),
    ("Slate", "#64748b"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddClusterTab {
    #[default]
    File,
    Yaml,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddClusterFieldId {
    YamlContent,
    ClusterName,
    ContextName,
    ServerUrl,
    Token,
    ClientCert,
    ClientKey,
    Username,
    Password,
    CaCert,
    Alias,
    TagInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddClusterKeyResult {
    Ignored,
    Consumed,
    CloseModal,
    Submit,
}

#[derive(Debug, Clone)]
pub struct AddClusterModalState {
    pub is_open: bool,
    pub active_tab: AddClusterTab,
    pub focused_field: Option<AddClusterFieldId>,
    pub existing_cluster_names: HashSet<String>,

    // File tab
    pub file_path: Option<String>,
    pub file_yaml_content: Option<String>,
    pub file_error: Option<String>,

    // YAML tab
    pub yaml_content: String,
    pub yaml_error: Option<String>,

    // Manual endpoint tab
    pub manual: ManualEndpointConfig,
    pub manual_error: Option<String>,

    // Discovery & selection
    pub discovered_contexts: Vec<DiscoveredContextInfo>,
    pub selected_contexts: Vec<String>,

    // Customization
    pub customization_expanded: bool,
    pub alias: String,
    pub color: Option<String>,
    pub tags: Vec<String>,
    pub tag_input: String,

    // Submission
    pub switch_context: bool,
    pub is_submitting: bool,
    pub submit_error: Option<String>,
}

impl Default for AddClusterModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: AddClusterTab::File,
            focused_field: None,
            existing_cluster_names: HashSet::new(),
            file_path: None,
            file_yaml_content: None,
            file_error: None,
            yaml_content: String::new(),
            yaml_error: None,
            manual: ManualEndpointConfig::default(),
            manual_error: None,
            discovered_contexts: Vec::new(),
            selected_contexts: Vec::new(),
            customization_expanded: false,
            alias: String::new(),
            color: None,
            tags: Vec::new(),
            tag_input: String::new(),
            switch_context: true,
            is_submitting: false,
            submit_error: None,
        }
    }
}

impl AddClusterModalState {
    pub fn open(&mut self, existing_clusters: &[ClusterInfo]) {
        let existing_cluster_names = existing_clusters
            .iter()
            .map(|c| c.name.clone())
            .collect();
        *self = Self {
            is_open: true,
            existing_cluster_names,
            switch_context: true,
            ..Self::default()
        };
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.is_submitting = false;
        self.focused_field = None;
    }

    pub fn set_tab(&mut self, tab: AddClusterTab) {
        self.active_tab = tab;
        self.submit_error = None;
        self.focused_field = match tab {
            AddClusterTab::File => None,
            AddClusterTab::Yaml => Some(AddClusterFieldId::YamlContent),
            AddClusterTab::Manual => Some(AddClusterFieldId::ClusterName),
        };
        self.recompute_discovery();
    }

    pub fn set_file_loaded(&mut self, path: String, result: Result<String, String>) {
        self.file_path = Some(path);
        self.submit_error = None;
        match result {
            Ok(content) => {
                self.file_yaml_content = Some(content);
                self.file_error = None;
            }
            Err(err) => {
                self.file_yaml_content = None;
                self.file_error = Some(err);
            }
        }
        self.recompute_discovery();
    }

    pub fn set_yaml_content(&mut self, content: String) {
        self.yaml_content = content;
        self.submit_error = None;
        self.recompute_discovery();
    }

    pub fn set_manual_auth_type(&mut self, auth_type: ManualAuthType) {
        self.manual.auth_type = auth_type;
        self.submit_error = None;
        self.recompute_discovery();
    }

    pub fn toggle_insecure_skip_tls(&mut self) {
        self.manual.insecure_skip_tls_verify = !self.manual.insecure_skip_tls_verify;
        self.recompute_discovery();
    }

    pub fn is_context_existing(&self, name: &str) -> bool {
        self.existing_cluster_names.contains(name)
    }

    pub fn toggle_context(&mut self, name: &str) {
        if self.is_context_existing(name) {
            return;
        }
        if let Some(pos) = self.selected_contexts.iter().position(|c| c == name) {
            self.selected_contexts.remove(pos);
        } else {
            self.selected_contexts.push(name.to_string());
        }
    }

    pub fn select_all_contexts(&mut self) {
        self.selected_contexts = self
            .discovered_contexts
            .iter()
            .filter(|c| !self.existing_cluster_names.contains(&c.name))
            .map(|c| c.name.clone())
            .collect();
    }

    pub fn deselect_all_contexts(&mut self) {
        self.selected_contexts.clear();
    }

    pub fn recompute_discovery(&mut self) {
        self.discovered_contexts.clear();
        self.selected_contexts.clear();

        match self.active_tab {
            AddClusterTab::File => {
                let Some(ref yaml) = self.file_yaml_content else {
                    return;
                };
                match parse_kubeconfig_contexts(yaml) {
                    Ok(contexts) => {
                        self.file_error = None;
                        self.apply_discovered_contexts(contexts);
                    }
                    Err(err) => {
                        self.file_error = Some(err);
                    }
                }
            }
            AddClusterTab::Yaml => {
                if self.yaml_content.trim().is_empty() {
                    self.yaml_error = None;
                    return;
                }
                match parse_kubeconfig_contexts(&self.yaml_content) {
                    Ok(contexts) => {
                        self.yaml_error = None;
                        self.apply_discovered_contexts(contexts);
                    }
                    Err(err) => {
                        self.yaml_error = Some(err);
                    }
                }
            }
            AddClusterTab::Manual => {
                if self.manual.cluster_name.trim().is_empty()
                    && self.manual.server_url.trim().is_empty()
                {
                    self.manual_error = None;
                    return;
                }
                match build_kubeconfig_from_endpoint(&self.manual) {
                    Ok(yaml) => match parse_kubeconfig_contexts(&yaml) {
                        Ok(contexts) => {
                            self.manual_error = None;
                            self.apply_discovered_contexts(contexts);
                        }
                        Err(err) => {
                            self.manual_error = Some(err);
                        }
                    },
                    Err(err) => {
                        self.manual_error = Some(err);
                    }
                }
            }
        }
    }

    fn apply_discovered_contexts(&mut self, contexts: Vec<DiscoveredContextInfo>) {
        self.selected_contexts = contexts
            .iter()
            .filter(|c| !self.existing_cluster_names.contains(&c.name))
            .map(|c| c.name.clone())
            .collect();
        self.discovered_contexts = contexts;
    }

    pub fn can_submit(&self) -> bool {
        if self.is_submitting || self.selected_contexts.is_empty() {
            return false;
        }
        match self.active_tab {
            AddClusterTab::File => self.file_error.is_none() && self.file_yaml_content.is_some(),
            AddClusterTab::Yaml => self.yaml_error.is_none() && !self.yaml_content.trim().is_empty(),
            AddClusterTab::Manual => self.manual_error.is_none(),
        }
    }

    pub fn submit_label(&self) -> String {
        if self.is_submitting {
            "Adding...".to_string()
        } else if self.selected_contexts.len() > 1 {
            format!("Add {} Clusters", self.selected_contexts.len())
        } else {
            "Add Cluster".to_string()
        }
    }

    pub fn add_tag(&mut self, raw: &str) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return;
        }
        if !self.tags.iter().any(|t| t == trimmed) {
            self.tags.push(trimmed.to_string());
        }
    }

    pub fn commit_tag_input(&mut self) {
        let raw = std::mem::take(&mut self.tag_input);
        self.add_tag(&raw);
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn build_request(&self) -> Result<AddClusterRequest, String> {
        if self.selected_contexts.is_empty() {
            return Err("Select at least one context to add".to_string());
        }

        let source = match self.active_tab {
            AddClusterTab::File => {
                let path = self
                    .file_path
                    .clone()
                    .ok_or_else(|| "Select a kubeconfig file first".to_string())?;
                AddClusterSource::FilePath(path)
            }
            AddClusterTab::Yaml => {
                if self.yaml_content.trim().is_empty() {
                    return Err("Paste kubeconfig YAML content first".to_string());
                }
                AddClusterSource::YamlContent(self.yaml_content.clone())
            }
            AddClusterTab::Manual => {
                let yaml = build_kubeconfig_from_endpoint(&self.manual)?;
                AddClusterSource::YamlContent(yaml)
            }
        };

        let alias_opt = {
            let trimmed = self.alias.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };
        let customization = if alias_opt.is_some() || !self.tags.is_empty() || self.color.is_some()
        {
            Some(ClusterCustomization {
                alias: alias_opt,
                tags: self.tags.clone(),
                color: self.color.clone(),
                description: None,
            })
        } else {
            None
        };

        Ok(AddClusterRequest {
            source,
            selected_contexts: self.selected_contexts.clone(),
            customization,
            switch_context: self.switch_context,
        })
    }

    pub fn field_text(&self, field: AddClusterFieldId) -> String {
        match field {
            AddClusterFieldId::YamlContent => self.yaml_content.clone(),
            AddClusterFieldId::ClusterName => self.manual.cluster_name.clone(),
            AddClusterFieldId::ContextName => self.manual.context_name.clone(),
            AddClusterFieldId::ServerUrl => self.manual.server_url.clone(),
            AddClusterFieldId::Token => self.manual.token.clone().unwrap_or_default(),
            AddClusterFieldId::ClientCert => {
                self.manual.client_cert_data.clone().unwrap_or_default()
            }
            AddClusterFieldId::ClientKey => self.manual.client_key_data.clone().unwrap_or_default(),
            AddClusterFieldId::Username => self.manual.username.clone().unwrap_or_default(),
            AddClusterFieldId::Password => self.manual.password.clone().unwrap_or_default(),
            AddClusterFieldId::CaCert => self.manual.ca_cert_data.clone().unwrap_or_default(),
            AddClusterFieldId::Alias => self.alias.clone(),
            AddClusterFieldId::TagInput => self.tag_input.clone(),
        }
    }

    fn mutate_field_text<F>(&mut self, field: AddClusterFieldId, f: F)
    where
        F: FnOnce(&mut String),
    {
        match field {
            AddClusterFieldId::YamlContent => f(&mut self.yaml_content),
            AddClusterFieldId::ClusterName => f(&mut self.manual.cluster_name),
            AddClusterFieldId::ContextName => f(&mut self.manual.context_name),
            AddClusterFieldId::ServerUrl => f(&mut self.manual.server_url),
            AddClusterFieldId::Token => {
                let mut val = self.manual.token.take().unwrap_or_default();
                f(&mut val);
                self.manual.token = (!val.is_empty()).then_some(val);
            }
            AddClusterFieldId::ClientCert => {
                let mut val = self.manual.client_cert_data.take().unwrap_or_default();
                f(&mut val);
                self.manual.client_cert_data = (!val.is_empty()).then_some(val);
            }
            AddClusterFieldId::ClientKey => {
                let mut val = self.manual.client_key_data.take().unwrap_or_default();
                f(&mut val);
                self.manual.client_key_data = (!val.is_empty()).then_some(val);
            }
            AddClusterFieldId::Username => {
                let mut val = self.manual.username.take().unwrap_or_default();
                f(&mut val);
                self.manual.username = (!val.is_empty()).then_some(val);
            }
            AddClusterFieldId::Password => {
                let mut val = self.manual.password.take().unwrap_or_default();
                f(&mut val);
                self.manual.password = (!val.is_empty()).then_some(val);
            }
            AddClusterFieldId::CaCert => {
                let mut val = self.manual.ca_cert_data.take().unwrap_or_default();
                f(&mut val);
                self.manual.ca_cert_data = (!val.is_empty()).then_some(val);
            }
            AddClusterFieldId::Alias => f(&mut self.alias),
            AddClusterFieldId::TagInput => f(&mut self.tag_input),
        }

        self.submit_error = None;
        if !matches!(
            field,
            AddClusterFieldId::Alias | AddClusterFieldId::TagInput
        ) {
            self.recompute_discovery();
        }
    }

    pub fn append_text_to_focused_field(&mut self, text: &str) {
        let Some(field) = self.focused_field else {
            return;
        };
        self.mutate_field_text(field, |buf| buf.push_str(text));
    }

    pub fn backspace_focused_field(&mut self) {
        let Some(field) = self.focused_field else {
            return;
        };
        self.mutate_field_text(field, |buf| {
            buf.pop();
        });
    }

    pub fn handle_key_input(
        &mut self,
        key: &str,
        key_char: Option<&str>,
    ) -> AddClusterKeyResult {
        if !self.is_open {
            return AddClusterKeyResult::Ignored;
        }
        match key {
            "escape" => {
                if !self.is_submitting {
                    AddClusterKeyResult::CloseModal
                } else {
                    AddClusterKeyResult::Consumed
                }
            }
            "backspace" => {
                self.backspace_focused_field();
                AddClusterKeyResult::Consumed
            }
            "enter" => match self.focused_field {
                Some(AddClusterFieldId::TagInput) => {
                    self.commit_tag_input();
                    AddClusterKeyResult::Consumed
                }
                Some(AddClusterFieldId::YamlContent) => {
                    self.append_text_to_focused_field("\n");
                    AddClusterKeyResult::Consumed
                }
                _ => {
                    if self.can_submit() {
                        AddClusterKeyResult::Submit
                    } else {
                        AddClusterKeyResult::Consumed
                    }
                }
            },
            "space" => {
                self.append_text_to_focused_field(" ");
                AddClusterKeyResult::Consumed
            }
            _ => {
                if let Some(ch) = key_char
                    && !ch.is_empty()
                    && !ch.chars().any(char::is_control)
                {
                    self.append_text_to_focused_field(ch);
                    return AddClusterKeyResult::Consumed;
                }
                AddClusterKeyResult::Consumed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cluster(name: &str) -> ClusterInfo {
        ClusterInfo {
            id: name.to_string(),
            name: name.to_string(),
            status: "disconnected".to_string(),
            alias: None,
            tags: Vec::new(),
            color: None,
            description: None,
            server_url: None,
        }
    }

    #[test]
    fn test_add_cluster_modal_state_yaml_discovery_excludes_existing_contexts_by_default() {
        let mut state = AddClusterModalState::default();
        state.open(&[sample_cluster("existing-ctx")]);
        state.set_tab(AddClusterTab::Yaml);

        let yaml = r#"
apiVersion: v1
kind: Config
clusters:
  - name: c1
    cluster:
      server: https://10.0.0.1:6443
  - name: c2
    cluster:
      server: https://10.0.0.2:6443
contexts:
  - name: existing-ctx
    context:
      cluster: c1
      user: u1
  - name: new-ctx
    context:
      cluster: c2
      user: u2
users:
  - name: u1
    user:
      token: t1
  - name: u2
    user:
      token: t2
"#;
        state.set_yaml_content(yaml.to_string());

        assert_eq!(state.discovered_contexts.len(), 2);
        assert!(state.is_context_existing("existing-ctx"));
        assert!(!state.is_context_existing("new-ctx"));
        assert_eq!(state.selected_contexts, vec!["new-ctx".to_string()]);
        assert!(state.can_submit());

        // Toggling existing context must remain a no-op
        state.toggle_context("existing-ctx");
        assert_eq!(state.selected_contexts, vec!["new-ctx".to_string()]);
    }

    #[test]
    fn test_add_cluster_modal_state_manual_endpoint_builds_valid_request() {
        let mut state = AddClusterModalState::default();
        state.open(&[]);
        state.set_tab(AddClusterTab::Manual);

        state.focused_field = Some(AddClusterFieldId::ClusterName);
        state.append_text_to_focused_field("prod-cluster");
        state.focused_field = Some(AddClusterFieldId::ContextName);
        state.append_text_to_focused_field("prod-ctx");
        state.focused_field = Some(AddClusterFieldId::ServerUrl);
        state.append_text_to_focused_field("api.prod.example.com:6443");
        state.focused_field = Some(AddClusterFieldId::Token);
        state.append_text_to_focused_field("secret-token");

        assert!(state.manual_error.is_none());
        assert_eq!(state.selected_contexts, vec!["prod-ctx".to_string()]);
        assert!(state.can_submit());

        let req = state.build_request().expect("manual request should build");
        assert_eq!(req.selected_contexts, vec!["prod-ctx".to_string()]);
        assert!(req.switch_context);
        match req.source {
            AddClusterSource::YamlContent(yaml) => {
                assert!(yaml.contains("https://api.prod.example.com:6443"));
            }
            AddClusterSource::FilePath(_) => panic!("expected YamlContent for manual endpoint"),
        }
    }

    #[test]
    fn test_add_cluster_modal_state_keyboard_editing_tags_and_duplicate_detection() {
        let mut state = AddClusterModalState::default();
        state.open(&[sample_cluster("taken")]);
        state.set_tab(AddClusterTab::Yaml);

        state.focused_field = Some(AddClusterFieldId::TagInput);
        state.handle_key_input("p", Some("p"));
        state.handle_key_input("r", Some("r"));
        state.handle_key_input("o", Some("o"));
        state.handle_key_input("d", Some("d"));
        assert_eq!(
            state.handle_key_input("enter", None),
            AddClusterKeyResult::Consumed
        );
        assert_eq!(state.tags, vec!["prod".to_string()]);

        // Duplicate tag ignored
        state.add_tag("prod");
        assert_eq!(state.tags.len(), 1);

        // Backspace on alias
        state.focused_field = Some(AddClusterFieldId::Alias);
        state.append_text_to_focused_field("MyAlias!");
        state.handle_key_input("backspace", None);
        assert_eq!(state.alias, "MyAlias");

        state.remove_tag("prod");
        assert!(state.tags.is_empty());

        assert_eq!(
            state.handle_key_input("escape", None),
            AddClusterKeyResult::CloseModal
        );
    }
}
