use std::collections::HashMap;
use iced::{
    widget::{button, column, container, horizontal_space, mouse_area, pick_list, row, text},
    Element, Length, Subscription, Task,
};

use orbit_core::command::{CoreCommand, ResourceKind};
use orbit_core::events::CoreEvent;
use orbit_core::ui_state::UiStateStore;

use crate::icons::{button_icon, icon, tab_content_icon, Icon};
use crate::state::tabs::{TabContent, TabManager, WorkspaceTab};
use crate::store::{ClusterStore, FilterCriteria};
use crate::theme::{
    accent_color_from_name, action_button_style, empty_state_container_style, header_badge_style,
    header_container_style, pick_list_style, search_pill_style, status_dot_pill,
    tab_button_style, tab_close_button_style, tab_plus_button_style, tab_strip_container_style,
    titlebar_container_style, window_control_button_style, OrbitPalette,
};
use crate::views::dashboard::dashboard_view;
use crate::views::details_drawer::{details_drawer_view, ResourceAction, SelectedResourceDetails};
use crate::views::modals::{modal_view, ActiveModal, SearchResultItem, SearchResultKind};
use crate::views::navigation::{
    activity_bar_view, get_cluster_display_name, is_local_cluster, sub_sidebar_view,
    SidebarCategory,
};
use crate::views::resource_table::resource_table_view;
use crate::views::settings::{settings_view, SettingsViewState};

pub const ALL_NAMESPACES_LABEL: &str = "All Namespaces";
pub const DOUBLE_CLICK_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(300);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterConnectionStatus {
    Loading,
    Connected(String),
    Disconnected,
}

static COMMAND_TX: std::sync::OnceLock<tokio::sync::mpsc::Sender<CoreCommand>> =
    std::sync::OnceLock::new();

