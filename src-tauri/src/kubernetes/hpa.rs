use std::collections::BTreeMap;
use k8s_openapi::api::autoscaling::v2::{HorizontalPodAutoscaler, MetricSpec, MetricStatus};
use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    Client,
};
use serde_json::json;
use crate::kubernetes::{format_age, models::*};

fn format_quantity_string(q: &k8s_openapi::apimachinery::pkg::api::resource::Quantity) -> String {
    q.0.clone()
}

pub fn map_hpa(hpa: &HorizontalPodAutoscaler) -> HpaInfo {
    let name = hpa.metadata.name.clone().unwrap_or_default();
    let namespace = hpa.metadata.namespace.clone().unwrap_or_default();
    let age = format_age(&hpa.metadata.creation_timestamp);
    let created = hpa
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.to_rfc3339())
        .unwrap_or_default();
    let labels: BTreeMap<String, String> = hpa.metadata.labels.clone().unwrap_or_default();
    let annotations: BTreeMap<String, String> = hpa.metadata.annotations.clone().unwrap_or_default();

    let spec = hpa.spec.as_ref();
    let status = hpa.status.as_ref();

    let scale_target_ref = spec
        .map(|s| HpaTargetRef {
            kind: s.scale_target_ref.kind.clone(),
            name: s.scale_target_ref.name.clone(),
            api_version: s.scale_target_ref.api_version.clone(),
        })
        .unwrap_or_else(|| HpaTargetRef {
            kind: "Unknown".to_string(),
            name: "Unknown".to_string(),
            api_version: None,
        });

    let min_replicas = spec.and_then(|s| s.min_replicas).unwrap_or(1);
    let max_replicas = spec.map(|s| s.max_replicas).unwrap_or(0);
    let current_replicas = status.and_then(|s| s.current_replicas).unwrap_or(0);
    let desired_replicas = status.map(|s| s.desired_replicas).unwrap_or(0);

    let last_scale_time = status
        .and_then(|s| s.last_scale_time.as_ref())
        .map(|t| t.0.to_rfc3339());

    let conditions: Vec<HpaConditionInfo> = status
        .and_then(|s| s.conditions.as_ref())
        .map(|conds| {
            conds
                .iter()
                .map(|c| HpaConditionInfo {
                    r#type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                    last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
                })
                .collect()
        })
        .unwrap_or_default();

    // Map metrics and current values
    let mut metrics = Vec::new();
    let mut metrics_summary = Vec::new();

    let spec_metrics: &[MetricSpec] = spec.and_then(|s| s.metrics.as_deref()).unwrap_or(&[]);
    let current_metrics: &[MetricStatus] = status.and_then(|s| s.current_metrics.as_deref()).unwrap_or(&[]);

    for m in spec_metrics {
        let metric_type = m.type_.clone();
        let mut name = String::new();
        let mut target_type = String::new();
        let mut target_value = String::new();
        let mut current_value: Option<String> = None;

        match metric_type.as_str() {
            "Resource" => {
                if let Some(res) = &m.resource {
                    name = res.name.clone();
                    let target = &res.target;
                    target_type = target.type_.clone();
                    if let Some(util) = target.average_utilization {
                        target_value = format!("{}%", util);
                    } else if let Some(avg_val) = &target.average_value {
                        target_value = format_quantity_string(avg_val);
                    } else if let Some(val) = &target.value {
                        target_value = format_quantity_string(val);
                    }

                    // Look for current metric status
                    for cur in current_metrics {
                        if cur.type_ == "Resource"
                            && let Some(cur_res) = &cur.resource
                            && cur_res.name == name
                        {
                            let current = &cur_res.current;
                            if let Some(util) = current.average_utilization {
                                current_value = Some(format!("{}%", util));
                            } else if let Some(avg_val) = &current.average_value {
                                current_value = Some(format_quantity_string(avg_val));
                            } else if let Some(val) = &current.value {
                                current_value = Some(format_quantity_string(val));
                            }
                            break;
                        }
                    }
                }
            }
            "Pods" => {
                if let Some(pods) = &m.pods {
                    name = pods.metric.name.clone();
                    target_type = pods.target.type_.clone();
                    if let Some(avg_val) = &pods.target.average_value {
                        target_value = format_quantity_string(avg_val);
                    }

                    for cur in current_metrics {
                        if cur.type_ == "Pods"
                            && let Some(cur_pods) = &cur.pods
                            && cur_pods.metric.name == name
                        {
                            if let Some(avg_val) = &cur_pods.current.average_value {
                                current_value = Some(format_quantity_string(avg_val));
                            } else if let Some(val) = &cur_pods.current.value {
                                current_value = Some(format_quantity_string(val));
                            }
                            break;
                        }
                    }
                }
            }
            "Object" => {
                if let Some(obj) = &m.object {
                    name = obj.metric.name.clone();
                    target_type = obj.target.type_.clone();
                    if let Some(val) = &obj.target.value {
                        target_value = format_quantity_string(val);
                    } else if let Some(avg_val) = &obj.target.average_value {
                        target_value = format_quantity_string(avg_val);
                    }

                    for cur in current_metrics {
                        if cur.type_ == "Object"
                            && let Some(cur_obj) = &cur.object
                            && cur_obj.metric.name == name
                        {
                            if let Some(val) = &cur_obj.current.value {
                                current_value = Some(format_quantity_string(val));
                            } else if let Some(avg_val) = &cur_obj.current.average_value {
                                current_value = Some(format_quantity_string(avg_val));
                            }
                            break;
                        }
                    }
                }
            }
            "External" => {
                if let Some(ext) = &m.external {
                    name = ext.metric.name.clone();
                    target_type = ext.target.type_.clone();
                    if let Some(val) = &ext.target.value {
                        target_value = format_quantity_string(val);
                    } else if let Some(avg_val) = &ext.target.average_value {
                        target_value = format_quantity_string(avg_val);
                    }

                    for cur in current_metrics {
                        if cur.type_ == "External"
                            && let Some(cur_ext) = &cur.external
                            && cur_ext.metric.name == name
                        {
                            if let Some(val) = &cur_ext.current.value {
                                current_value = Some(format_quantity_string(val));
                            } else if let Some(avg_val) = &cur_ext.current.average_value {
                                current_value = Some(format_quantity_string(avg_val));
                            }
                            break;
                        }
                    }
                }
            }
            "ContainerResource" => {
                if let Some(c_res) = &m.container_resource {
                    name = format!("{}/{}", c_res.container, c_res.name);
                    target_type = c_res.target.type_.clone();
                    if let Some(util) = c_res.target.average_utilization {
                        target_value = format!("{}%", util);
                    } else if let Some(avg_val) = &c_res.target.average_value {
                        target_value = format_quantity_string(avg_val);
                    } else if let Some(val) = &c_res.target.value {
                        target_value = format_quantity_string(val);
                    }

                    for cur in current_metrics {
                        if cur.type_ == "ContainerResource"
                            && let Some(cur_c_res) = &cur.container_resource
                            && format!("{}/{}", cur_c_res.container, cur_c_res.name) == name
                        {
                            if let Some(util) = cur_c_res.current.average_utilization {
                                current_value = Some(format!("{}%", util));
                            } else if let Some(avg_val) = &cur_c_res.current.average_value {
                                current_value = Some(format_quantity_string(avg_val));
                            } else if let Some(val) = &cur_c_res.current.value {
                                current_value = Some(format_quantity_string(val));
                            }
                            break;
                        }
                    }
                }
            }
            _ => {
                name = "unknown".to_string();
                target_type = "unknown".to_string();
            }
        }

        let summary_str = match &current_value {
            Some(curr) => format!("{}: {} / {}", name, curr, target_value),
            None => format!("{}: <unknown> / {}", name, target_value),
        };
        metrics_summary.push(summary_str);

        metrics.push(HpaMetricTarget {
            metric_type,
            name,
            target_type,
            target_value,
            current_value,
        });
    }

    // Resolve overall status: Active, ScalingLimited, Failed, Pending
    let mut resolved_status = "Active".to_string();

    let is_scaling_limited = conditions
        .iter()
        .any(|c| c.r#type == "ScalingLimited" && c.status == "True");
    let is_scaling_active_false = conditions
        .iter()
        .any(|c| c.r#type == "ScalingActive" && c.status == "False");
    let is_able_to_scale_false = conditions
        .iter()
        .any(|c| c.r#type == "AbleToScale" && c.status == "False");

    if is_scaling_active_false || is_able_to_scale_false {
        resolved_status = "Failed".to_string();
    } else if is_scaling_limited {
        resolved_status = "ScalingLimited".to_string();
    } else if current_replicas == 0 && desired_replicas == 0 && conditions.is_empty() {
        resolved_status = "Pending".to_string();
    }

    HpaInfo {
        kind: "HorizontalPodAutoscaler".to_string(),
        name,
        namespace,
        status: resolved_status,
        scale_target_ref,
        min_replicas,
        max_replicas,
        current_replicas,
        desired_replicas,
        metrics,
        metrics_summary,
        conditions,
        last_scale_time,
        age,
        created,
        labels,
        annotations,
    }
}

pub fn build_hpa_scale_patch(min_replicas: i32, max_replicas: i32) -> Result<serde_json::Value, String> {
    if min_replicas < 1 {
        return Err(format!("min_replicas must be at least 1, got {}", min_replicas));
    }
    if max_replicas < min_replicas {
        return Err(format!(
            "max_replicas ({}) cannot be less than min_replicas ({})",
            max_replicas, min_replicas
        ));
    }

    Ok(json!({
        "spec": {
            "minReplicas": min_replicas,
            "maxReplicas": max_replicas
        }
    }))
}

pub async fn list_hpas(client: &Client, namespace: Option<String>) -> Result<Vec<HpaInfo>, kube::Error> {
    let api: Api<HorizontalPodAutoscaler> = match namespace {
        Some(ns) if !ns.is_empty() && ns != "all" => Api::namespaced(client.clone(), &ns),
        _ => Api::all(client.clone()),
    };

    let lp = ListParams::default();
    let hpa_list = api.list(&lp).await?;

    Ok(hpa_list.items.iter().map(map_hpa).collect())
}

pub async fn update_hpa_replicas(
    client: &Client,
    namespace: &str,
    name: &str,
    min_replicas: i32,
    max_replicas: i32,
) -> Result<(), kube::Error> {
    let patch_val = build_hpa_scale_patch(min_replicas, max_replicas).map_err(|e| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: e,
            reason: "BadRequest".to_string(),
            code: 400,
        })
    })?;

    let api: Api<HorizontalPodAutoscaler> = Api::namespaced(client.clone(), namespace);
    let patch = Patch::Merge(&patch_val);
    let params = PatchParams::default();

    api.patch(name, &params, &patch).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::autoscaling::v2::{
        CrossVersionObjectReference, HorizontalPodAutoscalerCondition, HorizontalPodAutoscalerSpec,
        HorizontalPodAutoscalerStatus, MetricTarget, MetricValueStatus,
        ResourceMetricSource, ResourceMetricStatus,
    };
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_map_hpa_extracts_replicas_target_metrics_and_conditions() {
        let hpa = HorizontalPodAutoscaler {
            metadata: ObjectMeta {
                name: Some("api-server-hpa".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(HorizontalPodAutoscalerSpec {
                max_replicas: 10,
                min_replicas: None, // defaults to 1
                scale_target_ref: CrossVersionObjectReference {
                    api_version: Some("apps/v1".to_string()),
                    kind: "Deployment".to_string(),
                    name: "api-server".to_string(),
                },
                metrics: Some(vec![
                    MetricSpec {
                        type_: "Resource".to_string(),
                        resource: Some(ResourceMetricSource {
                            name: "cpu".to_string(),
                            target: MetricTarget {
                                type_: "Utilization".to_string(),
                                average_utilization: Some(80),
                                average_value: None,
                                value: None,
                            },
                        }),
                        container_resource: None,
                        external: None,
                        object: None,
                        pods: None,
                    },
                    MetricSpec {
                        type_: "Resource".to_string(),
                        resource: Some(ResourceMetricSource {
                            name: "memory".to_string(),
                            target: MetricTarget {
                                type_: "AverageValue".to_string(),
                                average_utilization: None,
                                average_value: Some(Quantity("512Mi".to_string())),
                                value: None,
                            },
                        }),
                        container_resource: None,
                        external: None,
                        object: None,
                        pods: None,
                    },
                ]),
                behavior: None,
            }),
            status: Some(HorizontalPodAutoscalerStatus {
                current_replicas: Some(3),
                desired_replicas: 4,
                last_scale_time: None,
                observed_generation: None,
                conditions: Some(vec![
                    HorizontalPodAutoscalerCondition {
                        type_: "ScalingActive".to_string(),
                        status: "True".to_string(),
                        reason: Some("ValidMetricFound".to_string()),
                        message: Some("HPA was able to find and read metrics".to_string()),
                        last_transition_time: None,
                    },
                    HorizontalPodAutoscalerCondition {
                        type_: "ScalingLimited".to_string(),
                        status: "False".to_string(),
                        reason: Some("DesiredWithinRange".to_string()),
                        message: Some("Replicas within bounds".to_string()),
                        last_transition_time: None,
                    },
                ]),
                current_metrics: Some(vec![
                    MetricStatus {
                        type_: "Resource".to_string(),
                        resource: Some(ResourceMetricStatus {
                            name: "cpu".to_string(),
                            current: MetricValueStatus {
                                average_utilization: Some(65),
                                average_value: None,
                                value: None,
                            },
                        }),
                        container_resource: None,
                        external: None,
                        object: None,
                        pods: None,
                    },
                    MetricStatus {
                        type_: "Resource".to_string(),
                        resource: Some(ResourceMetricStatus {
                            name: "memory".to_string(),
                            current: MetricValueStatus {
                                average_utilization: None,
                                average_value: Some(Quantity("256Mi".to_string())),
                                value: None,
                            },
                        }),
                        container_resource: None,
                        external: None,
                        object: None,
                        pods: None,
                    },
                ]),
            }),
        };

        let info = map_hpa(&hpa);
        assert_eq!(info.kind, "HorizontalPodAutoscaler");
        assert_eq!(info.name, "api-server-hpa");
        assert_eq!(info.namespace, "default");
        assert_eq!(info.scale_target_ref.kind, "Deployment");
        assert_eq!(info.scale_target_ref.name, "api-server");
        assert_eq!(info.min_replicas, 1);
        assert_eq!(info.max_replicas, 10);
        assert_eq!(info.current_replicas, 3);
        assert_eq!(info.desired_replicas, 4);
        assert_eq!(info.status, "Active");
        assert_eq!(info.metrics.len(), 2);
        assert_eq!(info.metrics[0].name, "cpu");
        assert_eq!(info.metrics[0].target_value, "80%");
        assert_eq!(info.metrics[0].current_value, Some("65%".to_string()));
        assert_eq!(info.metrics[1].name, "memory");
        assert_eq!(info.metrics[1].target_value, "512Mi");
        assert_eq!(info.metrics[1].current_value, Some("256Mi".to_string()));
        assert_eq!(
            info.metrics_summary,
            vec!["cpu: 65% / 80%".to_string(), "memory: 256Mi / 512Mi".to_string()]
        );
        assert_eq!(info.conditions.len(), 2);
        assert_eq!(info.conditions[0].r#type, "ScalingActive");
        assert_eq!(info.conditions[0].status, "True");
    }

    #[test]
    fn test_map_hpa_status_scaling_limited_and_failed() {
        // Test ScalingLimited
        let mut hpa = HorizontalPodAutoscaler {
            metadata: ObjectMeta {
                name: Some("hpa-limited".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(HorizontalPodAutoscalerSpec {
                max_replicas: 5,
                min_replicas: Some(1),
                scale_target_ref: CrossVersionObjectReference {
                    api_version: None,
                    kind: "Deployment".to_string(),
                    name: "app".to_string(),
                },
                metrics: None,
                behavior: None,
            }),
            status: Some(HorizontalPodAutoscalerStatus {
                current_replicas: Some(5),
                desired_replicas: 5,
                last_scale_time: None,
                observed_generation: None,
                conditions: Some(vec![HorizontalPodAutoscalerCondition {
                    type_: "ScalingLimited".to_string(),
                    status: "True".to_string(),
                    reason: Some("TooManyReplicas".to_string()),
                    message: None,
                    last_transition_time: None,
                }]),
                current_metrics: None,
            }),
        };

        let info = map_hpa(&hpa);
        assert_eq!(info.status, "ScalingLimited");

        // Test Failed when ScalingActive is False
        hpa.status.as_mut().unwrap().conditions = Some(vec![HorizontalPodAutoscalerCondition {
            type_: "ScalingActive".to_string(),
            status: "False".to_string(),
            reason: Some("FailedGetResourceMetric".to_string()),
            message: None,
            last_transition_time: None,
        }]);
        let info = map_hpa(&hpa);
        assert_eq!(info.status, "Failed");

        // Test Pending when replicas are 0 and no conditions
        hpa.status = Some(HorizontalPodAutoscalerStatus {
            current_replicas: Some(0),
            desired_replicas: 0,
            last_scale_time: None,
            observed_generation: None,
            conditions: None,
            current_metrics: None,
        });
        let info = map_hpa(&hpa);
        assert_eq!(info.status, "Pending");
    }

    #[test]
    fn test_build_hpa_scale_patch_validation() {
        let valid = build_hpa_scale_patch(2, 10).unwrap();
        assert_eq!(valid["spec"]["minReplicas"], 2);
        assert_eq!(valid["spec"]["maxReplicas"], 10);

        assert!(build_hpa_scale_patch(0, 10).is_err());
        assert!(build_hpa_scale_patch(5, 3).is_err());
    }
}
