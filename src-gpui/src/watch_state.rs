use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use orbit_core::kubernetes::models::{
    ConfigMapInfo, CronJobInfo, DaemonSetInfo, DeploymentInfo, EventInfo, HpaInfo, IngressInfo,
    JobInfo, NamespaceInfo, NodeInfo, PersistentVolumeClaimInfo, PersistentVolumeInfo, PodInfo,
    PodMetricItem, PolicyInfo, ReplicaSetInfo, SecretInfo, ServiceInfo, StatefulSetInfo,
    StorageClassInfo,
};
use orbit_core::kubernetes::services::ServiceEndpointsSnapshot;
use orbit_core::kubernetes::watchers::{WatchAction, WatchUpdate};

use crate::components::data_table::SortState;
use crate::navigation::WorkspaceView;

pub const MAX_RETAINED_EVENTS: usize = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WatchResourceKind {
    Namespaces,
    Deployments,
    StatefulSets,
    DaemonSets,
    ReplicaSets,
    Jobs,
    CronJobs,
    Pods,
    Services,
    Ingresses,
    ConfigMaps,
    Secrets,
    PersistentVolumes,
    PersistentVolumeClaims,
    StorageClasses,
    Hpas,
    NetworkPolicies,
    PodSecurity,
    ResourceQuotas,
    AdmissionWebhooks,
    SecurityOverview,
    Nodes,
    Events,
}

impl WatchResourceKind {
    pub const fn from_workspace_view(view: WorkspaceView) -> Option<Self> {
        match view {
            WorkspaceView::Namespaces => Some(Self::Namespaces),
            WorkspaceView::Deployments => Some(Self::Deployments),
            WorkspaceView::StatefulSets => Some(Self::StatefulSets),
            WorkspaceView::DaemonSets => Some(Self::DaemonSets),
            WorkspaceView::ReplicaSets => Some(Self::ReplicaSets),
            WorkspaceView::Jobs => Some(Self::Jobs),
            WorkspaceView::CronJobs => Some(Self::CronJobs),
            WorkspaceView::Pods => Some(Self::Pods),
            WorkspaceView::Services => Some(Self::Services),
            WorkspaceView::Ingresses => Some(Self::Ingresses),
            WorkspaceView::ConfigMaps => Some(Self::ConfigMaps),
            WorkspaceView::Secrets => Some(Self::Secrets),
            WorkspaceView::PersistentVolumes => Some(Self::PersistentVolumes),
            WorkspaceView::PersistentVolumeClaims => Some(Self::PersistentVolumeClaims),
            WorkspaceView::StorageClasses => Some(Self::StorageClasses),
            WorkspaceView::Hpas => Some(Self::Hpas),
            WorkspaceView::NetworkPolicies => Some(Self::NetworkPolicies),
            WorkspaceView::PodSecurity => Some(Self::PodSecurity),
            WorkspaceView::ResourceQuotas => Some(Self::ResourceQuotas),
            WorkspaceView::AdmissionWebhooks => Some(Self::AdmissionWebhooks),
            WorkspaceView::SecurityOverview => Some(Self::SecurityOverview),
            WorkspaceView::Nodes => Some(Self::Nodes),
            WorkspaceView::Events => Some(Self::Events),
            WorkspaceView::Overview
            | WorkspaceView::WorkloadsOverview
            | WorkspaceView::StorageOverview
            | WorkspaceView::PortForward
            | WorkspaceView::Settings => None,
        }
    }