pub fn send_core_command(cmd: CoreCommand) {
    if let Some(tx) = COMMAND_TX.get() {
        let _ = tx.try_send(cmd);
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    CoreEventReceived(CoreEvent),
    SendCommand(CoreCommand),
    SelectCluster(Option<String>),
    SelectNamespace(Option<String>),
    SearchFilterChanged(String),
    OpenTab(WorkspaceTab),
    CloseTab(String),
    SelectTab(String),
    OpenModal(ActiveModal),
    CloseModal,
    SelectResource(Option<SelectedResourceDetails>),
    ExecuteResourceAction(ResourceAction, ResourceKind, String, Option<String>),
    ToggleTheme,
    SearchEverywhereQueryChanged(String),
    SearchEverywhereSelect(usize),
    ToggleCategory(SidebarCategory),
    ClusterFilterChanged(String),
    SelectClusterTag(Option<String>),
    OpenNewTabAction,
    WindowMinimize,
    WindowToggleMaximize,
    WindowClose,
    TitlebarPressed,
    UpdateSetting(String, serde_json::Value),
    UpdateShellInput(String),
    ResetSettingsToDefaults,
}

pub struct OrbitApp {
    pub is_engine_ready: bool,
    pub is_loading_cluster: bool,
    pub has_auto_connected: bool,
    pub palette: OrbitPalette,
    pub cluster_store: ClusterStore,
    pub filter: FilterCriteria,
    pub tab_manager: TabManager,
    pub active_modal: ActiveModal,
    pub settings: HashMap<String, serde_json::Value>,
    pub settings_state: SettingsViewState,
    pub ui_state: UiStateStore,
    pub selected_resource: Option<SelectedResourceDetails>,
    pub active_category: Option<SidebarCategory>,
    pub cluster_filter: String,
    pub selected_cluster_tag: Option<String>,
    pub last_titlebar_click: Option<std::time::Instant>,
}

impl Default for OrbitApp {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbitApp {
    pub fn new() -> Self {
        let mut tab_mgr = TabManager::new();
        tab_mgr.open_tab(WorkspaceTab {
            id: "tab-overview".to_string(),
            title: "Overview".to_string(),
            content: TabContent::Dashboard,
        });

        Self {
            is_engine_ready: false,
            is_loading_cluster: false,
            has_auto_connected: false,
            palette: OrbitPalette::dark(),
            cluster_store: ClusterStore::new(),
            filter: FilterCriteria::default(),
            tab_manager: tab_mgr,
            active_modal: ActiveModal::None,
            settings: HashMap::new(),
            settings_state: SettingsViewState::default(),
            ui_state: UiStateStore::default(),
            selected_resource: None,
            active_category: Some(SidebarCategory::Workloads),
            cluster_filter: String::new(),
            selected_cluster_tag: None,
            last_titlebar_click: None,
        }
    }

    pub fn k8s_version_label(&self) -> String {
        if let Some(node) = self.cluster_store.nodes.first()
            && !node.version.is_empty()
        {
            return format!("K8S {}", node.version);
        }
        "K8S —".to_string()
    }

    pub fn active_cluster_is_local(&self) -> bool {
        if let Some(active_id) = &self.cluster_store.active_cluster_id
            && let Some(cluster) = self.cluster_store.clusters.iter().find(|c| &c.id == active_id)
        {
            return is_local_cluster(cluster);
        }
        false
    }

    pub fn active_cluster_display_name(&self) -> &str {
        if let Some(active_id) = &self.cluster_store.active_cluster_id
            && let Some(cluster) = self.cluster_store.clusters.iter().find(|c| &c.id == active_id)
        {
            return get_cluster_display_name(cluster);
        }
        "No Cluster"
    }

    pub fn empty_state_view<'a>(palette: &'a OrbitPalette) -> Element<'a, Message> {
        let icon_el = icon(Icon::Boxes, 36, palette.text_secondary);
        let title = text("No Open Tabs").size(18).color(palette.text_primary);
        let desc = text("Select a resource from the sidebar or open a new tab to start inspecting your cluster.")
            .size(13)
            .color(palette.text_secondary);
        let action_btn = button(
            row![
                icon(Icon::Plus, 14, palette.text_primary),
                text("Open New Tab").size(13).color(palette.text_primary),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .padding([8, 14]),
        )
        .style(action_button_style(palette, false))
        .on_press(Message::OpenNewTabAction);

        let content = column![icon_el, title, desc, action_btn]
            .spacing(16)
            .align_x(iced::Alignment::Center)
            .padding(32);

        container(content)
            .style(empty_state_container_style(palette))
            .into()
    }

    /// Returns selectable cluster context IDs/names from `self.cluster_store.clusters`.
    pub fn cluster_options(&self) -> Vec<String> {
        self.cluster_store
            .clusters
            .iter()
            .map(|c| c.id.clone())
            .collect()
    }

    /// Returns deduplicated namespace options for the namespace `pick_list`:
    /// `["All Namespaces", "default", "kube-system"]` plus sorted live namespaces from `self.cluster_store.namespaces`.
    pub fn namespace_options(&self) -> Vec<String> {
        let mut options = vec![
            ALL_NAMESPACES_LABEL.to_string(),
            "default".to_string(),
            "kube-system".to_string(),
        ];
        let mut live: Vec<String> = self
            .cluster_store
            .namespaces
            .iter()
            .map(|n| n.name.clone())
            .collect();
        live.sort();
        for ns in live {
            if !options.contains(&ns) {
                options.push(ns);
            }
        }
        if let Some(ref current) = self.filter.namespace
            && !options.contains(current)
        {
            options.push(current.clone());
        }
        options
    }

    /// Returns the currently selected label in the namespace `pick_list`
    /// (`self.filter.namespace.clone().unwrap_or_else(|| ALL_NAMESPACES_LABEL.to_string())`).
    pub fn selected_namespace_label(&self) -> String {
        self.filter
            .namespace
            .clone()
            .unwrap_or_else(|| ALL_NAMESPACES_LABEL.to_string())
    }

    /// Returns the active cluster connection status for the top bar indicator.
    pub fn cluster_connection_status(&self) -> ClusterConnectionStatus {
        if self.is_loading_cluster {
            ClusterConnectionStatus::Loading
        } else if let Some(ref id) = self.cluster_store.active_cluster_id {
            ClusterConnectionStatus::Connected(id.clone())
        } else {
            ClusterConnectionStatus::Disconnected
        }
    }

    pub fn search_everywhere(&self, query: &str) -> Vec<SearchResultItem> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // 1. Navigation items
        let nav_targets = [
            ("Dashboard", "Cluster overview & health", TabContent::Dashboard),
            ("Pods", "Workloads > Pods list", TabContent::ResourceTable(ResourceKind::Pod)),
            ("Deployments", "Workloads > Deployments list", TabContent::ResourceTable(ResourceKind::Deployment)),
            ("StatefulSets", "Workloads > StatefulSets list", TabContent::ResourceTable(ResourceKind::StatefulSet)),
            ("DaemonSets", "Workloads > DaemonSets list", TabContent::ResourceTable(ResourceKind::DaemonSet)),
            ("Nodes", "Infrastructure > Nodes", TabContent::ResourceTable(ResourceKind::Node)),
            ("Services", "Network > Services", TabContent::ResourceTable(ResourceKind::Service)),
            ("Ingresses", "Network > Ingresses", TabContent::ResourceTable(ResourceKind::Ingress)),
            ("ConfigMaps", "Config > ConfigMaps", TabContent::ResourceTable(ResourceKind::ConfigMap)),
            ("Secrets", "Config > Secrets", TabContent::ResourceTable(ResourceKind::Secret)),
            ("Settings", "App preferences & configuration", TabContent::Settings),
        ];

        for (title, sub, content) in nav_targets {
            if title.to_lowercase().contains(&q) || sub.to_lowercase().contains(&q) {
                let tab_id = if content == TabContent::Dashboard {
                    "tab-overview".to_string()
                } else {
                    format!("tab-{}", title.to_lowercase())
                };
                results.push(SearchResultItem {
                    title: title.to_string(),
                    subtitle: sub.to_string(),
                    kind: SearchResultKind::Navigation,
                    target_tab: Some(WorkspaceTab {
                        id: tab_id,
                        title: title.to_string(),
                        content,
                    }),
                });
            }
        }

        // 2. Live Kubernetes resources
        for pod in &self.cluster_store.pods {
            if pod.name.to_lowercase().contains(&q) {
                results.push(SearchResultItem {
                    title: pod.name.clone(),
                    subtitle: format!("Pod · {} · {}", pod.namespace, pod.status),
                    kind: SearchResultKind::Resource(ResourceKind::Pod),
                    target_tab: Some(WorkspaceTab {
                        id: format!("tab-pod-{}", pod.name),
                        title: format!("Pod: {}", pod.name),
                        content: TabContent::ResourceTable(ResourceKind::Pod),
                    }),
                });
            }
        }

        for deploy in &self.cluster_store.deployments {
            if deploy.name.to_lowercase().contains(&q) {
                results.push(SearchResultItem {
                    title: deploy.name.clone(),
                    subtitle: format!("Deployment · {} · {}/{}", deploy.namespace, deploy.replicas.current, deploy.replicas.desired),
                    kind: SearchResultKind::Resource(ResourceKind::Deployment),
                    target_tab: Some(WorkspaceTab {
                        id: format!("tab-deploy-{}", deploy.name),
                        title: format!("Deployment: {}", deploy.name),
                        content: TabContent::ResourceTable(ResourceKind::Deployment),
                    }),
                });
            }
        }

        results
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CoreEventReceived(event) => match event {
                CoreEvent::EngineReady { clusters, active_cluster_id, settings, .. } => {
                    self.is_engine_ready = true;
                    self.cluster_store.clusters = clusters;
                    self.settings = settings;

                    if let Some(theme) = self.settings.get("themeMode").and_then(|v| v.as_str()) {
                        self.palette = if theme == "light" {
                            OrbitPalette::light()
                        } else {
                            OrbitPalette::dark()
                        };
                    }
                    if let Some(accent) = self.settings.get("accentColor").and_then(|v| v.as_str()) {
                        self.palette.accent = accent_color_from_name(accent, self.palette.is_dark);
                    }
                    if let Some(shell) = self.settings.get("defaultTerminalShell").and_then(|v| v.as_str())
                        && self.settings_state.terminal_shell_input.is_empty()
                    {
                        self.settings_state.terminal_shell_input = shell.to_string();
                    }

                    if !self.has_auto_connected {
                        let target = active_cluster_id.or_else(|| {
                            self.cluster_store.clusters.first().map(|c| c.id.clone())
                        });
                        if let Some(target_id) = target {
                            self.has_auto_connected = true;
                            return self.update(Message::SelectCluster(Some(target_id)));
                        } else {
                            self.cluster_store.active_cluster_id = None;
                            self.is_loading_cluster = false;
                        }
                    } else {
                        self.cluster_store.active_cluster_id = active_cluster_id;
                    }
                }
                CoreEvent::ClustersUpdated { clusters } => {
                    self.cluster_store.clusters = clusters;
                    if !self.has_auto_connected
                        && self.cluster_store.active_cluster_id.is_none()
                        && let Some(first) = self.cluster_store.clusters.first().map(|c| c.id.clone())
                    {
                        self.has_auto_connected = true;
                        return self.update(Message::SelectCluster(Some(first)));
                    }
                }
                CoreEvent::ActiveClusterChanged { active_cluster_id } => {
                    self.cluster_store.active_cluster_id = active_cluster_id;
                    self.is_loading_cluster = false;
                }
                CoreEvent::ResourceBatchUpdated(batch) => {
                    self.cluster_store.apply_batch(batch);
                }
                CoreEvent::PodMetricsUpdated { metrics } => {
                    self.cluster_store.apply_pod_metrics(metrics);
                }
                CoreEvent::UserProfileUpdated { profile } => {
                    self.cluster_store.user_profile = Some(profile);
                }
                CoreEvent::ClusterAdded { .. } => {
                    self.active_modal = ActiveModal::None;
                }
                CoreEvent::ClusterAddFailed { error, .. } => {
                    if let ActiveModal::AddCluster { ref mut mode, .. } = self.active_modal {
                        self.active_modal = ActiveModal::AddCluster {
                            mode: mode.clone(),
                            error: Some(error),
                        };
                    }
                }
                _ => {}
            },
            Message::SendCommand(cmd) => {
                send_core_command(cmd);
            }
            Message::SelectCluster(cluster_id) => {
                self.cluster_store.clear_cluster_resources();
                self.cluster_store.active_cluster_id = cluster_id.clone();
                self.filter.namespace = None;
                self.is_loading_cluster = true;
                send_core_command(CoreCommand::SwitchCluster { context_name: cluster_id });
            }
            Message::SelectNamespace(ns) => {
                self.filter.namespace = match ns {
                    Some(s) if s == ALL_NAMESPACES_LABEL || s.is_empty() => None,
                    other => other,
                };
            }
            Message::SearchFilterChanged(query) => {
                self.filter.search_query = query.trim().to_lowercase();
            }
            Message::OpenTab(tab) => {
                self.tab_manager.open_tab(tab);
            }
            Message::CloseTab(tab_id) => {
                self.tab_manager.close_tab(&tab_id);
            }
            Message::SelectTab(tab_id) => {
                self.tab_manager.select_tab(&tab_id);
            }
            Message::OpenModal(modal) => {
                self.active_modal = modal;
            }
            Message::CloseModal => {
                self.active_modal = ActiveModal::None;
            }
            Message::SelectResource(details) => {
                self.selected_resource = details;
            }
            Message::ExecuteResourceAction(_action, _kind, _name, _ns) => {
                // Dispatches corresponding CoreCommand
            }
            Message::ToggleTheme => {
                self.palette = if self.palette.is_dark {
                    OrbitPalette::light()
                } else {
                    OrbitPalette::dark()
                };
            }
            Message::SearchEverywhereQueryChanged(query) => {
                self.active_modal = ActiveModal::SearchEverywhere {
                    query,
                    selected_index: 0,
                };
            }
            Message::SearchEverywhereSelect(idx) => {
                if let ActiveModal::SearchEverywhere { ref query, .. } = self.active_modal {
                    let results = self.search_everywhere(query);
                    if let Some(tab) = results.get(idx).and_then(|item| item.target_tab.as_ref()) {
                        self.tab_manager.open_tab(tab.clone());
                    }
                }
                self.active_modal = ActiveModal::None;
            }
            Message::ToggleCategory(cat) => {
                if self.active_category == Some(cat) {
                    self.active_category = None;
                } else {
                    self.active_category = Some(cat);
                }
            }
            Message::ClusterFilterChanged(query) => {
                self.cluster_filter = query;
            }
            Message::SelectClusterTag(tag) => {
                if self.selected_cluster_tag == tag {
                    self.selected_cluster_tag = None;
                } else {
                    self.selected_cluster_tag = tag;
                }
            }
            Message::OpenNewTabAction => {
                self.active_modal = ActiveModal::SearchEverywhere {
                    query: String::new(),
                    selected_index: 0,
                };
            }
            Message::TitlebarPressed => {
                let now = std::time::Instant::now();
                if self.last_titlebar_click.is_some_and(|last| {
                    now.duration_since(last) < DOUBLE_CLICK_TIMEOUT
                }) {
                    self.last_titlebar_click = None;
                    return iced::window::get_latest().and_then(iced::window::toggle_maximize);
                } else {
                    self.last_titlebar_click = Some(now);
                    return iced::window::get_latest().and_then(iced::window::drag);
                }
            }
            Message::WindowMinimize => {
                return iced::window::get_latest().and_then(|id| iced::window::minimize(id, true));
            }
            Message::WindowToggleMaximize => {
                return iced::window::get_latest().and_then(iced::window::toggle_maximize);
            }
            Message::WindowClose => {
                return iced::window::get_latest().and_then(iced::window::close);
            }
            Message::UpdateShellInput(val) => {
                self.settings_state.terminal_shell_input = val;
            }
            Message::UpdateSetting(key, val) => {
                self.settings.insert(key.clone(), val.clone());
                if key == "themeMode" {
                    self.palette = if val == "light" {
                        OrbitPalette::light()
                    } else {
                        OrbitPalette::dark()
                    };
                } else if key == "accentColor" {
                    let accent_str = val.as_str().unwrap_or("blue");
                    self.palette.accent = accent_color_from_name(accent_str, self.palette.is_dark);
                } else if key == "launchOnStartup" {
                    let enabled = val.as_bool().unwrap_or(false);
                    send_core_command(CoreCommand::SetLaunchOnStartup { enabled });
                }
                send_core_command(CoreCommand::UpdateAppSettings {
                    values: [(key, val)].into_iter().collect(),
                });
                self.settings_state.feedback_message = Some("Settings saved".to_string());
            }
            Message::ResetSettingsToDefaults => {
                let defs = orbit_core::config::Configuration::system_definitions();
                let mut reset_map = HashMap::new();
                for def in defs {
                    if let Some(default_val) = def.default_value {
                        self.settings.insert(def.key.clone(), default_val.clone());
                        reset_map.insert(def.key, default_val);
                    }
                }
                send_core_command(CoreCommand::UpdateAppSettings { values: reset_map });
                self.palette = OrbitPalette::dark();
                self.settings_state.terminal_shell_input = self
                    .settings
                    .get("defaultTerminalShell")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                self.settings_state.feedback_message =
                    Some("Default settings restored".to_string());
            }
        }
        Task::none()
    }

    pub fn titlebar_view(&self) -> Element<'_, Message> {
        let logo_and_title = row![
            icon(Icon::Boxes, 14, self.palette.accent),
            text("Orbit")
                .size(12)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                })
                .color(self.palette.text_primary),
        ]
        .spacing(6)
        .padding([0, 10])
        .align_y(iced::Alignment::Center);

        let drag_area = mouse_area(
            container(horizontal_space())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::TitlebarPressed);

        let window_btn = |icon_variant: Icon, is_close: bool, msg: Message| {
            button(
                container(button_icon(icon_variant, 12))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .width(Length::Fixed(44.0))
            .height(Length::Fixed(32.0))
            .padding(0)
            .style(window_control_button_style(&self.palette, is_close))
            .on_press(msg)
        };

        let controls = row![
            window_btn(Icon::Minus, false, Message::WindowMinimize),
            window_btn(Icon::Square, false, Message::WindowToggleMaximize),
            window_btn(Icon::X, true, Message::WindowClose),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center);

        container(
            row![logo_and_title, drag_area, controls]
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(32.0))
        .style(titlebar_container_style(&self.palette))
        .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // 1. Dual-Rail Activity Bar & Collapsible Sub-Sidebar
        let activity_bar = activity_bar_view(&self.palette, self.active_category);

        let sub_sidebar: Element<'_, Message> = if let Some(cat) = self.active_category {
            sub_sidebar_view(
                &self.palette,
                cat,
                &self.cluster_store.clusters,
                self.cluster_store.active_cluster_id.as_deref(),
                &self.cluster_filter,
                self.selected_cluster_tag.as_deref(),
                self.tab_manager.active_tab().map(|t| &t.content),
            )
        } else {
            horizontal_space().width(Length::Fixed(0.0)).into()
        };

        // 2. Desktop Top Header
        let (is_conn, is_load) = match self.cluster_connection_status() {
            ClusterConnectionStatus::Connected(_) => (true, false),
            ClusterConnectionStatus::Loading => (false, true),
            ClusterConnectionStatus::Disconnected => (false, false),
        };
        let connection_pill = status_dot_pill(
            &self.palette,
            is_conn,
            is_load,
            if is_conn {
                "Connected"
            } else if is_load {
                "Connecting..."
            } else {
                "Disconnected"
            },
        );

        let cluster_name = text(self.active_cluster_display_name())
            .size(12)
            .color(self.palette.text_primary);

        let divider = text("|").size(12).color(self.palette.border);

        let version_badge = container(
            row![
                icon(Icon::Cpu, 11, self.palette.text_secondary),
                text(self.k8s_version_label())
                    .size(11)
                    .color(self.palette.text_secondary),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .padding([2, 6]),
        )
        .style(header_badge_style(&self.palette));

        let mut left_badges = row![
            connection_pill,
            cluster_name,
            divider,
            version_badge,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        if self.cluster_store.active_cluster_id.is_some() {
            let is_local = self.active_cluster_is_local();
            let local_pill = container(
                text(if is_local { "Local" } else { "Remote" })
                    .size(10)
                    .color(if is_local {
                        self.palette.info
                    } else {
                        self.palette.accent
                    }),
            )
            .padding([2, 6])
            .style(header_badge_style(&self.palette));

            left_badges = left_badges.push(local_pill);
        }

        let search_pill = button(
            row![
                icon(Icon::Search, 12, self.palette.text_secondary),
                text("Search everywhere... (Ctrl+K)")
                    .size(12)
                    .color(self.palette.text_secondary),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .style(search_pill_style(&self.palette))
        .padding([4, 16])
        .on_press(Message::OpenModal(ActiveModal::SearchEverywhere {
            query: String::new(),
            selected_index: 0,
        }));

        let namespace_selector = pick_list(
            self.namespace_options(),
            Some(self.selected_namespace_label()),
            |ns| Message::SelectNamespace(Some(ns)),
        )
        .style(pick_list_style(&self.palette))
        .padding([4, 8]);

        let desktop_header_row = row![
            left_badges,
            horizontal_space(),
            search_pill,
            horizontal_space(),
            namespace_selector,
        ]
        .spacing(12)
        .padding([6, 12])
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

        let desktop_header = container(desktop_header_row)
            .width(Length::Fill)
            .style(header_container_style(&self.palette));

        // 3. Document Tab Bar Strip
        let mut tabs_row = row![].spacing(4).padding(2).align_y(iced::Alignment::Center);
        for tab in &self.tab_manager.tabs {
            let is_active = self.tab_manager.active_tab_id.as_deref() == Some(&tab.id);
            let tab_id = tab.id.clone();
            let tab_icon = tab_content_icon(&tab.content);
            let tab_color = if is_active {
                self.palette.text_primary
            } else {
                self.palette.text_secondary
            };

            let tab_btn = button(
                row![
                    icon(tab_icon, 12, tab_color),
                    text(tab.title.clone()).size(12).color(tab_color),
                    button(text("×").size(11))
                        .style(tab_close_button_style(&self.palette))
                        .padding([2, 4])
                        .on_press(Message::CloseTab(tab_id.clone())),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            )
            .style(tab_button_style(&self.palette, is_active))
            .padding([4, 8])
            .on_press(Message::SelectTab(tab_id));

            tabs_row = tabs_row.push(tab_btn);
        }

        let plus_tab_btn = button(
            container(icon(Icon::Plus, 12, self.palette.text_secondary))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .style(tab_plus_button_style(&self.palette))
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .on_press(Message::OpenNewTabAction);

        tabs_row = tabs_row.push(plus_tab_btn);

        let tab_strip = container(tabs_row)
            .width(Length::Fill)
            .padding([2, 8])
            .style(tab_strip_container_style(&self.palette));

        // 4. Main Workspace (or Empty State)
        let main_workspace: Element<'_, Message> = if self.tab_manager.tabs.is_empty() {
            container(Self::empty_state_view(&self.palette))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            match self.tab_manager.active_tab().map(|t| &t.content) {
                Some(TabContent::Dashboard) => dashboard_view(&self.cluster_store, &self.palette),
                Some(TabContent::ResourceTable(kind)) => {
                    resource_table_view(*kind, &self.cluster_store, &self.filter, &self.palette, |d| {
                        Message::SelectResource(Some(d))
                    })
                }
                Some(TabContent::Settings) => settings_view(
                    &self.settings,
                    &self.settings_state,
                    &self.palette,
                    Message::UpdateSetting,
                    Message::UpdateShellInput,
                    Message::ResetSettingsToDefaults,
                ),
                _ => container(Self::empty_state_view(&self.palette))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into(),
            }
        };

        let content_with_drawer: Element<'_, Message> = if let Some(ref details) = self.selected_resource {
            row![
                container(main_workspace).width(Length::Fill).height(Length::Fill),
                details_drawer_view(
                    details,
                    &self.palette,
                    Message::SelectResource(None),
                    Message::ExecuteResourceAction,
                ),
            ]
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            main_workspace
        };

        // 5. Status Bar
        let status_bar = row![
            text(format!(
                "Cluster: {}",
                self.cluster_store.active_cluster_id.as_deref().unwrap_or("Disconnected")
            ))
            .size(11)
            .color(self.palette.text_muted),
            text(format!(
                "Namespace: {}",
                self.filter.namespace.as_deref().unwrap_or("All Namespaces")
            ))
            .size(11)
            .color(self.palette.text_muted),
            text(format!("Pods: {}", self.cluster_store.pods.len()))
                .size(11)
                .color(self.palette.text_muted),
        ]
        .spacing(20)
        .padding(6);

        let main_panel = column![
            desktop_header,
            tab_strip,
            container(content_with_drawer)
                .width(Length::Fill)
                .height(Length::Fill),
            status_bar,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        let full_layout = container(
            column![
                self.titlebar_view(),
                row![activity_bar, sub_sidebar, main_panel]
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(self.palette.bg_app)),
            ..Default::default()
        });

        // 5. Active Modal Overlay
        if self.active_modal != ActiveModal::None {
            let search_results = if let ActiveModal::SearchEverywhere { ref query, .. } = self.active_modal {
                self.search_everywhere(query)
            } else {
                Vec::new()
            };

            if let Some(modal_el) = modal_view(
                &self.active_modal,
                &self.palette,
                Message::CloseModal,
                Message::SearchEverywhereQueryChanged,
                Message::SearchEverywhereSelect,
                search_results,
            ) {
                return modal_el;
            }
        }

        full_layout.into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::futures::SinkExt;

        Subscription::run_with_id(
            "engine-bridge",
            iced::stream::channel(500, |mut output| async move {
                let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<CoreEvent>();
                let sink = std::sync::Arc::new(orbit_core::sink::MpscEventSink::new(event_tx));
                let manager = std::sync::Arc::new(tokio::sync::RwLock::new(
                    orbit_core::kubernetes::manager::KubeManager::new().await,
                ));
                let dispatcher = orbit_core::dispatcher::CoreDispatcher::new(manager, sink.clone());

                let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<CoreCommand>(100);
                let _ = COMMAND_TX.set(cmd_tx);

                dispatcher.dispatch(CoreCommand::Initialize).await;

                loop {
                    tokio::select! {
                        Some(event) = event_rx.recv() => {
                            let _ = output.send(Message::CoreEventReceived(event)).await;
                        }
                        Some(cmd) = cmd_rx.recv() => {
                            dispatcher.dispatch(cmd).await;
                        }
                        else => break,
                    }
                }
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::kubernetes::models::{ClusterInfo, DeploymentInfo, PodInfo};

    #[test]
    fn test_app_update_handles_engine_ready_and_populates_clusters_and_settings() {
        let mut app = OrbitApp::new();

        let mut settings = HashMap::new();
        settings.insert("maxLogFiles".to_string(), serde_json::json!(10));

        let clusters = vec![
            ClusterInfo {
                id: "dev-cluster".to_string(),
                name: "dev-cluster".to_string(),
                status: "healthy".to_string(),
                ..Default::default()
            },
            ClusterInfo {
                id: "prod-cluster".to_string(),
                name: "prod-cluster".to_string(),
                status: "offline".to_string(),
                ..Default::default()
            },
        ];

        let event = CoreEvent::EngineReady {
            status: "ready".to_string(),
            message: "Connected".to_string(),
            clusters,
            active_cluster_id: Some("dev-cluster".to_string()),
            settings,
        };

        let _ = app.update(Message::CoreEventReceived(event));

        assert!(app.is_engine_ready);
        assert_eq!(app.cluster_store.clusters.len(), 2);
        assert!(!app.settings.is_empty());
        assert_eq!(app.cluster_store.active_cluster_id.as_deref(), Some("dev-cluster"));
    }

    #[test]
    fn test_app_update_switch_cluster_clears_previous_cluster_resources_and_sets_loading() {
        let mut app = OrbitApp::new();
        app.cluster_store.pods.push(PodInfo {
            name: "old-pod".to_string(),
            namespace: "default".to_string(),
            ..Default::default()
        });

        assert_eq!(app.cluster_store.pods.len(), 1);

        let _ = app.update(Message::SelectCluster(Some("prod-cluster".to_string())));

        assert!(app.cluster_store.pods.is_empty());
        assert!(app.is_loading_cluster);
        assert_eq!(app.cluster_store.active_cluster_id.as_deref(), Some("prod-cluster"));
    }

    #[test]
    fn test_search_everywhere_matches_navigation_and_live_resources() {
        let mut app = OrbitApp::new();
        app.cluster_store.pods.push(PodInfo {
            name: "nginx-ingress-controller".to_string(),
            namespace: "ingress-nginx".to_string(),
            status: "Running".to_string(),
            ..Default::default()
        });
        app.cluster_store.deployments.push(DeploymentInfo {
            name: "nginx-deployment".to_string(),
            namespace: "default".to_string(),
            available: 2,
            replicas: orbit_core::kubernetes::models::Replicas { current: 2, desired: 2 },
            ..Default::default()
        });

        let results = app.search_everywhere("nginx");

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title.contains("nginx-ingress-controller")));
        assert!(results.iter().any(|r| r.title.contains("nginx-deployment")));
    }

    #[test]
    fn test_engine_ready_auto_connects_to_active_or_first_cluster() {
        // Case A: EngineReady with active_cluster_id: Some("dev-cluster")
        let mut app_a = OrbitApp::new();
        let event_a = CoreEvent::EngineReady {
            status: "ready".to_string(),
            message: "Connected".to_string(),
            clusters: vec![ClusterInfo {
                id: "dev-cluster".to_string(),
                name: "dev-cluster".to_string(),
                status: "healthy".to_string(),
                ..Default::default()
            }],
            active_cluster_id: Some("dev-cluster".to_string()),
            settings: HashMap::new(),
        };
        let _ = app_a.update(Message::CoreEventReceived(event_a));
        assert_eq!(app_a.cluster_store.active_cluster_id.as_deref(), Some("dev-cluster"));
        assert!(app_a.is_loading_cluster);
        assert_eq!(app_a.cluster_connection_status(), ClusterConnectionStatus::Loading);

        // Case B: EngineReady with active_cluster_id: None and clusters: [prod-cluster, dev-cluster]
        let mut app_b = OrbitApp::new();
        let event_b = CoreEvent::EngineReady {
            status: "ready".to_string(),
            message: "Connected".to_string(),
            clusters: vec![
                ClusterInfo {
                    id: "prod-cluster".to_string(),
                    name: "prod-cluster".to_string(),
                    status: "offline".to_string(),
                    ..Default::default()
                },
                ClusterInfo {
                    id: "dev-cluster".to_string(),
                    name: "dev-cluster".to_string(),
                    status: "healthy".to_string(),
                    ..Default::default()
                },
            ],
            active_cluster_id: None,
            settings: HashMap::new(),
        };
        let _ = app_b.update(Message::CoreEventReceived(event_b));
        assert_eq!(app_b.cluster_store.active_cluster_id.as_deref(), Some("prod-cluster"));
        assert!(app_b.is_loading_cluster);
        assert_eq!(app_b.cluster_connection_status(), ClusterConnectionStatus::Loading);
    }

    #[test]
    fn test_clusters_updated_auto_connects_when_disconnected_once() {
        let mut app = OrbitApp::new();
        let clusters = vec![ClusterInfo {
            id: "staging-cluster".to_string(),
            name: "staging-cluster".to_string(),
            status: "healthy".to_string(),
            ..Default::default()
        }];
        let _ = app.update(Message::CoreEventReceived(CoreEvent::ClustersUpdated { clusters }));
        assert_eq!(app.cluster_store.active_cluster_id.as_deref(), Some("staging-cluster"));
        assert!(app.is_loading_cluster);

        let _ = app.update(Message::CoreEventReceived(CoreEvent::ActiveClusterChanged {
            active_cluster_id: None,
        }));
        assert_eq!(app.cluster_store.active_cluster_id, None);
        assert!(!app.is_loading_cluster);
        assert_eq!(app.cluster_connection_status(), ClusterConnectionStatus::Disconnected);
    }

    #[test]
    fn test_cluster_and_namespace_options_and_selection_flow() {
        let mut app = OrbitApp::new();
        app.cluster_store.clusters = vec![
            ClusterInfo {
                id: "prod-cluster".to_string(),
                name: "prod-cluster".to_string(),
                ..Default::default()
            },
            ClusterInfo {
                id: "dev-cluster".to_string(),
                name: "dev-cluster".to_string(),
                ..Default::default()
            },
        ];
        assert_eq!(app.cluster_options(), vec!["prod-cluster", "dev-cluster"]);

        app.cluster_store.namespaces = vec![
            orbit_core::kubernetes::models::NamespaceInfo {
                name: "monitoring".to_string(),
                status: "Active".to_string(),
                is_system: false,
                age: "1d".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
                uid: "u1".to_string(),
                created: "".to_string(),
            },
            orbit_core::kubernetes::models::NamespaceInfo {
                name: "default".to_string(),
                status: "Active".to_string(),
                is_system: false,
                age: "1d".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
                uid: "u2".to_string(),
                created: "".to_string(),
            },
            orbit_core::kubernetes::models::NamespaceInfo {
                name: "kube-system".to_string(),
                status: "Active".to_string(),
                is_system: true,
                age: "1d".to_string(),
                labels: Default::default(),
                annotations: Default::default(),
                uid: "u3".to_string(),
                created: "".to_string(),
            },
        ];
        let ns_options = app.namespace_options();
        assert_eq!(ns_options, vec!["All Namespaces", "default", "kube-system", "monitoring"]);

        assert_eq!(app.selected_namespace_label(), ALL_NAMESPACES_LABEL);

        let _ = app.update(Message::SelectNamespace(Some("kube-system".into())));
        assert_eq!(app.selected_namespace_label(), "kube-system");
        assert_eq!(app.filter.namespace.as_deref(), Some("kube-system"));

        app.filter.namespace = Some("custom-temp".into());
        assert!(app.namespace_options().contains(&"custom-temp".to_string()));
        app.filter.namespace = None;

        let _ = app.update(Message::SelectCluster(Some("prod-cluster".into())));
        assert_eq!(app.selected_namespace_label(), ALL_NAMESPACES_LABEL);
        assert_eq!(app.filter.namespace, None);
    }

    #[test]
    fn test_top_bar_rendering_uses_custom_styles_and_pills() {
        let app = OrbitApp::new();
        assert_eq!(app.cluster_connection_status(), ClusterConnectionStatus::Disconnected);
        let _view = app.view();
    }

    #[test]
    fn test_sidebar_nav_active_item_matching() {
        let mut app = OrbitApp::new();
        let pod_tab = WorkspaceTab {
            id: "nav-pods".into(),
            title: "Pods".into(),
            content: TabContent::ResourceTable(ResourceKind::Pod),
        };
        let _ = app.update(Message::OpenTab(pod_tab.clone()));
        assert_eq!(
            app.tab_manager.active_tab().map(|t| &t.content),
            Some(&TabContent::ResourceTable(ResourceKind::Pod))
        );
        let _view = app.view();
    }

    #[test]
    fn test_sidebar_click_deduplicates_tabs_and_uses_clean_labels() {
        let mut app = OrbitApp::new();
        // Check initial tab is clean "Overview"
        assert_eq!(app.tab_manager.tabs.len(), 1);
        assert_eq!(app.tab_manager.tabs[0].id, "tab-overview");
        assert_eq!(app.tab_manager.tabs[0].title, "Overview");

        let pod_tab = WorkspaceTab {
            id: "nav-pods".into(),
            title: "Pods".into(),
            content: TabContent::ResourceTable(ResourceKind::Pod),
        };

        // Open Pods once
        let _ = app.update(Message::OpenTab(pod_tab.clone()));
        assert_eq!(app.tab_manager.tabs.len(), 2);
        assert_eq!(app.tab_manager.active_tab().unwrap().title, "Pods");

        // Open Pods second time (duplicate)
        let _ = app.update(Message::OpenTab(pod_tab));
        assert_eq!(app.tab_manager.tabs.len(), 2, "Tab count must remain 2 after duplicate open");
        assert_eq!(app.tab_manager.active_tab().unwrap().title, "Pods");

        // Verify all tab titles contain no emoji glyphs
        for tab in &app.tab_manager.tabs {
            assert!(
                !tab.title.contains('📊') && !tab.title.contains('📦') && !tab.title.contains('🚀'),
                "Tab title '{}' must not contain emoji",
                tab.title
            );
        }
    }

    #[test]
    fn test_sidebar_and_tab_icons_rendering() {
        let mut app = OrbitApp::new();
        let _ = app.update(Message::OpenTab(WorkspaceTab {
            id: "tab-settings".into(),
            title: "Settings".into(),
            content: TabContent::Settings,
        }));
        let _ = app.update(Message::OpenTab(WorkspaceTab {
            id: "tab-yaml".into(),
            title: "YAML".into(),
            content: TabContent::YamlEditor {
                kind: "Pod".into(),
                name: "my-pod".into(),
                namespace: Some("default".into()),
            },
        }));
        let _view = app.view();
    }

    #[test]
    fn test_active_cluster_badges_format_k8s_version_and_local_tag() {
        use orbit_core::kubernetes::models::{ClusterInfo, NodeInfo};

        let mut app = OrbitApp::new();
        assert_eq!(app.k8s_version_label(), "K8S —");
        assert!(!app.active_cluster_is_local());

        let local_cluster = ClusterInfo {
            id: "c-local".to_string(),
            name: "docker-desktop".to_string(),
            status: "Connected".to_string(),
            alias: Some("Dev Desktop".to_string()),
            server_url: Some("https://localhost:6443".to_string()),
            ..Default::default()
        };
        app.cluster_store.clusters = vec![local_cluster];
        app.cluster_store.active_cluster_id = Some("c-local".to_string());
        app.cluster_store.nodes = vec![NodeInfo {
            name: "node-1".to_string(),
            version: "v1.32.1".to_string(),
            ..Default::default()
        }];

        assert_eq!(app.k8s_version_label(), "K8S v1.32.1");
        assert!(app.active_cluster_is_local());
        assert_eq!(app.active_cluster_display_name(), "Dev Desktop");
    }

    #[test]
    fn test_dual_rail_category_toggle_collapses_and_expands_sidebar() {
        let mut app = OrbitApp::new();
        assert_eq!(app.active_category, Some(SidebarCategory::Workloads));

        // Click same category -> collapses to None
        let _ = app.update(Message::ToggleCategory(SidebarCategory::Workloads));
        assert_eq!(app.active_category, None);

        // Click different category -> expands to Some(Clusters)
        let _ = app.update(Message::ToggleCategory(SidebarCategory::Clusters));
        assert_eq!(app.active_category, Some(SidebarCategory::Clusters));

        // Click again -> collapses to None
        let _ = app.update(Message::ToggleCategory(SidebarCategory::Clusters));
        assert_eq!(app.active_category, None);
    }

    #[test]
    fn test_empty_state_rendered_when_all_tabs_closed() {
        let mut app = OrbitApp::new();
        assert_eq!(app.tab_manager.tabs.len(), 1);
        let tab_id = app.tab_manager.tabs[0].id.clone();

        // Close the only open tab
        let _ = app.update(Message::CloseTab(tab_id));
        assert!(app.tab_manager.tabs.is_empty());
        assert_eq!(app.tab_manager.active_tab(), None);

        // Verify view renders empty state without panic
        let _view = app.view();
    }

    #[test]
    fn test_window_control_messages_and_double_click_detection() {
        let mut app = OrbitApp::new();
        assert!(app.last_titlebar_click.is_none());

        // First click -> records timestamp
        let _ = app.update(Message::TitlebarPressed);
        assert!(app.last_titlebar_click.is_some());

        // Second rapid click -> detects double-click and resets
        let _ = app.update(Message::TitlebarPressed);
        assert!(app.last_titlebar_click.is_none());

        // Window control messages execute without panic
        let _ = app.update(Message::WindowMinimize);
        let _ = app.update(Message::WindowToggleMaximize);
        let _ = app.update(Message::WindowClose);
    }

    #[test]
    fn test_view_renders_titlebar_and_window_controls() {
        let mut app = OrbitApp::new();
        {
            let _dark_view = app.view();
        }

        let _ = app.update(Message::ToggleTheme);
        assert!(!app.palette.is_dark);
        {
            let _light_view = app.view();
        }
    }

    #[test]
    fn test_app_settings_update_palette_and_auto_saves() {
        let mut app = OrbitApp::new();
        assert!(app.palette.is_dark);

        // Update theme mode to light
        let _ = app.update(Message::UpdateSetting(
            "themeMode".to_string(),
            serde_json::json!("light"),
        ));
        assert!(!app.palette.is_dark);
        assert_eq!(app.settings.get("themeMode"), Some(&serde_json::json!("light")));
        assert_eq!(
            app.settings_state.feedback_message,
            Some("Settings saved".to_string())
        );

        // Update accent color to emerald
        let _ = app.update(Message::UpdateSetting(
            "accentColor".to_string(),
            serde_json::json!("emerald"),
        ));
        assert_eq!(
            app.palette.accent,
            accent_color_from_name("emerald", app.palette.is_dark)
        );
        assert_eq!(app.settings.get("accentColor"), Some(&serde_json::json!("emerald")));

        // Update terminal shell input
        let _ = app.update(Message::UpdateShellInput("pwsh.exe".to_string()));
        assert_eq!(app.settings_state.terminal_shell_input, "pwsh.exe");

        // Verify view renders with settings tab active
        app.tab_manager.open_tab(WorkspaceTab {
            id: "tab-settings".to_string(),
            title: "Settings".to_string(),
            content: TabContent::Settings,
        });
        let _settings_view = app.view();
    }

    #[test]
    fn test_app_reset_settings_to_defaults() {
        let mut app = OrbitApp::new();
        let _ = app.update(Message::UpdateSetting(
            "themeMode".to_string(),
            serde_json::json!("light"),
        ));
        assert!(!app.palette.is_dark);

        let _ = app.update(Message::ResetSettingsToDefaults);
        assert!(app.palette.is_dark);
        assert_eq!(app.settings.get("themeMode"), Some(&serde_json::json!("dark")));
        assert_eq!(
            app.settings_state.feedback_message,
            Some("Default settings restored".to_string())
        );
    }
}
