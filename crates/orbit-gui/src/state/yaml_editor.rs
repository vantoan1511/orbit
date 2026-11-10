#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlValidationResult {
    pub is_valid: bool,
    pub detected_kind: Option<String>,
    pub detected_name: Option<String>,
    pub detected_namespace: Option<String>,
    pub error_message: Option<String>,
}

pub fn validate_k8s_manifest_yaml(yaml: &str) -> YamlValidationResult {
    let parsed: serde_json::Value = match serde_yaml::from_str(yaml) {
        Ok(v) => v,
        Err(e) => {
            return YamlValidationResult {
                is_valid: false,
                detected_kind: None,
                detected_name: None,
                detected_namespace: None,
                error_message: Some(format!("YAML syntax error: {}", e)),
            };
        }
    };

    let Some(obj) = parsed.as_object() else {
        return YamlValidationResult {
            is_valid: false,
            detected_kind: None,
            detected_name: None,
            detected_namespace: None,
            error_message: Some("Manifest must be a YAML mapping/object".to_string()),
        };
    };

    let api_version = obj.get("apiVersion").and_then(|v| v.as_str());
    let kind = obj.get("kind").and_then(|v| v.as_str());

    if api_version.is_none() || kind.is_none() {
        return YamlValidationResult {
            is_valid: false,
            detected_kind: kind.map(String::from),
            detected_name: None,
            detected_namespace: None,
            error_message: Some("Missing required Kubernetes manifest field: apiVersion or kind".to_string()),
        };
    }

    let metadata = obj.get("metadata").and_then(|v| v.as_object());
    let name = metadata.and_then(|m| m.get("name")).and_then(|v| v.as_str()).map(String::from);
    let namespace = metadata.and_then(|m| m.get("namespace")).and_then(|v| v.as_str()).map(String::from);

    YamlValidationResult {
        is_valid: true,
        detected_kind: kind.map(String::from),
        detected_name: name,
        detected_namespace: namespace,
        error_message: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlEditorState {
    pub content: String,
    pub target_kind: String,
    pub target_name: String,
    pub target_namespace: Option<String>,
    pub is_dirty: bool,
    pub validation: YamlValidationResult,
}

impl YamlEditorState {
    pub fn new(content: String, target_kind: String, target_name: String, target_namespace: Option<String>) -> Self {
        let validation = validate_k8s_manifest_yaml(&content);
        Self {
            content,
            target_kind,
            target_name,
            target_namespace,
            is_dirty: false,
            validation,
        }
    }

    pub fn update_content(&mut self, new_content: String) {
        self.validation = validate_k8s_manifest_yaml(&new_content);
        self.content = new_content;
        self.is_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_k8s_manifest_yaml_extracts_kind_name_namespace_and_catches_syntax_errors() {
        let valid_deployment = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
  namespace: default
spec:
  replicas: 2
"#;
        let res_valid = validate_k8s_manifest_yaml(valid_deployment);
        assert!(res_valid.is_valid);
        assert_eq!(res_valid.detected_kind.as_deref(), Some("Deployment"));
        assert_eq!(res_valid.detected_name.as_deref(), Some("nginx"));
        assert_eq!(res_valid.detected_namespace.as_deref(), Some("default"));
        assert!(res_valid.error_message.is_none());

        // Missing apiVersion or kind
        let missing_kind = r#"
metadata:
  name: nginx
"#;
        let res_missing = validate_k8s_manifest_yaml(missing_kind);
        assert!(!res_missing.is_valid);
        assert!(res_missing.error_message.unwrap().contains("Missing required"));

        // Syntax error (invalid indentation)
        let invalid_indentation = r#"
apiVersion: apps/v1
kind: Deployment
  metadata:
 name: bad-indent
"#;
        let res_syntax = validate_k8s_manifest_yaml(invalid_indentation);
        assert!(!res_syntax.is_valid);
        assert!(res_syntax.error_message.unwrap().contains("YAML syntax error"));
    }
}
