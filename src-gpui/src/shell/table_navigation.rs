use gpui::{Context, Window};

use crate::components::{ResourceDrawerTabId, ResourceDrawerTarget};
use crate::navigation::{WorkspaceTab, WorkspaceView};
use crate::{OrbitShell, views};

impl OrbitShell {
    pub(crate) fn is_text_input_focused(&self) -> bool {
        if self.add_cluster.is_open
            || self.port_forward_modal.is_some()
            || self.workload_action_modal.is_open()
            || self.command_palette.is_open
            || self.tabs.picker_open
            || self.resources.namespaces.dropdown_open
        {
            return true;
        }
        if let Some(WorkspaceTab::Logs(target)) = self.tabs.active_tab()
            && self
                .log_tabs
                .get(&target)
                .is_some_and(|log_state| log_state.search_focused)
        {
            return true;
        }
        match self.sidebar.active_view {
            WorkspaceView::Pods => self.resources.pods.table.filters.search_focused,
            WorkspaceView::Deployments => self.resources.deployments.table.filters.search_focused,
            WorkspaceView::Services => self.resources.services.table.filters.search_focused,
            WorkspaceView::Events => self.resources.events.table.filters.search_focused,
            WorkspaceView::PortForward => self.resources.port_forwards.search_focused,
            _ => false,
        }
    }

