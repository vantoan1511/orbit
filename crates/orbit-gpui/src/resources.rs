use orbit_core::command::ResourceKind;
use orbit_core::kubernetes::models::{
    ConfigMapInfo, DaemonSetInfo, DeploymentInfo, IngressInfo, NodeInfo, PodInfo, SecretInfo,
    ServiceInfo, StatefulSetInfo,
};
use std::collections::BTreeMap;

pub const SUPPORTED_RESOURCE_KINDS: [ResourceKind; 9] = [
    ResourceKind::Pod,
    ResourceKind::Deployment,
    ResourceKind::DaemonSet,
    ResourceKind::StatefulSet,
    ResourceKind::Node,
    ResourceKind::Service,
    ResourceKind::Ingress,
    ResourceKind::ConfigMap,
    ResourceKind::Secret,
];

fn map_labels(labels: &BTreeMap<String, String>) -> Vec<(String, String)> {
    labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

fn or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub header: &'static str,
    pub width_px: Option<f32>,
    pub right_aligned: bool,
}

impl ColumnDef {
    pub const fn flex(header: &'static str) -> Self {
        Self {
            header,
            width_px: None,
            right_aligned: false,
        }
    }

    pub const fn fixed(header: &'static str, width_px: f32, right_aligned: bool) -> Self {
        Self {
            header,
            width_px: Some(width_px),
            right_aligned,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRow {
    pub kind: ResourceKind,
    pub name: String,
    pub namespace: Option<String>,
    pub status: String,
    pub cells: Vec<String>,
    pub details: Vec<(String, String)>,
    pub labels: Vec<(String, String)>,
}

pub fn kind_display_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Pod => "Pods",
        ResourceKind::Deployment => "Deployments",
        ResourceKind::DaemonSet => "DaemonSets",
        ResourceKind::StatefulSet => "StatefulSets",
        ResourceKind::Node => "Nodes",
        ResourceKind::Service => "Services",
        ResourceKind::ConfigMap => "ConfigMaps",
        ResourceKind::Secret => "Secrets",
        ResourceKind::Namespace => "Namespaces",
        ResourceKind::ReplicaSet => "ReplicaSets",
        ResourceKind::Job => "Jobs",
        ResourceKind::CronJob => "CronJobs",
        ResourceKind::HorizontalPodAutoscaler => "HorizontalPodAutoscalers",
        ResourceKind::Ingress => "Ingresses",
        ResourceKind::Event => "Events",
        ResourceKind::PersistentVolume => "PersistentVolumes",
        ResourceKind::PersistentVolumeClaim => "PersistentVolumeClaims",
        ResourceKind::StorageClass => "StorageClasses",
        ResourceKind::Policy => "Policies",
    }
}

pub fn kind_api_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Pod => "Pod",
        ResourceKind::Deployment => "Deployment",
        ResourceKind::DaemonSet => "DaemonSet",
        ResourceKind::StatefulSet => "StatefulSet",
        ResourceKind::Node => "Node",
        ResourceKind::Service => "Service",
        ResourceKind::ConfigMap => "ConfigMap",
        ResourceKind::Secret => "Secret",
        ResourceKind::Namespace => "Namespace",
        ResourceKind::ReplicaSet => "ReplicaSet",
        ResourceKind::Job => "Job",
        ResourceKind::CronJob => "CronJob",
        ResourceKind::HorizontalPodAutoscaler => "HorizontalPodAutoscaler",
        ResourceKind::Ingress => "Ingress",
        ResourceKind::Event => "Event",
        ResourceKind::PersistentVolume => "PersistentVolume",
        ResourceKind::PersistentVolumeClaim => "PersistentVolumeClaim",
        ResourceKind::StorageClass => "StorageClass",
        ResourceKind::Policy => "Policy",
    }
}

