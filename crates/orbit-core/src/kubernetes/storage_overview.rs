use std::collections::{BTreeMap, HashMap};

use kube::Client;
use serde::{Deserialize, Serialize};
use tokio::join;

use crate::kubernetes::models::{
    PersistentVolumeClaimInfo, PersistentVolumeInfo, StorageClassInfo,
};
use crate::kubernetes::nodes::parse_memory_quantity;
use crate::kubernetes::storage::{fetch_pvcs, fetch_pvs, fetch_storage_classes};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageResourceSnapshot<T> {
    pub items: Vec<T>,
    pub error: Option<String>,
}

impl<T> Default for StorageResourceSnapshot<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StorageOverviewSnapshot {
    pub persistent_volumes: StorageResourceSnapshot<PersistentVolumeInfo>,
    pub persistent_volume_claims: StorageResourceSnapshot<PersistentVolumeClaimInfo>,
    pub storage_classes: StorageResourceSnapshot<StorageClassInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PvStatusCounts {
    pub total: usize,
    pub bound: usize,
    pub available: usize,
    pub released: usize,
    pub failed: usize,
    pub other: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PvcStatusCounts {
    pub total: usize,
    pub bound: usize,
    pub pending: usize,
    pub lost: usize,
    pub other: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StorageCapacitySummary {
    pub total_gib: f64,
    pub bound_gib: f64,
    pub available_gib: f64,
    pub released_gib: f64,
    pub failed_gib: f64,
    pub bound_pct: u8,
    pub available_pct: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageClassCapacityEntry {
    pub name: String,
    pub volume_count: usize,
    pub total_gib: f64,
    pub share_pct: u8,
    pub provisioner: Option<String>,
    pub allow_volume_expansion: Option<bool>,
}

fn parse_storage_capacity_gib(raw: &str) -> f64 {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return 0.0;
    }
    let parsed = parse_memory_quantity(trimmed);
    if parsed.is_finite() && parsed > 0.0 {
        parsed
    } else {
        0.0
    }
}

fn ratio_pct(part: f64, total: f64) -> u8 {
    if !part.is_finite() || !total.is_finite() || total <= 0.0 || part <= 0.0 {
        return 0;
    }
    ((part / total) * 100.0).round().clamp(0.0, 100.0) as u8
}

/// Formats a capacity value in GiB into human-readable MiB, GiB, or TiB units.
pub fn format_storage_gib(gib: f64) -> String {
    if !gib.is_finite() || gib <= 0.0 {
        return "0.0 GiB".to_string();
    }
    if gib < 1.0 {
        let mib = (gib * 1024.0).round();
        return format!("{mib:.0} MiB");
    }
    if gib >= 1024.0 {
        let tib = gib / 1024.0;
        return format!("{tib:.1} TiB");
    }
    format!("{gib:.1} GiB")
}

impl StorageOverviewSnapshot {
    pub fn pv_status_counts(&self) -> PvStatusCounts {
        let mut counts = PvStatusCounts {
            total: self.persistent_volumes.items.len(),
            ..PvStatusCounts::default()
        };
        for pv in &self.persistent_volumes.items {
            if pv.status.eq_ignore_ascii_case("Bound") {
                counts.bound += 1;
            } else if pv.status.eq_ignore_ascii_case("Available") {
                counts.available += 1;
            } else if pv.status.eq_ignore_ascii_case("Released") {
                counts.released += 1;
            } else if pv.status.eq_ignore_ascii_case("Failed") {
                counts.failed += 1;
            } else {
                counts.other += 1;
            }
        }
        counts
    }

    pub fn pvc_status_counts_for_namespaces(&self, selected: &[String]) -> PvcStatusCounts {
        let mut counts = PvcStatusCounts::default();
        for pvc in &self.persistent_volume_claims.items {
            if !selected.is_empty() && !selected.iter().any(|ns| ns == &pvc.namespace) {
                continue;
            }
            counts.total += 1;
            if pvc.status.eq_ignore_ascii_case("Bound") {
                counts.bound += 1;
            } else if pvc.status.eq_ignore_ascii_case("Pending") {
                counts.pending += 1;
            } else if pvc.status.eq_ignore_ascii_case("Lost") {
                counts.lost += 1;
            } else {
                counts.other += 1;
            }
        }
        counts
    }

    pub fn capacity_summary(&self) -> StorageCapacitySummary {
        let mut summary = StorageCapacitySummary::default();
        for pv in &self.persistent_volumes.items {
            let gib = parse_storage_capacity_gib(&pv.capacity);
            summary.total_gib += gib;
            if pv.status.eq_ignore_ascii_case("Bound") {
                summary.bound_gib += gib;
            } else if pv.status.eq_ignore_ascii_case("Available") {
                summary.available_gib += gib;
            } else if pv.status.eq_ignore_ascii_case("Released") {
                summary.released_gib += gib;
            } else if pv.status.eq_ignore_ascii_case("Failed") {
                summary.failed_gib += gib;
            }
        }
        summary.bound_pct = ratio_pct(summary.bound_gib, summary.total_gib);
        summary.available_pct = ratio_pct(summary.available_gib, summary.total_gib);
        summary
    }

    pub fn storage_class_capacity_breakdown(&self) -> Vec<StorageClassCapacityEntry> {
        let sc_meta: HashMap<&str, &StorageClassInfo> = self
            .storage_classes
            .items
            .iter()
            .map(|sc| (sc.name.as_str(), sc))
            .collect();

        let mut grouped: BTreeMap<String, (usize, f64)> = BTreeMap::new();
        for sc in &self.storage_classes.items {
            grouped.entry(sc.name.clone()).or_insert((0, 0.0));
        }

        let mut total_gib = 0.0_f64;
        for pv in &self.persistent_volumes.items {
            let class_name = {
                let trimmed = pv.storage_class.trim();
                if trimmed.is_empty() || trimmed == "-" {
                    "Unassigned".to_string()
                } else {
                    trimmed.to_string()
                }
            };
            let gib = parse_storage_capacity_gib(&pv.capacity);
            total_gib += gib;
            let entry = grouped.entry(class_name).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += gib;
        }

        let mut entries: Vec<StorageClassCapacityEntry> = grouped
            .into_iter()
            .map(|(name, (volume_count, class_gib))| {
                let meta = sc_meta.get(name.as_str()).copied();
                StorageClassCapacityEntry {
                    name,
                    volume_count,
                    total_gib: class_gib,
                    share_pct: ratio_pct(class_gib, total_gib),
                    provisioner: meta.map(|sc| sc.provisioner.clone()),
                    allow_volume_expansion: meta.map(|sc| sc.allow_volume_expansion),
                }
            })
            .collect();

        entries.sort_by(|a, b| {
            b.total_gib
                .total_cmp(&a.total_gib)
                .then_with(|| b.volume_count.cmp(&a.volume_count))
                .then_with(|| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
        });

        entries
    }

    pub fn storage_classes_expandable_count(&self) -> usize {
        self.storage_classes
            .items
            .iter()
            .filter(|sc| sc.allow_volume_expansion)
            .count()
    }

    pub fn has_any_error(&self) -> bool {
        self.persistent_volumes.error.is_some()
            || self.persistent_volume_claims.error.is_some()
            || self.storage_classes.error.is_some()
    }

    pub fn all_failed(&self) -> bool {
        self.persistent_volumes.error.is_some()
            && self.persistent_volume_claims.error.is_some()
            && self.storage_classes.error.is_some()
    }
}

fn into_resource_snapshot<T>(result: Result<Vec<T>, String>) -> StorageResourceSnapshot<T> {
    match result {
        Ok(items) => StorageResourceSnapshot { items, error: None },
        Err(error) => StorageResourceSnapshot {
            items: Vec::new(),
            error: Some(error),
        },
    }
}

pub fn build_storage_overview_snapshot(
    pvs: Result<Vec<PersistentVolumeInfo>, String>,
    pvcs: Result<Vec<PersistentVolumeClaimInfo>, String>,
    storage_classes: Result<Vec<StorageClassInfo>, String>,
) -> StorageOverviewSnapshot {
    StorageOverviewSnapshot {
        persistent_volumes: into_resource_snapshot(pvs),
        persistent_volume_claims: into_resource_snapshot(pvcs),
        storage_classes: into_resource_snapshot(storage_classes),
    }
}

/// Fetches PersistentVolumes, PersistentVolumeClaims (all namespaces), and StorageClasses in parallel.
pub async fn fetch_storage_overview(client: &Client) -> StorageOverviewSnapshot {
    let (pvs, pvcs, storage_classes) = join!(
        fetch_pvs(client),
        fetch_pvcs(client, None),
        fetch_storage_classes(client),
    );
    build_storage_overview_snapshot(pvs, pvcs, storage_classes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pv(name: &str, capacity: &str, status: &str, storage_class: &str) -> PersistentVolumeInfo {
        PersistentVolumeInfo {
            name: name.to_string(),
            capacity: capacity.to_string(),
            access_mode: "ReadWriteOnce".to_string(),
            reclaim_policy: "Delete".to_string(),
            status: status.to_string(),
            storage_class: storage_class.to_string(),
            age: "2d".to_string(),
            volume_mode: "Filesystem".to_string(),
            reason: None,
        }
    }

    fn sample_pvc(name: &str, namespace: &str, status: &str) -> PersistentVolumeClaimInfo {
        PersistentVolumeClaimInfo {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: status.to_string(),
            volume: "pv-1".to_string(),
            capacity: "10Gi".to_string(),
            access_mode: "ReadWriteOnce".to_string(),
            storage_class: "gp3".to_string(),
            age: "1d".to_string(),
        }
    }

    fn sample_sc(name: &str, provisioner: &str, expandable: bool) -> StorageClassInfo {
        StorageClassInfo {
            name: name.to_string(),
            provisioner: provisioner.to_string(),
            reclaim_policy: "Delete".to_string(),
            volume_binding_mode: "WaitForFirstConsumer".to_string(),
            allow_volume_expansion: expandable,
            age: "30d".to_string(),
        }
    }

    #[test]
    fn test_format_storage_gib_formats_zero_mib_gib_and_tib() {
        assert_eq!(format_storage_gib(0.0), "0.0 GiB");
        assert_eq!(format_storage_gib(0.5), "512 MiB");
        assert_eq!(format_storage_gib(12.5), "12.5 GiB");
        assert_eq!(format_storage_gib(2048.0), "2.0 TiB");
        assert_eq!(format_storage_gib(-5.0), "0.0 GiB");
        assert_eq!(format_storage_gib(f64::NAN), "0.0 GiB");
    }

    #[test]
    fn test_storage_overview_snapshot_aggregates_pv_status_and_capacity() {
        let snapshot = build_storage_overview_snapshot(
            Ok(vec![
                sample_pv("pv-bound", "10Gi", "Bound", "gp3"),
                sample_pv("pv-avail", "20Gi", "Available", "gp3"),
                sample_pv("pv-rel", "5Gi", "Released", "standard"),
                sample_pv("pv-fail", "5Gi", "Failed", "standard"),
            ]),
            Ok(vec![]),
            Ok(vec![]),
        );

        let counts = snapshot.pv_status_counts();
        assert_eq!(counts.total, 4);
        assert_eq!(counts.bound, 1);
        assert_eq!(counts.available, 1);
        assert_eq!(counts.released, 1);
        assert_eq!(counts.failed, 1);
        assert_eq!(counts.other, 0);

        let cap = snapshot.capacity_summary();
        assert!((cap.total_gib - 40.0).abs() < 1e-6);
        assert!((cap.bound_gib - 10.0).abs() < 1e-6);
        assert!((cap.available_gib - 20.0).abs() < 1e-6);
        assert!((cap.released_gib - 5.0).abs() < 1e-6);
        assert!((cap.failed_gib - 5.0).abs() < 1e-6);
        assert_eq!(cap.bound_pct, 25);
        assert_eq!(cap.available_pct, 50);
    }

    #[test]
    fn test_storage_overview_pvc_counts_respect_namespace_filter() {
        let snapshot = build_storage_overview_snapshot(
            Ok(vec![]),
            Ok(vec![
                sample_pvc("pvc-1", "default", "Bound"),
                sample_pvc("pvc-2", "prod", "Bound"),
                sample_pvc("pvc-3", "prod", "Pending"),
                sample_pvc("pvc-4", "staging", "Lost"),
            ]),
            Ok(vec![]),
        );

        let all = snapshot.pvc_status_counts_for_namespaces(&[]);
        assert_eq!(all.total, 4);
        assert_eq!(all.bound, 2);
        assert_eq!(all.pending, 1);
        assert_eq!(all.lost, 1);

        let prod_only = snapshot.pvc_status_counts_for_namespaces(&["prod".to_string()]);
        assert_eq!(prod_only.total, 2);
        assert_eq!(prod_only.bound, 1);
        assert_eq!(prod_only.pending, 1);
        assert_eq!(prod_only.lost, 0);

        let prod_and_staging = snapshot
            .pvc_status_counts_for_namespaces(&["prod".to_string(), "staging".to_string()]);
        assert_eq!(prod_and_staging.total, 3);
        assert_eq!(prod_and_staging.bound, 1);
        assert_eq!(prod_and_staging.pending, 1);
        assert_eq!(prod_and_staging.lost, 1);
    }

    #[test]
    fn test_storage_class_capacity_breakdown_groups_by_class_includes_unassigned_and_sorts() {
        let snapshot = build_storage_overview_snapshot(
            Ok(vec![
                sample_pv("pv-1", "10Gi", "Bound", "gp3"),
                sample_pv("pv-2", "20Gi", "Bound", "gp3"),
                sample_pv("pv-3", "10Gi", "Available", "standard"),
                sample_pv("pv-4", "5Gi", "Available", "-"),
            ]),
            Ok(vec![]),
            Ok(vec![
                sample_sc("gp3", "ebs.csi.aws.com", true),
                sample_sc("standard", "kubernetes.io/aws-ebs", false),
                sample_sc("empty-sc", "efs.csi.aws.com", true),
            ]),
        );

        assert_eq!(snapshot.storage_classes_expandable_count(), 2);

        let breakdown = snapshot.storage_class_capacity_breakdown();
        let names: Vec<&str> = breakdown.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["gp3", "standard", "Unassigned", "empty-sc"]);

        assert_eq!(breakdown[0].volume_count, 2);
        assert!((breakdown[0].total_gib - 30.0).abs() < 1e-6);
        assert_eq!(breakdown[0].share_pct, 67);
        assert_eq!(breakdown[0].provisioner.as_deref(), Some("ebs.csi.aws.com"));
        assert_eq!(breakdown[0].allow_volume_expansion, Some(true));

        assert_eq!(breakdown[2].name, "Unassigned");
        assert_eq!(breakdown[2].volume_count, 1);
        assert_eq!(breakdown[2].provisioner, None);
        assert_eq!(breakdown[2].allow_volume_expansion, None);

        assert_eq!(breakdown[3].name, "empty-sc");
        assert_eq!(breakdown[3].volume_count, 0);
        assert_eq!(breakdown[3].share_pct, 0);
    }

    #[test]
    fn test_storage_overview_partial_and_full_error_flags() {
        let clean = build_storage_overview_snapshot(Ok(vec![]), Ok(vec![]), Ok(vec![]));
        assert!(!clean.has_any_error());
        assert!(!clean.all_failed());

        let partial = build_storage_overview_snapshot(
            Err("forbidden".to_string()),
            Ok(vec![]),
            Ok(vec![]),
        );
        assert!(partial.has_any_error());
        assert!(!partial.all_failed());

        let all_err = build_storage_overview_snapshot(
            Err("pv err".to_string()),
            Err("pvc err".to_string()),
            Err("sc err".to_string()),
        );
        assert!(all_err.has_any_error());
        assert!(all_err.all_failed());
    }
}
