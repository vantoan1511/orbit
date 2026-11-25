pub mod config;

pub mod kubernetes {
    pub mod batch;
    pub mod configmaps;
    pub mod delete;
    pub mod events;
    pub mod hpa;
    pub mod ingresses;
    pub mod kubeconfig_import;
    pub mod manager;
    pub mod metrics;
    pub mod models;
    pub mod namespaces;
    pub mod nodes;
    pub mod overview;
    pub mod policies;
    pub mod raw;
    pub mod secrets;
    pub mod services;
    pub mod storage;
    pub mod storage_overview;
    pub mod workloads;
    pub mod workloads_overview;
}
