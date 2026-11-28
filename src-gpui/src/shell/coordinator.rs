use std::collections::HashMap;

use crate::components::{ActionToastKind, ActionToastState, NotificationSeverity};
use crate::navigation::{LogsTabTarget, SidebarLayoutState, WorkspaceView};
use crate::state::{
    AddClusterModalState, ClusterBridgeCommand, ClusterBridgeEvent, ClusterSwitcherState,
    PortForwardsState, WorkloadActionKind, WorkspaceResourcesState, WorkspaceTabsState,
};
use crate::views;
use crate::{PortForwardModalState, ResourceDrawerState, WorkloadActionModalState};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyShortcutModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub platform: bool,
}

pub fn select_workspace_view_with_state(
    state: &ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    resources: &mut WorkspaceResourcesState,
    view: WorkspaceView,
) -> Option<ClusterBridgeCommand> {
    sidebar.select_view(view);
    load_command_for_active_view(state, resources, Some(view))
}

pub fn select_workspace_view_with_tabs(
    state: &ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    resources: &mut WorkspaceResourcesState,
    tabs: &mut WorkspaceTabsState,
    view: WorkspaceView,
) -> Vec<ClusterBridgeCommand> {
    let mut commands = Vec::new();
    tabs.open_tab(view);
    if let Some(cmd) = select_workspace_view_with_state(state, sidebar, resources, view) {
        commands.push(cmd);
    }
    if let Some(save_cmd) = tabs.persistence_command() {
        commands.push(save_cmd);
    }
    commands
}

pub fn load_command_for_active_view(
    state: &ClusterSwitcherState,
    resources: &mut WorkspaceResourcesState,
    view: Option<WorkspaceView>,
) -> Option<ClusterBridgeCommand> {
    resources.load_command_for_active_view(state, view)
}

pub fn reset_all_workspace_states_for_disconnect(resources: &mut WorkspaceResourcesState) {
    resources.reset_all_for_disconnect();
}

pub fn disconnect_cluster_with_state(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    resources: &mut WorkspaceResourcesState,
    tabs: &mut WorkspaceTabsState,
    drawer: &mut ResourceDrawerState,
) -> Option<ClusterBridgeCommand> {
    let cmd = state.begin_disconnect()?;
    reset_all_workspace_states_for_disconnect(resources);
    tabs.clear_active_cluster();
    drawer.reset_for_context_change();
    sidebar.on_cluster_disconnected();
    Some(cmd)
}

pub fn apply_cluster_event_with_tabs(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    resources: &mut WorkspaceResourcesState,
    tabs: &mut WorkspaceTabsState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let is_connected = matches!(
        &event,
        ClusterBridgeEvent::ClusterConnected { .. }
            | ClusterBridgeEvent::ClusterAdded {
                switch_outcome: Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome {
                    healthy: true,
                    ..
                }),
                ..
            }
    );
    let is_disconnected = matches!(&event, ClusterBridgeEvent::ClusterDisconnected { .. });
    let mut commands = apply_cluster_event_with_workspace(state, sidebar, resources, event);

    if is_connected {
        if let Some(id) = state.active_cluster_id.clone() {
            let restored = tabs.activate_cluster(&id);
            sidebar.sync_for_view(restored);
            if let Some(load_cmd) = load_command_for_active_view(state, resources, restored)
                && !commands.contains(&load_cmd)
            {
                commands.push(load_cmd);
            }
        }
    } else if is_disconnected {
        tabs.clear_active_cluster();
        sidebar.on_cluster_disconnected();
    }

    commands
}

pub fn handle_tab_shortcut_with_state(
    state: &ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    resources: &mut WorkspaceResourcesState,
    tabs: &mut WorkspaceTabsState,
    key: &str,
    modifiers: KeyShortcutModifiers,
) -> (bool, Vec<ClusterBridgeCommand>) {
    if !(modifiers.ctrl || modifiers.platform) {
        return (false, Vec::new());
    }

    let handled = if key.eq_ignore_ascii_case("tab") {
        if modifiers.shift {
            tabs.activate_prev_tab();
        } else {
            tabs.activate_next_tab();
        }
        true
    } else if key.eq_ignore_ascii_case("w") {
        tabs.close_active_tab();
        true
    } else {
        false
    };

    if !handled {
        return (false, Vec::new());
    }

    let active = tabs.active_view();
    sidebar.sync_for_view(active);

    let mut commands = Vec::new();
    if let Some(cmd) = load_command_for_active_view(state, resources, active) {
        commands.push(cmd);
    }
    if let Some(save_cmd) = tabs.persistence_command() {
        commands.push(save_cmd);
    }

    (true, commands)
}

