use std::time::Duration;

use k8s_openapi::api::core::v1::PersistentVolume;
use kube::{
    api::{Api, ListParams},
    Client,
};

use crate::kubernetes::{models, namespaces::format_age};

pub const STORAGE_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Formats a Kubernetes `PersistentVolume` into `models::PersistentVolumeInfo`.
pub fn format_pv(pv: &PersistentVolume) -> models::PersistentVolumeInfo {
    let name = pv.metadata.name.clone().unwrap_or_default();
    let age = format_age(&pv.metadata.creation_timestamp);

    let capacity = pv
        .spec
        .as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|cap| cap.get("storage"))
        .map(|q| q.0.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "-".to_string());

    let access_mode = pv
        .spec
        .as_ref()
        .and_then(|s| s.access_modes.as_ref())
        .filter(|modes| !modes.is_empty())
        .map(|modes| modes.join(", "))
        .unwrap_or_else(|| "-".to_string());

    let reclaim_policy = pv
        .spec
        .as_ref()
        .and_then(|s| s.persistent_volume_reclaim_policy.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "-".to_string());

    let status = pv
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    let storage_class = pv
        .spec
        .as_ref()
        .and_then(|s| s.storage_class_name.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "-".to_string());

    let volume_mode = pv
        .spec
        .as_ref()
        .and_then(|s| s.volume_mode.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "Filesystem".to_string());

    let reason = pv
        .status
        .as_ref()
        .and_then(|s| s.reason.clone())
        .filter(|v| !v.trim().is_empty());

    models::PersistentVolumeInfo {
        name,
        capacity,
        access_mode,
        reclaim_policy,
        status,
        storage_class,
        age,
        volume_mode,
        reason,
    }
}

/// Lists all PersistentVolumes in the cluster.
pub async fn list_pvs(client: &Client) -> Result<Vec<models::PersistentVolumeInfo>, kube::Error> {
    let api: Api<PersistentVolume> = Api::all(client.clone());
    let mut list = Vec::new();
    for pv in api.list(&ListParams::default()).await? {
        list.push(format_pv(&pv));
    }
    Ok(list)
}

/// Fetches all PersistentVolumes with a bounded timeout and human-readable error mapping.
pub async fn fetch_pvs(client: &Client) -> Result<Vec<models::PersistentVolumeInfo>, String> {
    match tokio::time::timeout(STORAGE_FETCH_TIMEOUT, list_pvs(client)).await {
        Ok(Ok(list)) => Ok(list),
        Ok(Err(kube::Error::Api(api_err))) => Err(api_err.message),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("Timed out fetching persistent volumes after 10s".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use k8s_openapi::api::core::v1::{PersistentVolumeSpec, PersistentVolumeStatus};
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use std::collections::BTreeMap;

    #[test]
    fn test_format_pv_extracts_capacity_access_modes_reclaim_status_class_and_reason() {
        let mut capacity = BTreeMap::new();
        capacity.insert("storage".to_string(), Quantity("10Gi".to_string()));

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let pv = PersistentVolume {
            metadata: ObjectMeta {
                name: Some("pv-data-01".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                ..Default::default()
            },
            spec: Some(PersistentVolumeSpec {
                capacity: Some(capacity),
                access_modes: Some(vec![
                    "ReadWriteOnce".to_string(),
                    "ReadOnlyMany".to_string(),
                ]),
                persistent_volume_reclaim_policy: Some("Retain".to_string()),
                storage_class_name: Some("gp3".to_string()),
                volume_mode: Some("Block".to_string()),
                ..Default::default()
            }),
            status: Some(PersistentVolumeStatus {
                phase: Some("Failed".to_string()),
                reason: Some("VolumeLost".to_string()),
                ..Default::default()
            }),
        };

        let info = format_pv(&pv);
        assert_eq!(info.name, "pv-data-01");
        assert_eq!(info.capacity, "10Gi");
        assert_eq!(info.access_mode, "ReadWriteOnce, ReadOnlyMany");
        assert_eq!(info.reclaim_policy, "Retain");
        assert_eq!(info.status, "Failed");
        assert_eq!(info.storage_class, "gp3");
        assert_eq!(info.volume_mode, "Block");
        assert_eq!(info.reason, Some("VolumeLost".to_string()));
        assert!(!info.age.is_empty());
    }

    #[test]
    fn test_format_pv_defaults_missing_spec_and_status_fields() {
        let pv = PersistentVolume {
            metadata: ObjectMeta {
                name: Some("empty-pv".to_string()),
                ..Default::default()
            },
            spec: None,
            status: None,
        };

        let info = format_pv(&pv);
        assert_eq!(info.name, "empty-pv");
        assert_eq!(info.capacity, "-");
        assert_eq!(info.access_mode, "-");
        assert_eq!(info.reclaim_policy, "-");
        assert_eq!(info.status, "Unknown");
        assert_eq!(info.storage_class, "-");
        assert_eq!(info.volume_mode, "Filesystem");
        assert_eq!(info.reason, None);
    }
}
