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
    #[cfg(debug_assertions)]
    Showcase,
}

pub const MAX_OPEN_TABS: usize = 4;
pub const RESOURCES_TAB_ID: &str = "tab-resources";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTab {
    pub id: String,
    pub title: String,
    pub content: TabContent,
}

impl WorkspaceTab {
    pub fn pinned(&self) -> bool {
        matches!(self.content, TabContent::Dashboard)
    }
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

        while self.tabs.len() >= MAX_OPEN_TABS {
            if let Some(pos) = self
                .tabs
                .iter()
                .position(|t| !t.pinned() && self.active_tab_id.as_deref() != Some(&t.id))
            {
                self.tabs.remove(pos);
            } else {
                break;
            }
        }

        let new_id = tab.id.clone();
        self.tabs.push(tab);
        self.active_tab_id = Some(new_id);
    }

    pub fn open_resource(&mut self, kind: ResourceKind, title: String) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == RESOURCES_TAB_ID) {
            tab.content = TabContent::ResourceTable(kind);
            tab.title = title;
            self.active_tab_id = Some(RESOURCES_TAB_ID.to_string());
        } else {
            self.open_tab(WorkspaceTab {
                id: RESOURCES_TAB_ID.to_string(),
                title,
                content: TabContent::ResourceTable(kind),
            });
        }
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
    fn test_open_resource_reuses_single_tab() {
        let mut mgr = TabManager::new();
        mgr.open_resource(ResourceKind::Pod, "Pods".to_string());
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.tabs[0].id, RESOURCES_TAB_ID);
        assert_eq!(mgr.tabs[0].title, "Pods");
        assert_eq!(mgr.tabs[0].content, TabContent::ResourceTable(ResourceKind::Pod));
        assert_eq!(mgr.active_tab_id.as_deref(), Some(RESOURCES_TAB_ID));

        mgr.open_resource(ResourceKind::Deployment, "Deployments".to_string());
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.tabs[0].id, RESOURCES_TAB_ID);
        assert_eq!(mgr.tabs[0].title, "Deployments");
        assert_eq!(mgr.tabs[0].content, TabContent::ResourceTable(ResourceKind::Deployment));
        assert_eq!(mgr.active_tab_id.as_deref(), Some(RESOURCES_TAB_ID));

        mgr.open_resource(ResourceKind::Service, "Services".to_string());
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.tabs[0].id, RESOURCES_TAB_ID);
        assert_eq!(mgr.tabs[0].title, "Services");
        assert_eq!(mgr.tabs[0].content, TabContent::ResourceTable(ResourceKind::Service));
        assert_eq!(mgr.active_tab_id.as_deref(), Some(RESOURCES_TAB_ID));
    }

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

    #[cfg(debug_assertions)]
    #[test]
    fn test_showcase_tab_open_deduplication() {
        let mut mgr = TabManager::new();
        let tab = WorkspaceTab {
            id: "tab-showcase".to_string(),
            title: "UI Showcase".to_string(),
            content: TabContent::Showcase,
        };
        mgr.open_tab(tab.clone());
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-showcase"));

        // Opening duplicate focuses existing
        let tab2 = WorkspaceTab {
            id: "tab-showcase-2".to_string(),
            title: "UI Showcase".to_string(),
            content: TabContent::Showcase,
        };
        mgr.open_tab(tab2);
        assert_eq!(mgr.tabs.len(), 1);
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-showcase"));
    }

    #[test]
    fn test_open_tab_caps_at_four_and_evicts_oldest_unpinned() {
        let mut mgr = TabManager::new();
        // Open Overview (pinned)
        mgr.open_tab(WorkspaceTab {
            id: "tab-overview".to_string(),
            title: "Overview".to_string(),
            content: TabContent::Dashboard,
        });
        // Open Settings
        mgr.open_tab(WorkspaceTab {
            id: "tab-settings".to_string(),
            title: "Settings".to_string(),
            content: TabContent::Settings,
        });
        // Open Logs(a)
        mgr.open_tab(WorkspaceTab {
            id: "tab-logs-a".to_string(),
            title: "Logs: nginx-a".to_string(),
            content: TabContent::LogViewer {
                namespace: "default".to_string(),
                pod: "nginx-a".to_string(),
                container: None,
            },
        });
        // Open Yaml(a)
        mgr.open_tab(WorkspaceTab {
            id: "tab-yaml-a".to_string(),
            title: "YAML: pod-a".to_string(),
            content: TabContent::YamlEditor {
                kind: "Pod".to_string(),
                name: "pod-a".to_string(),
                namespace: Some("default".to_string()),
            },
        });
        assert_eq!(mgr.tabs.len(), 4);

        // Open Logs(b) - 5th tab
        mgr.open_tab(WorkspaceTab {
            id: "tab-logs-b".to_string(),
            title: "Logs: nginx-b".to_string(),
            content: TabContent::LogViewer {
                namespace: "default".to_string(),
                pod: "nginx-b".to_string(),
                container: None,
            },
        });

        assert_eq!(mgr.tabs.len(), 4);
        assert!(mgr.tabs.iter().any(|t| t.id == "tab-overview"));
        assert!(!mgr.tabs.iter().any(|t| t.id == "tab-settings"));
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-logs-b"));
    }

    #[test]
    fn test_active_and_overview_never_evicted() {
        let mut mgr = TabManager::new();
        mgr.open_tab(WorkspaceTab {
            id: "tab-overview".to_string(),
            title: "Overview".to_string(),
            content: TabContent::Dashboard,
        });
        mgr.open_tab(WorkspaceTab {
            id: "tab-a".to_string(),
            title: "Tab A".to_string(),
            content: TabContent::Settings,
        });
        mgr.open_tab(WorkspaceTab {
            id: "tab-b".to_string(),
            title: "Tab B".to_string(),
            content: TabContent::LogViewer {
                namespace: "default".to_string(),
                pod: "pod-b".to_string(),
                container: None,
            },
        });
        mgr.open_tab(WorkspaceTab {
            id: "tab-c".to_string(),
            title: "Tab C".to_string(),
            content: TabContent::YamlEditor {
                kind: "Pod".to_string(),
                name: "pod-c".to_string(),
                namespace: Some("default".to_string()),
            },
        });
        assert_eq!(mgr.tabs.len(), 4);

        // Select Tab A so it becomes active
        mgr.select_tab("tab-a");
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-a"));

        // Open Tab D: active (tab-a) and Overview must not be evicted; tab-b is oldest unpinned & non-active
        mgr.open_tab(WorkspaceTab {
            id: "tab-d".to_string(),
            title: "Tab D".to_string(),
            content: TabContent::LogViewer {
                namespace: "default".to_string(),
                pod: "pod-d".to_string(),
                container: None,
            },
        });

        assert_eq!(mgr.tabs.len(), 4);
        assert!(mgr.tabs.iter().any(|t| t.id == "tab-overview"));
        assert!(mgr.tabs.iter().any(|t| t.id == "tab-a"));
        assert!(!mgr.tabs.iter().any(|t| t.id == "tab-b"));
        assert!(mgr.tabs.iter().any(|t| t.id == "tab-c"));
        assert!(mgr.tabs.iter().any(|t| t.id == "tab-d"));
        assert_eq!(mgr.active_tab_id.as_deref(), Some("tab-d"));
    }
}