pub fn apply_cluster_event_with_workspace(
    state: &mut ClusterSwitcherState,
    sidebar: &mut SidebarLayoutState,
    resources: &mut WorkspaceResourcesState,
    event: ClusterBridgeEvent,
) -> Vec<ClusterBridgeCommand> {
    let mut commands = Vec::new();
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterAdded {
            switch_outcome:
                Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome { healthy: true, .. }),
            ..
        } => {
            state.apply_event(event);
            sidebar.on_cluster_connected();
            resources.namespaces_view.reset_for_cluster();
            if let Some(id) = state.active_cluster_id.as_deref() {
                commands.push(resources.overview.begin_load_for_cluster(id));
                commands.push(resources.namespaces.begin_load_for_cluster(id));

                macro_rules! sync_view_on_connect {
                    ($view:expr, $target:expr) => {
                        if sidebar.active_view == $view {
                            commands.push($target.begin_load_for_cluster(id));
                        } else {
                            $target.reset_for_cluster(id);
                        }
                    };
                }

                sync_view_on_connect!(
                    WorkspaceView::WorkloadsOverview,
                    resources.workloads_overview
                );
                sync_view_on_connect!(WorkspaceView::Deployments, resources.deployments);
                sync_view_on_connect!(WorkspaceView::StatefulSets, resources.statefulsets);
                sync_view_on_connect!(WorkspaceView::DaemonSets, resources.daemonsets);
                sync_view_on_connect!(WorkspaceView::ReplicaSets, resources.replicasets);
                sync_view_on_connect!(WorkspaceView::Jobs, resources.jobs);
                sync_view_on_connect!(WorkspaceView::CronJobs, resources.cronjobs);
                sync_view_on_connect!(WorkspaceView::Pods, resources.pods);
                sync_view_on_connect!(WorkspaceView::Services, resources.services);
                sync_view_on_connect!(WorkspaceView::Ingresses, resources.ingresses);
                sync_view_on_connect!(WorkspaceView::ConfigMaps, resources.configmaps);
                sync_view_on_connect!(WorkspaceView::Secrets, resources.secrets);
                sync_view_on_connect!(
                    WorkspaceView::PersistentVolumes,
                    resources.persistent_volumes
                );
                sync_view_on_connect!(
                    WorkspaceView::PersistentVolumeClaims,
                    resources.persistent_volume_claims
                );
                sync_view_on_connect!(WorkspaceView::NetworkPolicies, resources.network_policies);
                sync_view_on_connect!(WorkspaceView::StorageOverview, resources.storage_overview);
                sync_view_on_connect!(WorkspaceView::PodSecurity, resources.pod_security);
                sync_view_on_connect!(WorkspaceView::ResourceQuotas, resources.resource_quotas);
                sync_view_on_connect!(WorkspaceView::Nodes, resources.nodes);
                sync_view_on_connect!(WorkspaceView::Events, resources.events);
                resources
                    .list_views
                    .on_cluster_connected(sidebar.active_view, id, &mut commands);
            }
        }
        ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. }
        | ClusterBridgeEvent::ClusterDisconnected { .. }
        | ClusterBridgeEvent::ClusterAdded {
            switch_outcome:
                Some(orbit_core::kubernetes::manager::ClusterSwitchOutcome { healthy: false, .. }),
            ..
        } => {
            state.apply_event(event);
            reset_all_workspace_states_for_disconnect(resources);
        }
        ClusterBridgeEvent::OverviewLoaded {
            cluster_id,
            generation,
            summary,
        } => {
            resources
                .overview
                .apply_loaded(&cluster_id, generation, summary);
        }
        ClusterBridgeEvent::OverviewLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .overview
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::NamespacesLoaded {
            cluster_id,
            generation,
            namespaces: loaded_namespaces,
        } => {
            let (_applied, pruned) =
                resources
                    .namespaces
                    .apply_loaded(&cluster_id, generation, loaded_namespaces);
            if let Some(pruned_selection) = pruned {
                commands.push(ClusterBridgeCommand::SaveNamespaceSelection {
                    cluster_id,
                    namespaces: pruned_selection,
                });
            }
        }
        ClusterBridgeEvent::NamespacesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .namespaces
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::DeploymentsLoaded {
            cluster_id,
            generation,
            deployments: loaded_deployments,
        } => {
            resources
                .deployments
                .apply_loaded(&cluster_id, generation, loaded_deployments);
        }
        ClusterBridgeEvent::DeploymentsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .deployments
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::StatefulSetsLoaded {
            cluster_id,
            generation,
            statefulsets: loaded_statefulsets,
        } => {
            resources
                .statefulsets
                .apply_loaded(&cluster_id, generation, loaded_statefulsets);
        }
        ClusterBridgeEvent::StatefulSetsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .statefulsets
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::DaemonSetsLoaded {
            cluster_id,
            generation,
            daemonsets: loaded_daemonsets,
        } => {
            resources
                .daemonsets
                .apply_loaded(&cluster_id, generation, loaded_daemonsets);
        }
        ClusterBridgeEvent::DaemonSetsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .daemonsets
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ReplicaSetsLoaded {
            cluster_id,
            generation,
            replicasets: loaded_replicasets,
        } => {
            resources
                .replicasets
                .apply_loaded(&cluster_id, generation, loaded_replicasets);
        }
        ClusterBridgeEvent::ReplicaSetsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .replicasets
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::JobsLoaded {
            cluster_id,
            generation,
            jobs: loaded_jobs,
        } => {
            resources
                .jobs
                .apply_loaded(&cluster_id, generation, loaded_jobs);
        }
        ClusterBridgeEvent::JobsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .jobs
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::CronJobsLoaded {
            cluster_id,
            generation,
            cronjobs: loaded_cronjobs,
        } => {
            resources
                .cronjobs
                .apply_loaded(&cluster_id, generation, loaded_cronjobs);
        }
        ClusterBridgeEvent::CronJobsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .cronjobs
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PodsLoaded {
            cluster_id,
            generation,
            pods: loaded_pods,
        } => {
            resources
                .pods
                .apply_loaded(&cluster_id, generation, loaded_pods);
        }
        ClusterBridgeEvent::PodsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .pods
                .apply_failed(&cluster_id, generation, message);
        }
        other @ ClusterBridgeEvent::ContextsLoaded { .. } => {
            state.apply_event(other);
        }
        ClusterBridgeEvent::NodesLoaded {
            cluster_id,
            generation,
            nodes: loaded_nodes,
        } => {
            resources
                .nodes
                .apply_loaded(&cluster_id, generation, loaded_nodes);
        }
        ClusterBridgeEvent::NodesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .nodes
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::EventsLoaded {
            cluster_id,
            generation,
            events: loaded_events,
        } => {
            resources
                .events
                .apply_loaded(&cluster_id, generation, loaded_events);
        }
        ClusterBridgeEvent::EventsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .events
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ServicesLoaded {
            cluster_id,
            generation,
            services: loaded_services,
        } => {
            resources
                .services
                .apply_loaded(&cluster_id, generation, loaded_services);
        }
        ClusterBridgeEvent::ServicesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .services
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::IngressesLoaded {
            cluster_id,
            generation,
            ingresses: loaded_ingresses,
        } => {
            resources
                .ingresses
                .apply_loaded(&cluster_id, generation, loaded_ingresses);
        }
        ClusterBridgeEvent::IngressesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .ingresses
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ConfigMapsLoaded {
            cluster_id,
            generation,
            configmaps: loaded_configmaps,
        } => {
            resources
                .configmaps
                .apply_loaded(&cluster_id, generation, loaded_configmaps);
        }
        ClusterBridgeEvent::ConfigMapsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .configmaps
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::SecretsLoaded {
            cluster_id,
            generation,
            secrets: loaded_secrets,
        } => {
            resources
                .secrets
                .apply_loaded(&cluster_id, generation, loaded_secrets);
        }
        ClusterBridgeEvent::SecretsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .secrets
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::WorkloadsOverviewLoaded {
            cluster_id,
            generation,
            snapshot,
        } => {
            resources
                .workloads_overview
                .apply_loaded(&cluster_id, generation, snapshot);
        }
        ClusterBridgeEvent::WorkloadsOverviewLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .workloads_overview
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PersistentVolumesLoaded {
            cluster_id,
            generation,
            persistent_volumes: loaded_pvs,
        } => {
            resources
                .persistent_volumes
                .apply_loaded(&cluster_id, generation, loaded_pvs);
        }
        ClusterBridgeEvent::PersistentVolumesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .persistent_volumes
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PersistentVolumeClaimsLoaded {
            cluster_id,
            generation,
            persistent_volume_claims: loaded_pvcs,
        } => {
            resources
                .persistent_volume_claims
                .apply_loaded(&cluster_id, generation, loaded_pvcs);
        }
        ClusterBridgeEvent::PersistentVolumeClaimsLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .persistent_volume_claims
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::NetworkPoliciesLoaded {
            cluster_id,
            generation,
            network_policies: loaded_network_policies,
        } => {
            resources.network_policies.apply_loaded(
                &cluster_id,
                generation,
                loaded_network_policies,
            );
        }
        ClusterBridgeEvent::NetworkPoliciesLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .network_policies
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::StorageOverviewLoaded {
            cluster_id,
            generation,
            snapshot,
        } => {
            resources
                .storage_overview
                .apply_loaded(&cluster_id, generation, snapshot);
        }
        ClusterBridgeEvent::StorageOverviewLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .storage_overview
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::PodSecurityLoaded {
            cluster_id,
            generation,
            policies,
        } => {
            resources
                .pod_security
                .apply_loaded(&cluster_id, generation, policies);
        }
        ClusterBridgeEvent::PodSecurityLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .pod_security
                .apply_failed(&cluster_id, generation, message);
        }
        ClusterBridgeEvent::ResourceQuotasLoaded {
            cluster_id,
            generation,
            policies,
        } => {
            resources
                .resource_quotas
                .apply_loaded(&cluster_id, generation, policies);
        }
        ClusterBridgeEvent::ResourceQuotasLoadFailed {
            cluster_id,
            generation,
            message,
        } => {
            resources
                .resource_quotas
                .apply_failed(&cluster_id, generation, message);
        }
        other @ (ClusterBridgeEvent::HpasLoaded { .. }
        | ClusterBridgeEvent::HpasLoadFailed { .. }
        | ClusterBridgeEvent::StorageClassesLoaded { .. }
        | ClusterBridgeEvent::StorageClassesLoadFailed { .. }
        | ClusterBridgeEvent::SecurityOverviewLoaded { .. }
        | ClusterBridgeEvent::SecurityOverviewLoadFailed { .. }
        | ClusterBridgeEvent::AdmissionWebhooksLoaded { .. }
        | ClusterBridgeEvent::AdmissionWebhooksLoadFailed { .. }) => {
            resources.list_views.apply_event(&other);
        }
        ClusterBridgeEvent::ResourceYamlLoaded { .. }
        | ClusterBridgeEvent::ResourceYamlLoadFailed { .. }
        | ClusterBridgeEvent::AddClusterFailed { .. }
        | ClusterBridgeEvent::SettingsSaveFailed { .. }
        | ClusterBridgeEvent::WorkloadActionSucceeded { .. }
        | ClusterBridgeEvent::WorkloadActionFailed { .. }
        | ClusterBridgeEvent::LogTargetsResolved { .. }
        | ClusterBridgeEvent::LogLinesReceived { .. }
        | ClusterBridgeEvent::LogStreamStatus { .. }
        | ClusterBridgeEvent::LogStreamEnded { .. }
        | ClusterBridgeEvent::PortForwardsLoaded { .. }
        | ClusterBridgeEvent::PortForwardStarted { .. }
        | ClusterBridgeEvent::PortForwardStopped { .. }
        | ClusterBridgeEvent::PortForwardsCleared { .. }
        | ClusterBridgeEvent::PortForwardFailed { .. } => {}
        ClusterBridgeEvent::ClusterAdded { .. } => {
            state.apply_event(event);
        }
    }
    commands
}

