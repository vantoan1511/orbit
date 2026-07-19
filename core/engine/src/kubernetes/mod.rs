pub mod models;
pub mod manager;
pub mod configmaps;
pub mod secrets;
pub mod storage;
pub mod workloads;
pub mod batch;
pub mod nodes;
pub mod services;
pub mod namespaces;
pub mod events;
pub mod policies;
pub mod watchers;
pub mod metrics;
pub mod logs;

pub use configmaps::list_configmaps;
pub use secrets::list_secrets;
pub use storage::{list_pvs, list_pvcs, list_storage_classes};
pub use workloads::{list_pods, list_deployments, list_statefulsets, list_daemonsets, list_replicasets};
pub use batch::{list_jobs, list_cronjobs};
pub use nodes::list_nodes;
pub use namespaces::list_namespaces;
pub use services::list_services;
pub use events::list_events;
pub use policies::*;
pub use logs::{stream_pod_logs, get_workload_pods};


/// Formats a Kubernetes creation timestamp into a human-readable age string.
pub(crate) fn format_age(creation_timestamp: &Option<k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> String {
    if let Some(creation) = creation_timestamp {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(creation.0);
        let seconds = duration.num_seconds();
        if seconds < 0 {
            "0s".to_string()
        } else if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m", duration.num_minutes())
        } else if seconds < 86400 {
            format!("{}h", duration.num_hours())
        } else {
            format!("{}d", duration.num_days())
        }
    } else {
        "Unknown".to_string()
    }
}

/// Formats a byte count into a human-readable size string.
pub(crate) fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    }
}