pub fn columns_for_kind(kind: ResourceKind) -> Vec<ColumnDef> {
    match kind {
        ResourceKind::Pod => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 130.0, false),
            ColumnDef::fixed("STATUS", 140.0, false),
            ColumnDef::fixed("RESTARTS", 72.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
            ColumnDef::fixed("NODE", 140.0, false),
        ],
        ResourceKind::Deployment => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 130.0, false),
            ColumnDef::fixed("STATUS", 120.0, false),
            ColumnDef::fixed("READY", 72.0, true),
            ColumnDef::fixed("UP-TO-DATE", 84.0, true),
            ColumnDef::fixed("AVAILABLE", 80.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        ResourceKind::DaemonSet => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 130.0, false),
            ColumnDef::fixed("STATUS", 120.0, false),
            ColumnDef::fixed("DESIRED", 72.0, true),
            ColumnDef::fixed("CURRENT", 72.0, true),
            ColumnDef::fixed("READY", 72.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        ResourceKind::StatefulSet => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 130.0, false),
            ColumnDef::fixed("STATUS", 120.0, false),
            ColumnDef::fixed("READY", 72.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
            ColumnDef::fixed("IMAGES", 200.0, false),
        ],
        ResourceKind::Node => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("STATUS", 110.0, false),
            ColumnDef::fixed("ROLE", 120.0, false),
            ColumnDef::fixed("VERSION", 110.0, false),
            ColumnDef::fixed("INTERNAL IP", 120.0, false),
            ColumnDef::fixed("CPU", 90.0, true),
            ColumnDef::fixed("MEMORY", 90.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        ResourceKind::Service => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 130.0, false),
            ColumnDef::fixed("TYPE", 110.0, false),
            ColumnDef::fixed("CLUSTER IP", 120.0, false),
            ColumnDef::fixed("PORTS", 150.0, false),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        ResourceKind::Ingress => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 130.0, false),
            ColumnDef::fixed("CLASS", 110.0, false),
            ColumnDef::fixed("HOSTS", 180.0, false),
            ColumnDef::fixed("ADDRESS", 130.0, false),
            ColumnDef::fixed("PORTS", 110.0, false),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        ResourceKind::ConfigMap => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 140.0, false),
            ColumnDef::fixed("KEYS", 72.0, true),
            ColumnDef::fixed("SIZE", 84.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        ResourceKind::Secret => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 140.0, false),
            ColumnDef::fixed("TYPE", 180.0, false),
            ColumnDef::fixed("KEYS", 72.0, true),
            ColumnDef::fixed("AGE", 64.0, true),
        ],
        _ => vec![
            ColumnDef::flex("NAME"),
            ColumnDef::fixed("NAMESPACE", 140.0, false),
            ColumnDef::fixed("STATUS", 120.0, false),
        ],
    }
}

impl From<&PodInfo> for ResourceRow {
    fn from(pod: &PodInfo) -> Self {
        let node = pod.node.clone().unwrap_or_else(|| "-".to_string());
        let mut details = vec![
            ("Namespace".to_string(), pod.namespace.clone()),
            ("Status".to_string(), pod.status.clone()),
            ("Node".to_string(), node.clone()),
            ("Restarts".to_string(), pod.restarts.to_string()),
            ("Age".to_string(), pod.age.clone()),
        ];
        if let Some(ip) = &pod.ip {
            details.push(("Pod IP".to_string(), ip.clone()));
        }
        if let Some(qos) = &pod.qos_class {
            details.push(("QoS Class".to_string(), qos.clone()));
        }
        if !pod.images.is_empty() {
            details.push(("Images".to_string(), pod.images.join(", ")));
        }

        Self {
            kind: ResourceKind::Pod,
            name: pod.name.clone(),
            namespace: Some(pod.namespace.clone()),
            status: pod.status.clone(),
            cells: vec![
                pod.name.clone(),
                pod.namespace.clone(),
                pod.status.clone(),
                pod.restarts.to_string(),
                pod.age.clone(),
                node,
            ],
            details,
            labels: map_labels(&pod.labels),
        }
    }
}