pub type PortForwardEventOutcome = (Vec<ClusterBridgeCommand>, Option<(u64, ActionToastKind)>);

pub fn apply_port_forward_event(
    port_forwards: &mut PortForwardsState,
    modal: &mut Option<PortForwardModalState>,
    toast: &mut ActionToastState,
    event: &ClusterBridgeEvent,
) -> Option<PortForwardEventOutcome> {
    match event {
        ClusterBridgeEvent::ClusterConnected { outcome, .. } => {
            if outcome.healthy {
                let cmd = port_forwards.begin_load_for_cluster(&outcome.context_name);
                Some((vec![cmd], None))
            } else {
                port_forwards.reset_for_disconnected();
                *modal = None;
                Some((Vec::new(), None))
            }
        }
        ClusterBridgeEvent::ClusterDisconnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. } => {
            port_forwards.reset_for_disconnected();
            *modal = None;
            Some((Vec::new(), None))
        }
        ClusterBridgeEvent::PortForwardsLoaded {
            cluster_id,
            generation,
            port_forwards: loaded,
        } => {
            port_forwards.apply_loaded(cluster_id, *generation, loaded.clone());
            Some((Vec::new(), None))
        }
        ClusterBridgeEvent::PortForwardStarted {
            cluster_id,
            port_forward,
        } => {
            port_forwards.apply_started(cluster_id, port_forward.clone());
            if let Some(m) = modal.as_ref()
                && m.cluster_id == *cluster_id
                && m.namespace == port_forward.namespace
                && m.kind.eq_ignore_ascii_case(&port_forward.kind)
                && m.name == port_forward.name
                && m.parsed_local_port() == Some(port_forward.local_port)
            {
                *modal = None;
            }
            let toast_id = toast.show_success(
                "Port Forward Active",
                format!(
                    "Forwarding 127.0.0.1:{} -> {}/{}:{}",
                    port_forward.local_port,
                    port_forward.kind,
                    port_forward.name,
                    port_forward.remote_port
                ),
            );
            Some((Vec::new(), Some((toast_id, ActionToastKind::Success))))
        }
        ClusterBridgeEvent::PortForwardStopped { cluster_id, id } => {
            port_forwards.apply_stopped(cluster_id, id);
            Some((Vec::new(), None))
        }
        ClusterBridgeEvent::PortForwardsCleared { cluster_id } => {
            port_forwards.apply_cleared(cluster_id);
            let toast_id =
                toast.show_success("Port Forwards Stopped", "Stopped all active port forwards");
            Some((Vec::new(), Some((toast_id, ActionToastKind::Success))))
        }
        ClusterBridgeEvent::PortForwardFailed {
            cluster_id,
            id,
            message,
        } => {
            port_forwards.apply_failed(cluster_id, id.as_deref(), message.clone());
            if let Some(m) = modal.as_mut()
                && m.cluster_id == *cluster_id
            {
                m.submitting = false;
                m.error = Some(message.clone());
            }
            let toast_id = toast.show_error("Port Forward Failed", message.clone());
            Some((Vec::new(), Some((toast_id, ActionToastKind::Error))))
        }
        _ => None,
    }
}

