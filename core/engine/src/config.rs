use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OrbitConfig {
    #[serde(default)]
    pub custom_kubeconfig_paths: Vec<String>,
}

impl OrbitConfig {
    /// Returns the path to the Orbit configuration directory (`~/.orbit`).
    pub fn config_dir() -> Option<PathBuf> {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
        let mut path = PathBuf::from(home);
        path.push(".orbit");
        Some(path)
    }

    /// Returns the path to the Orbit configuration file (`~/.orbit/config.json`).
    pub fn file_path() -> Option<PathBuf> {
        let mut path = Self::config_dir()?;
        path.push("config.json");
        Some(path)
    }

    /// Load the configuration from disk, or return default if it does not exist or fails to parse.
    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            return Self::default();
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(err) => {
                    tracing::warn!(path = ?path, error = %err, "Failed to parse Orbit configuration file");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Save the configuration to disk at `~/.orbit/config.json`.
    pub fn save(&self) -> Result<(), String> {
        let config_dir = Self::config_dir()
            .ok_or_else(|| "Could not determine user home directory for config storage".to_string())?;

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory {:?}: {}", config_dir, e))?;
        }

        let file_path = Self::file_path()
            .ok_or_else(|| "Could not determine configuration file path".to_string())?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize Orbit configuration: {}", e))?;

        std::fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write configuration file {:?}: {}", file_path, e))?;

        Ok(())
    }

    /// Normalizes a file path string for cross-platform comparisons (canonicalizing slashes and casing).
    pub fn normalize_path(path_str: &str) -> String {
        if let Ok(canonical) = std::fs::canonicalize(path_str) {
            let s = canonical.to_string_lossy().to_string();
            let clean = if let Some(stripped) = s.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                s
            };
            clean.replace('\\', "/")
        } else {
            path_str.replace('\\', "/")
        }
    }

    /// Add a custom kubeconfig file path to the configuration and save it.
    /// If the path is already present, it will not be duplicated.
    pub fn add_kubeconfig_path(&mut self, path: &str) -> Result<(), String> {
        let norm_path = Self::normalize_path(path);
        if !self.custom_kubeconfig_paths.iter().any(|p| Self::normalize_path(p).eq_ignore_ascii_case(&norm_path)) {
            self.custom_kubeconfig_paths.push(norm_path);
            self.save()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_config_default() {
        let config = OrbitConfig::default();
        assert!(config.custom_kubeconfig_paths.is_empty());
    }

    #[test]
    fn test_orbit_config_serde() {
        let json = r#"{"custom_kubeconfig_paths": ["/path/to/kubeconfig.yaml"]}"#;
        let config: OrbitConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.custom_kubeconfig_paths.len(), 1);
        assert_eq!(config.custom_kubeconfig_paths[0], "/path/to/kubeconfig.yaml");

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("/path/to/kubeconfig.yaml"));
    }

    #[test]
    fn test_add_kubeconfig_path_unique_in_memory() {
        let mut config = OrbitConfig::default();
        let path1 = "C:\\Users\\test\\.kube\\config";
        let path2 = "C:/Users/test/.kube/config";
        let norm1 = OrbitConfig::normalize_path(path1);
        let norm2 = OrbitConfig::normalize_path(path2);
        assert_eq!(norm1, norm2);

        config.custom_kubeconfig_paths.push(norm1.clone());
        if !config.custom_kubeconfig_paths.iter().any(|p| OrbitConfig::normalize_path(p).eq_ignore_ascii_case(&norm2)) {
            config.custom_kubeconfig_paths.push(norm2);
        }
        assert_eq!(config.custom_kubeconfig_paths.len(), 1);
    }
}
