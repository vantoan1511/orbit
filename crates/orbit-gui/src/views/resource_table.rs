use orbit_core::command::ResourceKind;

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub header: String,
    pub width: f32,
    pub is_numeric: bool,
}

pub fn columns_for_kind(kind: ResourceKind) -> Vec<TableColumn> {
    match kind {
        ResourceKind::Pod => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Status".into(), width: 110.0, is_numeric: false },
            TableColumn { header: "Restarts".into(), width: 80.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 80.0, is_numeric: false },
            TableColumn { header: "CPU".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Memory".into(), width: 100.0, is_numeric: true },
            TableColumn { header: "Node".into(), width: 160.0, is_numeric: false },
            TableColumn { header: "IP".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 140.0, is_numeric: false },
        ],
        ResourceKind::Deployment => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Ready".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Up-to-date".into(), width: 100.0, is_numeric: true },
            TableColumn { header: "Available".into(), width: 100.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Images".into(), width: 260.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 120.0, is_numeric: false },
        ],
        ResourceKind::StatefulSet => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Ready".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Images".into(), width: 260.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::DaemonSet => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Desired".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Current".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Ready".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Available".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::ReplicaSet => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Desired".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Current".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Ready".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Job => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Completions".into(), width: 110.0, is_numeric: true },
            TableColumn { header: "Duration".into(), width: 100.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::CronJob => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Schedule".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Suspend".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Active".into(), width: 80.0, is_numeric: true },
            TableColumn { header: "Last Schedule".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::HorizontalPodAutoscaler => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Targets".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Min Pods".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Max Pods".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Replicas".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Node => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Status".into(), width: 110.0, is_numeric: false },
            TableColumn { header: "Roles".into(), width: 120.0, is_numeric: false },
            TableColumn { header: "Version".into(), width: 110.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Internal IP".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "OS Image".into(), width: 200.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 120.0, is_numeric: false },
        ],
        ResourceKind::Service => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Type".into(), width: 120.0, is_numeric: false },
            TableColumn { header: "Cluster IP".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "External IP".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Ports".into(), width: 150.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Ingress => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Class".into(), width: 110.0, is_numeric: false },
            TableColumn { header: "Hosts".into(), width: 200.0, is_numeric: false },
            TableColumn { header: "Address".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Ports".into(), width: 100.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::ConfigMap => vec![
            TableColumn { header: "Name".into(), width: 260.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Data".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Secret => vec![
            TableColumn { header: "Name".into(), width: 260.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Type".into(), width: 180.0, is_numeric: false },
            TableColumn { header: "Data".into(), width: 90.0, is_numeric: true },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Event => vec![
            TableColumn { header: "Type".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Reason".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Object".into(), width: 180.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Message".into(), width: 380.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 80.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::PersistentVolume => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Capacity".into(), width: 100.0, is_numeric: true },
            TableColumn { header: "Access Modes".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Reclaim Policy".into(), width: 120.0, is_numeric: false },
            TableColumn { header: "Status".into(), width: 100.0, is_numeric: false },
            TableColumn { header: "Claim".into(), width: 180.0, is_numeric: false },
            TableColumn { header: "StorageClass".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::PersistentVolumeClaim => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Status".into(), width: 100.0, is_numeric: false },
            TableColumn { header: "Volume".into(), width: 180.0, is_numeric: false },
            TableColumn { header: "Capacity".into(), width: 100.0, is_numeric: true },
            TableColumn { header: "Access Modes".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "StorageClass".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::StorageClass => vec![
            TableColumn { header: "Name".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Provisioner".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Reclaim Policy".into(), width: 130.0, is_numeric: false },
            TableColumn { header: "Volume Binding".into(), width: 160.0, is_numeric: false },
            TableColumn { header: "Allow Expansion".into(), width: 120.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Policy => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Namespace".into(), width: 140.0, is_numeric: false },
            TableColumn { header: "Kind".into(), width: 150.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
        ],
        ResourceKind::Namespace => vec![
            TableColumn { header: "Name".into(), width: 240.0, is_numeric: false },
            TableColumn { header: "Status".into(), width: 110.0, is_numeric: false },
            TableColumn { header: "Labels".into(), width: 220.0, is_numeric: false },
            TableColumn { header: "Age".into(), width: 90.0, is_numeric: false },
            TableColumn { header: "Actions".into(), width: 100.0, is_numeric: false },
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
    use iced::Length;
    use crate::theme::{action_button_style, header_container_style, status_pill, table_row_style};

    let columns = columns_for_kind(kind);

    // Header row
    let mut header_row = row![].spacing(8).padding([8, 12]).align_y(iced::Alignment::Center);
    for col in &columns {
        header_row = header_row.push(
            container(
                text(col.header.clone())
                    .size(12)
                    .color(palette.text_secondary),
            )
            .width(Length::Fixed(col.width)),
        );
    }

    let header_container = container(header_row)
        .width(Length::Fill)
        .style(header_container_style(palette));

    // Rows
    let mut rows_col = column![].spacing(2);

    match kind {
        ResourceKind::Pod => {
            let filtered_pods: Vec<_> = store.pods.iter().filter(|p| filter.matches_pod(p)).collect();
            if filtered_pods.is_empty() {
                rows_col = rows_col.push(
                    container(text("No pods found matching filter criteria").size(13).color(palette.text_muted))
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
                        button(text("Logs").size(11))
                            .style(action_button_style(palette, false))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                        button(text("Details").size(11))
                            .style(action_button_style(palette, false))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                    let pod_row = row![
                        container(text(pod.name.clone()).size(13).color(palette.text_primary)).width(Length::Fixed(220.0)),
                        container(text(pod.namespace.clone()).size(12).color(palette.text_secondary)).width(Length::Fixed(140.0)),
                        container(status_pill(palette, &pod.status)).width(Length::Fixed(110.0)),
                        container(text(pod.restarts.to_string()).size(12).color(palette.text_muted)).width(Length::Fixed(80.0)),
                        container(text(pod.age.clone()).size(12).color(palette.text_muted)).width(Length::Fixed(80.0)),
                        container(text(pod.cpu.clone().unwrap_or_else(|| "-".into())).size(12).font(iced::Font::MONOSPACE).color(palette.text_muted)).width(Length::Fixed(90.0)),
                        container(text(pod.memory.clone().unwrap_or_else(|| "-".into())).size(12).font(iced::Font::MONOSPACE).color(palette.text_muted)).width(Length::Fixed(100.0)),
                        container(text(pod.node.clone().unwrap_or_else(|| "-".into())).size(12).color(palette.text_secondary)).width(Length::Fixed(160.0)),
                        container(text(pod.ip.clone().unwrap_or_else(|| "-".into())).size(12).color(palette.text_muted)).width(Length::Fixed(130.0)),
                        container(actions_cell).width(Length::Fixed(140.0)),
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
                    container(text("No deployments found").size(13).color(palette.text_muted)).padding(24),
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
                        button(text("Scale").size(11))
                            .style(action_button_style(palette, false))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                        button(text("Details").size(11))
                            .style(action_button_style(palette, false))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                    let deploy_row = row![
                        container(text(deploy.name.clone()).size(13).color(palette.text_primary)).width(Length::Fixed(240.0)),
                        container(text(deploy.namespace.clone()).size(12).color(palette.text_secondary)).width(Length::Fixed(140.0)),
                        container(status_pill(palette, deploy_status_text)).width(Length::Fixed(90.0)),
                        container(text(deploy.up_to_date.to_string()).size(12).color(palette.text_muted)).width(Length::Fixed(100.0)),
                        container(text(deploy.available.to_string()).size(12).color(palette.text_muted)).width(Length::Fixed(100.0)),
                        container(text(deploy.age.clone()).size(12).color(palette.text_muted)).width(Length::Fixed(90.0)),
                        container(text(deploy.images.join(", ")).size(12).color(palette.text_muted)).width(Length::Fixed(260.0)),
                        container(actions_cell).width(Length::Fixed(120.0)),
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
                    container(text("No nodes found").size(13).color(palette.text_muted)).padding(24),
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
                        button(text("Details").size(11))
                            .style(action_button_style(palette, false))
                            .padding([2, 6])
                            .on_press(on_select(details.clone())),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                    let node_row = row![
                        container(text(node.name.clone()).size(13).color(palette.text_primary)).width(Length::Fixed(220.0)),
                        container(status_pill(palette, &node.status)).width(Length::Fixed(110.0)),
                        container(text(node.role.clone()).size(12).color(palette.text_secondary)).width(Length::Fixed(120.0)),
                        container(text(node.version.clone()).size(12).color(palette.text_muted)).width(Length::Fixed(110.0)),
                        container(text(node.uptime.clone()).size(12).color(palette.text_muted)).width(Length::Fixed(90.0)),
                        container(text(internal_ip).size(12).color(palette.text_muted)).width(Length::Fixed(130.0)),
                        container(text(os_img).size(12).color(palette.text_muted)).width(Length::Fixed(200.0)),
                        container(actions_cell).width(Length::Fixed(120.0)),
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
                container(text(format!("{:?} resources view", kind)).size(13).color(palette.text_muted)).padding(24),
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
    fn test_resource_table_columns_include_actions_column() {
        let pod_cols = columns_for_kind(ResourceKind::Pod);
        let last_pod_col = pod_cols.last().expect("Pod columns should not be empty");
        assert_eq!(last_pod_col.header, "Actions");
        assert!(last_pod_col.width >= 100.0);

        let deploy_cols = columns_for_kind(ResourceKind::Deployment);
        let last_deploy_col = deploy_cols.last().expect("Deployment columns should not be empty");
        assert_eq!(last_deploy_col.header, "Actions");
        assert!(last_deploy_col.width >= 100.0);

        let node_cols = columns_for_kind(ResourceKind::Node);
        let last_node_col = node_cols.last().expect("Node columns should not be empty");
        assert_eq!(last_node_col.header, "Actions");
        assert!(last_node_col.width >= 100.0);
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
                assert!(col.width > 0.0);
            }
        }
    }
}