pub fn open_logs_tab_in_state(
    tabs: &mut WorkspaceTabsState,
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
    next_stream_id: &mut u64,
    cluster_id: Option<&str>,
    kind: &str,
    namespace: &str,
    name: &str,
) -> Option<ClusterBridgeCommand> {
    let target = LogsTabTarget {
        kind: kind.to_string(),
        namespace: namespace.to_string(),
        name: name.to_string(),
    };
    let is_new = tabs.open_logs_tab(target.clone());
    if !is_new && log_tabs.contains_key(&target) {
        return None;
    }
    let stream_id = *next_stream_id;
    *next_stream_id = next_stream_id.saturating_add(1);
    let viewer = views::logs::LogViewerState::new(target.clone(), stream_id);
    let tail_lines = viewer.tail_lines;
    let include_unavailable_pods = viewer.include_unavailable_pods;
    log_tabs.insert(target.clone(), viewer);

    let cluster_id = cluster_id?.to_string();
    Some(ClusterBridgeCommand::StartLogStream {
        cluster_id,
        stream_id,
        namespace: target.namespace,
        workload_kind: target.kind,
        workload_name: target.name,
        pod_filter: None,
        container_filter: None,
        tail_lines,
        include_unavailable_pods,
    })
}

pub fn cleanup_closed_log_tabs_in_state(
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
    closed: Vec<LogsTabTarget>,
) -> Vec<ClusterBridgeCommand> {
    closed
        .into_iter()
        .filter_map(|target| {
            log_tabs
                .remove(&target)
                .map(|state| ClusterBridgeCommand::StopLogStream {
                    stream_id: state.stream_id,
                })
        })
        .collect()
}

