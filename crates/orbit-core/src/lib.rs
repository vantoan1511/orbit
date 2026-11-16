pub mod config;

pub mod kubernetes {
    pub mod events;
    pub mod manager;
    pub mod metrics;
    pub mod models;
    pub mod namespaces;
    pub mod nodes;
    pub mod overview;
    pub mod raw;
    pub mod services;
    pub mod workloads;
}
