use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{PersistentVolume, PersistentVolumeClaim};
use k8s_openapi::api::storage::v1::StorageClass;
use crate::kubernetes::models::{PersistentVolumeInfo, PersistentVolumeClaimInfo, StorageClassInfo};
use crate::kubernetes::format_age;

/// Formats a Kubernetes `PersistentVolume` into `PersistentVolumeInfo`.
pub fn format_pv(pv: &PersistentVolume) -> PersistentVolumeInfo {
    let name = pv.metadata.name.clone().unwrap_or_default();
    let age = format_age(&pv.metadata.creation_timestamp);

    let capacity = pv.spec.as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|cap| cap.get("storage"))
        .map(|q| q.0.clone())
        .unwrap_or_else(|| "-".to_string());

    let access_mode = pv.spec.as_ref()
        .and_then(|s| s.access_modes.as_ref())
        .map(|modes| modes.join(", "))
        .unwrap_or_else(|| "-".to_string());

    let reclaim_policy = pv.spec.as_ref()
        .and_then(|s| s.persistent_volume_reclaim_policy.clone())
        .unwrap_or_else(|| "-".to_string());

    let status = pv.status.as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let storage_class = pv.spec.as_ref()
        .and_then(|s| s.storage_class_name.clone())
        .unwrap_or_else(|| "-".to_string());

    let volume_mode = pv.spec.as_ref()
        .and_then(|s| s.volume_mode.clone())
        .unwrap_or_else(|| "Filesystem".to_string());

    let reason = pv.status.as_ref()
        .and_then(|s| s.reason.clone());

    PersistentVolumeInfo {
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

/// Formats a Kubernetes `PersistentVolumeClaim` into `PersistentVolumeClaimInfo`.
pub fn format_pvc(pvc: &PersistentVolumeClaim) -> PersistentVolumeClaimInfo {
    let name = pvc.metadata.name.clone().unwrap_or_default();
    let namespace = pvc.metadata.namespace.clone().unwrap_or_default();
    let age = format_age(&pvc.metadata.creation_timestamp);

    let status = pvc.status.as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let volume = pvc.spec.as_ref()
        .and_then(|s| s.volume_name.clone())
        .unwrap_or_else(|| "-".to_string());

    let capacity = pvc.status.as_ref()
        .and_then(|s| s.capacity.as_ref())
        .and_then(|cap| cap.get("storage"))
        .map(|q| q.0.clone())
        .or_else(|| {
            pvc.spec.as_ref()
                .and_then(|s| s.resources.as_ref())
                .and_then(|r| r.requests.as_ref())
                .and_then(|req| req.get("storage"))
                .map(|q| q.0.clone())
        })
        .unwrap_or_else(|| "-".to_string());

    let access_mode = pvc.spec.as_ref()
        .and_then(|s| s.access_modes.as_ref())
        .map(|modes| modes.join(", "))
        .unwrap_or_else(|| "-".to_string());

    let storage_class = pvc.spec.as_ref()
        .and_then(|s| s.storage_class_name.clone())
        .unwrap_or_else(|| "-".to_string());

    PersistentVolumeClaimInfo {
        name,
        namespace,
        status,
        volume,
        capacity,
        access_mode,
        storage_class,
        age,
    }
}

/// Formats a Kubernetes `StorageClass` into `StorageClassInfo`.
pub fn format_storage_class(sc: &StorageClass) -> StorageClassInfo {
    let name = sc.metadata.name.clone().unwrap_or_default();
    let age = format_age(&sc.metadata.creation_timestamp);
    let provisioner = sc.provisioner.clone();
    
    let reclaim_policy = sc.reclaim_policy.clone()
        .unwrap_or_else(|| "Delete".to_string());

    let volume_binding_mode = sc.volume_binding_mode.clone()
        .unwrap_or_else(|| "Immediate".to_string());

    let allow_volume_expansion = sc.allow_volume_expansion.unwrap_or(false);

    StorageClassInfo {
        name,
        provisioner,
        reclaim_policy,
        volume_binding_mode,
        allow_volume_expansion,
        age,
    }
}

/// Lists all PersistentVolumes in the cluster.
pub async fn list_pvs(client: &Client) -> Result<Vec<PersistentVolumeInfo>, kube::Error> {
    let api: Api<PersistentVolume> = Api::all(client.clone());
    let mut list = Vec::new();
    for pv in api.list(&ListParams::default()).await? {
        list.push(format_pv(&pv));
    }
    Ok(list)
}

/// Lists all PersistentVolumeClaims in the specified namespace or all namespaces.
pub async fn list_pvcs(client: &Client, namespace: Option<String>) -> Result<Vec<PersistentVolumeClaimInfo>, kube::Error> {
    let api: Api<PersistentVolumeClaim> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    let mut list = Vec::new();
    for pvc in api.list(&ListParams::default()).await? {
        list.push(format_pvc(&pvc));
    }
    Ok(list)
}

/// Lists all StorageClasses in the cluster.
pub async fn list_storage_classes(client: &Client) -> Result<Vec<StorageClassInfo>, kube::Error> {
    let api: Api<StorageClass> = Api::all(client.clone());
    let mut list = Vec::new();
    for sc in api.list(&ListParams::default()).await? {
        list.push(format_storage_class(&sc));
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
    use k8s_openapi::api::core::v1::{PersistentVolumeSpec, PersistentVolumeStatus, PersistentVolumeClaimSpec, PersistentVolumeClaimStatus, VolumeResourceRequirements};
    use std::collections::BTreeMap;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_pv() {
        let mut capacity = BTreeMap::new();
        capacity.insert("storage".to_string(), Quantity("10Gi".to_string()));

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let pv = PersistentVolume {
            metadata: ObjectMeta {
                name: Some("test-pv".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                ..Default::default()
            },
            spec: Some(PersistentVolumeSpec {
                capacity: Some(capacity),
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                persistent_volume_reclaim_policy: Some("Retain".to_string()),
                storage_class_name: Some("standard".to_string()),
                volume_mode: Some("Filesystem".to_string()),
                ..Default::default()
            }),
            status: Some(PersistentVolumeStatus {
                phase: Some("Bound".to_string()),
                ..Default::default()
            }),
        };

        let info = format_pv(&pv);
        assert_eq!(info.name, "test-pv");
        assert_eq!(info.capacity, "10Gi");
        assert_eq!(info.access_mode, "ReadWriteOnce");
        assert_eq!(info.reclaim_policy, "Retain");
        assert_eq!(info.status, "Bound");
        assert_eq!(info.storage_class, "standard");
        assert_eq!(info.volume_mode, "Filesystem");
    }

    #[test]
    fn test_format_pvc() {
        let mut requests = BTreeMap::new();
        requests.insert("storage".to_string(), Quantity("5Gi".to_string()));

        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let pvc = PersistentVolumeClaim {
            metadata: ObjectMeta {
                name: Some("test-pvc".to_string()),
                namespace: Some("default".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                ..Default::default()
            },
            spec: Some(PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                storage_class_name: Some("standard".to_string()),
                volume_name: Some("test-pv".to_string()),
                resources: Some(VolumeResourceRequirements {
                    requests: Some(requests),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            status: Some(PersistentVolumeClaimStatus {
                phase: Some("Bound".to_string()),
                ..Default::default()
            }),
        };

        let info = format_pvc(&pvc);
        assert_eq!(info.name, "test-pvc");
        assert_eq!(info.namespace, "default");
        assert_eq!(info.status, "Bound");
        assert_eq!(info.volume, "test-pv");
        assert_eq!(info.capacity, "5Gi");
        assert_eq!(info.access_mode, "ReadWriteOnce");
        assert_eq!(info.storage_class, "standard");
    }

    #[test]
    fn test_format_storage_class() {
        let creation_time = Utc.with_ymd_and_hms(2026, 7, 12, 12, 0, 0).unwrap();

        let sc = StorageClass {
            metadata: ObjectMeta {
                name: Some("standard".to_string()),
                creation_timestamp: Some(Time(creation_time)),
                ..Default::default()
            },
            provisioner: "kubernetes.io/aws-ebs".to_string(),
            reclaim_policy: Some("Delete".to_string()),
            volume_binding_mode: Some("Immediate".to_string()),
            allow_volume_expansion: Some(true),
            ..Default::default()
        };

        let info = format_storage_class(&sc);
        assert_eq!(info.name, "standard");
        assert_eq!(info.provisioner, "kubernetes.io/aws-ebs");
        assert_eq!(info.reclaim_policy, "Delete");
        assert_eq!(info.volume_binding_mode, "Immediate");
        assert!(info.allow_volume_expansion);
    }
}
