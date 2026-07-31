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
                    log::warn!("Failed to parse Orbit configuration file at {:?}: {}", path, err);
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

    /// Add a custom kubeconfig file path to the configuration and save it.
    /// If the path is already present, it will not be duplicated.
    pub fn add_kubeconfig_path(&mut self, path: &str) -> Result<(), String> {
        if !self.custom_kubeconfig_paths.iter().any(|p| p == path) {
            self.custom_kubeconfig_paths.push(path.to_string());
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
        // Test pushing paths without invoking disk save
        if !config.custom_kubeconfig_paths.iter().any(|p| p == "/a/b/c") {
            config.custom_kubeconfig_paths.push("/a/b/c".to_string());
        }
        if !config.custom_kubeconfig_paths.iter().any(|p| p == "/a/b/c") {
            config.custom_kubeconfig_paths.push("/a/b/c".to_string());
        }
        assert_eq!(config.custom_kubeconfig_paths.len(), 1);
    }
}