impl From<&DeploymentInfo> for ResourceRow {
    fn from(dep: &DeploymentInfo) -> Self {
        let ready = format!("{}/{}", dep.replicas.current, dep.replicas.desired);
        let mut details = vec![
            ("Namespace".to_string(), dep.namespace.clone()),
            ("Status".to_string(), dep.status.clone()),
            ("Ready".to_string(), ready.clone()),
            ("Up-to-date".to_string(), dep.up_to_date.to_string()),
            ("Available".to_string(), dep.available.to_string()),
            ("Age".to_string(), dep.age.clone()),
        ];
        if let Some(strategy) = &dep.strategy {
            details.push(("Strategy".to_string(), strategy.clone()));
        }
        if !dep.images.is_empty() {
            details.push(("Images".to_string(), dep.images.join(", ")));
        }

        Self {
            kind: ResourceKind::Deployment,
            name: dep.name.clone(),
            namespace: Some(dep.namespace.clone()),
            status: dep.status.clone(),
            cells: vec![
                dep.name.clone(),
                dep.namespace.clone(),
                dep.status.clone(),
                ready,
                dep.up_to_date.to_string(),
                dep.available.to_string(),
                dep.age.clone(),
            ],
            details,
            labels: map_labels(&dep.labels),
        }
    }
}

impl From<&DaemonSetInfo> for ResourceRow {
    fn from(ds: &DaemonSetInfo) -> Self {
        let mut details = vec![
            ("Namespace".to_string(), ds.namespace.clone()),
            ("Status".to_string(), ds.status.clone()),
            ("Desired".to_string(), ds.replicas.desired.to_string()),
            ("Current".to_string(), ds.replicas.current.to_string()),
            ("Ready".to_string(), ds.replicas.ready.to_string()),
            ("Age".to_string(), ds.age.clone()),
        ];
        if !ds.images.is_empty() {
            details.push(("Images".to_string(), ds.images.join(", ")));
        }

        Self {
            kind: ResourceKind::DaemonSet,
            name: ds.name.clone(),
            namespace: Some(ds.namespace.clone()),
            status: ds.status.clone(),
            cells: vec![
                ds.name.clone(),
                ds.namespace.clone(),
                ds.status.clone(),
                ds.replicas.desired.to_string(),
                ds.replicas.current.to_string(),
                ds.replicas.ready.to_string(),
                ds.age.clone(),
            ],
            details,
            labels: map_labels(&ds.labels),
        }
    }
}

impl From<&StatefulSetInfo> for ResourceRow {
    fn from(sts: &StatefulSetInfo) -> Self {
        let ready = format!("{}/{}", sts.replicas.current, sts.replicas.desired);
        let images = if sts.images.is_empty() {
            "-".to_string()
        } else {
            sts.images.join(", ")
        };
        let details = vec![
            ("Namespace".to_string(), sts.namespace.clone()),
            ("Status".to_string(), sts.status.clone()),
            ("Ready".to_string(), ready.clone()),
            ("Age".to_string(), sts.age.clone()),
            ("Images".to_string(), images.clone()),
        ];

        Self {
            kind: ResourceKind::StatefulSet,
            name: sts.name.clone(),
            namespace: Some(sts.namespace.clone()),
            status: sts.status.clone(),
            cells: vec![
                sts.name.clone(),
                sts.namespace.clone(),
                sts.status.clone(),
                ready,
                sts.age.clone(),
                images,
            ],
            details,
            labels: map_labels(&sts.labels),
        }
    }
}