pub fn clear_all_log_tabs_in_state(
    tabs: &mut WorkspaceTabsState,
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
) -> Vec<ClusterBridgeCommand> {
    tabs.strip_logs_tabs();
    log_tabs
        .drain()
        .map(|(_, state)| ClusterBridgeCommand::StopLogStream {
            stream_id: state.stream_id,
        })
        .collect()
}

pub fn apply_log_bridge_event(
    log_tabs: &mut HashMap<LogsTabTarget, views::logs::LogViewerState>,
    active_cluster_id: Option<&str>,
    event: &ClusterBridgeEvent,
) {
    match event {
        ClusterBridgeEvent::LogTargetsResolved {
            cluster_id,
            stream_id,
            pods,
            excluded_unavailable_count,
            ..
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.pods = pods.clone();
                state.excluded_unavailable_count = *excluded_unavailable_count;
            }
        }
        ClusterBridgeEvent::LogLinesReceived {
            cluster_id,
            stream_id,
            pod,
            container,
            lines,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.push_lines(pod, container, lines.clone());
            }
        }
        ClusterBridgeEvent::LogStreamStatus {
            cluster_id,
            stream_id,
            message,
            is_error,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.status_message = Some(message.clone());
                state.status_is_error = *is_error;
            }
        }
        ClusterBridgeEvent::LogStreamEnded {
            cluster_id,
            stream_id,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return;
            }
            if let Some(state) = log_tabs.values_mut().find(|s| s.stream_id == *stream_id) {
                state.is_streaming = false;
            }
        }
        _ => {}
    }
}

