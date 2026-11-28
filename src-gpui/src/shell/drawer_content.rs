use gpui::{AnyElement, App, ClipboardItem, Context, SharedString, Window, div, prelude::*};

use crate::components::{
    ResourceDrawerProps, ResourceDrawerTabDef, ResourceDrawerTabId, ResourceDrawerTarget,
    YamlLoadStatus, YamlViewerProps, render_resource_drawer, render_yaml_viewer,
};
use crate::state::EventsLoadStatus;
use crate::theme::OrbitTheme;
use crate::{OrbitShell, views};

type DrawerTargetContent = (
    SharedString,
    SharedString,
    Option<SharedString>,
    Option<SharedString>,
    Option<SharedString>,
    Vec<ResourceDrawerTabDef>,
    AnyElement,
);

impl OrbitShell {
    pub(crate) fn open_resource_drawer(
        &mut self,
        target: ResourceDrawerTarget,
        initial_tab: ResourceDrawerTabId,
        cx: &mut Context<Self>,
    ) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if matches!(
            target,
            ResourceDrawerTarget::Deployment { .. }
                | ResourceDrawerTarget::StatefulSet { .. }
                | ResourceDrawerTarget::DaemonSet { .. }
                | ResourceDrawerTarget::ReplicaSet { .. }
                | ResourceDrawerTarget::Job { .. }
                | ResourceDrawerTarget::CronJob { .. }
                | ResourceDrawerTarget::Pod { .. }
                | ResourceDrawerTarget::Service { .. }
                | ResourceDrawerTarget::Ingress { .. }
                | ResourceDrawerTarget::ConfigMap { .. }
                | ResourceDrawerTarget::Secret { .. }
                | ResourceDrawerTarget::PersistentVolume { .. }
                | ResourceDrawerTarget::PersistentVolumeClaim { .. }
                | ResourceDrawerTarget::NetworkPolicy { .. }
                | ResourceDrawerTarget::PodSecurity { .. }
                | ResourceDrawerTarget::ResourceQuota { .. }
                | ResourceDrawerTarget::LimitRange { .. }
                | ResourceDrawerTarget::Hpa { .. }
                | ResourceDrawerTarget::StorageClass { .. }
                | ResourceDrawerTarget::AdmissionWebhook { .. }
                | ResourceDrawerTarget::SecurityPolicy { .. }
                | ResourceDrawerTarget::Node { .. }
        ) {
            if let Some(cid) = cluster_id.as_deref() {
                if self.resources.events.needs_load_for_cluster(cid) {
                    let cmd = self.resources.events.begin_load_for_cluster(cid);
                    self.dispatch_command(cmd);
                }
            }
        }
        if let Some(cmd) = self
            .drawer
            .open_or_switch(cluster_id.as_deref(), target, initial_tab)
        {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn select_drawer_tab(&mut self, tab: ResourceDrawerTabId, cx: &mut Context<Self>) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if let Some(cmd) = self.drawer.select_tab(cluster_id.as_deref(), tab) {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn retry_drawer_yaml(&mut self, cx: &mut Context<Self>) {
        let cluster_id = self.active_cluster_id().map(str::to_string);
        if let Some(cmd) = self.drawer.retry_yaml(cluster_id.as_deref()) {
            self.dispatch_command(cmd);
        }
        cx.notify();
    }

    fn copy_drawer_yaml(&mut self, yaml: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(yaml));
        self.drawer.yaml_copied = true;
        cx.notify();
    }

    pub(crate) fn dismiss_resource_drawer(&mut self, cx: &mut Context<Self>) {
        if self.drawer.is_open {
            self.drawer.close();
            cx.notify();
        }
    }

    fn find_event_for_target(
        &self,
        uid: &str,
        reason: &str,
        namespace: &str,
    ) -> Option<&orbit_core::kubernetes::models::EventInfo> {
        self.resources.events.events.iter().find(|ev| {
            if !uid.is_empty() && !ev.uid.is_empty() {
                ev.uid == uid
            } else {
                ev.namespace == namespace && ev.reason == reason
            }
        })
    }

    pub(crate) fn reconcile_drawer_target_exists(&mut self) {
        let Some(target) = self.drawer.target.as_ref() else {
            return;
        };
        if !self.drawer.is_open {
            return;
        }
        let exists = match target {
            ResourceDrawerTarget::Deployment { name, namespace } => self
                .resources
                .deployments
                .deployments
                .iter()
                .any(|d| d.name == *name && d.namespace == *namespace),
            ResourceDrawerTarget::StatefulSet { name, namespace } => self
                .resources
                .statefulsets
                .statefulsets
                .iter()
                .any(|s| s.name == *name && s.namespace == *namespace),
            ResourceDrawerTarget::DaemonSet { name, namespace } => self
                .resources
                .daemonsets
                .daemonsets
                .iter()
                .any(|d| d.name == *name && d.namespace == *namespace),
            ResourceDrawerTarget::ReplicaSet { name, namespace } => self
                .resources
                .replicasets
                .replicasets
                .iter()
                .any(|r| r.name == *name && r.namespace == *namespace),
            ResourceDrawerTarget::Job { name, namespace } => self
                .resources
                .jobs
                .jobs
                .iter()
                .any(|j| j.name == *name && j.namespace == *namespace),
            ResourceDrawerTarget::CronJob { name, namespace } => self
                .resources
                .cronjobs
                .cronjobs
                .iter()
                .any(|cj| cj.name == *name && cj.namespace == *namespace),
            ResourceDrawerTarget::PersistentVolume { name } => self
                .resources
                .persistent_volumes
                .persistent_volumes
                .iter()
                .any(|pv| pv.name == *name),
            ResourceDrawerTarget::Node { name } => {
                self.resources.nodes.nodes.iter().any(|n| n.name == *name)
            }
            ResourceDrawerTarget::Namespace { name } => self
                .resources
                .namespaces
                .namespaces
                .iter()
                .any(|ns| ns.name == *name),
            ResourceDrawerTarget::Event {
                uid,
                reason,
                namespace,
                ..
            } => self.find_event_for_target(uid, reason, namespace).is_some(),
            ResourceDrawerTarget::Pod { name, namespace } => self
                .resources
                .pods
                .pods
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::Service { name, namespace } => self
                .resources
                .services
                .services
                .iter()
                .any(|s| s.name == *name && s.namespace == *namespace),
            ResourceDrawerTarget::Ingress { name, namespace } => self
                .resources
                .ingresses
                .ingresses
                .iter()
                .any(|ing| ing.name == *name && ing.namespace == *namespace),
            ResourceDrawerTarget::ConfigMap { name, namespace } => self
                .resources
                .configmaps
                .configmaps
                .iter()
                .any(|c| c.name == *name && c.namespace == *namespace),
            ResourceDrawerTarget::Secret { name, namespace } => self
                .resources
                .secrets
                .secrets
                .iter()
                .any(|s| s.name == *name && s.namespace == *namespace),
            ResourceDrawerTarget::PersistentVolumeClaim { name, namespace } => self
                .resources
                .persistent_volume_claims
                .persistent_volume_claims
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::NetworkPolicy { name, namespace } => self
                .resources
                .network_policies
                .network_policies
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::PodSecurity { name, namespace } => self
                .resources
                .pod_security
                .policies
                .iter()
                .any(|p| p.name == *name && p.namespace == *namespace),
            ResourceDrawerTarget::ResourceQuota { name, namespace } => {
                self.resources.resource_quotas.policies.iter().any(|p| {
                    p.kind.eq_ignore_ascii_case("ResourceQuota")
                        && p.name == *name
                        && p.namespace == *namespace
                })
            }
            ResourceDrawerTarget::LimitRange { name, namespace } => {
                self.resources.resource_quotas.policies.iter().any(|p| {
                    p.kind.eq_ignore_ascii_case("LimitRange")
                        && p.name == *name
                        && p.namespace == *namespace
                })
            }
            ResourceDrawerTarget::Hpa { name, namespace } => self
                .resources
                .list_views
                .hpas
                .hpas
                .iter()
                .any(|h| h.name == *name && h.namespace == *namespace),
            ResourceDrawerTarget::StorageClass { name } => self
                .resources
                .list_views
                .storage_classes
                .classes
                .iter()
                .any(|sc| sc.name == *name),
            ResourceDrawerTarget::AdmissionWebhook { is_mutating, name } => {
                let target_kind = if *is_mutating {
                    "MutatingWebhookConfiguration"
                } else {
                    "ValidatingWebhookConfiguration"
                };
                self.resources
                    .list_views
                    .admission_webhooks
                    .policies
                    .iter()
                    .any(|p| p.kind.eq_ignore_ascii_case(target_kind) && p.name == *name)
            }
            ResourceDrawerTarget::SecurityPolicy {
                kind,
                name,
                namespace,
            } => self
                .resources
                .list_views
                .security_overview
                .policies
                .iter()
                .any(|p| {
                    p.kind.eq_ignore_ascii_case(kind)
                        && p.name == *name
                        && p.namespace == *namespace
                }),
        };
        if !exists {
            self.drawer.close();
        }
    }

    fn render_live_yaml_tab(
        &self,
        subtitle: impl Into<SharedString>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let copy_yaml = self.drawer.yaml_content.clone();
        let copy_entity = cx.entity().clone();
        let on_copy = copy_yaml.map(|yaml| {
            std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
                copy_entity.update(cx, |shell, cx| {
                    shell.copy_drawer_yaml(yaml.clone(), cx);
                });
            }) as std::rc::Rc<dyn Fn(&mut Window, &mut App)>
        });
        let retry_entity = cx.entity().clone();
        let on_retry = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
            retry_entity.update(cx, |shell, cx| {
                shell.retry_drawer_yaml(cx);
            });
        });
        render_yaml_viewer(
            YamlViewerProps {
                status: self.drawer.yaml_status,
                yaml_content: self.drawer.yaml_content.as_deref(),
                cached_document: self.drawer.yaml_document.current(),
                scroll_handle: Some(self.drawer.yaml_scroll_handle.clone()),
                error_message: self.drawer.yaml_error.as_deref(),
                copied: self.drawer.yaml_copied,
                subtitle: subtitle.into(),
                on_copy,
                on_retry: Some(on_retry),
            },
            theme,
        )
    }

    fn build_drawer_target_content(
        &self,
        target: &ResourceDrawerTarget,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<DrawerTargetContent> {
        let events_loading = self.resources.events.status == EventsLoadStatus::Loading;
        match target {
            ResourceDrawerTarget::Deployment { .. }
            | ResourceDrawerTarget::StatefulSet { .. }
            | ResourceDrawerTarget::ReplicaSet { .. }
            | ResourceDrawerTarget::DaemonSet { .. }
            | ResourceDrawerTarget::Job { .. }
            | ResourceDrawerTarget::CronJob { .. }
            | ResourceDrawerTarget::Pod { .. }
            | ResourceDrawerTarget::Hpa { .. } => {
                self.build_workload_drawer_content(target, events_loading, theme, cx)
            }
            ResourceDrawerTarget::Ingress { .. }
            | ResourceDrawerTarget::Service { .. }
            | ResourceDrawerTarget::NetworkPolicy { .. } => {
                self.build_network_drawer_content(target, events_loading, theme, cx)
            }
            ResourceDrawerTarget::ConfigMap { .. }
            | ResourceDrawerTarget::Secret { .. }
            | ResourceDrawerTarget::PersistentVolume { .. }
            | ResourceDrawerTarget::PersistentVolumeClaim { .. }
            | ResourceDrawerTarget::StorageClass { .. } => {
                self.build_config_storage_drawer_content(target, events_loading, theme, cx)
            }
            ResourceDrawerTarget::Node { .. }
            | ResourceDrawerTarget::Namespace { .. }
            | ResourceDrawerTarget::Event { .. }
            | ResourceDrawerTarget::PodSecurity { .. }
            | ResourceDrawerTarget::ResourceQuota { .. }
            | ResourceDrawerTarget::LimitRange { .. }
            | ResourceDrawerTarget::AdmissionWebhook { .. }
            | ResourceDrawerTarget::SecurityPolicy { .. } => {
                self.build_cluster_security_drawer_content(target, events_loading, theme, cx)
            }
        }
    }

    fn build_workload_drawer_content(
        &self,
        target: &ResourceDrawerTarget,
        events_loading: bool,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<DrawerTargetContent> {
        match target {
            ResourceDrawerTarget::Deployment { name, namespace } => {
                let dep = self
                    .resources
                    .deployments
                    .deployments
                    .iter()
                    .find(|d| d.name == *name && d.namespace == *namespace)?;
                let matched_events = views::deployments::filter_events_for_deployment(
                    &self.resources.events.events,
                    dep,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::deployments::render_deployment_drawer_overview(dep, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::deployments::render_deployment_drawer_events(
                            &self.resources.events.events,
                            events_loading,
                            dep,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", dep.namespace, dep.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(dep.name.clone()),
                    SharedString::from("Deployment"),
                    Some(SharedString::from(dep.namespace.clone())),
                    Some(SharedString::from(dep.age.clone())),
                    Some(SharedString::from(dep.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::StatefulSet { name, namespace } => {
                let ss = self
                    .resources
                    .statefulsets
                    .statefulsets
                    .iter()
                    .find(|s| s.name == *name && s.namespace == *namespace)?;
                let matched_events = views::statefulsets::filter_events_for_statefulset(
                    &self.resources.events.events,
                    ss,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::statefulsets::render_statefulset_drawer_overview(ss, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::statefulsets::render_statefulset_drawer_events(
                            &self.resources.events.events,
                            events_loading,
                            ss,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", ss.namespace, ss.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(ss.name.clone()),
                    SharedString::from("StatefulSet"),
                    Some(SharedString::from(ss.namespace.clone())),
                    Some(SharedString::from(ss.age.clone())),
                    Some(SharedString::from(ss.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::ReplicaSet { name, namespace } => {
                let rs = self
                    .resources
                    .replicasets
                    .replicasets
                    .iter()
                    .find(|r| r.name == *name && r.namespace == *namespace)?;
                let matched_events = views::replicasets::filter_events_for_replicaset(
                    &self.resources.events.events,
                    rs,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::replicasets::render_replicaset_drawer_overview(rs, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::replicasets::render_replicaset_drawer_events(
                            &self.resources.events.events,
                            events_loading,
                            rs,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", rs.namespace, rs.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(rs.name.clone()),
                    SharedString::from("ReplicaSet"),
                    Some(SharedString::from(rs.namespace.clone())),
                    Some(SharedString::from(rs.age.clone())),
                    Some(SharedString::from(rs.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::DaemonSet { name, namespace } => {
                let ds = self
                    .resources
                    .daemonsets
                    .daemonsets
                    .iter()
                    .find(|d| d.name == *name && d.namespace == *namespace)?;
                let matched_events = views::daemonsets::filter_events_for_daemonset(
                    &self.resources.events.events,
                    ds,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::daemonsets::render_daemonset_drawer_overview(ds, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::daemonsets::render_daemonset_drawer_events(
                            &self.resources.events.events,
                            events_loading,
                            ds,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", ds.namespace, ds.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(ds.name.clone()),
                    SharedString::from("DaemonSet"),
                    Some(SharedString::from(ds.namespace.clone())),
                    Some(SharedString::from(ds.age.clone())),
                    Some(SharedString::from(ds.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Job { name, namespace } => {
                let job = self
                    .resources
                    .jobs
                    .jobs
                    .iter()
                    .find(|j| j.name == *name && j.namespace == *namespace)?;
                let matched_events =
                    views::jobs::filter_events_for_job(&self.resources.events.events, job);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::jobs::render_job_drawer_overview(job, theme)
                    }
                    ResourceDrawerTabId::Events => views::jobs::render_job_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        job,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", job.namespace, job.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(job.name.clone()),
                    SharedString::from("Job"),
                    Some(SharedString::from(job.namespace.clone())),
                    Some(SharedString::from(job.age.clone())),
                    Some(SharedString::from(job.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::CronJob { name, namespace } => {
                let cj = self
                    .resources
                    .cronjobs
                    .cronjobs
                    .iter()
                    .find(|c| c.name == *name && c.namespace == *namespace)?;
                let matched_events =
                    views::cronjobs::filter_events_for_cronjob(&self.resources.events.events, cj);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::cronjobs::render_cronjob_drawer_overview(cj, theme)
                    }
                    ResourceDrawerTabId::Events => views::cronjobs::render_cronjob_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        cj,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", cj.namespace, cj.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(cj.name.clone()),
                    SharedString::from("CronJob"),
                    Some(SharedString::from(cj.namespace.clone())),
                    Some(SharedString::from(cj.age.clone())),
                    Some(SharedString::from(views::cronjobs::cronjob_suspend_status(
                        cj.suspend,
                    ))),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Pod { name, namespace } => {
                let pod = self
                    .resources
                    .pods
                    .pods
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events =
                    views::pods::filter_events_for_pod(&self.resources.events.events, pod);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::pods::render_pod_drawer_overview(pod, theme)
                    }
                    ResourceDrawerTabId::Events => views::pods::render_pod_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        pod,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", pod.namespace, pod.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(pod.name.clone()),
                    SharedString::from("Pod"),
                    Some(SharedString::from(pod.namespace.clone())),
                    Some(SharedString::from(pod.age.clone())),
                    Some(SharedString::from(pod.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Hpa { name, namespace } => {
                let hpa = self
                    .resources
                    .list_views
                    .hpas
                    .hpas
                    .iter()
                    .find(|h| h.name == *name && h.namespace == *namespace)?;
                let matched_events =
                    views::hpas::filter_events_for_hpa(&self.resources.events.events, hpa);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::hpas::render_hpa_drawer_overview(hpa, theme)
                    }
                    ResourceDrawerTabId::Events => views::hpas::render_hpa_drawer_events(
                        &matched_events,
                        events_loading,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", hpa.namespace, hpa.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(hpa.name.clone()),
                    SharedString::from("HorizontalPodAutoscaler"),
                    Some(SharedString::from(hpa.namespace.clone())),
                    Some(SharedString::from(hpa.age.clone())),
                    Some(SharedString::from(hpa.status.clone())),
                    tabs,
                    body,
                ))
            }
            _ => None,
        }
    }

    fn build_network_drawer_content(
        &self,
        target: &ResourceDrawerTarget,
        events_loading: bool,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<DrawerTargetContent> {
        match target {
            ResourceDrawerTarget::Ingress { name, namespace } => {
                let ing = self
                    .resources
                    .ingresses
                    .ingresses
                    .iter()
                    .find(|i| i.name == *name && i.namespace == *namespace)?;
                let matched_events =
                    views::ingresses::filter_events_for_ingress(&self.resources.events.events, ing);
                let rules_tab_id = ResourceDrawerTabId::Custom(SharedString::from("rules"));
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: rules_tab_id.clone(),
                        label: SharedString::from("Rules"),
                        badge: Some(SharedString::from(ing.rules_summary.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match &self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::ingresses::render_ingress_drawer_overview(ing, theme)
                    }
                    ResourceDrawerTabId::Custom(id) if id.as_ref() == "rules" => {
                        views::ingresses::render_ingress_drawer_rules(ing, theme)
                    }
                    ResourceDrawerTabId::Events => views::ingresses::render_ingress_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        ing,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", ing.namespace, ing.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(ing.name.clone()),
                    SharedString::from("Ingress"),
                    Some(SharedString::from(ing.namespace.clone())),
                    Some(SharedString::from(ing.age.clone())),
                    ing.class_name
                        .as_deref()
                        .filter(|c| !c.trim().is_empty())
                        .map(|c| SharedString::from(c.to_string())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Service { name, namespace } => {
                let svc = self
                    .resources
                    .services
                    .services
                    .iter()
                    .find(|s| s.name == *name && s.namespace == *namespace)?;
                let matched_events =
                    views::services::filter_events_for_service(&self.resources.events.events, svc);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::services::render_service_drawer_overview(svc, theme)
                    }
                    ResourceDrawerTabId::Events => views::services::render_service_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        svc,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", svc.namespace, svc.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(svc.name.clone()),
                    SharedString::from("Service"),
                    Some(SharedString::from(svc.namespace.clone())),
                    Some(SharedString::from(svc.age.clone())),
                    Some(SharedString::from(svc.r#type.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::NetworkPolicy { name, namespace } => {
                let policy = self
                    .resources
                    .network_policies
                    .network_policies
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events = views::network_policies::filter_events_for_network_policy(
                    &self.resources.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::network_policies::render_network_policy_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::network_policies::render_network_policy_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.namespace, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from("NetworkPolicy"),
                    Some(SharedString::from(policy.namespace.clone())),
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            _ => None,
        }
    }

    fn build_config_storage_drawer_content(
        &self,
        target: &ResourceDrawerTarget,
        events_loading: bool,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<DrawerTargetContent> {
        match target {
            ResourceDrawerTarget::ConfigMap { name, namespace } => {
                let cm = self
                    .resources
                    .configmaps
                    .configmaps
                    .iter()
                    .find(|c| c.name == *name && c.namespace == *namespace)?;
                let matched_events = views::configmaps::filter_events_for_configmap(
                    &self.resources.events.events,
                    cm,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Custom(SharedString::from("data")),
                        label: SharedString::from("Data"),
                        badge: Some(SharedString::from(cm.keys_count.to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match &self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::configmaps::render_configmap_drawer_overview(cm, theme)
                    }
                    ResourceDrawerTabId::Custom(id) if id.as_ref() == "data" => {
                        views::configmaps::render_configmap_drawer_data(cm, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::configmaps::render_configmap_drawer_events(
                            &self.resources.events.events,
                            events_loading,
                            cm,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", cm.namespace, cm.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                let status_label = if cm.immutable { "Immutable" } else { "Mutable" };
                Some((
                    SharedString::from(cm.name.clone()),
                    SharedString::from("ConfigMap"),
                    Some(SharedString::from(cm.namespace.clone())),
                    Some(SharedString::from(cm.age.clone())),
                    Some(SharedString::from(status_label)),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Secret { name, namespace } => {
                let sec = self
                    .resources
                    .secrets
                    .secrets
                    .iter()
                    .find(|s| s.name == *name && s.namespace == *namespace)?;
                let matched_events =
                    views::secrets::filter_events_for_secret(&self.resources.events.events, sec);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Custom(SharedString::from("data")),
                        label: SharedString::from("Data"),
                        badge: Some(SharedString::from(sec.keys_count.to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let reveal_entity = cx.entity().clone();
                let copy_entity = cx.entity().clone();
                let body = match &self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::secrets::render_secret_drawer_overview(sec, theme)
                    }
                    ResourceDrawerTabId::Custom(id) if id.as_ref() == "data" => {
                        views::secrets::render_secret_drawer_data(
                            sec,
                            &self.drawer.revealed_secret_keys,
                            theme,
                            move |key, _window, cx| {
                                reveal_entity.update(cx, |shell, cx| {
                                    shell.drawer.toggle_secret_key_revealed(&key);
                                    cx.notify();
                                });
                            },
                            move |value, _window, cx| {
                                copy_entity.update(cx, |_shell, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(value));
                                });
                            },
                        )
                    }
                    ResourceDrawerTabId::Events => views::secrets::render_secret_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        sec,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", sec.namespace, sec.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                let status_label = if sec.immutable {
                    "Immutable"
                } else {
                    "Mutable"
                };
                Some((
                    SharedString::from(sec.name.clone()),
                    SharedString::from("Secret"),
                    Some(SharedString::from(sec.namespace.clone())),
                    Some(SharedString::from(sec.age.clone())),
                    Some(SharedString::from(status_label)),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::PersistentVolume { name } => {
                let pv = self
                    .resources
                    .persistent_volumes
                    .persistent_volumes
                    .iter()
                    .find(|p| p.name == *name)?;
                let matched_events = views::persistent_volumes::filter_events_for_persistent_volume(
                    &self.resources.events.events,
                    pv,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::persistent_volumes::render_persistent_volume_drawer_overview(
                            pv, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::persistent_volumes::render_persistent_volume_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(pv.name.clone(), theme, cx)
                    }
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(pv.name.clone()),
                    SharedString::from("PersistentVolume"),
                    None,
                    Some(SharedString::from(pv.age.clone())),
                    Some(SharedString::from(pv.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::PersistentVolumeClaim { name, namespace } => {
                let pvc = self
                    .resources
                    .persistent_volume_claims
                    .persistent_volume_claims
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events =
                    views::persistent_volume_claims::filter_events_for_persistent_volume_claim(
                        &self.resources.events.events,
                        pvc,
                    );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::persistent_volume_claims::render_persistent_volume_claim_drawer_overview(
                            pvc, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::persistent_volume_claims::render_persistent_volume_claim_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", pvc.namespace, pvc.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(pvc.name.clone()),
                    SharedString::from("PersistentVolumeClaim"),
                    Some(SharedString::from(pvc.namespace.clone())),
                    Some(SharedString::from(pvc.age.clone())),
                    Some(SharedString::from(pvc.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::StorageClass { name } => {
                let sc = self
                    .resources
                    .list_views
                    .storage_classes
                    .classes
                    .iter()
                    .find(|c| c.name == *name)?;
                let matched_events = views::storage_classes::filter_events_for_storage_class(
                    &self.resources.events.events,
                    sc,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::storage_classes::render_storage_class_drawer_overview(sc, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::storage_classes::render_storage_class_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(format!("StorageClass/{}", sc.name), theme, cx)
                    }
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(sc.name.clone()),
                    SharedString::from("StorageClass"),
                    None,
                    Some(SharedString::from(sc.age.clone())),
                    None,
                    tabs,
                    body,
                ))
            }
            _ => None,
        }
    }

    fn build_cluster_security_drawer_content(
        &self,
        target: &ResourceDrawerTarget,
        events_loading: bool,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<DrawerTargetContent> {
        match target {
            ResourceDrawerTarget::Node { name } => {
                let node = self
                    .resources
                    .nodes
                    .nodes
                    .iter()
                    .find(|n| n.name == *name)?;
                let matched_events =
                    views::nodes::filter_events_for_node(&self.resources.events.events, node);
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::nodes::render_node_drawer_overview(node, theme)
                    }
                    ResourceDrawerTabId::Events => views::nodes::render_node_drawer_events(
                        &self.resources.events.events,
                        events_loading,
                        node,
                        theme,
                    ),
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(node.name.clone(), theme, cx)
                    }
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(node.name.clone()),
                    SharedString::from("Node"),
                    None,
                    Some(SharedString::from(node.uptime.clone())),
                    Some(SharedString::from(node.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Namespace { name } => {
                let ns = self
                    .resources
                    .namespaces
                    .namespaces
                    .iter()
                    .find(|n| n.name == *name)?;
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Yaml => {
                        self.render_live_yaml_tab(ns.name.clone(), theme, cx)
                    }
                    _ => views::namespaces::render_namespace_drawer_overview(ns, theme),
                };
                Some((
                    SharedString::from(ns.name.clone()),
                    SharedString::from("Namespace"),
                    None,
                    Some(SharedString::from(ns.age.clone())),
                    Some(SharedString::from(ns.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::Event {
                uid,
                reason,
                namespace,
                ..
            } => {
                let ev = self.find_event_for_target(uid, reason, namespace)?;
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Yaml => {
                        let formatted = orbit_core::kubernetes::raw::format_event_yaml(ev);
                        let copy_yaml = formatted.clone();
                        let copy_entity = cx.entity().clone();
                        let on_copy = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
                            copy_entity.update(cx, |shell, cx| {
                                shell.copy_drawer_yaml(copy_yaml.clone(), cx);
                            });
                        });
                        render_yaml_viewer(
                            YamlViewerProps {
                                status: YamlLoadStatus::Loaded,
                                yaml_content: Some(&formatted),
                                cached_document: None,
                                scroll_handle: Some(self.drawer.yaml_scroll_handle.clone()),
                                error_message: None,
                                copied: self.drawer.yaml_copied,
                                subtitle: SharedString::from(format!(
                                    "{}/{}",
                                    ev.object_kind, ev.object_name
                                )),
                                on_copy: Some(on_copy),
                                on_retry: None,
                            },
                            theme,
                        )
                    }
                    _ => views::events::render_event_drawer_overview(ev, theme),
                };
                Some((
                    SharedString::from(ev.reason.clone()),
                    SharedString::from("Event"),
                    if ev.namespace.is_empty() {
                        None
                    } else {
                        Some(SharedString::from(ev.namespace.clone()))
                    },
                    Some(SharedString::from(ev.time.clone())),
                    Some(SharedString::from(ev.r#type.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::PodSecurity { name, namespace } => {
                let policy = self
                    .resources
                    .pod_security
                    .policies
                    .iter()
                    .find(|p| p.name == *name && p.namespace == *namespace)?;
                let matched_events = views::pod_security::filter_events_for_pod_security(
                    &self.resources.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::pod_security::render_pod_security_drawer_overview(policy, theme)
                    }
                    ResourceDrawerTabId::Events => {
                        views::pod_security::render_pod_security_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("Namespace/{}", policy.namespace),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from("PodSecurity"),
                    Some(SharedString::from(policy.namespace.clone())),
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::ResourceQuota { name, namespace }
            | ResourceDrawerTarget::LimitRange { name, namespace } => {
                let target_kind = target.kind();
                let policy = self.resources.resource_quotas.policies.iter().find(|p| {
                    p.kind.eq_ignore_ascii_case(target_kind)
                        && p.name == *name
                        && p.namespace == *namespace
                })?;
                let matched_events = views::resource_quotas::filter_events_for_resource_policy(
                    &self.resources.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::resource_quotas::render_resource_policy_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::resource_quotas::render_resource_policy_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.namespace, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from(policy.kind.clone()),
                    Some(SharedString::from(policy.namespace.clone())),
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::AdmissionWebhook { is_mutating, name } => {
                let target_kind = if *is_mutating {
                    "MutatingWebhookConfiguration"
                } else {
                    "ValidatingWebhookConfiguration"
                };
                let policy = self
                    .resources
                    .list_views
                    .admission_webhooks
                    .policies
                    .iter()
                    .find(|p| p.kind.eq_ignore_ascii_case(target_kind) && p.name == *name)?;
                let matched_events = views::admission_webhooks::filter_events_for_admission_webhook(
                    &self.resources.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::admission_webhooks::render_admission_webhook_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::admission_webhooks::render_admission_webhook_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.kind, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from(policy.kind.clone()),
                    None,
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            ResourceDrawerTarget::SecurityPolicy {
                kind,
                name,
                namespace,
            } => {
                let policy = self
                    .resources
                    .list_views
                    .security_overview
                    .policies
                    .iter()
                    .find(|p| {
                        p.kind.eq_ignore_ascii_case(kind)
                            && p.name == *name
                            && p.namespace == *namespace
                    })?;
                let matched_events = views::security_overview::filter_events_for_security_policy(
                    &self.resources.events.events,
                    policy,
                );
                let tabs = vec![
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Overview,
                        label: SharedString::from("Overview"),
                        badge: None,
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Events,
                        label: SharedString::from("Events"),
                        badge: Some(SharedString::from(matched_events.len().to_string())),
                    },
                    ResourceDrawerTabDef {
                        id: ResourceDrawerTabId::Yaml,
                        label: SharedString::from("YAML"),
                        badge: None,
                    },
                ];
                let body = match self.drawer.active_tab {
                    ResourceDrawerTabId::Overview => {
                        views::security_overview::render_security_policy_drawer_overview(
                            policy, theme,
                        )
                    }
                    ResourceDrawerTabId::Events => {
                        views::security_overview::render_security_policy_drawer_events(
                            &matched_events,
                            events_loading,
                            theme,
                        )
                    }
                    ResourceDrawerTabId::Yaml => self.render_live_yaml_tab(
                        format!("{}/{}", policy.kind, policy.name),
                        theme,
                        cx,
                    ),
                    ResourceDrawerTabId::Custom(_) => div().into_any_element(),
                };
                let ns_opt = if policy.namespace.is_empty() || policy.namespace == "-" {
                    None
                } else {
                    Some(SharedString::from(policy.namespace.clone()))
                };
                Some((
                    SharedString::from(policy.name.clone()),
                    SharedString::from(policy.kind.clone()),
                    ns_opt,
                    Some(SharedString::from(policy.last_updated.clone())),
                    Some(SharedString::from(policy.status.clone())),
                    tabs,
                    body,
                ))
            }
            _ => None,
        }
    }

    pub(crate) fn render_active_drawer(
        &self,
        window_width: f32,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if !self.drawer.is_open {
            return None;
        }
        let target = self.drawer.target.as_ref()?;
        let (title, kind, namespace, age, status, tabs, body) =
            self.build_drawer_target_content(target, theme, cx)?;

        let tab_entity = cx.entity().clone();
        let on_select_tab = std::rc::Rc::new(
            move |tab: ResourceDrawerTabId, _w: &mut Window, cx: &mut App| {
                tab_entity.update(cx, |shell, cx| {
                    shell.select_drawer_tab(tab.clone(), cx);
                });
            },
        );
        let close_entity = cx.entity().clone();
        let on_close = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
            close_entity.update(cx, |shell, cx| {
                shell.dismiss_resource_drawer(cx);
            });
        });
        let resize_entity = cx.entity().clone();
        let on_start_resize = std::rc::Rc::new(move |_w: &mut Window, cx: &mut App| {
            resize_entity.update(cx, |shell, cx| {
                shell.drawer.start_resize();
                cx.notify();
            });
        });

        render_resource_drawer(
            true,
            Some(ResourceDrawerProps {
                title,
                kind,
                namespace,
                age,
                status,
                tabs,
                active_tab: self.drawer.active_tab.clone(),
                width: self.drawer.width,
                window_width,
                on_select_tab,
                on_close,
                on_start_resize,
                body,
            }),
            theme,
        )
    }
}
