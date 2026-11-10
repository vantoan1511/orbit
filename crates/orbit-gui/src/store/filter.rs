use orbit_core::kubernetes::models::PodInfo;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterCriteria {
    pub namespace: Option<String>,
    pub search_query: String,
    pub status: Option<String>,
}

impl FilterCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_namespace(mut self, ns: Option<String>) -> Self {
        self.namespace = ns;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        self.search_query = query.into().trim().to_lowercase();
        self
    }

    pub fn with_status(mut self, status: Option<String>) -> Self {
        self.status = status;
        self
    }

    pub fn matches_pod(&self, pod: &PodInfo) -> bool {
        // Namespace filter
        if self
            .namespace
            .as_deref()
            .is_some_and(|ns| !ns.is_empty() && pod.namespace != ns)
        {
            return false;
        }

        // Status filter
        if self
            .status
            .as_deref()
            .is_some_and(|st| !st.is_empty() && !pod.status.eq_ignore_ascii_case(st))
        {
            return false;
        }

        // Search query filter
        if !self.search_query.is_empty() {
            let q = &self.search_query;
            let name_matches = pod.name.to_lowercase().contains(q);
            let node_matches = pod.node.as_ref().is_some_and(|n| n.to_lowercase().contains(q));
            let ip_matches = pod.ip.as_ref().is_some_and(|ip| ip.to_lowercase().contains(q))
                || pod.node_ip.as_ref().is_some_and(|ip| ip.to_lowercase().contains(q));

            if !name_matches && !node_matches && !ip_matches {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_pods_by_namespace_search_and_status() {
        let pod1 = PodInfo {
            name: "nginx-app".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            node: Some("node-worker-1".to_string()),
            ip: Some("10.244.0.15".to_string()),
            ..Default::default()
        };

        let pod2 = PodInfo {
            name: "coredns-xyz".to_string(),
            namespace: "kube-system".to_string(),
            status: "Running".to_string(),
            node: Some("node-master".to_string()),
            ip: Some("10.244.0.5".to_string()),
            ..Default::default()
        };

        let pod3 = PodInfo {
            name: "nginx-ingress".to_string(),
            namespace: "kube-system".to_string(),
            status: "Pending".to_string(),
            node: None,
            ip: None,
            ..Default::default()
        };

        // All Namespaces
        let filter_all = FilterCriteria::default();
        assert!(filter_all.matches_pod(&pod1));
        assert!(filter_all.matches_pod(&pod2));
        assert!(filter_all.matches_pod(&pod3));

        // Filter by namespace
        let filter_kube_system = FilterCriteria::default().with_namespace(Some("kube-system".to_string()));
        assert!(!filter_kube_system.matches_pod(&pod1));
        assert!(filter_kube_system.matches_pod(&pod2));
        assert!(filter_kube_system.matches_pod(&pod3));

        // Filter by search query
        let filter_nginx = FilterCriteria::default().with_search("NGINX");
        assert!(filter_nginx.matches_pod(&pod1));
        assert!(!filter_nginx.matches_pod(&pod2));
        assert!(filter_nginx.matches_pod(&pod3));

        // Filter by search node
        let filter_worker = FilterCriteria::default().with_search("worker-1");
        assert!(filter_worker.matches_pod(&pod1));
        assert!(!filter_worker.matches_pod(&pod2));

        // Filter by status
        let filter_pending = FilterCriteria::default().with_status(Some("Pending".to_string()));
        assert!(!filter_pending.matches_pod(&pod1));
        assert!(!filter_pending.matches_pod(&pod2));
        assert!(filter_pending.matches_pod(&pod3));
    }
}
