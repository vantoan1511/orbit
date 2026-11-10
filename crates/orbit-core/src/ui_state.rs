use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

impl Default for WindowBounds {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1280,
            height: 800,
            maximized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedTab {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub resource_name: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UiStateStore {
    pub selected_namespace: Option<String>,
    pub active_tab_id: Option<String>,
    pub tabs: Vec<PersistedTab>,
    pub table_filters: HashMap<String, String>,
    pub window_bounds: WindowBounds,
    pub notifications: Vec<String>,
}

impl UiStateStore {
    pub fn default_path() -> PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".orbit").join("state.json")
    }

    pub fn load_from_path(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let temp_path = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
        std::fs::write(&temp_path, json).map_err(|e| e.to_string())?;
        std::fs::rename(&temp_path, path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_store_roundtrip_and_corrupt_fallback() {
        let temp_dir = std::env::temp_dir().join(format!("orbit_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");
        let path = temp_dir.join("state.json");

        let store = UiStateStore {
            selected_namespace: Some("kube-system".to_string()),
            active_tab_id: Some("tab-pods".to_string()),
            tabs: vec![PersistedTab {
                id: "tab-pods".to_string(),
                title: "Pods".to_string(),
                kind: "Pod".to_string(),
                resource_name: None,
                namespace: Some("default".to_string()),
            }],
            table_filters: [("Pod".to_string(), "nginx".to_string())]
                .into_iter()
                .collect(),
            window_bounds: WindowBounds {
                x: 150,
                y: 200,
                width: 1440,
                height: 900,
                maximized: true,
            },
            notifications: Vec::new(),
        };

        // Roundtrip save & load
        store.save_to_path(&path).expect("save state");
        let loaded = UiStateStore::load_from_path(&path);
        assert_eq!(store, loaded);

        // Corrupt file fallback
        std::fs::write(&path, "{ invalid_json: true").expect("write corrupt state");
        let fallback = UiStateStore::load_from_path(&path);
        assert_eq!(fallback, UiStateStore::default());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