impl From<&NodeInfo> for ResourceRow {
    fn from(node: &NodeInfo) -> Self {
        let cpu = format!("{} / {}", node.cpu_used, node.cpu_total);
        let mem = format!("{} / {}", node.mem_used, node.mem_total);
        let internal_ip = node
            .addresses
            .iter()
            .find(|addr| addr.type_ == "InternalIP")
            .map(|addr| addr.address.clone())
            .unwrap_or_else(|| "-".to_string());
        let details = vec![
            ("Status".to_string(), node.status.clone()),
            ("Role".to_string(), node.role.clone()),
            ("Version".to_string(), node.version.clone()),
            ("Internal IP".to_string(), internal_ip.clone()),
            ("CPU".to_string(), cpu.clone()),
            ("Memory".to_string(), mem.clone()),
            (
                "Pods".to_string(),
                format!("{}/{}", node.pods_count, node.pods_limit),
            ),
            ("Uptime".to_string(), node.uptime.clone()),
        ];

        Self {
            kind: ResourceKind::Node,
            name: node.name.clone(),
            namespace: None,
            status: node.status.clone(),
            cells: vec![
                node.name.clone(),
                node.status.clone(),
                node.role.clone(),
                node.version.clone(),
                internal_ip,
                cpu,
                mem,
                node.uptime.clone(),
            ],
            details,
            labels: map_labels(&node.labels_map),
        }
    }
}

