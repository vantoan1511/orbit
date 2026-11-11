use iced::{
    widget::{column, container, row, text},
    Color, Element, Length,
};

use crate::icons::{icon, Icon};
use crate::store::ClusterStore;
use crate::theme::OrbitPalette;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterSummary {
    pub total_nodes: usize,
    pub ready_nodes: usize,
    pub total_pods: usize,
    pub running_pods: usize,
    pub failed_pods: usize,
    pub total_deployments: usize,
    pub total_services: usize,
    pub total_namespaces: usize,
    pub warning_events: usize,
}

pub fn compute_cluster_summary(store: &ClusterStore) -> ClusterSummary {
    let total_nodes = store.nodes.len();
    let ready_nodes = store
        .nodes
        .iter()
        .filter(|n| n.status.eq_ignore_ascii_case("ready"))
        .count();

    let total_pods = store.pods.len();
    let running_pods = store
        .pods
        .iter()
        .filter(|p| p.status.eq_ignore_ascii_case("running"))
        .count();
    let failed_pods = store
        .pods
        .iter()
        .filter(|p| {
            let s = p.status.to_ascii_lowercase();
            s.contains("fail") || s.contains("crash") || s.contains("error")
        })
        .count();

    let total_deployments = store.deployments.len();
    let total_services = store.services.len();
    let total_namespaces = store.namespaces.len();
    let warning_events = store
        .events
        .iter()
        .filter(|e| e.r#type.eq_ignore_ascii_case("warning"))
        .count();

    ClusterSummary {
        total_nodes,
        ready_nodes,
        total_pods,
        running_pods,
        failed_pods,
        total_deployments,
        total_services,
        total_namespaces,
        warning_events,
    }
}

pub fn render_summary_card<'a, Message: 'a>(
    title: &'static str,
    value: String,
    subtitle: String,
    watermark_icon: Option<Icon>,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let text_col = column![
        text(title).size(12).color(palette.text_secondary),
        text(value).size(24).color(palette.text_primary),
        text(subtitle).size(11).color(palette.text_muted),
    ]
    .spacing(4)
    .width(Length::Fill);

    let card_content = if let Some(ic) = watermark_icon {
        let watermark_color = Color::from_rgba(
            palette.text_primary.r,
            palette.text_primary.g,
            palette.text_primary.b,
            0.18,
        );
        let icon_el = icon(ic, 24, watermark_color);
        row![text_col, icon_el]
            .spacing(8)
            .align_y(iced::Alignment::Start)
    } else {
        row![text_col]
    };

    container(card_content)
        .padding(14)
        .width(Length::FillPortion(1))
        .style(crate::theme::card_container_style(palette))
        .into()
}

pub fn dashboard_view<'a, Message: 'a>(
    store: &ClusterStore,
    palette: &'a OrbitPalette,
) -> Element<'a, Message> {
    let summary = compute_cluster_summary(store);

    let card_nodes = render_summary_card(
        "NODES",
        format!("{}/{}", summary.ready_nodes, summary.total_nodes),
        "Ready / Total".to_string(),
        Some(Icon::Server),
        palette,
    );

    let card_pods = render_summary_card(
        "PODS",
        format!("{}/{}", summary.running_pods, summary.total_pods),
        format!("{} failed / degraded", summary.failed_pods),
        Some(Icon::Box),
        palette,
    );

    let card_deploy = render_summary_card(
        "DEPLOYMENTS",
        summary.total_deployments.to_string(),
        "Total active".to_string(),
        Some(Icon::Layers),
        palette,
    );

    let card_services = render_summary_card(
        "SERVICES",
        summary.total_services.to_string(),
        "Cluster endpoints".to_string(),
        Some(Icon::Network),
        palette,
    );

    let card_namespaces = render_summary_card(
        "NAMESPACES",
        summary.total_namespaces.to_string(),
        "Configured scopes".to_string(),
        Some(Icon::Folder),
        palette,
    );

    let card_events = render_summary_card(
        "WARNING EVENTS",
        summary.warning_events.to_string(),
        "Past 1 hour".to_string(),
        Some(Icon::TriangleAlert),
        palette,
    );

    let row1 = row![card_nodes, card_pods, card_deploy].spacing(16);
    let row2 = row![card_services, card_namespaces, card_events].spacing(16);

    column![
        text("Cluster Health & Workload Overview")
            .size(18)
            .color(palette.text_primary),
        row1,
        row2,
    ]
    .spacing(20)
    .padding(24)
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_card_styling_and_values() {
        let palette = OrbitPalette::dark();
        let store = ClusterStore::default();
        let summary = compute_cluster_summary(&store);
        assert_eq!(summary.total_nodes, 0);

        let _card: Element<'_, ()> = render_summary_card(
            "NODES",
            "3/3".into(),
            "Ready / Total".into(),
            Some(Icon::Server),
            &palette,
        );
    }
}
