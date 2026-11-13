use iced::{
    Alignment, Element, Length,
    widget::{
        button, column, container, horizontal_space, row, scrollable, text, text_input,
        vertical_space,
    },
};
use orbit_core::command::ResourceKind;
use orbit_core::kubernetes::models::ClusterInfo;
use std::collections::BTreeSet;

use crate::app::Message;
use crate::icons::{Icon, icon};
use crate::state::tabs::{TabContent, WorkspaceTab};
use crate::theme::{
    FONT_UI_SEMIBOLD, OrbitPalette, Typography, activity_bar_container_style,
    activity_bar_item_style, cluster_card_style, cluster_tag_pill_style,
    ghost_action_button_style, selection_indicator, sub_sidebar_container_style,
    sub_sidebar_item_style,
};
use crate::views::modals::{ActiveModal, AddClusterModalMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarCategory {
    Clusters,
    Workloads,
    Network,
    Storage,
    Config,
}

#[derive(Debug, Clone)]
pub struct SubNavItem {
    pub label: &'static str,
    pub icon: Icon,
    pub content: TabContent,
}

pub fn category_sub_nav_items(category: SidebarCategory) -> Vec<SubNavItem> {
    match category {
        SidebarCategory::Clusters => Vec::new(),
        SidebarCategory::Workloads => vec![
            SubNavItem {
                label: "Overview",
                icon: Icon::LayoutDashboard,
                content: TabContent::Dashboard,
            },
            SubNavItem {
                label: "Pods",
                icon: Icon::Box,
                content: TabContent::ResourceTable(ResourceKind::Pod),
            },
            SubNavItem {
                label: "Deployments",
                icon: Icon::Layers,
                content: TabContent::ResourceTable(ResourceKind::Deployment),
            },
            SubNavItem {
                label: "DaemonSets",
                icon: Icon::Cpu,
                content: TabContent::ResourceTable(ResourceKind::DaemonSet),
            },
            SubNavItem {
                label: "StatefulSets",
                icon: Icon::Database,
                content: TabContent::ResourceTable(ResourceKind::StatefulSet),
            },
            SubNavItem {
                label: "ReplicaSets",
                icon: Icon::Copy,
                content: TabContent::ResourceTable(ResourceKind::ReplicaSet),
            },
            SubNavItem {
                label: "Jobs",
                icon: Icon::Play,
                content: TabContent::ResourceTable(ResourceKind::Job),
            },
            SubNavItem {
                label: "CronJobs",
                icon: Icon::Clock,
                content: TabContent::ResourceTable(ResourceKind::CronJob),
            },
        ],
        SidebarCategory::Network => vec![
            SubNavItem {
                label: "Services",
                icon: Icon::Network,
                content: TabContent::ResourceTable(ResourceKind::Service),
            },
            SubNavItem {
                label: "Ingresses",
                icon: Icon::Globe,
                content: TabContent::ResourceTable(ResourceKind::Ingress),
            },
        ],
        SidebarCategory::Storage => vec![
            SubNavItem {
                label: "PersistentVolumes",
                icon: Icon::HardDrive,
                content: TabContent::ResourceTable(ResourceKind::PersistentVolume),
            },
            SubNavItem {
                label: "Volume Claims",
                icon: Icon::FolderArchive,
                content: TabContent::ResourceTable(ResourceKind::PersistentVolumeClaim),
            },
            SubNavItem {
                label: "StorageClasses",
                icon: Icon::Layers2,
                content: TabContent::ResourceTable(ResourceKind::StorageClass),
            },
        ],
        SidebarCategory::Config => vec![
            SubNavItem {
                label: "ConfigMaps",
                icon: Icon::FileCode,
                content: TabContent::ResourceTable(ResourceKind::ConfigMap),
            },
            SubNavItem {
                label: "Secrets",
                icon: Icon::Key,
                content: TabContent::ResourceTable(ResourceKind::Secret),
            },
        ],
    }
}

pub fn get_cluster_display_name(cluster: &ClusterInfo) -> &str {
    cluster.alias.as_deref().unwrap_or(&cluster.name)
}

pub fn is_local_cluster(cluster: &ClusterInfo) -> bool {
    if let Some(url) = &cluster.server_url {
        let url_lower = url.to_lowercase();
        if url_lower.contains("127.0.0.1")
            || url_lower.contains("localhost")
            || url_lower.contains("0.0.0.0")
        {
            return true;
        }
    }
    let name_lower = cluster.name.to_lowercase();
    name_lower.contains("minikube")
        || name_lower.contains("kind-")
        || name_lower.contains("k3d-")
        || name_lower.contains("docker-desktop")
}

pub fn extract_all_cluster_tags(clusters: &[ClusterInfo]) -> Vec<String> {
    let mut tags = BTreeSet::new();
    for c in clusters {
        for t in &c.tags {
            tags.insert(t.clone());
        }
    }
    tags.into_iter().collect()
}

pub fn filter_clusters<'a>(
    clusters: &'a [ClusterInfo],
    query: &str,
    tag_filter: Option<&str>,
) -> Vec<&'a ClusterInfo> {
    let q = query.trim().to_lowercase();
    clusters
        .iter()
        .filter(|c| {
            if let Some(tag) = tag_filter
                && !c.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
            {
                return false;
            }
            if q.is_empty() {
                return true;
            }
            c.name.to_lowercase().contains(&q)
                || c.alias
                    .as_deref()
                    .map(|a| a.to_lowercase().contains(&q))
                    .unwrap_or(false)
                || c.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}

pub fn activity_bar_view<'a>(
    palette: &'a OrbitPalette,
    active_category: Option<SidebarCategory>,
) -> Element<'a, Message> {
    let logo_btn = button(
        container(icon(Icon::Boxes, 20, palette.accent))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .style(activity_bar_item_style(palette, false))
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .on_press(Message::OpenTab(WorkspaceTab {
        id: "tab-overview".to_string(),
        title: "Overview".to_string(),
        content: TabContent::Dashboard,
    }));

    let logo_item = row![selection_indicator(palette, false), logo_btn]
        .spacing(4)
        .align_y(Alignment::Center);

    let categories = [
        (SidebarCategory::Clusters, Icon::Server),
        (SidebarCategory::Workloads, Icon::Boxes),
        (SidebarCategory::Network, Icon::Network),
        (SidebarCategory::Storage, Icon::HardDrive),
        (SidebarCategory::Config, Icon::FileCode),
    ];

    let mut cat_col = column![logo_item].spacing(6).align_x(Alignment::Start);

    for (cat, cat_icon) in categories {
        let is_active = active_category == Some(cat);
        let btn = button(
            container(icon(
                cat_icon,
                18,
                if is_active {
                    palette.text_primary
                } else {
                    palette.text_secondary
                },
            ))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        )
        .style(activity_bar_item_style(palette, is_active))
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .on_press(Message::ToggleCategory(cat));

        let item_row = row![selection_indicator(palette, is_active), btn]
            .spacing(4)
            .align_y(Alignment::Center);

        cat_col = cat_col.push(item_row);
    }

    cat_col = cat_col.push(vertical_space());

    #[cfg(debug_assertions)]
    {
        let showcase_btn = button(
            container(icon(Icon::Palette, 16, palette.accent))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .style(activity_bar_item_style(palette, false))
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .on_press(Message::OpenTab(WorkspaceTab {
            id: "tab-showcase".to_string(),
            title: "UI Showcase".to_string(),
            content: TabContent::Showcase,
        }));
        let showcase_item = row![selection_indicator(palette, false), showcase_btn]
            .spacing(4)
            .align_y(Alignment::Center);
        cat_col = cat_col.push(showcase_item);
    }

    // Utility buttons
    let settings_btn = button(
        container(icon(Icon::Settings, 16, palette.text_secondary))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .style(activity_bar_item_style(palette, false))
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .on_press(Message::OpenTab(WorkspaceTab {
        id: "tab-settings".to_string(),
        title: "Settings".to_string(),
        content: TabContent::Settings,
    }));

    let theme_icon = if palette.is_dark {
        Icon::Sun
    } else {
        Icon::Moon
    };
    let theme_btn = button(
        container(icon(theme_icon, 16, palette.text_secondary))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .style(activity_bar_item_style(palette, false))
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .on_press(Message::ToggleTheme);

    let notif_btn = button(
        container(icon(Icon::Bell, 16, palette.text_secondary))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .style(activity_bar_item_style(palette, false))
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .on_press(Message::OpenResource(
        ResourceKind::Event,
        "Events".to_string(),
    ));

    let settings_item = row![selection_indicator(palette, false), settings_btn]
        .spacing(4)
        .align_y(Alignment::Center);
    let theme_item = row![selection_indicator(palette, false), theme_btn]
        .spacing(4)
        .align_y(Alignment::Center);
    let notif_item = row![selection_indicator(palette, false), notif_btn]
        .spacing(4)
        .align_y(Alignment::Center);

    cat_col = cat_col
        .push(settings_item)
        .push(theme_item)
        .push(notif_item);

    container(cat_col.padding([10, 2]))
        .width(Length::Fixed(50.0))
        .height(Length::Fill)
        .style(activity_bar_container_style(palette))
        .into()
}

pub fn sub_sidebar_view<'a>(
    palette: &'a OrbitPalette,
    category: SidebarCategory,
    clusters: &'a [ClusterInfo],
    active_cluster_id: Option<&'a str>,
    cluster_query: &'a str,
    selected_tag: Option<&'a str>,
    active_tab_content: Option<&'a TabContent>,
) -> Element<'a, Message> {
    match category {
        SidebarCategory::Clusters => {
            let header = row![
                text("CLUSTERS")
                    .size(Typography::CAPTION)
                    .font(FONT_UI_SEMIBOLD)
                    .color(palette.text_secondary),
                horizontal_space(),
                text(format!("{}", clusters.len()))
                    .size(Typography::CAPTION)
                    .color(palette.text_muted),
            ]
            .padding([4, 4]);

            let search_input = text_input("Filter clusters or tags...", cluster_query)
                .on_input(Message::ClusterFilterChanged)
                .size(Typography::BODY)
                .padding(6)
                .width(Length::Fill);

            let all_tags = extract_all_cluster_tags(clusters);
            let mut tag_row = row![
                button(text("#all").size(Typography::CAPTION))
                    .style(cluster_tag_pill_style(palette, selected_tag.is_none()))
                    .padding([2, 6])
                    .on_press(Message::SelectClusterTag(None))
            ]
            .spacing(4);

            for tag in all_tags.iter().take(5) {
                let is_sel = selected_tag == Some(tag.as_str());
                tag_row = tag_row.push(
                    button(text(format!("#{}", tag)).size(Typography::CAPTION))
                        .style(cluster_tag_pill_style(palette, is_sel))
                        .padding([2, 6])
                        .on_press(Message::SelectClusterTag(Some(tag.clone()))),
                );
            }

            let filtered = filter_clusters(clusters, cluster_query, selected_tag);
            let mut list_col = column![].spacing(6);

            for cluster in filtered {
                let is_active = active_cluster_id == Some(&cluster.id);
                let status_color = if cluster.status.eq_ignore_ascii_case("connected")
                    || cluster.status.eq_ignore_ascii_case("healthy")
                {
                    palette.success
                } else if cluster.status.eq_ignore_ascii_case("connecting")
                    || cluster.status.eq_ignore_ascii_case("pending")
                {
                    palette.warning
                } else {
                    palette.danger
                };

                let display_name = get_cluster_display_name(cluster);
                let subtitle = cluster.server_url.as_deref().unwrap_or(&cluster.name);

                let card_content = row![
                    text("●").size(Typography::CAPTION).color(status_color),
                    column![
                        text(display_name.to_string())
                            .size(Typography::BODY)
                            .font(FONT_UI_SEMIBOLD)
                            .color(palette.text_primary),
                        text(subtitle.to_string())
                            .size(Typography::SECONDARY)
                            .color(palette.text_muted),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    if is_active {
                        icon(Icon::Check, 14, palette.accent)
                    } else {
                        horizontal_space().width(Length::Fixed(0.0)).into()
                    }
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .padding(8);

                let card_btn = button(card_content)
                    .style(cluster_card_style(palette, is_active))
                    .width(Length::Fill)
                    .on_press(Message::SelectCluster(Some(cluster.id.clone())));

                list_col = list_col.push(card_btn);
            }

            let add_cluster_btn = button(
                row![
                    icon(Icon::Plus, 12, palette.accent),
                    text("+ Add cluster")
                        .size(Typography::BODY)
                        .color(palette.accent),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(ghost_action_button_style(palette))
            .width(Length::Fill)
            .padding(6)
            .on_press(Message::OpenModal(ActiveModal::AddCluster {
                mode: AddClusterModalMode::FilePath {
                    path: String::new(),
                },
                error: None,
            }));

            let main_col = column![
                header,
                search_input,
                tag_row,
                scrollable(list_col).height(Length::Fill),
                add_cluster_btn,
            ]
            .spacing(10)
            .padding([12, 10]);

            container(main_col)
                .width(Length::Fixed(220.0))
                .height(Length::Fill)
                .style(sub_sidebar_container_style(palette))
                .into()
        }
        SidebarCategory::Workloads
        | SidebarCategory::Network
        | SidebarCategory::Storage
        | SidebarCategory::Config => {
            let cat_title = match category {
                SidebarCategory::Workloads => "WORKLOADS",
                SidebarCategory::Network => "NETWORK",
                SidebarCategory::Storage => "STORAGE",
                SidebarCategory::Config => "CONFIG",
                _ => "",
            };

            let header = container(
                text(cat_title)
                    .size(Typography::CAPTION)
                    .font(FONT_UI_SEMIBOLD)
                    .color(palette.text_secondary),
            )
            .padding([4, 4]);

            let mut sub_col = column![header].spacing(4);

            for item in category_sub_nav_items(category) {
                let is_active = active_tab_content == Some(&item.content);
                let btn = button(
                    row![
                        icon(
                            item.icon,
                            14,
                            if is_active {
                                palette.text_primary
                            } else {
                                palette.text_secondary
                            },
                        ),
                        text(item.label).size(Typography::BODY),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .padding([6, 8]),
                )
                .style(sub_sidebar_item_style(palette, is_active))
                .width(Length::Fill)
                .on_press(match &item.content {
                    TabContent::ResourceTable(kind) => {
                        Message::OpenResource(*kind, item.label.to_string())
                    }
                    _ => Message::OpenTab(WorkspaceTab {
                        id: format!("tab-{}", item.label.to_lowercase().replace(' ', "-")),
                        title: item.label.to_string(),
                        content: item.content.clone(),
                    }),
                });

                let nav_row = row![selection_indicator(palette, is_active), btn,]
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .width(Length::Fill);

                sub_col = sub_col.push(nav_row);
            }

            container(sub_col.padding([12, 8]))
                .width(Length::Fixed(220.0))
                .height(Length::Fill)
                .style(sub_sidebar_container_style(palette))
                .into()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_cluster_display_name_and_is_local_detection() {
        let remote_cluster = ClusterInfo {
            id: "c1".to_string(),
            name: "k8s-prod-cluster-us-east-1".to_string(),
            status: "Connected".to_string(),
            alias: Some("Prod East".to_string()),
            server_url: Some("https://10.0.0.1:6443".to_string()),
            ..Default::default()
        };

        let local_cluster = ClusterInfo {
            id: "c2".to_string(),
            name: "docker-desktop".to_string(),
            status: "Connected".to_string(),
            alias: None,
            server_url: Some("https://127.0.0.1:6443".to_string()),
            ..Default::default()
        };

        assert_eq!(get_cluster_display_name(&remote_cluster), "Prod East");
        assert_eq!(get_cluster_display_name(&local_cluster), "docker-desktop");

        assert!(!is_local_cluster(&remote_cluster));
        assert!(is_local_cluster(&local_cluster));
    }

    #[test]
    fn test_category_sub_nav_items_configured() {
        let workloads = category_sub_nav_items(SidebarCategory::Workloads);
        assert_eq!(workloads.len(), 8);
        assert_eq!(workloads[0].label, "Overview");
        assert_eq!(workloads[1].label, "Pods");

        let network = category_sub_nav_items(SidebarCategory::Network);
        assert_eq!(network.len(), 2);
        assert_eq!(network[0].label, "Services");

        let storage = category_sub_nav_items(SidebarCategory::Storage);
        assert_eq!(storage.len(), 3);
        assert_eq!(storage[0].label, "PersistentVolumes");

        let config = category_sub_nav_items(SidebarCategory::Config);
        assert_eq!(config.len(), 2);
        assert_eq!(config[0].label, "ConfigMaps");
    }

    #[test]
    fn test_sub_sidebar_filters_clusters_by_query_and_tag() {
        let clusters = vec![
            ClusterInfo {
                id: "c-prod".to_string(),
                name: "prod-us-east".to_string(),
                status: "Connected".to_string(),
                alias: Some("Production East".to_string()),
                tags: vec!["prod".to_string(), "us".to_string()],
                server_url: Some("https://10.0.0.1:6443".to_string()),
                ..Default::default()
            },
            ClusterInfo {
                id: "c-dev".to_string(),
                name: "dev-local".to_string(),
                status: "Connected".to_string(),
                alias: None,
                tags: vec!["dev".to_string()],
                server_url: Some("https://127.0.0.1:6443".to_string()),
                ..Default::default()
            },
            ClusterInfo {
                id: "c-default".to_string(),
                name: "staging-cluster".to_string(),
                status: "Connected".to_string(),
                alias: None,
                tags: vec!["default".to_string()],
                server_url: Some("https://10.1.0.1:6443".to_string()),
                ..Default::default()
            },
        ];

        // 1. Tag extraction
        let tags = extract_all_cluster_tags(&clusters);
        assert_eq!(tags, vec!["default", "dev", "prod", "us"]);

        // 2. Query filter matching alias
        let matched = filter_clusters(&clusters, "production", None);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "c-prod");

        // 3. Query filter matching tag
        let matched = filter_clusters(&clusters, "dev", None);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "c-dev");

        // 4. Tag filter explicit
        let matched = filter_clusters(&clusters, "", Some("default"));
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "c-default");

        // 5. Query + Tag filter together
        let matched = filter_clusters(&clusters, "staging", Some("default"));
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "c-default");

        let matched = filter_clusters(&clusters, "prod", Some("default"));
        assert_eq!(matched.len(), 0);
    }
}
