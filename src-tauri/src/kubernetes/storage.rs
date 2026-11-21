use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::storage::v1::StorageClass;
use crate::kubernetes::models::StorageClassInfo;
use crate::kubernetes::format_age;

pub use orbit_core::kubernetes::storage::{format_pv, format_pvc, list_pvcs, list_pvs};

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
    use k8s_openapi::api::core::v1::{PersistentVolume, PersistentVolumeClaim, PersistentVolumeSpec, PersistentVolumeStatus, PersistentVolumeClaimSpec, PersistentVolumeClaimStatus, VolumeResourceRequirements};
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
