use iced::Length;
use orbit_core::command::ResourceKind;

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub header: String,
    pub width: Length,
    pub is_numeric: bool,
}

pub fn truncate_ellipsis(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

pub fn columns_for_kind(kind: ResourceKind) -> Vec<TableColumn> {
    match kind {
        ResourceKind::Pod => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Status".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Restarts".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "CPU".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Memory".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Node".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "IP".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Deployment => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Ready".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Up-to-date".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Available".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Images".into(), width: Length::FillPortion(3), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::StatefulSet => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Ready".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Images".into(), width: Length::FillPortion(3), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::DaemonSet => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Desired".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Current".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Ready".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Available".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::ReplicaSet => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Desired".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Current".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Ready".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Job => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Completions".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Duration".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::CronJob => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Schedule".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Suspend".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Active".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Last Schedule".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::HorizontalPodAutoscaler => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Targets".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Min Pods".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Max Pods".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Replicas".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Node => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Status".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Roles".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Version".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Internal IP".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "OS Image".into(), width: Length::FillPortion(3), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Service => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Type".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Cluster IP".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "External IP".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Ports".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Ingress => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Class".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Hosts".into(), width: Length::FillPortion(3), is_numeric: false },
            TableColumn { header: "Address".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Ports".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::ConfigMap => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Data".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Secret => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Type".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Data".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Event => vec![
            TableColumn { header: "Type".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Reason".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Object".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Message".into(), width: Length::FillPortion(5), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::PersistentVolume => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Capacity".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Access Modes".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Reclaim Policy".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Status".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Claim".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "StorageClass".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::PersistentVolumeClaim => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Status".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Volume".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Capacity".into(), width: Length::FillPortion(1), is_numeric: true },
            TableColumn { header: "Access Modes".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "StorageClass".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::StorageClass => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Provisioner".into(), width: Length::FillPortion(3), is_numeric: false },
            TableColumn { header: "Reclaim Policy".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Volume Binding".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Allow Expansion".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Policy => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Namespace".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Kind".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
        ResourceKind::Namespace => vec![
            TableColumn { header: "Name".into(), width: Length::FillPortion(4), is_numeric: false },
            TableColumn { header: "Status".into(), width: Length::FillPortion(2), is_numeric: false },
            TableColumn { header: "Labels".into(), width: Length::FillPortion(3), is_numeric: false },
            TableColumn { header: "Age".into(), width: Length::FillPortion(1), is_numeric: false },
            TableColumn { header: "Actions".into(), width: Length::Fixed(100.0), is_numeric: false },
        ],
    }
}

pub fn resource_table_view<'a, Message: 'a + Clone>(
    kind: ResourceKind,
    store: &'a crate::store::ClusterStore,
    filter: &'a crate::store::FilterCriteria,
    palette: &'a crate::theme::OrbitPalette,
    on_select: impl Fn(crate::views::details_drawer::SelectedResourceDetails) -> Message + 'a + Copy,
) -> iced::Element<'a, Message> {
    use iced::widget::{button, column, container, row, scrollable, text};
    use crate::theme::{
        FONT_MONO, FONT_UI_SEMIBOLD, Typography, ghost_action_button_style, status_pill,
        table_row_style,
    };

    let columns = columns_for_kind(kind);

    // Header row
    let mut header_row = row![].spacing(8).padding([8, 12]).align_y(iced::Alignment::Center);
    for col in &columns {
        header_row = header_row.push(
            container(
                text(col.header.to_uppercase())
                    .size(Typography::SECONDARY)
                    .font(FONT_UI_SEMIBOLD)
                    .color(palette.text_muted),
            )
            .width(col.width),
        );
    }

    let header_bg = palette.bg_card;
    let header_container = container(header_row)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(header_bg)),
            ..Default::default()
        });

    // Rows
    let mut rows_col = column![].spacing(2);

    match kind {
        ResourceKind::Pod => {
            let filtered_pods: Vec<_> = store.pods.iter().filter(|p| filter.matches_pod(p)).collect();
            if filtered_pods.is_empty() {
                rows_col = rows_col.push(
                    container(
                        text("No pods found matching filter criteria")
                            .size(Typography::BODY)
                            .color(palette.text_muted),
                    )
                    .padding(24),
                );
            } else {
                for pod in filtered_pods {
                    let details = crate::views::details_drawer::SelectedResourceDetails {
                        kind: ResourceKind::Pod,
                        name: pod.name.clone(),
                        namespace: Some(pod.namespace.clone()),
                        status: pod.status.clone(),
                        age: pod.age.clone(),
                        labels: pod.labels.clone().into_iter().collect(),
                        annotations: pod.annotations.clone().into_iter().collect(),
                    };

                    let actions_cell = row![
                        button(text("Logs").size(Typography::CAPTION))
                            .style(ghost_action_button_style(palette))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                        button(text("Details").size(Typography::CAPTION))
                            .style(ghost_action_button_style(palette))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                    let pod_row = row![
                        container(text(pod.name.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_primary)).width(columns[0].width),
                        container(text(pod.namespace.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_secondary)).width(columns[1].width),
                        container(status_pill(palette, &pod.status)).width(columns[2].width),
                        container(text(pod.restarts.to_string()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[3].width),
                        container(text(pod.age.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[4].width),
                        container(text(pod.cpu.clone().unwrap_or_else(|| "-".into())).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[5].width),
                        container(text(pod.memory.clone().unwrap_or_else(|| "-".into())).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[6].width),
                        container(text(truncate_ellipsis(pod.node.as_deref().unwrap_or("-"), 24)).size(Typography::BODY).color(palette.text_secondary)).width(columns[7].width),
                        container(text(pod.ip.clone().unwrap_or_else(|| "-".into())).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[8].width),
                        container(actions_cell).width(columns[9].width),
                    ]
                    .spacing(8)
                    .padding([6, 12])
                    .align_y(iced::Alignment::Center);

                    rows_col = rows_col.push(
                        button(pod_row)
                            .style(table_row_style(palette))
                            .on_press(on_select(details))
                            .padding(0)
                            .width(Length::Fill),
                    );
                }
            }
        }
        ResourceKind::Deployment => {
            let filtered_deploys: Vec<_> = store
                .deployments
                .iter()
                .filter(|d| filter.matches_namespace(&d.namespace))
                .collect();
            if filtered_deploys.is_empty() {
                rows_col = rows_col.push(
                    container(text("No deployments found").size(Typography::BODY).color(palette.text_muted)).padding(24),
                );
            } else {
                for deploy in filtered_deploys {
                    let details = crate::views::details_drawer::SelectedResourceDetails {
                        kind: ResourceKind::Deployment,
                        name: deploy.name.clone(),
                        namespace: Some(deploy.namespace.clone()),
                        status: format!("{}/{}", deploy.replicas.current, deploy.replicas.desired),
                        age: deploy.age.clone(),
                        labels: deploy.labels.clone().into_iter().collect(),
                        annotations: deploy.annotations.clone().into_iter().collect(),
                    };

                    let is_ready = deploy.replicas.current == deploy.replicas.desired && deploy.replicas.desired > 0;
                    let deploy_status_text = if is_ready { "Ready" } else { "Progressing" };

                    let actions_cell = row![
                        button(text("Scale").size(Typography::CAPTION))
                            .style(ghost_action_button_style(palette))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                        button(text("Details").size(Typography::CAPTION))
                            .style(ghost_action_button_style(palette))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                    let deploy_row = row![
                        container(text(deploy.name.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_primary)).width(columns[0].width),
                        container(text(deploy.namespace.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_secondary)).width(columns[1].width),
                        container(status_pill(palette, deploy_status_text)).width(columns[2].width),
                        container(text(deploy.up_to_date.to_string()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[3].width),
                        container(text(deploy.available.to_string()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[4].width),
                        container(text(deploy.age.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[5].width),
                        container(text(truncate_ellipsis(&deploy.images.join(", "), 36)).size(Typography::BODY).color(palette.text_muted)).width(columns[6].width),
                        container(actions_cell).width(columns[7].width),
                    ]
                    .spacing(8)
                    .padding([6, 12])
                    .align_y(iced::Alignment::Center);

                    rows_col = rows_col.push(
                        button(deploy_row)
                            .style(table_row_style(palette))
                            .on_press(on_select(details))
                            .padding(0)
                            .width(Length::Fill),
                    );
                }
            }
        }
        ResourceKind::Node => {
            if store.nodes.is_empty() {
                rows_col = rows_col.push(
                    container(text("No nodes found").size(Typography::BODY).color(palette.text_muted)).padding(24),
                );
            } else {
                for node in &store.nodes {
                    let details = crate::views::details_drawer::SelectedResourceDetails {
                        kind: ResourceKind::Node,
                        name: node.name.clone(),
                        namespace: None,
                        status: node.status.clone(),
                        age: node.uptime.clone(),
                        labels: node.labels_map.clone().into_iter().collect(),
                        annotations: node.annotations.clone().into_iter().collect(),
                    };

                    let internal_ip = node.addresses.first().map(|a| a.address.clone()).unwrap_or_else(|| "-".into());
                    let os_img = node.node_info.as_ref().map(|i| i.os_image.clone()).unwrap_or_else(|| "-".into());

                    let actions_cell = row![
                        button(text("Details").size(Typography::CAPTION))
                            .style(ghost_action_button_style(palette))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                    let node_row = row![
                        container(text(node.name.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_primary)).width(columns[0].width),
                        container(status_pill(palette, &node.status)).width(columns[1].width),
                        container(text(node.role.clone()).size(Typography::BODY).color(palette.text_secondary)).width(columns[2].width),
                        container(text(node.version.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[3].width),
                        container(text(node.uptime.clone()).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[4].width),
                        container(text(internal_ip).size(Typography::BODY).font(FONT_MONO).color(palette.text_muted)).width(columns[5].width),
                        container(text(truncate_ellipsis(&os_img, 32)).size(Typography::BODY).color(palette.text_muted)).width(columns[6].width),
                        container(actions_cell).width(columns[7].width),
                    ]
                    .spacing(8)
                    .padding([6, 12])
                    .align_y(iced::Alignment::Center);

                    rows_col = rows_col.push(
                        button(node_row)
                            .style(table_row_style(palette))
                            .on_press(on_select(details))
                            .padding(0)
                            .width(Length::Fill),
                    );
                }
            }
        }
        _ => {
            rows_col = rows_col.push(
                container(text(format!("{:?} resources view", kind)).size(Typography::BODY).color(palette.text_muted)).padding(24),
            );
        }
    }

    column![
        header_container,
        scrollable(rows_col).width(Length::Fill).height(Length::Fill),
    ]
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_table_columns_use_fill_portion_and_fixed_actions() {
        let pod_cols = columns_for_kind(ResourceKind::Pod);
        for col in &pod_cols[..pod_cols.len() - 1] {
            assert!(matches!(col.width, iced::Length::FillPortion(_)));
        }
        let last_pod = pod_cols.last().unwrap();
        assert_eq!(last_pod.header, "Actions");
        assert_eq!(last_pod.width, iced::Length::Fixed(100.0));

        let deploy_cols = columns_for_kind(ResourceKind::Deployment);
        for col in &deploy_cols[..deploy_cols.len() - 1] {
            assert!(matches!(col.width, iced::Length::FillPortion(_)));
        }
        let last_deploy = deploy_cols.last().unwrap();
        assert_eq!(last_deploy.header, "Actions");
        assert_eq!(last_deploy.width, iced::Length::Fixed(100.0));
    }

    #[test]
    fn test_truncate_ellipsis() {
        assert_eq!(truncate_ellipsis("short", 10), "short");
        assert_eq!(truncate_ellipsis("exactly10chars", 14), "exactly10chars");
        let truncated = truncate_ellipsis("this is a very long container image string", 20);
        assert_eq!(truncated.chars().count(), 20);
        assert!(truncated.ends_with('…'));
        assert_eq!(truncate_ellipsis("", 5), "");
        assert_eq!(truncate_ellipsis("abc", 0), "");
    }

    #[test]
    fn test_columns_for_all_18_resource_kinds_non_empty() {
        let all_kinds = [
            ResourceKind::Pod,
            ResourceKind::Deployment,
            ResourceKind::StatefulSet,
            ResourceKind::DaemonSet,
            ResourceKind::ReplicaSet,
            ResourceKind::Job,
            ResourceKind::CronJob,
            ResourceKind::HorizontalPodAutoscaler,
            ResourceKind::Node,
            ResourceKind::Service,
            ResourceKind::Ingress,
            ResourceKind::ConfigMap,
            ResourceKind::Secret,
            ResourceKind::Event,
            ResourceKind::PersistentVolume,
            ResourceKind::PersistentVolumeClaim,
            ResourceKind::StorageClass,
            ResourceKind::Policy,
            ResourceKind::Namespace,
        ];

        for kind in all_kinds {
            let cols = columns_for_kind(kind);
            assert!(
                cols.len() >= 4 && cols.len() <= 10,
                "Expected between 4 and 10 columns for {:?}, got {}",
                kind,
                cols.len()
            );
            for col in &cols {
                assert!(!col.header.is_empty());
                assert!(matches!(col.width, Length::FillPortion(_) | Length::Fixed(_)));
            }
        }
    }

    #[test]
    fn test_table_and_drawer_typography_rendering() {
        let palette = crate::theme::OrbitPalette::dark();
        let mut store = crate::store::ClusterStore::default();
        store.pods.push(orbit_core::kubernetes::models::PodInfo {
            name: "api-server-0".into(),
            namespace: "default".into(),
            status: "Running".into(),
            restarts: 0,
            age: "2d".into(),
            cpu: Some("25m".into()),
            memory: Some("128Mi".into()),
            node: Some("node-1".into()),
            ip: Some("10.0.0.12".into()),
            ..Default::default()
        });
        store.deployments.push(orbit_core::kubernetes::models::DeploymentInfo {
            name: "web-frontend".into(),
            namespace: "prod".into(),
            replicas: orbit_core::kubernetes::models::Replicas { current: 3, desired: 3 },
            up_to_date: 3,
            available: 3,
            age: "14d".into(),
            ..Default::default()
        });
        let filter = crate::store::FilterCriteria::default();

        let _pod_table: iced::Element<'_, ()> =
            resource_table_view(ResourceKind::Pod, &store, &filter, &palette, |_| ());
        let _deploy_table: iced::Element<'_, ()> =
            resource_table_view(ResourceKind::Deployment, &store, &filter, &palette, |_| ());
    }
}