    pub const fn workspace_view(self) -> WorkspaceView {
        match self {
            Self::Namespaces => WorkspaceView::Namespaces,
            Self::Deployments => WorkspaceView::Deployments,
            Self::StatefulSets => WorkspaceView::StatefulSets,
            Self::DaemonSets => WorkspaceView::DaemonSets,
            Self::ReplicaSets => WorkspaceView::ReplicaSets,
            Self::Jobs => WorkspaceView::Jobs,
            Self::CronJobs => WorkspaceView::CronJobs,
            Self::Pods => WorkspaceView::Pods,
            Self::Services => WorkspaceView::Services,
            Self::Ingresses => WorkspaceView::Ingresses,
            Self::ConfigMaps => WorkspaceView::ConfigMaps,
            Self::Secrets => WorkspaceView::Secrets,
            Self::PersistentVolumes => WorkspaceView::PersistentVolumes,
            Self::PersistentVolumeClaims => WorkspaceView::PersistentVolumeClaims,
            Self::StorageClasses => WorkspaceView::StorageClasses,
            Self::Hpas => WorkspaceView::Hpas,
            Self::NetworkPolicies => WorkspaceView::NetworkPolicies,
            Self::PodSecurity => WorkspaceView::PodSecurity,
            Self::ResourceQuotas => WorkspaceView::ResourceQuotas,
            Self::AdmissionWebhooks => WorkspaceView::AdmissionWebhooks,
            Self::SecurityOverview => WorkspaceView::SecurityOverview,
            Self::Nodes => WorkspaceView::Nodes,
            Self::Events => WorkspaceView::Events,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResourceWatchPayload {
    Namespaces(WatchUpdate<NamespaceInfo>),
    Deployments(WatchUpdate<DeploymentInfo>),
    StatefulSets(WatchUpdate<StatefulSetInfo>),
    DaemonSets(WatchUpdate<DaemonSetInfo>),
    ReplicaSets(WatchUpdate<ReplicaSetInfo>),
    Jobs(WatchUpdate<JobInfo>),
    CronJobs(WatchUpdate<CronJobInfo>),
    Pods(WatchUpdate<PodInfo>),
    PodMetrics(Vec<PodMetricItem>),
    Services(WatchUpdate<ServiceInfo>),
    ServiceEndpoints(WatchUpdate<ServiceEndpointsSnapshot>),
    Ingresses(WatchUpdate<IngressInfo>),
    ConfigMaps(WatchUpdate<ConfigMapInfo>),
    Secrets(WatchUpdate<SecretInfo>),
    PersistentVolumes(WatchUpdate<PersistentVolumeInfo>),
    PersistentVolumeClaims(WatchUpdate<PersistentVolumeClaimInfo>),
    StorageClasses(WatchUpdate<StorageClassInfo>),
    Hpas(WatchUpdate<HpaInfo>),
    NetworkPolicies(WatchUpdate<PolicyInfo>),
    PodSecurity(WatchUpdate<PolicyInfo>),
    ResourceQuotas {
        prefix: &'static str,
        update: WatchUpdate<PolicyInfo>,
    },
    AdmissionWebhooks {
        prefix: &'static str,
        update: WatchUpdate<PolicyInfo>,
    },
    SecurityOverview {
        prefix: &'static str,
        update: WatchUpdate<PolicyInfo>,
    },
    Nodes(WatchUpdate<NodeInfo>),
    Events(WatchUpdate<EventInfo>),
}

impl ResourceWatchPayload {
    pub const fn kind(&self) -> WatchResourceKind {
        match self {
            Self::Namespaces(_) => WatchResourceKind::Namespaces,
            Self::Deployments(_) => WatchResourceKind::Deployments,
            Self::StatefulSets(_) => WatchResourceKind::StatefulSets,
            Self::DaemonSets(_) => WatchResourceKind::DaemonSets,
            Self::ReplicaSets(_) => WatchResourceKind::ReplicaSets,
            Self::Jobs(_) => WatchResourceKind::Jobs,
            Self::CronJobs(_) => WatchResourceKind::CronJobs,
            Self::Pods(_) | Self::PodMetrics(_) => WatchResourceKind::Pods,
            Self::Services(_) | Self::ServiceEndpoints(_) => WatchResourceKind::Services,
            Self::Ingresses(_) => WatchResourceKind::Ingresses,
            Self::ConfigMaps(_) => WatchResourceKind::ConfigMaps,
            Self::Secrets(_) => WatchResourceKind::Secrets,
            Self::PersistentVolumes(_) => WatchResourceKind::PersistentVolumes,
            Self::PersistentVolumeClaims(_) => WatchResourceKind::PersistentVolumeClaims,
            Self::StorageClasses(_) => WatchResourceKind::StorageClasses,
            Self::Hpas(_) => WatchResourceKind::Hpas,
            Self::NetworkPolicies(_) => WatchResourceKind::NetworkPolicies,
            Self::PodSecurity(_) => WatchResourceKind::PodSecurity,
            Self::ResourceQuotas { .. } => WatchResourceKind::ResourceQuotas,
            Self::AdmissionWebhooks { .. } => WatchResourceKind::AdmissionWebhooks,
            Self::SecurityOverview { .. } => WatchResourceKind::SecurityOverview,
            Self::Nodes(_) => WatchResourceKind::Nodes,
            Self::Events(_) => WatchResourceKind::Events,
        }
    }
}

/// Maps each resource model back to the canonical watch key (`"ns/name"` or `"name"`),
/// keeping manual `apply_loaded` snapshots aligned with live `WatchUpdate` deltas.
pub trait IndexedResourceKey {
    fn indexed_key(&self) -> String;
}

fn namespaced_key(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_string()
    } else {
        format!("{namespace}/{name}")
    }
}

impl IndexedResourceKey for NamespaceInfo {
    fn indexed_key(&self) -> String {
        self.name.clone()
    }
}
impl IndexedResourceKey for DeploymentInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for StatefulSetInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for DaemonSetInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for ReplicaSetInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for JobInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for CronJobInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for PodInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for ServiceInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for IngressInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for ConfigMapInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for SecretInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for PersistentVolumeInfo {
    fn indexed_key(&self) -> String {
        self.name.clone()
    }
}
impl IndexedResourceKey for PersistentVolumeClaimInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for StorageClassInfo {
    fn indexed_key(&self) -> String {
        self.name.clone()
    }
}
impl IndexedResourceKey for HpaInfo {
    fn indexed_key(&self) -> String {
        namespaced_key(&self.namespace, &self.name)
    }
}
impl IndexedResourceKey for PolicyInfo {
    fn indexed_key(&self) -> String {
        if self.r#type == "Pod Security" {
            return format!("PodSecurity:{}", self.namespace);
        }
        let base = if !self.namespace.is_empty()
            && self.namespace != "Cluster"
            && self.namespace != "-"
        {
            format!("{}/{}", self.namespace, self.name)
        } else {
            self.name.clone()
        };
        if self.kind.is_empty() {
            base
        } else {
            format!("{}:{base}", self.kind)
        }
    }
}
impl IndexedResourceKey for NodeInfo {
    fn indexed_key(&self) -> String {
        self.name.clone()
    }
}
impl IndexedResourceKey for EventInfo {
    fn indexed_key(&self) -> String {
        if !self.name.is_empty() {
            namespaced_key(&self.namespace, &self.name)
        } else {
            self.uid.clone()
        }
    }
}

/// O(1) bidirectional key <-> vector index supporting `swap_remove` repair without scanning.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyIndex {
    keys: Vec<String>,
    map: HashMap<String, usize>,
}

