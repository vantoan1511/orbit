use orbit_core::command::ResourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabContent {
    Dashboard,
    ResourceTable(ResourceKind),
    LogViewer {
        namespace: String,
        pod: String,
        container: Option<String>,
    },
    YamlEditor {
        kind: String,
        name: String,
        namespace: Option<String>,
    },
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTab {
    pub id: String,
    pub title: String,
    pub content: TabContent,
}

#[derive(Debug, Clone, Default)]
pub struct TabManager {
    pub tabs: Vec<WorkspaceTab>,
    pub active_tab_id: Option<String>,
}

impl TabManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_tab(&mut self, tab: WorkspaceTab) {
        if let Some(existing) = self.tabs.iter().find(|t| t.id == tab.id || t.content == tab.content) {
            self.active_tab_id = Some(existing.id.clone());
            return;
        }

        let new_id = tab.id.clone();
        self.tabs.push(tab);
        self.active_tab_id = Some(new_id);
    }

    pub fn select_tab(&mut self, tab_id: &str) {
        if self.tabs.iter().any(|t| t.id == tab_id) {
            self.active_tab_id = Some(tab_id.to_string());
        }
    }

    pub fn close_tab(&mut self, tab_id: &str) {
        let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) else {
            return;
        };

        let is_active = self.active_tab_id.as_deref() == Some(tab_id);
        self.tabs.remove(pos);

        if self.tabs.is_empty() {
            self.active_tab_id = None;
        } else if is_active {
            let next_pos = if pos < self.tabs.len() {
                pos
            } else {
                pos.saturating_sub(1)
            };
            self.active_tab_id = Some(self.tabs[next_pos].id.clone());
        }
    }

    pub fn active_tab(&self) -> Option<&WorkspaceTab> {
        let active_id = self.active_tab_id.as_ref()?;
        self.tabs.iter().find(|t| &t.id == active_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_manager_open_deduplicates_existing_tab_and_close_selects_neighbor() {
        let mut mgr = TabManager::new();

        let tab_pods = WorkspaceTab {
            id: "tab-pods".to_string(),
            title: "Pods".to_string(),
            content: TabContent::ResourceTable(ResourceKind::Pod),
        };

        let tab_deployments = WorkspaceTab {
            id: "tab-deployments".to_string(),
            title: "Deployments".to_string(),
            content: TabContent::ResourceTable(ResourceKind::Deployment),
        };

        let tab_logs = WorkspaceTab {
            id: "tab-logs-1".to_string(),
            title: "Logs: nginx".to_string(),
            content: TabContent::LogViewer {
                namespace: "default".to_string(),
                pod: "nginx-1".to_string(),
                container: None,
            },
        };

        // Open Pods tab
        mgr.open_tab(tab_pods.clone());
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-pods"));

        // Open Deployments tab
        mgr.open_tab(tab_deployments.clone());
        assert_eq!(mgr.tabs.len(), 2);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-deployments"));

        // Open duplicate Pods tab (should focus existing tab)
        mgr.open_tab(tab_pods.clone());
        assert_eq!(mgr.tabs.len(), 2);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-pods"));

        // Open Logs tab
        mgr.open_tab(tab_logs);
        assert_eq!(mgr.tabs.len(), 3);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-logs-1"));

        // Close active Logs tab -> neighbor (deployments or pods) is focused
        mgr.close_tab("tab-logs-1");
        assert_eq!(mgr.tabs.len(), 2);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-deployments"));

        // Close active deployments tab
        mgr.close_tab("tab-deployments");
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-pods"));

        // Close remaining pods tab -> leaves empty workspace
        mgr.close_tab("tab-pods");
        assert_eq!(mgr.tabs.len(), 0);
        assert_eq!(mgr.active_tab_id, None);
    }
}