impl From<&ServiceInfo> for ResourceRow {
    fn from(svc: &ServiceInfo) -> Self {
        let ports = if svc.ports.is_empty() {
            "-".to_string()
        } else {
            svc.ports.clone()
        };
        let details = vec![
            ("Namespace".to_string(), svc.namespace.clone()),
            ("Type".to_string(), svc.r#type.clone()),
            ("Cluster IP".to_string(), svc.cluster_ip.clone()),
            ("External IP".to_string(), svc.external_ip.clone()),
            ("Ports".to_string(), ports.clone()),
            ("Endpoints".to_string(), svc.endpoints.clone()),
            ("Age".to_string(), svc.age.clone()),
        ];

        Self {
            kind: ResourceKind::Service,
            name: svc.name.clone(),
            namespace: Some(svc.namespace.clone()),
            status: "Active".to_string(),
            cells: vec![
                svc.name.clone(),
                svc.namespace.clone(),
                svc.r#type.clone(),
                svc.cluster_ip.clone(),
                ports,
                svc.age.clone(),
            ],
            details,
            labels: map_labels(&svc.labels),
        }
    }
}

impl From<&IngressInfo> for ResourceRow {
    fn from(ing: &IngressInfo) -> Self {
        let class_name = ing.class_name.clone().unwrap_or_else(|| "-".to_string());
        let hosts = or_dash(&ing.hosts);
        let address = or_dash(&ing.address);
        let ports = or_dash(&ing.ports);
        let mut details = vec![
            ("Namespace".to_string(), ing.namespace.clone()),
            ("Class".to_string(), class_name.clone()),
            ("Hosts".to_string(), hosts.clone()),
            ("Address".to_string(), address.clone()),
            ("Ports".to_string(), ports.clone()),
            ("Age".to_string(), ing.age.clone()),
        ];
        if !ing.rules_summary.is_empty() {
            details.push(("Rules".to_string(), ing.rules_summary.join(", ")));
        }

        Self {
            kind: ResourceKind::Ingress,
            name: ing.name.clone(),
            namespace: Some(ing.namespace.clone()),
            status: "Active".to_string(),
            cells: vec![
                ing.name.clone(),
                ing.namespace.clone(),
                class_name,
                hosts,
                address,
                ports,
                ing.age.clone(),
            ],
            details,
            labels: map_labels(&ing.labels),
        }
    }
}

impl From<&ConfigMapInfo> for ResourceRow {
    fn from(cm: &ConfigMapInfo) -> Self {
        let details = vec![
            ("Namespace".to_string(), cm.namespace.clone()),
            ("Keys".to_string(), cm.keys_count.to_string()),
            ("Size".to_string(), cm.size.clone()),
            ("Immutable".to_string(), cm.immutable.to_string()),
            ("Mounted Pods".to_string(), cm.mounted_pods.to_string()),
            ("Age".to_string(), cm.age.clone()),
        ];

        Self {
            kind: ResourceKind::ConfigMap,
            name: cm.name.clone(),
            namespace: Some(cm.namespace.clone()),
            status: "Active".to_string(),
            cells: vec![
                cm.name.clone(),
                cm.namespace.clone(),
                cm.keys_count.to_string(),
                cm.size.clone(),
                cm.age.clone(),
            ],
            details,
            labels: map_labels(&cm.labels),
        }
    }
}

impl From<&SecretInfo> for ResourceRow {
    fn from(sec: &SecretInfo) -> Self {
        let details = vec![
            ("Namespace".to_string(), sec.namespace.clone()),
            ("Type".to_string(), sec.r#type.clone()),
            ("Keys".to_string(), sec.keys_count.to_string()),
            ("Size".to_string(), sec.size.clone()),
            ("Immutable".to_string(), sec.immutable.to_string()),
            ("Mounted Pods".to_string(), sec.mounted_pods.to_string()),
            ("Age".to_string(), sec.age.clone()),
        ];

        Self {
            kind: ResourceKind::Secret,
            name: sec.name.clone(),
            namespace: Some(sec.namespace.clone()),
            status: "Active".to_string(),
            cells: vec![
                sec.name.clone(),
                sec.namespace.clone(),
                sec.r#type.clone(),
                sec.keys_count.to_string(),
                sec.age.clone(),
            ],
            details,
            labels: map_labels(&sec.labels),
        }
    }
}

pub fn mock_rows_for_kind(kind: ResourceKind) -> Vec<ResourceRow> {
    match kind {
        ResourceKind::Pod => vec![
            ResourceRow {
                kind,
                name: "coredns-5dd5756b68-7x9k2".to_string(),
                namespace: Some("kube-system".to_string()),
                status: "Running".to_string(),
                cells: vec![
                    "coredns-5dd5756b68-7x9k2".to_string(),
                    "kube-system".to_string(),
                    "Running".to_string(),
                    "0".to_string(),
                    "14d".to_string(),
                    "orb-control-plane".to_string(),
                ],
                details: vec![
                    ("Namespace".to_string(), "kube-system".to_string()),
                    ("Status".to_string(), "Running".to_string()),
                    ("Node".to_string(), "orb-control-plane".to_string()),
                ],
                labels: vec![("k8s-app".to_string(), "kube-dns".to_string())],
            },
            ResourceRow {
                kind,
                name: "orbit-api-gateway-6c8f9d4b-m2p9q".to_string(),
                namespace: Some("production".to_string()),
                status: "Running".to_string(),
                cells: vec![
                    "orbit-api-gateway-6c8f9d4b-m2p9q".to_string(),
                    "production".to_string(),
                    "Running".to_string(),
                    "0".to_string(),
                    "3d".to_string(),
                    "orb-worker-01".to_string(),
                ],
                details: vec![
                    ("Namespace".to_string(), "production".to_string()),
                    ("Status".to_string(), "Running".to_string()),
                    ("Node".to_string(), "orb-worker-01".to_string()),
                ],
                labels: vec![("app".to_string(), "orbit-api-gateway".to_string())],
            },
            ResourceRow {
                kind,
                name: "metrics-collector-74b9d8c6-v8n4w".to_string(),
                namespace: Some("monitoring".to_string()),
                status: "Pending".to_string(),
                cells: vec![
                    "metrics-collector-74b9d8c6-v8n4w".to_string(),
                    "monitoring".to_string(),
                    "Pending".to_string(),
                    "1".to_string(),
                    "12m".to_string(),
                    "orb-worker-02".to_string(),
                ],
                details: vec![
                    ("Namespace".to_string(), "monitoring".to_string()),
                    ("Status".to_string(), "Pending".to_string()),
                    ("Node".to_string(), "orb-worker-02".to_string()),
                ],
                labels: vec![("app".to_string(), "metrics-collector".to_string())],
            },
            ResourceRow {
                kind,
                name: "cache-warmer-cron-289104-x5k1z".to_string(),
                namespace: Some("jobs".to_string()),
                status: "CrashLoopBackOff".to_string(),
                cells: vec![
                    "cache-warmer-cron-289104-x5k1z".to_string(),
                    "jobs".to_string(),
                    "CrashLoopBackOff".to_string(),
                    "6".to_string(),
                    "45m".to_string(),
                    "orb-worker-01".to_string(),
                ],
                details: vec![
                    ("Namespace".to_string(), "jobs".to_string()),
                    ("Status".to_string(), "CrashLoopBackOff".to_string()),
                    ("Node".to_string(), "orb-worker-01".to_string()),
                ],
                labels: vec![("job-name".to_string(), "cache-warmer".to_string())],
            },
        ],
        ResourceKind::Deployment => vec![ResourceRow {
            kind,
            name: "orbit-api-gateway".to_string(),
            namespace: Some("production".to_string()),
            status: "Running".to_string(),
            cells: vec![
                "orbit-api-gateway".to_string(),
                "production".to_string(),
                "Running".to_string(),
                "3/3".to_string(),
                "3".to_string(),
                "3".to_string(),
                "14d".to_string(),
            ],
            details: vec![
                ("Namespace".to_string(), "production".to_string()),
                ("Strategy".to_string(), "RollingUpdate".to_string()),
            ],
            labels: vec![("app".to_string(), "orbit-api-gateway".to_string())],
        }],
        ResourceKind::DaemonSet => vec![ResourceRow {
            kind,
            name: "kube-proxy".to_string(),
            namespace: Some("kube-system".to_string()),
            status: "Running".to_string(),
            cells: vec![
                "kube-proxy".to_string(),
                "kube-system".to_string(),
                "Running".to_string(),
                "3".to_string(),
                "3".to_string(),
                "3".to_string(),
                "30d".to_string(),
            ],
            details: vec![("Namespace".to_string(), "kube-system".to_string())],
            labels: vec![("k8s-app".to_string(), "kube-proxy".to_string())],
        }],
        ResourceKind::StatefulSet => vec![ResourceRow {
            kind,
            name: "postgres-primary".to_string(),
            namespace: Some("database".to_string()),
            status: "Running".to_string(),
            cells: vec![
                "postgres-primary".to_string(),
                "database".to_string(),
                "Running".to_string(),
                "1/1".to_string(),
                "21d".to_string(),
                "postgres:16-alpine".to_string(),
            ],
            details: vec![("Namespace".to_string(), "database".to_string())],
            labels: vec![("app".to_string(), "postgres".to_string())],
        }],
        ResourceKind::Node => vec![ResourceRow {
            kind,
            name: "orb-control-plane".to_string(),
            namespace: None,
            status: "Ready".to_string(),
            cells: vec![
                "orb-control-plane".to_string(),
                "Ready".to_string(),
                "control-plane".to_string(),
                "v1.31.2".to_string(),
                "172.18.0.2".to_string(),
                "280m / 4".to_string(),
                "1.8Gi / 8Gi".to_string(),
                "30d".to_string(),
            ],
            details: vec![
                ("Version".to_string(), "v1.31.2".to_string()),
                ("Internal IP".to_string(), "172.18.0.2".to_string()),
            ],
            labels: vec![("kubernetes.io/os".to_string(), "linux".to_string())],
        }],
        ResourceKind::Service => vec![ResourceRow {
            kind,
            name: "orbit-api".to_string(),
            namespace: Some("production".to_string()),
            status: "Active".to_string(),
            cells: vec![
                "orbit-api".to_string(),
                "production".to_string(),
                "ClusterIP".to_string(),
                "10.96.42.18".to_string(),
                "443/TCP".to_string(),
                "14d".to_string(),
            ],
            details: vec![("Cluster IP".to_string(), "10.96.42.18".to_string())],
            labels: vec![("app".to_string(), "orbit-api-gateway".to_string())],
        }],
        ResourceKind::Ingress => vec![ResourceRow {
            kind,
            name: "orbit-gateway-ingress".to_string(),
            namespace: Some("production".to_string()),
            status: "Active".to_string(),
            cells: vec![
                "orbit-gateway-ingress".to_string(),
                "production".to_string(),
                "nginx".to_string(),
                "api.orbit.local".to_string(),
                "192.168.64.2".to_string(),
                "80, 443".to_string(),
                "14d".to_string(),
            ],
            details: vec![
                ("Namespace".to_string(), "production".to_string()),
                ("Class".to_string(), "nginx".to_string()),
                ("Hosts".to_string(), "api.orbit.local".to_string()),
            ],
            labels: vec![("app".to_string(), "orbit-api-gateway".to_string())],
        }],
        ResourceKind::ConfigMap => vec![ResourceRow {
            kind,
            name: "kube-root-ca.crt".to_string(),
            namespace: Some("default".to_string()),
            status: "Active".to_string(),
            cells: vec![
                "kube-root-ca.crt".to_string(),
                "default".to_string(),
                "1".to_string(),
                "1.2 KB".to_string(),
                "30d".to_string(),
            ],
            details: vec![("Keys".to_string(), "1".to_string())],
            labels: vec![],
        }],
        ResourceKind::Secret => vec![ResourceRow {
            kind,
            name: "orbit-tls-cert".to_string(),
            namespace: Some("production".to_string()),
            status: "Active".to_string(),
            cells: vec![
                "orbit-tls-cert".to_string(),
                "production".to_string(),
                "kubernetes.io/tls".to_string(),
                "2".to_string(),
                "14d".to_string(),
            ],
            details: vec![("Type".to_string(), "kubernetes.io/tls".to_string())],
            labels: vec![],
        }],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::kubernetes::models::{DaemonSetReplicas, IngressInfo, NodeAddress, Replicas};
    use std::collections::BTreeMap;

    #[test]
    fn test_columns_match_cell_counts_for_all_supported_kinds() {
        assert!(SUPPORTED_RESOURCE_KINDS.contains(&ResourceKind::Ingress));
        for kind in SUPPORTED_RESOURCE_KINDS {
            let cols = columns_for_kind(kind);
            let rows = mock_rows_for_kind(kind);
            assert!(
                !rows.is_empty(),
                "mock_rows_for_kind({:?}) should not be empty",
                kind
            );
            for row in rows {
                assert_eq!(
                    row.cells.len(),
                    cols.len(),
                    "mock row cell count mismatch for {:?}",
                    kind
                );
            }
        }

        let pod = PodInfo {
            name: "p1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            age: "1d".to_string(),
            ..Default::default()
        };
        assert_eq!(
            ResourceRow::from(&pod).cells.len(),
            columns_for_kind(ResourceKind::Pod).len()
        );

        let dep = DeploymentInfo {
            name: "d1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            replicas: Replicas {
                current: 2,
                desired: 2,
            },
            available: 2,
            up_to_date: 2,
            age: "1d".to_string(),
            ..Default::default()
        };
        assert_eq!(
            ResourceRow::from(&dep).cells.len(),
            columns_for_kind(ResourceKind::Deployment).len()
        );

        let ds = DaemonSetInfo {
            kind: "DaemonSet".to_string(),
            name: "ds1".to_string(),
            namespace: "kube-system".to_string(),
            status: "Running".to_string(),
            replicas: DaemonSetReplicas {
                desired: 3,
                current: 3,
                ready: 3,
                up_to_date: 3,
                available: 3,
            },
            age: "5d".to_string(),
            images: vec![],
            containers: vec![],
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
        };
        assert_eq!(
            ResourceRow::from(&ds).cells.len(),
            columns_for_kind(ResourceKind::DaemonSet).len()
        );

        let sts = StatefulSetInfo {
            kind: "StatefulSet".to_string(),
            name: "sts1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            replicas: Replicas {
                current: 1,
                desired: 1,
            },
            age: "3d".to_string(),
            images: vec!["redis:7".to_string()],
            containers: vec![],
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
        };
        assert_eq!(
            ResourceRow::from(&sts).cells.len(),
            columns_for_kind(ResourceKind::StatefulSet).len()
        );

        let node = NodeInfo {
            name: "node-1".to_string(),
            status: "Ready".to_string(),
            role: "worker".to_string(),
            version: "v1.31.0".to_string(),
            cpu_used: "100m".to_string(),
            cpu_total: "4".to_string(),
            mem_used: "1Gi".to_string(),
            mem_total: "8Gi".to_string(),
            uptime: "10d".to_string(),
            addresses: vec![NodeAddress {
                type_: "InternalIP".to_string(),
                address: "10.0.0.5".to_string(),
            }],
            ..Default::default()
        };
        let node_row = ResourceRow::from(&node);
        assert_eq!(
            node_row.cells.len(),
            columns_for_kind(ResourceKind::Node).len()
        );
        assert!(node_row.cells.contains(&"10.0.0.5".to_string()));

        let svc = ServiceInfo {
            name: "svc1".to_string(),
            namespace: "default".to_string(),
            r#type: "ClusterIP".to_string(),
            cluster_ip: "10.96.0.1".to_string(),
            external_ip: "-".to_string(),
            ports: "80/TCP".to_string(),
            endpoints: "10.0.0.1:80".to_string(),
            age: "2d".to_string(),
            session_affinity: "None".to_string(),
            internal_traffic_policy: None,
            created: "".to_string(),
            uid: "".to_string(),
            selector: BTreeMap::new(),
            labels: BTreeMap::new(),
            ports_list: vec![],
            endpoints_list: vec![],
            events: vec![],
        };
        assert_eq!(
            ResourceRow::from(&svc).cells.len(),
            columns_for_kind(ResourceKind::Service).len()
        );

        let cm = ConfigMapInfo {
            name: "cm1".to_string(),
            namespace: "default".to_string(),
            labels: BTreeMap::new(),
            annotations: 0,
            created: "".to_string(),
            age: "2d".to_string(),
            resource_version: "1".to_string(),
            immutable: false,
            keys_count: 3,
            size: "512 B".to_string(),
            mounted_pods: 1,
            used_by: vec![],
            data: BTreeMap::new(),
        };
        assert_eq!(
            ResourceRow::from(&cm).cells.len(),
            columns_for_kind(ResourceKind::ConfigMap).len()
        );

        let sec = SecretInfo {
            name: "sec1".to_string(),
            namespace: "default".to_string(),
            labels: BTreeMap::new(),
            annotations: 0,
            r#type: "Opaque".to_string(),
            created: "".to_string(),
            age: "2d".to_string(),
            resource_version: "1".to_string(),
            immutable: false,
            keys_count: 2,
            size: "256 B".to_string(),
            mounted_pods: 1,
            used_by: vec![],
            data: BTreeMap::new(),
        };
        assert_eq!(
            ResourceRow::from(&sec).cells.len(),
            columns_for_kind(ResourceKind::Secret).len()
        );

        let ing = IngressInfo {
            name: "ing1".to_string(),
            namespace: "default".to_string(),
            class_name: Some("nginx".to_string()),
            hosts: "example.com".to_string(),
            address: "10.0.0.10".to_string(),
            ports: "80, 443".to_string(),
            age: "2d".to_string(),
            created: "".to_string(),
            uid: "".to_string(),
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            rules_summary: vec![],
        };
        let ing_cols = columns_for_kind(ResourceKind::Ingress);
        assert_eq!(ing_cols.len(), 7);
        assert_eq!(ResourceRow::from(&ing).cells.len(), ing_cols.len());
    }
}