impl KeyIndex {
    pub fn clear(&mut self) {
        self.keys.clear();
        self.map.clear();
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<usize> {
        self.map.get(key).copied()
    }

    pub fn key_at(&self, idx: usize) -> Option<&str> {
        self.keys.get(idx).map(String::as_str)
    }

    pub fn insert_new(&mut self, key: String) -> usize {
        let idx = self.keys.len();
        self.keys.push(key.clone());
        self.map.insert(key, idx);
        idx
    }

    pub fn swap_remove(&mut self, key: &str) -> Option<usize> {
        let idx = self.map.remove(key)?;
        self.keys.swap_remove(idx);
        if let Some(moved_key) = self.keys.get(idx) {
            self.map.insert(moved_key.clone(), idx);
        }
        Some(idx)
    }

    pub fn swap_remove_at(&mut self, idx: usize) -> Option<String> {
        if idx >= self.keys.len() {
            return None;
        }
        let removed = self.keys.swap_remove(idx);
        self.map.remove(&removed);
        if let Some(moved_key) = self.keys.get(idx) {
            self.map.insert(moved_key.clone(), idx);
        }
        Some(removed)
    }

    pub fn rebuild<T: IndexedResourceKey>(&mut self, items: &[T]) {
        self.clear();
        self.keys.reserve(items.len());
        self.map.reserve(items.len());
        for (idx, item) in items.iter().enumerate() {
            let key = item.indexed_key();
            self.keys.push(key.clone());
            self.map.insert(key, idx);
        }
    }
}

/// Applies a single-kind `WatchUpdate<T>` to `items` and `index` in O(1) per delta item.
/// `merge_existing` lets callers preserve orthogonal fields (e.g. Pod metrics or Service endpoints).
pub fn apply_indexed_watch_update<T, M>(
    items: &mut Vec<T>,
    index: &mut KeyIndex,
    data_revision: &mut u64,
    update: WatchUpdate<T>,
    mut merge_existing: M,
) -> bool
where
    M: FnMut(&T, &mut T),
{
    match update {
        WatchUpdate::Reset(entries) => {
            let mut previous: HashMap<String, T> = HashMap::with_capacity(items.len());
            for (idx, old_item) in items.drain(..).enumerate() {
                if let Some(k) = index.key_at(idx) {
                    previous.insert(k.to_string(), old_item);
                }
            }
            index.clear();
            items.reserve(entries.len());
            for (key, mut new_item) in entries {
                if let Some(old_item) = previous.get(&key) {
                    merge_existing(old_item, &mut new_item);
                }
                if let Some(existing_idx) = index.get(&key) {
                    items[existing_idx] = new_item;
                } else {
                    index.insert_new(key);
                    items.push(new_item);
                }
            }
            *data_revision = data_revision.wrapping_add(1);
            true
        }
        WatchUpdate::Delta(deltas) => {
            if deltas.is_empty() {
                return false;
            }
            let mut mutated = false;
            for delta in deltas {
                match delta.action {
                    WatchAction::Applied => {
                        let mut new_item = delta.item;
                        if let Some(idx) = index.get(&delta.key) {
                            merge_existing(&items[idx], &mut new_item);
                            items[idx] = new_item;
                        } else {
                            index.insert_new(delta.key);
                            items.push(new_item);
                        }
                        mutated = true;
                    }
                    WatchAction::Deleted => {
                        if let Some(idx) = index.swap_remove(&delta.key) {
                            items.swap_remove(idx);
                            mutated = true;
                        }
                    }
                }
            }
            if mutated {
                *data_revision = data_revision.wrapping_add(1);
            }
            mutated
        }
    }
}

/// Applies a sub-kind `WatchUpdate<T>` to a multi-kind collection whose keys share `"{prefix}:"`.
/// `Reset` only replaces entries matching `"{prefix}:"`, preserving sibling kinds in the same table.
pub fn apply_indexed_watch_update_prefixed<T>(
    items: &mut Vec<T>,
    index: &mut KeyIndex,
    data_revision: &mut u64,
    prefix: &str,
    update: WatchUpdate<T>,
) -> bool {
    match update {
        WatchUpdate::Reset(entries) => {
            let match_prefix = format!("{prefix}:");
            let mut idx = 0;
            while idx < items.len() {
                let belongs_to_prefix = index
                    .key_at(idx)
                    .is_some_and(|k| k.starts_with(&match_prefix));
                if belongs_to_prefix {
                    index.swap_remove_at(idx);
                    items.swap_remove(idx);
                } else {
                    idx += 1;
                }
            }
            for (key, item) in entries {
                if let Some(existing_idx) = index.get(&key) {
                    items[existing_idx] = item;
                } else {
                    index.insert_new(key);
                    items.push(item);
                }
            }
            *data_revision = data_revision.wrapping_add(1);
            true
        }
        WatchUpdate::Delta(deltas) => {
            apply_indexed_watch_update(items, index, data_revision, WatchUpdate::Delta(deltas), |_, _| {})
        }
    }
}

/// Deterministic hash of `TableFilterState::extra_filters` for `RowCacheKey`.
pub fn hash_extra_filters(extra_filters: &HashMap<String, String>) -> u64 {
    if extra_filters.is_empty() {
        return 0;
    }
    let mut pairs: Vec<(&String, &String)> = extra_filters.iter().collect();
    pairs.sort_unstable_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));
    let mut hasher = DefaultHasher::new();
    for (k, v) in pairs {
        k.hash(&mut hasher);
        v.hash(&mut hasher);
    }
    hasher.finish()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowCacheKey {
    pub data_revision: u64,
    pub namespaces_revision: u64,
    pub search_query: String,
    pub extra_filters_hash: u64,
    pub sort: Option<SortState>,
    pub aux_filter_tag: u64,
}

