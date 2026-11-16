use std::collections::HashMap;

use kube::api::DynamicObject;

use crate::kubernetes::models::PodMetricItem;
use crate::kubernetes::nodes::{format_decimal, parse_cpu_quantity, parse_memory_quantity};

pub fn format_memory_mib(mib: f64) -> String {
    format!("{}Mi", format_decimal(mib, 2))
}

pub fn format_cpu_cores(cores: f64) -> String {
    if cores >= 1.0 {
        format!("{} cores", format_decimal(cores, 2))
    } else if cores > 0.0 {
        format!("{}m", format_decimal(cores * 1000.0, 2))
    } else {
        "0m".to_string()
    }
}

pub fn parse_pod_metric_obj(obj: &DynamicObject) -> Option<PodMetricItem> {
    let name = obj.metadata.name.clone()?;
    let namespace = obj.metadata.namespace.clone().unwrap_or_default();

    let containers = obj.data.get("containers")?.as_array()?;

    let mut total_cpu_cores: f64 = 0.0;
    let mut total_mem_gib: f64 = 0.0;

    for container in containers {
        if let Some(usage) = container.get("usage") {
            if let Some(cpu) = usage.get("cpu").and_then(|v| v.as_str()) {
                total_cpu_cores += parse_cpu_quantity(cpu);
            }
            if let Some(mem) = usage.get("memory").and_then(|v| v.as_str()) {
                total_mem_gib += parse_memory_quantity(mem);
            }
        }
    }

    let cpu_str = format_cpu_cores(total_cpu_cores);
    let mem_mib = total_mem_gib * 1024.0;
    let memory_str = format_memory_mib(mem_mib);
    let memory_bytes = total_mem_gib * 1024.0 * 1024.0 * 1024.0;

    Some(PodMetricItem {
        name,
        namespace,
        cpu: cpu_str,
        memory: memory_str,
        cpu_cores: Some(total_cpu_cores),
        memory_bytes: Some(memory_bytes),
    })
}

pub fn parse_pod_metrics_map(items: &[DynamicObject]) -> HashMap<(String, String), PodMetricItem> {
    let mut map = HashMap::new();
    for obj in items {
        if let Some(item) = parse_pod_metric_obj(obj) {
            map.insert((item.name.clone(), item.namespace.clone()), item);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::api::{ApiResource, GroupVersionKind};
    use serde_json::json;

    #[test]
    fn test_format_memory_mib() {
        assert_eq!(format_memory_mib(300.25), "300.25Mi");
        assert_eq!(format_memory_mib(300.0), "300Mi");
        assert_eq!(format_memory_mib(45.5), "45.5Mi");
    }

    #[test]
    fn test_format_cpu_cores() {
        assert_eq!(format_cpu_cores(2.25), "2.25 cores");
        assert_eq!(format_cpu_cores(1.0), "1 cores");
        assert_eq!(format_cpu_cores(0.25), "250m");
        assert_eq!(format_cpu_cores(0.05), "50m");
        assert_eq!(format_cpu_cores(0.0), "0m");
    }

    #[test]
    fn test_parse_pod_metric_obj_and_map() {
        let ar = ApiResource::from_gvk(&GroupVersionKind::gvk(
            "metrics.k8s.io",
            "v1beta1",
            "PodMetrics",
        ));

        let mut obj1 = DynamicObject::new("web-0", &ar);
        obj1.metadata.namespace = Some("prod".to_string());
        obj1.data = json!({
            "containers": [
                {
                    "name": "app",
                    "usage": {
                        "cpu": "1500m",
                        "memory": "204800Ki"
                    }
                },
                {
                    "name": "sidecar",
                    "usage": {
                        "cpu": "750m",
                        "memory": "102400Ki"
                    }
                }
            ]
        });

        let mut empty_obj = DynamicObject::new("empty-pod", &ar);
        empty_obj.metadata.namespace = Some("default".to_string());
        empty_obj.data = json!({});

        let parsed = parse_pod_metric_obj(&obj1).expect("should parse multi-container metric");
        assert_eq!(parsed.name, "web-0");
        assert_eq!(parsed.namespace, "prod");
        assert_eq!(parsed.cpu, "2.25 cores");
        assert_eq!(parsed.memory, "300Mi");
        assert_eq!(parsed.cpu_cores, Some(2.25));
        assert!(parsed.memory_bytes.is_some());

        assert!(parse_pod_metric_obj(&empty_obj).is_none());

        let map = parse_pod_metrics_map(&[obj1, empty_obj]);
        assert_eq!(map.len(), 1);
        let item = map
            .get(&("web-0".to_string(), "prod".to_string()))
            .expect("web-0 in map");
        assert_eq!(item.cpu, "2.25 cores");
        assert_eq!(item.memory, "300Mi");
    }
}