pub fn apply_add_cluster_modal_event(modal: &mut AddClusterModalState, event: &ClusterBridgeEvent) {
    match event {
        ClusterBridgeEvent::ClusterAdded { .. } => {
            modal.close();
        }
        ClusterBridgeEvent::AddClusterFailed { message } => {
            modal.is_submitting = false;
            modal.submit_error = Some(message.clone());
        }
        _ => {}
    }
}

pub fn apply_workload_action_event(
    active_cluster_id: Option<&str>,
    modal: &mut WorkloadActionModalState,
    toast: &mut ActionToastState,
    drawer: &mut ResourceDrawerState,
    resources: &mut WorkspaceResourcesState,
    event: &ClusterBridgeEvent,
) -> Option<(u64, ActionToastKind, Vec<ClusterBridgeCommand>)> {
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. }
        | ClusterBridgeEvent::ClusterDisconnected { .. } => {
            modal.reset_for_disconnect();
            toast.reset_for_disconnect();
            None
        }
        ClusterBridgeEvent::WorkloadActionSucceeded {
            cluster_id,
            action,
            kind,
            namespace,
            name,
            message,
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return None;
            }
            modal.close();
            if *action == WorkloadActionKind::Delete {
                drawer.close_if_deleted(kind, namespace, name);
            }
            let title = match action {
                WorkloadActionKind::Scale => format!("Scaled {kind}"),
                WorkloadActionKind::Restart => format!("Restarted {kind}"),
                WorkloadActionKind::RestartPod => "Restarted Pod".to_string(),
                WorkloadActionKind::TriggerCronJob => "Triggered CronJob".to_string(),
                WorkloadActionKind::Delete => format!("Deleted {kind}"),
            };
            let toast_id = toast.show_success(title, message.clone());
            let mut refresh_commands = Vec::new();
            match kind.as_str() {
                "Deployment" => {
                    if let Some(cmd) = resources.deployments.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "StatefulSet" => {
                    if let Some(cmd) = resources.statefulsets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "DaemonSet" => {
                    if let Some(cmd) = resources.daemonsets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "ReplicaSet" => {
                    if let Some(cmd) = resources.replicasets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Pod" => {
                    if let Some(cmd) = resources.pods.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "CronJob" => {
                    if let Some(cmd) = resources.cronjobs.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                    if !resources.jobs.jobs.is_empty()
                        && let Some(cmd) = resources.jobs.begin_refresh()
                    {
                        refresh_commands.push(cmd);
                    }
                }
                "Job" => {
                    if let Some(cmd) = resources.jobs.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Service" => {
                    if let Some(cmd) = resources.services.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Ingress" => {
                    if let Some(cmd) = resources.ingresses.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "ConfigMap" => {
                    if let Some(cmd) = resources.configmaps.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Secret" => {
                    if let Some(cmd) = resources.secrets.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "NetworkPolicy" => {
                    if let Some(cmd) = resources.network_policies.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "ResourceQuota" | "LimitRange" => {
                    if let Some(cmd) = resources.resource_quotas.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Namespace" => {
                    if let Some(cmd) = resources.namespaces.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "PersistentVolume" => {
                    if let Some(cmd) = resources.persistent_volumes.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "PersistentVolumeClaim" => {
                    if let Some(cmd) = resources.persistent_volume_claims.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Node" => {
                    if let Some(cmd) = resources.nodes.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                "Event" => {
                    if let Some(cmd) = resources.events.begin_refresh() {
                        refresh_commands.push(cmd);
                    }
                }
                _ => {}
            }
            Some((toast_id, ActionToastKind::Success, refresh_commands))
        }
        ClusterBridgeEvent::WorkloadActionFailed {
            cluster_id,
            action,
            kind,
            message,
            ..
        } => {
            if active_cluster_id != Some(cluster_id.as_str()) {
                return None;
            }
            if modal.is_open() {
                modal.set_error(message.clone());
            }
            let title = match action {
                WorkloadActionKind::Scale => format!("Scale {kind} Failed"),
                WorkloadActionKind::Restart => format!("Restart {kind} Failed"),
                WorkloadActionKind::RestartPod => "Restart Pod Failed".to_string(),
                WorkloadActionKind::TriggerCronJob => "Trigger CronJob Failed".to_string(),
                WorkloadActionKind::Delete => format!("Delete {kind} Failed"),
            };
            let toast_id = toast.show_error(title, message.clone());
            Some((toast_id, ActionToastKind::Error, Vec::new()))
        }
        _ => None,
    }
}

pub fn apply_drawer_lifecycle_for_event(
    drawer: &mut ResourceDrawerState,
    event: &ClusterBridgeEvent,
) {
    match event {
        ClusterBridgeEvent::ClusterConnected { .. }
        | ClusterBridgeEvent::ClusterUnreachable { .. }
        | ClusterBridgeEvent::ConnectionError { .. }
        | ClusterBridgeEvent::ClusterDisconnected { .. }
        | ClusterBridgeEvent::ClusterAdded {
            switch_outcome: Some(_),
            ..
        } => {
            drawer.reset_for_context_change();
        }
        ClusterBridgeEvent::ResourceYamlLoaded {
            cluster_id,
            generation,
            kind,
            namespace,
            name,
            yaml,
        } => {
            drawer.apply_yaml_loaded(
                cluster_id,
                *generation,
                kind,
                namespace.as_deref(),
                name,
                yaml.clone(),
            );
        }
        ClusterBridgeEvent::ResourceYamlLoadFailed {
            cluster_id,
            generation,
            kind,
            namespace,
            name,
            message,
        } => {
            drawer.apply_yaml_failed(
                cluster_id,
                *generation,
                kind,
                namespace.as_deref(),
                name,
                message.clone(),
            );
        }
        _ => {}
    }
}

pub fn notification_from_bridge_event(
    event: &ClusterBridgeEvent,
) -> Option<(NotificationSeverity, String, String)> {
    match event {
        ClusterBridgeEvent::WorkloadActionSucceeded {
            kind,
            name,
            message,
            ..
        } => Some((
            NotificationSeverity::Success,
            format!("{kind} {name}"),
            message.clone(),
        )),
        ClusterBridgeEvent::WorkloadActionFailed {
            kind,
            name,
            message,
            ..
        } => Some((
            NotificationSeverity::Error,
            format!("Action failed: {kind} {name}"),
            message.clone(),
        )),
        ClusterBridgeEvent::PortForwardFailed { message, .. } => Some((
            NotificationSeverity::Error,
            "Port Forward Failed".to_string(),
            message.clone(),
        )),
        ClusterBridgeEvent::SettingsSaveFailed { message } => Some((
            NotificationSeverity::Error,
            "Settings Save Failed".to_string(),
            message.clone(),
        )),
        ClusterBridgeEvent::ConnectionError {
            cluster_id,
            message,
            ..
        } => Some((
            NotificationSeverity::Error,
            cluster_id
                .as_deref()
                .map(|id| format!("Connection Error ({id})"))
                .unwrap_or_else(|| "Connection Error".to_string()),
            message.clone(),
        )),
        ClusterBridgeEvent::ClusterUnreachable { outcome, .. } => Some((
            NotificationSeverity::Warning,
            format!("Cluster Unreachable ({})", outcome.context_name),
            outcome
                .error_message
                .clone()
                .unwrap_or_else(|| "Cluster did not respond.".to_string()),
        )),
        ClusterBridgeEvent::AddClusterFailed { message } => Some((
            NotificationSeverity::Error,
            "Add Cluster Failed".to_string(),
            message.clone(),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ClusterConnectionState, DeploymentsLoadStatus};
    use orbit_core::kubernetes::manager::ClusterSwitchOutcome;
    use orbit_core::kubernetes::models::{ClusterInfo, UserProfileInfo};

    #[test]
    fn coordinator_select_view_and_disconnect_roundtrip_via_workspace_resources() {
        let mut switcher = ClusterSwitcherState::default();
        let mut sidebar = SidebarLayoutState::default();
        let mut resources = WorkspaceResourcesState::default();
        let mut tabs = WorkspaceTabsState::default();
        let mut drawer = ResourceDrawerState::default();

        let connect_event = ClusterBridgeEvent::ClusterConnected {
            outcome: ClusterSwitchOutcome {
                context_name: "ctx-prod".to_string(),
                healthy: true,
                k8s_version: Some("v1.30.0".to_string()),
                error_message: None,
            },
            clusters: vec![ClusterInfo {
                id: "ctx-prod".to_string(),
                name: "ctx-prod".to_string(),
                status: "online".to_string(),
                alias: None,
                tags: Vec::new(),
                color: None,
                description: None,
                server_url: Some("https://127.0.0.1:6443".to_string()),
            }],
            profile: UserProfileInfo {
                active_context: Some("ctx-prod".to_string()),
                user_name: None,
                auth_type: "token".to_string(),
                cluster_name: Some("ctx-prod".to_string()),
                server_url: Some("https://127.0.0.1:6443".to_string()),
                kubeconfig_paths: Vec::new(),
                k8s_version: Some("v1.30.0".to_string()),
            },
        };

        let connect_cmds = apply_cluster_event_with_tabs(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            connect_event,
        );
        assert!(matches!(
            switcher.connection_state,
            ClusterConnectionState::Connected { .. }
        ));
        assert!(
            connect_cmds
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadOverview { .. }))
        );

        let select_cmds = select_workspace_view_with_tabs(
            &switcher,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            WorkspaceView::Deployments,
        );
        assert_eq!(sidebar.active_view, WorkspaceView::Deployments);
        assert_eq!(resources.deployments.status, DeploymentsLoadStatus::Loading);
        assert!(
            select_cmds
                .iter()
                .any(|c| matches!(c, ClusterBridgeCommand::LoadDeployments { .. }))
        );

        let (handled, shortcut_cmds) = handle_tab_shortcut_with_state(
            &switcher,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            "tab",
            KeyShortcutModifiers {
                ctrl: true,
                shift: false,
                platform: false,
            },
        );
        assert!(handled);
        assert!(!shortcut_cmds.is_empty());

        let disconnect_cmd = disconnect_cluster_with_state(
            &mut switcher,
            &mut sidebar,
            &mut resources,
            &mut tabs,
            &mut drawer,
        );
        assert_eq!(
            disconnect_cmd,
            Some(ClusterBridgeCommand::DisconnectCluster)
        );
        assert_eq!(resources.deployments.status, DeploymentsLoadStatus::Idle);
        assert!(resources.deployments.cluster_id.is_none());
    }
}
