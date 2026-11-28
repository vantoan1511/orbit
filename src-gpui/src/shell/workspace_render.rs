use gpui::{
    AnyElement, App, ClipboardItem, Context, FontWeight, MouseDownEvent, SharedString, Window, div,
    prelude::*, px,
};

use crate::components::{
    ResourceDrawerTabId, ResourceDrawerTarget, extract_ports_from_deployment,
    extract_ports_from_pod, extract_ports_from_replicaset, extract_ports_from_service,
    extract_ports_from_statefulset,
};
use crate::navigation::{WorkspaceTab, WorkspaceView};
use crate::state::{
    ClusterConnectionState, EventTypeFilter, PortForwardKindFilter, ResourceQuotaKindFilter,
    WorkloadActionKind,
};
use crate::theme::{OrbitTheme, apply_text_step};
use crate::{OrbitShell, views};

impl OrbitShell {
    pub(crate) fn render_connected_workspace(
        &mut self,
        cluster_id: &str,
        display_name: &str,
        k8s_version: Option<&str>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(WorkspaceTab::Logs(target)) = self.tabs.active_tab() {
            return self.render_logs_tab_content(&target, theme, cx);
        }

        match self.sidebar.active_view {
            WorkspaceView::Overview
            | WorkspaceView::WorkloadsOverview
            | WorkspaceView::StorageOverview => {
                self.render_overview_workspace(cluster_id, display_name, k8s_version, theme, cx)
            }
            WorkspaceView::Deployments
            | WorkspaceView::StatefulSets
            | WorkspaceView::ReplicaSets
            | WorkspaceView::DaemonSets
            | WorkspaceView::Jobs
            | WorkspaceView::CronJobs
            | WorkspaceView::Pods
            | WorkspaceView::Hpas => self.render_workloads_workspace(theme, cx),
            WorkspaceView::Services
            | WorkspaceView::Ingresses
            | WorkspaceView::PortForward
            | WorkspaceView::NetworkPolicies => self.render_network_workspace(theme, cx),
            WorkspaceView::ConfigMaps | WorkspaceView::Secrets => {
                self.render_config_workspace(theme, cx)
            }
            WorkspaceView::PersistentVolumes
            | WorkspaceView::PersistentVolumeClaims
            | WorkspaceView::StorageClasses => self.render_storage_workspace(theme, cx),
            WorkspaceView::PodSecurity
            | WorkspaceView::ResourceQuotas
            | WorkspaceView::AdmissionWebhooks
            | WorkspaceView::SecurityOverview => self.render_security_workspace(theme, cx),
            _ => self.render_cluster_workspace(cluster_id, display_name, k8s_version, theme, cx),
        }
    }

