use orbit_core::command::ResourceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAction {
    Scale,
    Redeploy,
    Rollback,
    Clone,
    ViewLogs,
    PortForward,
    Restart,
    Trigger,
    ToggleSuspend,
    CordonOrUncordon,
    Drain,
    EditYaml,
    Delete,
}

pub fn available_actions_for_resource_kind(kind: ResourceKind) -> Vec<ResourceAction> {
    match kind {
        ResourceKind::Deployment => vec![
            ResourceAction::Scale,
            ResourceAction::Redeploy,
            ResourceAction::Rollback,
            ResourceAction::Clone,
            ResourceAction::ViewLogs,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        ResourceKind::CronJob => vec![
            ResourceAction::Trigger,
            ResourceAction::ToggleSuspend,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        ResourceKind::Pod => vec![
            ResourceAction::ViewLogs,
            ResourceAction::PortForward,
            ResourceAction::Restart,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        ResourceKind::Node => vec![
            ResourceAction::CordonOrUncordon,
            ResourceAction::Drain,
            ResourceAction::EditYaml,
        ],
        ResourceKind::StatefulSet | ResourceKind::DaemonSet => vec![
            ResourceAction::Scale,
            ResourceAction::Redeploy,
            ResourceAction::ViewLogs,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        ResourceKind::ReplicaSet => vec![
            ResourceAction::Scale,
            ResourceAction::ViewLogs,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        ResourceKind::Job => vec![
            ResourceAction::ViewLogs,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        ResourceKind::Service => vec![
            ResourceAction::PortForward,
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
        _ => vec![
            ResourceAction::EditYaml,
            ResourceAction::Delete,
        ],
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedResourceDetails {
    pub kind: ResourceKind,
    pub name: String,
    pub namespace: Option<String>,
    pub status: String,
    pub age: String,
    pub labels: Vec<(String, String)>,
    pub annotations: Vec<(String, String)>,
}

pub fn details_drawer_view<'a, Message: 'a + Clone>(
    details: &'a SelectedResourceDetails,
    palette: &'a crate::theme::OrbitPalette,
    on_close: Message,
    on_action: impl Fn(ResourceAction, ResourceKind, String, Option<String>) -> Message + 'a,
) -> iced::Element<'a, Message> {
    use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
    use iced::Length;
    use crate::theme::{action_button_style, drawer_container_style, tab_close_button_style};

    let title_bar = row![
        text(format!("{:?}: {}", details.kind, details.name))
            .size(15)
            .color(palette.text_primary),
        horizontal_space(),
        button(text("✕ Close").size(11))
            .style(tab_close_button_style(palette))
            .padding([3, 6])
            .on_press(on_close),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let actions = available_actions_for_resource_kind(details.kind);
    let mut actions_row = row![].spacing(6).align_y(iced::Alignment::Center);
    for act in actions {
        let is_destruct = act == ResourceAction::Delete;
        let label = format!("{:?}", act);
        let msg = on_action(act, details.kind, details.name.clone(), details.namespace.clone());
        actions_row = actions_row.push(
            button(text(label).size(11))
                .style(action_button_style(palette, is_destruct))
                .padding([3, 8])
                .on_press(msg),
        );
    }

    let meta_info = column![
        text(format!("Namespace: {}", details.namespace.as_deref().unwrap_or("Cluster-scoped"))).size(12).color(palette.text_secondary),
        text(format!("Status: {}", details.status)).size(12).color(palette.text_secondary),
        text(format!("Age: {}", details.age)).size(12).color(palette.text_muted),
    ]
    .spacing(4);

    let mut labels_col = column![text("Labels").size(13).color(palette.text_primary)].spacing(4);
    if details.labels.is_empty() {
        labels_col = labels_col.push(text("No labels").size(11).color(palette.text_muted));
    } else {
        for (k, v) in &details.labels {
            labels_col = labels_col.push(text(format!("{}: {}", k, v)).size(11).color(palette.text_secondary));
        }
    }

    container(
        column![
            title_bar,
            scrollable(actions_row).width(Length::Fill),
            meta_info,
            scrollable(labels_col).height(Length::Fill),
        ]
        .spacing(16)
        .padding(16),
    )
    .width(Length::Fixed(360.0))
    .height(Length::Fill)
    .style(drawer_container_style(palette))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_actions_for_resource_kind() {
        assert_eq!(
            available_actions_for_resource_kind(ResourceKind::Deployment),
            vec![
                ResourceAction::Scale,
                ResourceAction::Redeploy,
                ResourceAction::Rollback,
                ResourceAction::Clone,
                ResourceAction::ViewLogs,
                ResourceAction::EditYaml,
                ResourceAction::Delete,
            ]
        );

        assert_eq!(
            available_actions_for_resource_kind(ResourceKind::CronJob),
            vec![
                ResourceAction::Trigger,
                ResourceAction::ToggleSuspend,
                ResourceAction::EditYaml,
                ResourceAction::Delete,
            ]
        );

        assert_eq!(
            available_actions_for_resource_kind(ResourceKind::Pod),
            vec![
                ResourceAction::ViewLogs,
                ResourceAction::PortForward,
                ResourceAction::Restart,
                ResourceAction::EditYaml,
                ResourceAction::Delete,
            ]
        );

        assert_eq!(
            available_actions_for_resource_kind(ResourceKind::Node),
            vec![
                ResourceAction::CordonOrUncordon,
                ResourceAction::Drain,
                ResourceAction::EditYaml,
            ]
        );
    }

    #[test]
    fn test_details_drawer_rendering() {
        let palette = crate::theme::OrbitPalette::dark();
        let details = SelectedResourceDetails {
            kind: ResourceKind::Pod,
            name: "test-pod-1".into(),
            namespace: Some("default".into()),
            status: "Running".into(),
            age: "5m".into(),
            labels: vec![("app".into(), "web".into())],
            annotations: vec![],
        };
        let _drawer: iced::Element<'_, ()> = details_drawer_view(
            &details,
            &palette,
            (),
            |_act, _kind, _name, _ns| (),
        );
    }
}