/// Revision-keyed `Rc<Vec<T>>` cache that eliminates per-frame filtering, sorting, and cloning
/// during GPUI hover, scroll, and unrelated UI repaints.
#[derive(Debug, Clone)]
pub struct FilteredRowsCache<T> {
    entry: RefCell<Option<(RowCacheKey, Rc<Vec<T>>)>>,
}

impl<T> Default for FilteredRowsCache<T> {
    fn default() -> Self {
        Self {
            entry: RefCell::new(None),
        }
    }
}

impl<T> FilteredRowsCache<T> {
    pub fn get_or_compute<F>(&self, key: RowCacheKey, compute: F) -> Rc<Vec<T>>
    where
        F: FnOnce() -> Vec<T>,
    {
        if let Some((cached_key, cached_rows)) = self.entry.borrow().as_ref()
            && *cached_key == key
        {
            return Rc::clone(cached_rows);
        }
        let rows = Rc::new(compute());
        *self.entry.borrow_mut() = Some((key, Rc::clone(&rows)));
        rows
    }

    pub fn clear(&self) {
        *self.entry.borrow_mut() = None;
    }
}

impl RowCacheKey {
    pub fn from_table(
        data_revision: u64,
        namespaces_revision: u64,
        table: &crate::components::data_table::DataTableState,
        aux_filter_tag: u64,
    ) -> Self {
        Self {
            data_revision,
            namespaces_revision,
            search_query: table.filters.search_query.clone(),
            extra_filters_hash: hash_extra_filters(&table.filters.extra_filters),
            sort: table.sort.clone(),
            aux_filter_tag,
        }
    }
}