    fn render_overview_workspace(
        &mut self,
        cluster_id: &str,
        display_name: &str,
        k8s_version: Option<&str>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.sidebar.active_view {
            WorkspaceView::WorkloadsOverview => {
                let nav_entity = cx.entity().clone();
                let on_navigate = std::rc::Rc::new(
                    move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                        nav_entity.update(cx, |shell, cx| {
                            shell.select_workspace_view(view, cx);
                        });
                    },
                );
                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                views::workloads_overview::render_workloads_overview_view(
                    &self.resources.workloads_overview,
                    &self.resources.namespaces,
                    theme,
                    on_navigate,
                    on_refresh,
                )
            }
            WorkspaceView::StorageOverview => {
                let nav_entity = cx.entity().clone();
                let on_navigate = std::rc::Rc::new(
                    move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                        nav_entity.update(cx, |shell, cx| {
                            shell.select_workspace_view(view, cx);
                        });
                    },
                );
                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };
                views::storage_overview::render_storage_overview_view(
                    &self.resources.storage_overview,
                    &self.resources.namespaces,
                    theme,
                    on_navigate,
                    on_refresh,
                )
            }
            _ => {
                let nav_entity = cx.entity().clone();
                let on_navigate = std::rc::Rc::new(
                    move |view: WorkspaceView, _window: &mut Window, cx: &mut App| {
                        nav_entity.update(cx, |shell, cx| {
                            shell.select_workspace_view(view, cx);
                        });
                    },
                );
                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                views::overview::render_overview_view(
                    cluster_id,
                    display_name,
                    k8s_version,
                    self.state.active_profile.as_ref(),
                    self.state.active_cluster(),
                    &self.resources.overview,
                    theme,
                    on_navigate,
                    on_refresh,
                )
            }
        }
    }

    fn render_workloads_workspace(
        &mut self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.sidebar.active_view {
            WorkspaceView::Deployments => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .deployments
                            .table
                            .toggle_sort(col_id.as_ref());
                        let cfg = shell.resources.deployments.table.to_persisted_config();
                        shell.persist_table_config("deployments", cfg);
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.deployments.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Deployment {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          deployment: &orbit_core::kubernetes::models::DeploymentInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = deployment.name.clone();
                        let namespace = deployment.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Deployment {
                            name: deployment.name.clone(),
                            namespace: deployment.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let scale_action = {
                            let e = ctx_entity.clone();
                            let ns = deployment.namespace.clone();
                            let name = deployment.name.clone();
                            let replicas = deployment.replicas.desired;
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_scale_modal(
                                        "Deployment",
                                        &ns,
                                        &name,
                                        replicas,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let restart_action = {
                            let e = ctx_entity.clone();
                            let ns = deployment.namespace.clone();
                            let name = deployment.name.clone();
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_restart_confirm_modal(
                                        WorkloadActionKind::Restart,
                                        "Deployment",
                                        &ns,
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = deployment.namespace.clone();
                            let name = deployment.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Deployment",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let view_logs = {
                            let entity = ctx_entity.clone();
                            let name = deployment.name.clone();
                            let ns = deployment.namespace.clone();
                            move |_window: &mut Window, cx: &mut App| {
                                entity.update(cx, |shell, cx| {
                                    shell.open_logs_tab("Deployment", &ns, &name, cx);
                                });
                            }
                        };
                        let dep_ns = deployment.namespace.clone();
                        let dep_name = deployment.name.clone();
                        let dep_ports = extract_ports_from_deployment(deployment);
                        let pf_entity = ctx_entity.clone();
                        let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                            let ns = dep_ns.clone();
                            let name = dep_name.clone();
                            let ports = dep_ports.clone();
                            pf_entity.update(cx, |shell, cx| {
                                shell.open_port_forward_target_modal(
                                    &ns,
                                    "Deployment",
                                    &name,
                                    ports,
                                    cx,
                                );
                            });
                        };
                        let entries = views::deployments::build_deployment_context_menu_entries(
                            deployment,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            view_logs,
                            port_forward_action,
                            move |w, cx| scale_action(w, cx),
                            move |w, cx| restart_action(w, cx),
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.deployments.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.deployments.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Deployment {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::deployments::render_deployments_view(
                    &self.resources.deployments,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                    Some(self.table_enhancement_callbacks(WorkspaceView::Deployments, cx)),
                )
            }
            WorkspaceView::StatefulSets => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .statefulsets
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.statefulsets.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::StatefulSet {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          statefulset: &orbit_core::kubernetes::models::StatefulSetInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = statefulset.name.clone();
                        let namespace = statefulset.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::StatefulSet {
                            name: statefulset.name.clone(),
                            namespace: statefulset.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let scale_action = {
                            let e = ctx_entity.clone();
                            let ns = statefulset.namespace.clone();
                            let name = statefulset.name.clone();
                            let replicas = statefulset.replicas.desired;
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_scale_modal(
                                        "StatefulSet",
                                        &ns,
                                        &name,
                                        replicas,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let restart_action = {
                            let e = ctx_entity.clone();
                            let ns = statefulset.namespace.clone();
                            let name = statefulset.name.clone();
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_restart_confirm_modal(
                                        WorkloadActionKind::Restart,
                                        "StatefulSet",
                                        &ns,
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = statefulset.namespace.clone();
                            let name = statefulset.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "StatefulSet",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let view_logs = {
                            let entity = ctx_entity.clone();
                            let name = statefulset.name.clone();
                            let ns = statefulset.namespace.clone();
                            move |_window: &mut Window, cx: &mut App| {
                                entity.update(cx, |shell, cx| {
                                    shell.open_logs_tab("StatefulSet", &ns, &name, cx);
                                });
                            }
                        };
                        let ss_ns = statefulset.namespace.clone();
                        let ss_name = statefulset.name.clone();
                        let ss_ports = extract_ports_from_statefulset(statefulset);
                        let pf_entity = ctx_entity.clone();
                        let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                            let ns = ss_ns.clone();
                            let name = ss_name.clone();
                            let ports = ss_ports.clone();
                            pf_entity.update(cx, |shell, cx| {
                                shell.open_port_forward_target_modal(
                                    &ns,
                                    "StatefulSet",
                                    &name,
                                    ports,
                                    cx,
                                );
                            });
                        };
                        let entries = views::statefulsets::build_statefulset_context_menu_entries(
                            statefulset,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            view_logs,
                            port_forward_action,
                            move |w, cx| scale_action(w, cx),
                            move |w, cx| restart_action(w, cx),
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.statefulsets.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.statefulsets.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::StatefulSet {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::statefulsets::render_statefulsets_view(
                    &self.resources.statefulsets,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::ReplicaSets => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .replicasets
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.replicasets.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::ReplicaSet {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          replicaset: &orbit_core::kubernetes::models::ReplicaSetInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = replicaset.name.clone();
                        let namespace = replicaset.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::ReplicaSet {
                            name: replicaset.name.clone(),
                            namespace: replicaset.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let scale_action = {
                            let e = ctx_entity.clone();
                            let ns = replicaset.namespace.clone();
                            let name = replicaset.name.clone();
                            let replicas = replicaset.replicas.desired;
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_scale_modal(
                                        "ReplicaSet",
                                        &ns,
                                        &name,
                                        replicas,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = replicaset.namespace.clone();
                            let name = replicaset.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "ReplicaSet",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let view_logs = {
                            let entity = ctx_entity.clone();
                            let name = replicaset.name.clone();
                            let ns = replicaset.namespace.clone();
                            move |_window: &mut Window, cx: &mut App| {
                                entity.update(cx, |shell, cx| {
                                    shell.open_logs_tab("ReplicaSet", &ns, &name, cx);
                                });
                            }
                        };
                        let rs_ns = replicaset.namespace.clone();
                        let rs_name = replicaset.name.clone();
                        let rs_ports = extract_ports_from_replicaset(replicaset);
                        let pf_entity = ctx_entity.clone();
                        let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                            let ns = rs_ns.clone();
                            let name = rs_name.clone();
                            let ports = rs_ports.clone();
                            pf_entity.update(cx, |shell, cx| {
                                shell.open_port_forward_target_modal(
                                    &ns,
                                    "ReplicaSet",
                                    &name,
                                    ports,
                                    cx,
                                );
                            });
                        };
                        let entries = views::replicasets::build_replicaset_context_menu_entries(
                            replicaset,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            view_logs,
                            port_forward_action,
                            move |w, cx| scale_action(w, cx),
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.replicasets.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.replicasets.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::ReplicaSet {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::replicasets::render_replicasets_view(
                    &self.resources.replicasets,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::DaemonSets => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .daemonsets
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.daemonsets.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::DaemonSet {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          daemonset: &orbit_core::kubernetes::models::DaemonSetInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = daemonset.name.clone();
                        let namespace = daemonset.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::DaemonSet {
                            name: daemonset.name.clone(),
                            namespace: daemonset.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let restart_action = {
                            let e = ctx_entity.clone();
                            let ns = daemonset.namespace.clone();
                            let name = daemonset.name.clone();
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_restart_confirm_modal(
                                        WorkloadActionKind::Restart,
                                        "DaemonSet",
                                        &ns,
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = daemonset.namespace.clone();
                            let name = daemonset.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "DaemonSet",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let view_logs = {
                            let entity = ctx_entity.clone();
                            let name = daemonset.name.clone();
                            let ns = daemonset.namespace.clone();
                            move |_window: &mut Window, cx: &mut App| {
                                entity.update(cx, |shell, cx| {
                                    shell.open_logs_tab("DaemonSet", &ns, &name, cx);
                                });
                            }
                        };
                        let entries = views::daemonsets::build_daemonset_context_menu_entries(
                            daemonset,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            view_logs,
                            move |w, cx| restart_action(w, cx),
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.daemonsets.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.daemonsets.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::DaemonSet {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::daemonsets::render_daemonsets_view(
                    &self.resources.daemonsets,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::Jobs => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.jobs.table.toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.jobs.table.selected_row_key = Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Job {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          job: &orbit_core::kubernetes::models::JobInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = job.name.clone();
                        let namespace = job.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Job {
                            name: job.name.clone(),
                            namespace: job.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = job.namespace.clone();
                            let name = job.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Job",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let view_logs = {
                            let entity = ctx_entity.clone();
                            let name = job.name.clone();
                            let ns = job.namespace.clone();
                            move |_window: &mut Window, cx: &mut App| {
                                entity.update(cx, |shell, cx| {
                                    shell.open_logs_tab("Job", &ns, &name, cx);
                                });
                            }
                        };
                        let entries = views::jobs::build_job_context_menu_entries(
                            job,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            view_logs,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.jobs.table.selected_row_key = Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.jobs.table.selected_row_key = Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Job {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::jobs::render_jobs_view(
                    &self.resources.jobs,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::CronJobs => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.cronjobs.table.toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.cronjobs.table.selected_row_key = Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::CronJob {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          cj: &orbit_core::kubernetes::models::CronJobInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = cj.name.clone();
                        let namespace = cj.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::CronJob {
                            name: cj.name.clone(),
                            namespace: cj.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let trigger_action = {
                            let e = ctx_entity.clone();
                            let ns = cj.namespace.clone();
                            let name = cj.name.clone();
                            std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.trigger_cronjob_now(&ns, &name, cx);
                                });
                            })
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = cj.namespace.clone();
                            let name = cj.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "CronJob",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries = views::cronjobs::build_cronjob_context_menu_entries(
                            cj,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            move |w, cx| trigger_action(w, cx),
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.cronjobs.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.cronjobs.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::CronJob {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::cronjobs::render_cronjobs_view(
                    &self.resources.cronjobs,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::Pods => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.pods.table.toggle_sort(col_id.as_ref());
                        let cfg = shell.resources.pods.table.to_persisted_config();
                        shell.persist_table_config("pods", cfg);
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.pods.table.selected_row_key = Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Pod {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          pod: &orbit_core::kubernetes::models::PodInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = pod.name.clone();
                        let namespace = pod.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Pod {
                            name: pod.name.clone(),
                            namespace: pod.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let restart_pod_action = {
                            let e = ctx_entity.clone();
                            let ns = pod.namespace.clone();
                            let name = pod.name.clone();
                            std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_restart_confirm_modal(
                                        WorkloadActionKind::RestartPod,
                                        "Pod",
                                        &ns,
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            })
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = pod.namespace.clone();
                            let name = pod.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Pod",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let view_logs = {
                            let entity = ctx_entity.clone();
                            let name = pod.name.clone();
                            let ns = pod.namespace.clone();
                            move |_window: &mut Window, cx: &mut App| {
                                entity.update(cx, |shell, cx| {
                                    shell.open_logs_tab("Pod", &ns, &name, cx);
                                });
                            }
                        };
                        let pod_ns = pod.namespace.clone();
                        let pod_name = pod.name.clone();
                        let pod_ports = extract_ports_from_pod(pod);
                        let pf_entity = ctx_entity.clone();
                        let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                            let ns = pod_ns.clone();
                            let name = pod_name.clone();
                            let ports = pod_ports.clone();
                            pf_entity.update(cx, |shell, cx| {
                                shell.open_port_forward_target_modal(&ns, "Pod", &name, ports, cx);
                            });
                        };
                        let entries = views::pods::build_pod_context_menu_entries(
                            pod,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            view_logs,
                            port_forward_action,
                            move |w, cx| restart_pod_action(w, cx),
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.pods.table.selected_row_key = Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.pods.table.selected_row_key = Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Pod {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::pods::render_pods_view(
                    &self.resources.pods,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                    Some(self.table_enhancement_callbacks(WorkspaceView::Pods, cx)),
                )
            }
            WorkspaceView::Hpas => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .hpas
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.list_views.hpas.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Hpa {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.list_views.hpas.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Hpa {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          hpa: &orbit_core::kubernetes::models::HpaInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = hpa.name.clone();
                        let namespace = hpa.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Hpa {
                            name: hpa.name.clone(),
                            namespace: hpa.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_ctx_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_ctx_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let entries = views::hpas::build_hpa_context_menu_entries(
                            hpa,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.list_views.hpas.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                views::hpas::render_hpas_view(
                    &self.resources.list_views.hpas,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            _ => div().into_any_element(),
        }
    }

    fn render_network_workspace(
        &mut self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.sidebar.active_view {
            WorkspaceView::Services => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.services.table.toggle_sort(col_id.as_ref());
                        let cfg = shell.resources.services.table.to_persisted_config();
                        shell.persist_table_config("services", cfg);
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.services.table.selected_row_key = Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Service {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          svc: &orbit_core::kubernetes::models::ServiceInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = svc.name.clone();
                        let namespace = svc.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Service {
                            name: svc.name.clone(),
                            namespace: svc.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = svc.namespace.clone();
                            let name = svc.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Service",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let svc_ns = svc.namespace.clone();
                        let svc_name = svc.name.clone();
                        let svc_ports = extract_ports_from_service(svc);
                        let pf_entity = ctx_entity.clone();
                        let port_forward_action = move |_w: &mut Window, cx: &mut App| {
                            let ns = svc_ns.clone();
                            let name = svc_name.clone();
                            let ports = svc_ports.clone();
                            pf_entity.update(cx, |shell, cx| {
                                shell.open_port_forward_target_modal(
                                    &ns, "Service", &name, ports, cx,
                                );
                            });
                        };
                        let entries = views::services::build_service_context_menu_entries(
                            svc,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            port_forward_action,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.services.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let type_filter_entity = cx.entity().clone();
                let on_type_filter = move |filter: crate::state::ServiceTypeFilter,
                                           _window: &mut Window,
                                           cx: &mut App| {
                    type_filter_entity.update(cx, |shell, cx| {
                        shell.resources.services.type_filter = filter;
                        cx.notify();
                    });
                };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        if let ClusterConnectionState::Connected { cluster_id, .. } =
                            &shell.state.connection_state
                        {
                            let cmd =
                                shell.resources.services.begin_refresh().unwrap_or_else(|| {
                                    shell.resources.services.begin_load_for_cluster(cluster_id)
                                });
                            shell.dispatch_command(cmd);
                            cx.notify();
                        }
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.services.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Service {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::services::render_services_view(
                    &self.resources.services,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_type_filter,
                    on_refresh,
                    Some(self.table_enhancement_callbacks(WorkspaceView::Services, cx)),
                )
            }
            WorkspaceView::Ingresses => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.ingresses.table.toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.ingresses.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Ingress {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          ing: &orbit_core::kubernetes::models::IngressInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = ing.name.clone();
                        let namespace = ing.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Ingress {
                            name: ing.name.clone(),
                            namespace: ing.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = ing.namespace.clone();
                            let name = ing.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Ingress",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries = views::ingresses::build_ingress_context_menu_entries(
                            ing,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.ingresses.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh =
                    move |_window: &mut Window, cx: &mut App| {
                        refresh_entity.update(cx, |shell, cx| {
                            if let ClusterConnectionState::Connected { cluster_id, .. } =
                                &shell.state.connection_state
                            {
                                let cmd = shell.resources.ingresses.begin_refresh().unwrap_or_else(
                                    || shell.resources.ingresses.begin_load_for_cluster(cluster_id),
                                );
                                shell.dispatch_command(cmd);
                                cx.notify();
                            }
                        });
                    };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.ingresses.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Ingress {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::ingresses::render_ingresses_view(
                    &self.resources.ingresses,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::PortForward => {
                let sort_entity = cx.entity().clone();
                let on_sort = std::rc::Rc::new(
                    move |col_id: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        sort_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .port_forwards
                                .table
                                .toggle_sort(col_id.as_ref());
                            cx.notify();
                        });
                    },
                );

                let row_entity = cx.entity().clone();
                let on_row_click = std::rc::Rc::new(
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        row_entity.update(cx, |shell, cx| {
                            shell.resources.port_forwards.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.resources.port_forwards.search_focused = false;
                            cx.notify();
                        });
                    },
                );

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu = std::rc::Rc::new(
                    move |row_key: SharedString,
                          pf: &orbit_core::kubernetes::port_forward::PersistedPortForward,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let local_port = pf.local_port;
                        let pf_id = pf.id.clone();
                        let anchor = event.position;

                        let open_entity = ctx_entity.clone();
                        let on_open_browser = move |_w: &mut Window, cx: &mut App| {
                            open_entity.update(cx, |shell, cx| {
                                shell.open_port_forward_in_browser(local_port, cx);
                            });
                        };
                        let copy_entity = ctx_entity.clone();
                        let on_copy_url = move |_w: &mut Window, cx: &mut App| {
                            copy_entity.update(cx, |shell, cx| {
                                shell.copy_port_forward_url(local_port, cx);
                            });
                        };
                        let stop_entity = ctx_entity.clone();
                        let on_stop = move |_w: &mut Window, cx: &mut App| {
                            let id = pf_id.clone();
                            stop_entity.update(cx, |shell, cx| {
                                shell.stop_port_forward(id, cx);
                            });
                        };

                        let entries = views::port_forward::build_port_forward_context_menu_entries(
                            pf,
                            on_open_browser,
                            on_copy_url,
                            on_stop,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.port_forwards.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    },
                );

                let select_entity = cx.entity().clone();
                let on_toggle_select =
                    std::rc::Rc::new(move |id: String, _window: &mut Window, cx: &mut App| {
                        select_entity.update(cx, |shell, cx| {
                            shell.resources.port_forwards.toggle_select(&id);
                            cx.notify();
                        });
                    });

                let select_all_entity = cx.entity().clone();
                let on_toggle_select_all = std::rc::Rc::new(
                    move |ids: Vec<String>, _window: &mut Window, cx: &mut App| {
                        select_all_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .port_forwards
                                .toggle_select_all_visible(&ids);
                            cx.notify();
                        });
                    },
                );

                let kind_entity = cx.entity().clone();
                let on_kind_filter = std::rc::Rc::new(
                    move |filter: PortForwardKindFilter, _window: &mut Window, cx: &mut App| {
                        kind_entity.update(cx, |shell, cx| {
                            shell.resources.port_forwards.set_kind_filter(filter);
                            cx.notify();
                        });
                    },
                );

                let focus_entity = cx.entity().clone();
                let on_focus_search =
                    std::rc::Rc::new(move |focused: bool, _window: &mut Window, cx: &mut App| {
                        focus_entity.update(cx, |shell, cx| {
                            shell.resources.port_forwards.search_focused = focused;
                            cx.notify();
                        });
                    });

                let clear_entity = cx.entity().clone();
                let on_clear_search =
                    std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                        clear_entity.update(cx, |shell, cx| {
                            shell.resources.port_forwards.search_query.clear();
                            cx.notify();
                        });
                    });

                let create_entity = cx.entity().clone();
                let on_open_create_modal =
                    std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                        create_entity.update(cx, |shell, cx| {
                            shell.open_port_forward_create_modal(cx);
                        });
                    });

                let browser_entity = cx.entity().clone();
                let on_open_browser =
                    std::rc::Rc::new(move |port: u16, _window: &mut Window, cx: &mut App| {
                        browser_entity.update(cx, |shell, cx| {
                            shell.open_port_forward_in_browser(port, cx);
                        });
                    });

                let copy_entity = cx.entity().clone();
                let on_copy_url =
                    std::rc::Rc::new(move |port: u16, _window: &mut Window, cx: &mut App| {
                        copy_entity.update(cx, |shell, cx| {
                            shell.copy_port_forward_url(port, cx);
                        });
                    });

                let stop_entity = cx.entity().clone();
                let on_stop_tunnel =
                    std::rc::Rc::new(move |id: String, _window: &mut Window, cx: &mut App| {
                        stop_entity.update(cx, |shell, cx| {
                            shell.stop_port_forward(id, cx);
                        });
                    });

                let stop_sel_entity = cx.entity().clone();
                let on_stop_selected =
                    std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                        stop_sel_entity.update(cx, |shell, cx| {
                            shell.stop_selected_port_forwards(cx);
                        });
                    });

                let stop_all_entity = cx.entity().clone();
                let on_stop_all = std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                    stop_all_entity.update(cx, |shell, cx| {
                        shell.stop_all_port_forwards(cx);
                    });
                });

                let refresh_entity = cx.entity().clone();
                let on_refresh = std::rc::Rc::new(move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        if let ClusterConnectionState::Connected { cluster_id, .. } =
                            &shell.state.connection_state
                        {
                            let cmd = shell
                                .resources
                                .port_forwards
                                .begin_refresh()
                                .unwrap_or_else(|| {
                                    shell
                                        .resources
                                        .port_forwards
                                        .begin_load_for_cluster(cluster_id)
                                });
                            shell.dispatch_command(cmd);
                            cx.notify();
                        }
                    });
                });

                views::port_forward::render_port_forward_view(
                    &self.resources.port_forwards,
                    &self.resources.namespaces,
                    theme,
                    views::port_forward::PortForwardViewCallbacks {
                        on_sort,
                        on_row_click,
                        on_row_context_menu,
                        on_toggle_select,
                        on_toggle_select_all,
                        on_kind_filter,
                        on_focus_search,
                        on_clear_search,
                        on_open_create_modal,
                        on_open_browser,
                        on_copy_url,
                        on_stop_tunnel,
                        on_stop_selected,
                        on_stop_all,
                        on_refresh,
                    },
                )
            }
            WorkspaceView::NetworkPolicies => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .network_policies
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.network_policies.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::NetworkPolicy {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          policy: &orbit_core::kubernetes::models::PolicyInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = policy.name.clone();
                        let namespace = policy.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::NetworkPolicy {
                            name: policy.name.clone(),
                            namespace: policy.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = policy.namespace.clone();
                            let name = policy.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "NetworkPolicy",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries =
                            views::network_policies::build_network_policy_context_menu_entries(
                                policy,
                                copy_name,
                                filter_ns,
                                refresh_cb,
                                view_details,
                                view_yaml,
                                delete_action,
                            );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.network_policies.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.network_policies.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::NetworkPolicy {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::network_policies::render_network_policies_view(
                    &self.resources.network_policies,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            _ => div().into_any_element(),
        }
    }

    fn render_config_workspace(
        &mut self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.sidebar.active_view {
            WorkspaceView::ConfigMaps => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .configmaps
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.configmaps.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::ConfigMap {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          cm: &orbit_core::kubernetes::models::ConfigMapInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = cm.name.clone();
                        let namespace = cm.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::ConfigMap {
                            name: cm.name.clone(),
                            namespace: cm.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = cm.namespace.clone();
                            let name = cm.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "ConfigMap",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries = views::configmaps::build_configmap_context_menu_entries(
                            cm,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.configmaps.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh =
                    move |_window: &mut Window, cx: &mut App| {
                        refresh_entity.update(cx, |shell, cx| {
                            if let ClusterConnectionState::Connected { cluster_id, .. } =
                                &shell.state.connection_state
                            {
                                let cmd =
                                    shell.resources.configmaps.begin_refresh().unwrap_or_else(
                                        || {
                                            shell
                                                .resources
                                                .configmaps
                                                .begin_load_for_cluster(cluster_id)
                                        },
                                    );
                                shell.dispatch_command(cmd);
                                cx.notify();
                            }
                        });
                    };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.configmaps.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::ConfigMap {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::configmaps::render_configmaps_view(
                    &self.resources.configmaps,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::Secrets => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.secrets.table.toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.secrets.table.selected_row_key = Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::Secret {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          sec: &orbit_core::kubernetes::models::SecretInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = sec.name.clone();
                        let namespace = sec.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Secret {
                            name: sec.name.clone(),
                            namespace: sec.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = sec.namespace.clone();
                            let name = sec.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Secret",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries = views::secrets::build_secret_context_menu_entries(
                            sec,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.secrets.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        if let ClusterConnectionState::Connected { cluster_id, .. } =
                            &shell.state.connection_state
                        {
                            let cmd =
                                shell.resources.secrets.begin_refresh().unwrap_or_else(|| {
                                    shell.resources.secrets.begin_load_for_cluster(cluster_id)
                                });
                            shell.dispatch_command(cmd);
                            cx.notify();
                        }
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.secrets.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::Secret {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::secrets::render_secrets_view(
                    &self.resources.secrets,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            _ => div().into_any_element(),
        }
    }

    fn render_storage_workspace(
        &mut self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.sidebar.active_view {
            WorkspaceView::PersistentVolumes => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .persistent_volumes
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.persistent_volumes.table.selected_row_key =
                            Some(row_key.to_string());
                        let target = ResourceDrawerTarget::PersistentVolume {
                            name: row_key.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          pv: &orbit_core::kubernetes::models::PersistentVolumeInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = pv.name.clone();
                        let drawer_target = ResourceDrawerTarget::PersistentVolume {
                            name: pv.name.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let name = pv.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "PersistentVolume",
                                        None,
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries =
                            views::persistent_volumes::build_persistent_volume_context_menu_entries(
                                pv,
                                copy_name,
                                refresh_cb,
                                view_details,
                                view_yaml,
                                delete_action,
                            );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.persistent_volumes.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.persistent_volumes.table.selected_row_key =
                                Some(row_key.to_string());
                            let target = ResourceDrawerTarget::PersistentVolume {
                                name: row_key.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                        });
                    };

                views::persistent_volumes::render_persistent_volumes_view(
                    &self.resources.persistent_volumes,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::PersistentVolumeClaims => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .persistent_volume_claims
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .persistent_volume_claims
                            .table
                            .selected_row_key = Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::PersistentVolumeClaim {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          pvc: &orbit_core::kubernetes::models::PersistentVolumeClaimInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = pvc.name.clone();
                        let namespace = pvc.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::PersistentVolumeClaim {
                            name: pvc.name.clone(),
                            namespace: pvc.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = pvc.namespace.clone();
                            let name = pvc.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "PersistentVolumeClaim",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries =
                    views::persistent_volume_claims::build_persistent_volume_claim_context_menu_entries(
                        pvc,
                        copy_name,
                        filter_ns,
                        refresh_cb,
                        view_details,
                        view_yaml,
                        delete_action,
                    );
                        ctx_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .persistent_volume_claims
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .persistent_volume_claims
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::PersistentVolumeClaim {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::persistent_volume_claims::render_persistent_volume_claims_view(
                    &self.resources.persistent_volume_claims,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::StorageClasses => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .storage_classes
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .storage_classes
                            .table
                            .selected_row_key = Some(row_key.to_string());
                        let target = ResourceDrawerTarget::StorageClass {
                            name: row_key.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .list_views
                                .storage_classes
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            let target = ResourceDrawerTarget::StorageClass {
                                name: row_key.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                        });
                    };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          sc: &orbit_core::kubernetes::models::StorageClassInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = sc.name.clone();
                        let drawer_target = ResourceDrawerTarget::StorageClass {
                            name: sc.name.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_ctx_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_ctx_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let entries =
                            views::storage_classes::build_storage_class_context_menu_entries(
                                copy_name,
                                refresh_cb,
                                view_details,
                                view_yaml,
                            );
                        ctx_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .list_views
                                .storage_classes
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                views::storage_classes::render_storage_classes_view(
                    &self.resources.list_views.storage_classes,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            _ => div().into_any_element(),
        }
    }

    fn render_security_workspace(
        &mut self,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.sidebar.active_view {
            WorkspaceView::PodSecurity => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .pod_security
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.pod_security.table.selected_row_key =
                            Some(row_key.to_string());
                        if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::PodSecurity {
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          policy: &orbit_core::kubernetes::models::PolicyInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = policy.name.clone();
                        let namespace = policy.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::PodSecurity {
                            name: policy.name.clone(),
                            namespace: policy.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let entries = views::pod_security::build_pod_security_context_menu_entries(
                            policy,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.pod_security.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.pod_security.table.selected_row_key =
                                Some(row_key.to_string());
                            if let Some((ns, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::PodSecurity {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::pod_security::render_pod_security_view(
                    &self.resources.pod_security,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::ResourceQuotas => {
                let kind_entity = cx.entity().clone();
                let on_kind_filter =
                    move |filter: ResourceQuotaKindFilter, _window: &mut Window, cx: &mut App| {
                        kind_entity.update(cx, |shell, cx| {
                            shell.resources.resource_quotas.set_kind_filter(filter);
                            cx.notify();
                        });
                    };

                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .resource_quotas
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.resource_quotas.table.selected_row_key =
                            Some(row_key.to_string());
                        let mut parts = row_key.as_ref().splitn(3, '/');
                        if let (Some(kind), Some(ns), Some(name)) =
                            (parts.next(), parts.next(), parts.next())
                        {
                            let target = if kind.eq_ignore_ascii_case("LimitRange") {
                                ResourceDrawerTarget::LimitRange {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                }
                            } else {
                                ResourceDrawerTarget::ResourceQuota {
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                }
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          policy: &orbit_core::kubernetes::models::PolicyInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = policy.name.clone();
                        let namespace = policy.namespace.clone();
                        let drawer_target = if policy.kind.eq_ignore_ascii_case("LimitRange") {
                            ResourceDrawerTarget::LimitRange {
                                name: policy.name.clone(),
                                namespace: policy.namespace.clone(),
                            }
                        } else {
                            ResourceDrawerTarget::ResourceQuota {
                                name: policy.name.clone(),
                                namespace: policy.namespace.clone(),
                            }
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let kind = policy.kind.clone();
                            let ns = policy.namespace.clone();
                            let name = policy.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        &kind,
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries =
                            views::resource_quotas::build_resource_policy_context_menu_entries(
                                policy,
                                copy_name,
                                filter_ns,
                                refresh_cb,
                                view_details,
                                view_yaml,
                                delete_action,
                            );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.resource_quotas.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.resource_quotas.table.selected_row_key =
                                Some(row_key.to_string());
                            let mut parts = row_key.as_ref().splitn(3, '/');
                            if let (Some(kind), Some(ns), Some(name)) =
                                (parts.next(), parts.next(), parts.next())
                            {
                                let target = if kind.eq_ignore_ascii_case("LimitRange") {
                                    ResourceDrawerTarget::LimitRange {
                                        name: name.to_string(),
                                        namespace: ns.to_string(),
                                    }
                                } else {
                                    ResourceDrawerTarget::ResourceQuota {
                                        name: name.to_string(),
                                        namespace: ns.to_string(),
                                    }
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                views::resource_quotas::render_resource_quotas_view(
                    &self.resources.resource_quotas,
                    &self.resources.namespaces,
                    theme,
                    on_kind_filter,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::AdmissionWebhooks => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .admission_webhooks
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .admission_webhooks
                            .table
                            .selected_row_key = Some(row_key.to_string());
                        if let Some((kind, name)) = row_key.as_ref().split_once('/') {
                            let target = ResourceDrawerTarget::AdmissionWebhook {
                                is_mutating: kind
                                    .eq_ignore_ascii_case("MutatingWebhookConfiguration"),
                                name: name.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .list_views
                                .admission_webhooks
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            if let Some((kind, name)) = row_key.as_ref().split_once('/') {
                                let target = ResourceDrawerTarget::AdmissionWebhook {
                                    is_mutating: kind
                                        .eq_ignore_ascii_case("MutatingWebhookConfiguration"),
                                    name: name.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          policy: &orbit_core::kubernetes::models::PolicyInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = policy.name.clone();
                        let drawer_target = ResourceDrawerTarget::AdmissionWebhook {
                            is_mutating: policy
                                .kind
                                .eq_ignore_ascii_case("MutatingWebhookConfiguration"),
                            name: policy.name.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_ctx_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_ctx_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let entries =
                            views::admission_webhooks::build_admission_webhook_context_menu_entries(
                                copy_name,
                                refresh_cb,
                                view_details,
                                view_yaml,
                            );
                        ctx_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .list_views
                                .admission_webhooks
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                views::admission_webhooks::render_admission_webhooks_view(
                    &self.resources.list_views.admission_webhooks,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::SecurityOverview => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .security_overview
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .list_views
                            .security_overview
                            .table
                            .selected_row_key = Some(row_key.to_string());
                        let mut parts = row_key.as_ref().splitn(3, '/');
                        if let (Some(kind), Some(ns), Some(name)) =
                            (parts.next(), parts.next(), parts.next())
                        {
                            let target = ResourceDrawerTarget::SecurityPolicy {
                                kind: kind.to_string(),
                                name: name.to_string(),
                                namespace: ns.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .list_views
                                .security_overview
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            let mut parts = row_key.as_ref().splitn(3, '/');
                            if let (Some(kind), Some(ns), Some(name)) =
                                (parts.next(), parts.next(), parts.next())
                            {
                                let target = ResourceDrawerTarget::SecurityPolicy {
                                    kind: kind.to_string(),
                                    name: name.to_string(),
                                    namespace: ns.to_string(),
                                };
                                shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                            } else {
                                cx.notify();
                            }
                        });
                    };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          policy: &orbit_core::kubernetes::models::PolicyInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = policy.name.clone();
                        let namespace = policy.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::SecurityPolicy {
                            kind: policy.kind.clone(),
                            name: policy.name.clone(),
                            namespace: policy.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns =
                            (!namespace.is_empty() && namespace != "-").then(move || {
                                let ns_clone = namespace.clone();
                                move |_w: &mut Window, cx: &mut App| {
                                    filter_entity.update(cx, |shell, cx| {
                                        shell.filter_by_namespace(&ns_clone, cx);
                                    });
                                }
                            });
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_ctx_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_ctx_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let entries =
                            views::security_overview::build_security_policy_context_menu_entries(
                                policy,
                                copy_name,
                                filter_ns,
                                refresh_cb,
                                view_details,
                                view_yaml,
                            );
                        ctx_entity.update(cx, |shell, cx| {
                            shell
                                .resources
                                .list_views
                                .security_overview
                                .table
                                .selected_row_key = Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                views::security_overview::render_security_overview_view(
                    &self.resources.list_views.security_overview,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            _ => div().into_any_element(),
        }
    }

    fn render_cluster_workspace(
        &mut self,
        cluster_id: &str,
        display_name: &str,
        k8s_version: Option<&str>,
        theme: &OrbitTheme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let version_label = k8s_version.unwrap_or("Connected").to_string();
        match self.sidebar.active_view {
            WorkspaceView::Events => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.events.table.toggle_sort(col_id.as_ref());
                        let cfg = shell.resources.events.table.to_persisted_config();
                        shell.persist_table_config("events", cfg);
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.events.table.selected_row_key = Some(row_key.to_string());
                        if let Some(ev) = shell.resources.events.events.iter().find(|e| {
                            if !e.uid.is_empty() {
                                e.uid == row_key.as_ref()
                            } else {
                                format!("{}/{}/{}", e.namespace, e.reason, e.object_name)
                                    == row_key.as_ref()
                            }
                        }) {
                            let target = ResourceDrawerTarget::Event {
                                uid: ev.uid.clone(),
                                name: ev.name.clone(),
                                reason: ev.reason.clone(),
                                namespace: ev.namespace.clone(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                        } else {
                            cx.notify();
                        }
                    });
                };

                let filter_entity = cx.entity().clone();
                let on_type_filter =
                    move |filter: EventTypeFilter, _window: &mut Window, cx: &mut App| {
                        filter_entity.update(cx, |shell, cx| {
                            shell.resources.events.set_type_filter(filter);
                            cx.notify();
                        });
                    };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          ev: &orbit_core::kubernetes::models::EventInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let object_name = ev.object_name.clone();
                        let message = ev.message.clone();
                        let namespace = ev.namespace.clone();
                        let drawer_target = ResourceDrawerTarget::Event {
                            uid: ev.uid.clone(),
                            name: ev.name.clone(),
                            reason: ev.reason.clone(),
                            namespace: ev.namespace.clone(),
                        };
                        let anchor = event.position;
                        let copy_obj = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(object_name.clone()));
                        };
                        let copy_msg = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(message.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&namespace, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let ns = ev.namespace.clone();
                            let name = ev.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Event",
                                        Some(&ns),
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries = views::events::build_event_context_menu_entries(
                            ev,
                            copy_obj,
                            copy_msg,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.events.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                views::events::render_events_view(
                    &self.resources.events,
                    &self.resources.namespaces,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_context_menu,
                    on_type_filter,
                    on_refresh,
                    Some(self.table_enhancement_callbacks(WorkspaceView::Events, cx)),
                )
            }
            WorkspaceView::Namespaces => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell
                            .resources
                            .namespaces_view
                            .table
                            .toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.namespaces_view.table.selected_row_key =
                            Some(row_key.to_string());
                        let target = ResourceDrawerTarget::Namespace {
                            name: row_key.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          ns: &orbit_core::kubernetes::models::NamespaceInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = ns.name.clone();
                        let filter_name = ns.name.clone();
                        let drawer_target = ResourceDrawerTarget::Namespace {
                            name: ns.name.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let filter_entity = ctx_entity.clone();
                        let filter_ns = move |_w: &mut Window, cx: &mut App| {
                            filter_entity.update(cx, |shell, cx| {
                                shell.filter_by_namespace(&filter_name, cx);
                            });
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let name = ns.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell.open_delete_confirm_modal(
                                        "Namespace",
                                        None,
                                        &name,
                                        window,
                                        cx,
                                    );
                                });
                            }
                        };
                        let entries = views::namespaces::build_namespace_context_menu_entries(
                            ns,
                            copy_name,
                            filter_ns,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.namespaces_view.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.namespaces_view.table.selected_row_key =
                                Some(row_key.to_string());
                            let target = ResourceDrawerTarget::Namespace {
                                name: row_key.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                        });
                    };

                views::namespaces::render_namespaces_view(
                    &self.resources.namespaces,
                    &self.resources.namespaces_view,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            WorkspaceView::Nodes => {
                let sort_entity = cx.entity().clone();
                let on_sort = move |col_id: SharedString,
                                    _event: &gpui::ClickEvent,
                                    _window: &mut Window,
                                    cx: &mut App| {
                    sort_entity.update(cx, |shell, cx| {
                        shell.resources.nodes.table.toggle_sort(col_id.as_ref());
                        cx.notify();
                    });
                };

                let row_entity = cx.entity().clone();
                let on_row_click = move |_row_idx: usize,
                                         row_key: SharedString,
                                         _event: &gpui::ClickEvent,
                                         _window: &mut Window,
                                         cx: &mut App| {
                    row_entity.update(cx, |shell, cx| {
                        shell.resources.nodes.table.selected_row_key = Some(row_key.to_string());
                        let target = ResourceDrawerTarget::Node {
                            name: row_key.to_string(),
                        };
                        shell.open_resource_drawer(target, ResourceDrawerTabId::Overview, cx);
                    });
                };

                let ctx_entity = cx.entity().clone();
                let on_row_context_menu =
                    move |row_key: SharedString,
                          node: &orbit_core::kubernetes::models::NodeInfo,
                          event: &MouseDownEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        let name = node.name.clone();
                        let drawer_target = ResourceDrawerTarget::Node {
                            name: node.name.clone(),
                        };
                        let anchor = event.position;
                        let copy_name = move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                        };
                        let refresh_ctx_entity = ctx_entity.clone();
                        let refresh_cb = move |_w: &mut Window, cx: &mut App| {
                            refresh_ctx_entity.update(cx, |shell, cx| {
                                shell.refresh_workspace(cx);
                            });
                        };
                        let details_entity = ctx_entity.clone();
                        let details_target = drawer_target.clone();
                        let view_details = move |_w: &mut Window, cx: &mut App| {
                            details_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    details_target.clone(),
                                    ResourceDrawerTabId::Overview,
                                    cx,
                                );
                            });
                        };
                        let yaml_entity = ctx_entity.clone();
                        let view_yaml = move |_w: &mut Window, cx: &mut App| {
                            yaml_entity.update(cx, |shell, cx| {
                                shell.open_resource_drawer(
                                    drawer_target.clone(),
                                    ResourceDrawerTabId::Yaml,
                                    cx,
                                );
                            });
                        };
                        let delete_action = {
                            let e = ctx_entity.clone();
                            let name = node.name.clone();
                            move |window: &mut Window, cx: &mut App| {
                                e.update(cx, |shell, cx| {
                                    shell
                                        .open_delete_confirm_modal("Node", None, &name, window, cx);
                                });
                            }
                        };
                        let entries = views::nodes::build_node_context_menu_entries(
                            node,
                            copy_name,
                            refresh_cb,
                            view_details,
                            view_yaml,
                            delete_action,
                        );
                        ctx_entity.update(cx, |shell, cx| {
                            shell.resources.nodes.table.selected_row_key =
                                Some(row_key.to_string());
                            shell.open_context_menu(anchor, entries, cx);
                        });
                    };

                let refresh_entity = cx.entity().clone();
                let on_refresh = move |_window: &mut Window, cx: &mut App| {
                    refresh_entity.update(cx, |shell, cx| {
                        shell.refresh_workspace(cx);
                    });
                };

                let yaml_entity = cx.entity().clone();
                let on_row_quick_yaml =
                    move |_row_idx: usize,
                          row_key: SharedString,
                          _event: &gpui::ClickEvent,
                          _window: &mut Window,
                          cx: &mut App| {
                        yaml_entity.update(cx, |shell, cx| {
                            shell.resources.nodes.table.selected_row_key =
                                Some(row_key.to_string());
                            let target = ResourceDrawerTarget::Node {
                                name: row_key.to_string(),
                            };
                            shell.open_resource_drawer(target, ResourceDrawerTabId::Yaml, cx);
                        });
                    };

                views::nodes::render_nodes_view(
                    &self.resources.nodes,
                    theme,
                    on_sort,
                    on_row_click,
                    on_row_quick_yaml,
                    on_row_context_menu,
                    on_refresh,
                )
            }
            _ => {
                let view = self.sidebar.active_view;
                theme
                    .style_card(div())
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_6()
                    .w_full()
                    .max_w(px(720.0))
                    .bg(theme.surface.bg_panel)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                apply_text_step(div(), theme.typography.size_base)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("{} Workspace", view.label())),
                            )
                            .child(
                                apply_text_step(div(), theme.typography.size_xs)
                                    .px_2p5()
                                    .py_1()
                                    .rounded(theme.style.radius_control)
                                    .bg(theme.semantic.success.soft)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.semantic.success.solid)
                                    .child(format!("{display_name} • {version_label}")),
                            ),
                    )
                    .child(views::render_detail_row(
                        "View ID",
                        view.id().to_string(),
                        true,
                        theme,
                    ))
                    .child(views::render_detail_row(
                        "Active Context",
                        cluster_id.to_string(),
                        true,
                        theme,
                    ))
                    .child(views::render_detail_row(
                        "Kubernetes Version",
                        version_label,
                        true,
                        theme,
                    ))
                    .into_any_element()
            }
        }
    }
}