    pub(crate) fn focus_active_view_search(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(WorkspaceTab::Logs(target)) = self.tabs.active_tab() {
            if let Some(log_state) = self.log_tabs.get_mut(&target) {
                log_state.search_focused = true;
                self.log_search_focus.focus(window);
                cx.notify();
                return true;
            }
            return false;
        }
        match self.sidebar.active_view {
            WorkspaceView::Pods => {
                self.resources.pods.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::Deployments => {
                self.resources.deployments.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::Services => {
                self.resources.services.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::Events => {
                self.resources.events.table.filters.search_focused = true;
                cx.notify();
                true
            }
            WorkspaceView::PortForward => {
                self.resources.port_forwards.search_focused = true;
                cx.notify();
                true
            }
            _ => false,
        }
    }

    pub(crate) fn move_active_table_selection(
        &mut self,
        delta: isize,
        cx: &mut Context<Self>,
    ) -> bool {
        if matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_))) {
            return false;
        }
        let moved = match self.sidebar.active_view {
            WorkspaceView::Pods => {
                let keys: Vec<String> = views::pods::filter_and_sort_pods_with_filters(
                    &self.resources.pods.pods,
                    &self.resources.namespaces,
                    &self.resources.pods.table.filters,
                    self.resources.pods.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}", p.namespace, p.name))
                .collect();
                self.resources
                    .pods
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Deployments => {
                let keys: Vec<String> =
                    views::deployments::filter_and_sort_deployments_with_filters(
                        &self.resources.deployments.deployments,
                        &self.resources.namespaces,
                        &self.resources.deployments.table.filters,
                        self.resources.deployments.table.sort.as_ref(),
                    )
                    .into_iter()
                    .map(|d| format!("{}/{}", d.namespace, d.name))
                    .collect();
                self.resources
                    .deployments
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Services => {
                let keys: Vec<String> = views::services::filter_and_sort_services_with_filters(
                    &self.resources.services.services,
                    &self.resources.namespaces,
                    self.resources.services.type_filter,
                    &self.resources.services.table.filters,
                    self.resources.services.table.sort.as_ref(),
                )
                .into_iter()
                .map(|s| format!("{}/{}", s.namespace, s.name))
                .collect();
                self.resources
                    .services
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Events => {
                let keys: Vec<String> = views::events::filter_and_sort_events_with_filters(
                    &self.resources.events.events,
                    &self.resources.namespaces,
                    self.resources.events.type_filter,
                    &self.resources.events.table.filters,
                    self.resources.events.table.sort.as_ref(),
                )
                .into_iter()
                .map(|ev| {
                    if !ev.uid.is_empty() {
                        ev.uid.clone()
                    } else {
                        format!("{}/{}/{}", ev.namespace, ev.reason, ev.object_name)
                    }
                })
                .collect();
                self.resources
                    .events
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Nodes => {
                let keys: Vec<String> = views::nodes::sort_nodes(
                    &self.resources.nodes.nodes,
                    self.resources.nodes.table.sort.as_ref(),
                )
                .into_iter()
                .map(|n| n.name.clone())
                .collect();
                self.resources
                    .nodes
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Namespaces => {
                let keys: Vec<String> = views::namespaces::sort_namespaces(
                    &self.resources.namespaces.namespaces,
                    self.resources.namespaces_view.table.sort.as_ref(),
                )
                .into_iter()
                .map(|ns| ns.name.clone())
                .collect();
                self.resources
                    .namespaces_view
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::StatefulSets => {
                let keys: Vec<String> = views::statefulsets::filter_and_sort_statefulsets(
                    &self.resources.statefulsets.statefulsets,
                    &self.resources.namespaces,
                    self.resources.statefulsets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|s| format!("{}/{}", s.namespace, s.name))
                .collect();
                self.resources
                    .statefulsets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::DaemonSets => {
                let keys: Vec<String> = views::daemonsets::filter_and_sort_daemonsets(
                    &self.resources.daemonsets.daemonsets,
                    &self.resources.namespaces,
                    self.resources.daemonsets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|d| format!("{}/{}", d.namespace, d.name))
                .collect();
                self.resources
                    .daemonsets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::ReplicaSets => {
                let keys: Vec<String> = views::replicasets::filter_and_sort_replicasets(
                    &self.resources.replicasets.replicasets,
                    &self.resources.namespaces,
                    self.resources.replicasets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|r| format!("{}/{}", r.namespace, r.name))
                .collect();
                self.resources
                    .replicasets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Jobs => {
                let keys: Vec<String> = views::jobs::filter_and_sort_jobs(
                    &self.resources.jobs.jobs,
                    &self.resources.namespaces,
                    self.resources.jobs.table.sort.as_ref(),
                )
                .into_iter()
                .map(|j| format!("{}/{}", j.namespace, j.name))
                .collect();
                self.resources
                    .jobs
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::CronJobs => {
                let keys: Vec<String> = views::cronjobs::filter_and_sort_cronjobs(
                    &self.resources.cronjobs.cronjobs,
                    &self.resources.namespaces,
                    self.resources.cronjobs.table.sort.as_ref(),
                )
                .into_iter()
                .map(|c| format!("{}/{}", c.namespace, c.name))
                .collect();
                self.resources
                    .cronjobs
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Hpas => {
                let keys: Vec<String> = views::hpas::filter_and_sort_hpas(
                    &self.resources.list_views.hpas.hpas,
                    &self.resources.namespaces,
                    "",
                    self.resources.list_views.hpas.table.sort.as_ref(),
                )
                .into_iter()
                .map(|h| format!("{}/{}", h.namespace, h.name))
                .collect();
                self.resources
                    .list_views
                    .hpas
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Ingresses => {
                let keys: Vec<String> = views::ingresses::filter_and_sort_ingresses(
                    &self.resources.ingresses.ingresses,
                    &self.resources.namespaces,
                    self.resources.ingresses.table.sort.as_ref(),
                )
                .into_iter()
                .map(|i| format!("{}/{}", i.namespace, i.name))
                .collect();
                self.resources
                    .ingresses
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::PersistentVolumes => {
                let keys: Vec<String> = views::persistent_volumes::sort_persistent_volumes(
                    &self.resources.persistent_volumes.persistent_volumes,
                    self.resources.persistent_volumes.table.sort.as_ref(),
                )
                .into_iter()
                .map(|pv| pv.name.clone())
                .collect();
                self.resources
                    .persistent_volumes
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::PersistentVolumeClaims => {
                let keys: Vec<String> =
                    views::persistent_volume_claims::filter_and_sort_persistent_volume_claims(
                        &self
                            .resources
                            .persistent_volume_claims
                            .persistent_volume_claims,
                        &self.resources.namespaces,
                        self.resources.persistent_volume_claims.table.sort.as_ref(),
                    )
                    .into_iter()
                    .map(|pvc| format!("{}/{}", pvc.namespace, pvc.name))
                    .collect();
                self.resources
                    .persistent_volume_claims
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::StorageClasses => {
                let keys: Vec<String> = views::storage_classes::filter_and_sort_storage_classes(
                    &self.resources.list_views.storage_classes.classes,
                    "",
                    self.resources
                        .list_views
                        .storage_classes
                        .table
                        .sort
                        .as_ref(),
                )
                .into_iter()
                .map(|sc| sc.name.clone())
                .collect();
                self.resources
                    .list_views
                    .storage_classes
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::ConfigMaps => {
                let keys: Vec<String> = views::configmaps::filter_and_sort_configmaps(
                    &self.resources.configmaps.configmaps,
                    &self.resources.namespaces,
                    self.resources.configmaps.table.sort.as_ref(),
                )
                .into_iter()
                .map(|cm| format!("{}/{}", cm.namespace, cm.name))
                .collect();
                self.resources
                    .configmaps
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::Secrets => {
                let keys: Vec<String> = views::secrets::filter_and_sort_secrets(
                    &self.resources.secrets.secrets,
                    &self.resources.namespaces,
                    self.resources.secrets.table.sort.as_ref(),
                )
                .into_iter()
                .map(|s| format!("{}/{}", s.namespace, s.name))
                .collect();
                self.resources
                    .secrets
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::NetworkPolicies => {
                let keys: Vec<String> = views::network_policies::filter_and_sort_network_policies(
                    &self.resources.network_policies.network_policies,
                    &self.resources.namespaces,
                    self.resources.network_policies.table.sort.as_ref(),
                )
                .into_iter()
                .map(|np| format!("{}/{}", np.namespace, np.name))
                .collect();
                self.resources
                    .network_policies
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::PodSecurity => {
                let keys: Vec<String> = views::pod_security::filter_and_sort_pod_security_policies(
                    &self.resources.pod_security.policies,
                    &self.resources.namespaces,
                    self.resources.pod_security.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}", p.namespace, p.name))
                .collect();
                self.resources
                    .pod_security
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::ResourceQuotas => {
                let keys: Vec<String> = views::resource_quotas::filter_and_sort_resource_policies(
                    &self.resources.resource_quotas.policies,
                    &self.resources.namespaces,
                    self.resources.resource_quotas.kind_filter,
                    self.resources.resource_quotas.table.sort.as_ref(),
                )
                .into_iter()
                .map(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name))
                .collect();
                self.resources
                    .resource_quotas
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::AdmissionWebhooks => {
                let keys: Vec<String> =
                    views::admission_webhooks::filter_and_sort_admission_webhooks(
                        &self.resources.list_views.admission_webhooks.policies,
                        self.resources
                            .list_views
                            .admission_webhooks
                            .table
                            .sort
                            .as_ref(),
                    )
                    .into_iter()
                    .map(|w| format!("{}/{}", w.kind, w.name))
                    .collect();
                self.resources
                    .list_views
                    .admission_webhooks
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            WorkspaceView::SecurityOverview => {
                let keys: Vec<String> =
                    views::security_overview::filter_and_sort_security_policies(
                        &self.resources.list_views.security_overview.policies,
                        &self.resources.namespaces,
                        self.resources
                            .list_views
                            .security_overview
                            .table
                            .sort
                            .as_ref(),
                    )
                    .into_iter()
                    .map(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name))
                    .collect();
                self.resources
                    .list_views
                    .security_overview
                    .table
                    .move_selection_by_delta(delta, &keys)
                    .is_some()
            }
            _ => false,
        };
        if moved {
            cx.notify();
        }
        moved
    }

    pub(crate) fn open_selected_table_row_drawer(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(target) = self.selected_table_row_drawer_target() else {
            return false;
        };
        self.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
        true
    }

    fn selected_table_row_drawer_target(&self) -> Option<ResourceDrawerTarget> {
        if matches!(self.tabs.active_tab(), Some(WorkspaceTab::Logs(_))) {
            return None;
        }
        match self.sidebar.active_view {
            WorkspaceView::Pods => {
                let key = self.resources.pods.table.selected_row_key.as_deref()?;
                views::pods::filter_and_sort_pods_with_filters(
                    &self.resources.pods.pods,
                    &self.resources.namespaces,
                    &self.resources.pods.table.filters,
                    self.resources.pods.table.sort.as_ref(),
                )
                .into_iter()
                .find(|p| format!("{}/{}", p.namespace, p.name) == key)
                .map(|p| ResourceDrawerTarget::Pod {
                    name: p.name.clone(),
                    namespace: p.namespace.clone(),
                })
            }
            WorkspaceView::Deployments => {
                let key = self
                    .resources
                    .deployments
                    .table
                    .selected_row_key
                    .as_deref()?;
                views::deployments::filter_and_sort_deployments_with_filters(
                    &self.resources.deployments.deployments,
                    &self.resources.namespaces,
                    &self.resources.deployments.table.filters,
                    self.resources.deployments.table.sort.as_ref(),
                )
                .into_iter()
                .find(|d| format!("{}/{}", d.namespace, d.name) == key)
                .map(|d| ResourceDrawerTarget::Deployment {
                    name: d.name.clone(),
                    namespace: d.namespace.clone(),
                })
            }
            WorkspaceView::Services => {
                let key = self.resources.services.table.selected_row_key.as_deref()?;
                views::services::filter_and_sort_services_with_filters(
                    &self.resources.services.services,
                    &self.resources.namespaces,
                    self.resources.services.type_filter,
                    &self.resources.services.table.filters,
                    self.resources.services.table.sort.as_ref(),
                )
                .into_iter()
                .find(|s| format!("{}/{}", s.namespace, s.name) == key)
                .map(|s| ResourceDrawerTarget::Service {
                    name: s.name.clone(),
                    namespace: s.namespace.clone(),
                })
            }
            WorkspaceView::Events => {
                let key = self.resources.events.table.selected_row_key.as_deref()?;
                views::events::filter_and_sort_events_with_filters(
                    &self.resources.events.events,
                    &self.resources.namespaces,
                    self.resources.events.type_filter,
                    &self.resources.events.table.filters,
                    self.resources.events.table.sort.as_ref(),
                )
                .into_iter()
                .find(|ev| {
                    let ev_key = if !ev.uid.is_empty() {
                        ev.uid.clone()
                    } else {
                        format!("{}/{}/{}", ev.namespace, ev.reason, ev.object_name)
                    };
                    ev_key == key
                })
                .map(|ev| ResourceDrawerTarget::Event {
                    uid: ev.uid.clone(),
                    name: ev.object_name.clone(),
                    reason: ev.reason.clone(),
                    namespace: ev.namespace.clone(),
                })
            }
            WorkspaceView::Nodes => {
                let key = self.resources.nodes.table.selected_row_key.as_deref()?;
                self.resources
                    .nodes
                    .nodes
                    .iter()
                    .find(|n| n.name == key)
                    .map(|n| ResourceDrawerTarget::Node {
                        name: n.name.clone(),
                    })
            }
            WorkspaceView::Namespaces => {
                let key = self
                    .resources
                    .namespaces_view
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .namespaces
                    .namespaces
                    .iter()
                    .find(|ns| ns.name == key)
                    .map(|ns| ResourceDrawerTarget::Namespace {
                        name: ns.name.clone(),
                    })
            }
            WorkspaceView::StatefulSets => {
                let key = self
                    .resources
                    .statefulsets
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .statefulsets
                    .statefulsets
                    .iter()
                    .find(|s| format!("{}/{}", s.namespace, s.name) == key)
                    .map(|s| ResourceDrawerTarget::StatefulSet {
                        name: s.name.clone(),
                        namespace: s.namespace.clone(),
                    })
            }
            WorkspaceView::DaemonSets => {
                let key = self
                    .resources
                    .daemonsets
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .daemonsets
                    .daemonsets
                    .iter()
                    .find(|d| format!("{}/{}", d.namespace, d.name) == key)
                    .map(|d| ResourceDrawerTarget::DaemonSet {
                        name: d.name.clone(),
                        namespace: d.namespace.clone(),
                    })
            }
            WorkspaceView::ReplicaSets => {
                let key = self
                    .resources
                    .replicasets
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .replicasets
                    .replicasets
                    .iter()
                    .find(|r| format!("{}/{}", r.namespace, r.name) == key)
                    .map(|r| ResourceDrawerTarget::ReplicaSet {
                        name: r.name.clone(),
                        namespace: r.namespace.clone(),
                    })
            }
            WorkspaceView::Jobs => {
                let key = self.resources.jobs.table.selected_row_key.as_deref()?;
                self.resources
                    .jobs
                    .jobs
                    .iter()
                    .find(|j| format!("{}/{}", j.namespace, j.name) == key)
                    .map(|j| ResourceDrawerTarget::Job {
                        name: j.name.clone(),
                        namespace: j.namespace.clone(),
                    })
            }
            WorkspaceView::CronJobs => {
                let key = self.resources.cronjobs.table.selected_row_key.as_deref()?;
                self.resources
                    .cronjobs
                    .cronjobs
                    .iter()
                    .find(|c| format!("{}/{}", c.namespace, c.name) == key)
                    .map(|c| ResourceDrawerTarget::CronJob {
                        name: c.name.clone(),
                        namespace: c.namespace.clone(),
                    })
            }
            WorkspaceView::Hpas => {
                let key = self
                    .resources
                    .list_views
                    .hpas
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .list_views
                    .hpas
                    .hpas
                    .iter()
                    .find(|h| format!("{}/{}", h.namespace, h.name) == key)
                    .map(|h| ResourceDrawerTarget::Hpa {
                        name: h.name.clone(),
                        namespace: h.namespace.clone(),
                    })
            }
            WorkspaceView::Ingresses => {
                let key = self.resources.ingresses.table.selected_row_key.as_deref()?;
                self.resources
                    .ingresses
                    .ingresses
                    .iter()
                    .find(|i| format!("{}/{}", i.namespace, i.name) == key)
                    .map(|i| ResourceDrawerTarget::Ingress {
                        name: i.name.clone(),
                        namespace: i.namespace.clone(),
                    })
            }
            WorkspaceView::PersistentVolumes => {
                let key = self
                    .resources
                    .persistent_volumes
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .persistent_volumes
                    .persistent_volumes
                    .iter()
                    .find(|pv| pv.name == key)
                    .map(|pv| ResourceDrawerTarget::PersistentVolume {
                        name: pv.name.clone(),
                    })
            }
            WorkspaceView::PersistentVolumeClaims => {
                let key = self
                    .resources
                    .persistent_volume_claims
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .persistent_volume_claims
                    .persistent_volume_claims
                    .iter()
                    .find(|pvc| format!("{}/{}", pvc.namespace, pvc.name) == key)
                    .map(|pvc| ResourceDrawerTarget::PersistentVolumeClaim {
                        name: pvc.name.clone(),
                        namespace: pvc.namespace.clone(),
                    })
            }
            WorkspaceView::StorageClasses => {
                let key = self
                    .resources
                    .list_views
                    .storage_classes
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .list_views
                    .storage_classes
                    .classes
                    .iter()
                    .find(|sc| sc.name == key)
                    .map(|sc| ResourceDrawerTarget::StorageClass {
                        name: sc.name.clone(),
                    })
            }
            WorkspaceView::ConfigMaps => {
                let key = self
                    .resources
                    .configmaps
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .configmaps
                    .configmaps
                    .iter()
                    .find(|cm| format!("{}/{}", cm.namespace, cm.name) == key)
                    .map(|cm| ResourceDrawerTarget::ConfigMap {
                        name: cm.name.clone(),
                        namespace: cm.namespace.clone(),
                    })
            }
            WorkspaceView::Secrets => {
                let key = self.resources.secrets.table.selected_row_key.as_deref()?;
                self.resources
                    .secrets
                    .secrets
                    .iter()
                    .find(|s| format!("{}/{}", s.namespace, s.name) == key)
                    .map(|s| ResourceDrawerTarget::Secret {
                        name: s.name.clone(),
                        namespace: s.namespace.clone(),
                    })
            }
            WorkspaceView::NetworkPolicies => {
                let key = self
                    .resources
                    .network_policies
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .network_policies
                    .network_policies
                    .iter()
                    .find(|np| format!("{}/{}", np.namespace, np.name) == key)
                    .map(|np| ResourceDrawerTarget::NetworkPolicy {
                        name: np.name.clone(),
                        namespace: np.namespace.clone(),
                    })
            }
            WorkspaceView::PodSecurity => {
                let key = self
                    .resources
                    .pod_security
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .pod_security
                    .policies
                    .iter()
                    .find(|p| format!("{}/{}", p.namespace, p.name) == key)
                    .map(|p| ResourceDrawerTarget::PodSecurity {
                        name: p.name.clone(),
                        namespace: p.namespace.clone(),
                    })
            }
            WorkspaceView::ResourceQuotas => {
                let key = self
                    .resources
                    .resource_quotas
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .resource_quotas
                    .policies
                    .iter()
                    .find(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name) == key)
                    .map(|p| {
                        if p.kind.eq_ignore_ascii_case("LimitRange") {
                            ResourceDrawerTarget::LimitRange {
                                name: p.name.clone(),
                                namespace: p.namespace.clone(),
                            }
                        } else {
                            ResourceDrawerTarget::ResourceQuota {
                                name: p.name.clone(),
                                namespace: p.namespace.clone(),
                            }
                        }
                    })
            }
            WorkspaceView::AdmissionWebhooks => {
                let key = self
                    .resources
                    .list_views
                    .admission_webhooks
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .list_views
                    .admission_webhooks
                    .policies
                    .iter()
                    .find(|w| format!("{}/{}", w.kind, w.name) == key)
                    .map(|w| ResourceDrawerTarget::AdmissionWebhook {
                        is_mutating: w.kind.contains("Mutating"),
                        name: w.name.clone(),
                    })
            }
            WorkspaceView::SecurityOverview => {
                let key = self
                    .resources
                    .list_views
                    .security_overview
                    .table
                    .selected_row_key
                    .as_deref()?;
                self.resources
                    .list_views
                    .security_overview
                    .policies
                    .iter()
                    .find(|p| format!("{}/{}/{}", p.kind, p.namespace, p.name) == key)
                    .map(|p| ResourceDrawerTarget::SecurityPolicy {
                        kind: p.kind.clone(),
                        name: p.name.clone(),
                        namespace: p.namespace.clone(),
                    })
            }
            _ => None,
        }
    }
}
