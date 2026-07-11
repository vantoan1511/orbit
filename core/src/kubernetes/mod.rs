use crate::kubernetes::models::PodInfo;
use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Namespace, Pod, Node};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};


pub mod models;
pub mod manager;

pub async fn list_namespaces(client: &Client) -> Result<Vec<String>, kube::Error> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let mut namespace_list = Vec::new();
    
    for ns in namespaces.list(&ListParams::default()).await? {
        if let Some(name) = ns.metadata.name {
            namespace_list.push(name);
        }
    }
    
    // Sort namespaces alphabetically for consistency
    namespace_list.sort();
    
    Ok(namespace_list)
}

pub async fn list_pods(client: &Client, namespace: Option<String>) -> Result<Vec<PodInfo>, kube::Error> {
    let pods: Api<Pod> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut pod_list = Vec::new();
    for p in pods.list(&ListParams::default()).await? {
        let name = p.metadata.name.clone().unwrap_or_default();
        let namespace_name = p.metadata.namespace.clone().unwrap_or_default();
        
        let status = p.status.as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string());
            
        // Calculate age
        let age = if let Some(creation) = &p.metadata.creation_timestamp {
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
        };
        
        pod_list.push(PodInfo {
            name,
            namespace: namespace_name,
            status,
            age,
            cpu: None,
            memory: None,
        });
    }
    
    Ok(pod_list)
}

fn format_age(creation_timestamp: &Option<k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> String {
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

pub async fn list_deployments(client: &Client, namespace: Option<String>) -> Result<Vec<models::DeploymentInfo>, kube::Error> {
    let api: Api<Deployment> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut list = Vec::new();
    for d in api.list(&ListParams::default()).await? {
        let name = d.metadata.name.clone().unwrap_or_default();
        let namespace_name = d.metadata.namespace.clone().unwrap_or_default();
        
        let age = format_age(&d.metadata.creation_timestamp);
        
        let desired = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
        let status_replicas = d.status.as_ref();
        let current = status_replicas.and_then(|s| s.replicas).unwrap_or(0);
        let available = status_replicas.and_then(|s| s.available_replicas).unwrap_or(0);
        let up_to_date = status_replicas.and_then(|s| s.updated_replicas).unwrap_or(0);
        
        let replicas = models::Replicas { current, desired };
        
        let mut status = "Progressing".to_string();
        if (desired == 0 && current == 0) || available == desired {
            status = "Running".to_string();
        } else if let Some(conds) = status_replicas.and_then(|st| st.conditions.as_ref()) {
            for c in conds {
                if c.type_ == "ReplicaFailure" && c.status == "True" {
                    status = "Failed".to_string();
                }
            }
        }
        
        let mut images = Vec::new();
        if let Some(spec) = d.spec.as_ref() {
            for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }
        
        let strategy = d.spec.as_ref()
            .and_then(|s| s.strategy.as_ref())
            .and_then(|strt| strt.type_.clone());
            
        let min_ready_seconds = d.spec.as_ref().and_then(|s| s.min_ready_seconds).unwrap_or(0);
        let revision_history = d.spec.as_ref().and_then(|s| s.revision_history_limit);
        
        let labels = d.metadata.labels.clone().unwrap_or_default();
        let annotations = d.metadata.annotations.clone().unwrap_or_default();
        
        list.push(models::DeploymentInfo {
            name,
            namespace: namespace_name,
            status,
            replicas,
            available,
            up_to_date,
            age,
            images,
            strategy,
            min_ready_seconds,
            revision_history,
            labels,
            annotations,
        });
    }
    
    Ok(list)
}

pub async fn list_statefulsets(client: &Client, namespace: Option<String>) -> Result<Vec<models::StatefulSetInfo>, kube::Error> {
    let api: Api<StatefulSet> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut list = Vec::new();
    for ss in api.list(&ListParams::default()).await? {
        let name = ss.metadata.name.clone().unwrap_or_default();
        let namespace_name = ss.metadata.namespace.clone().unwrap_or_default();
        let age = format_age(&ss.metadata.creation_timestamp);
        
        let desired = ss.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
        let status_replicas = ss.status.as_ref();
        let current = status_replicas.and_then(|s| s.ready_replicas).unwrap_or(0);
        let replicas = models::Replicas { current, desired };
        
        let status = if desired == current {
            "Running".to_string()
        } else {
            "Progressing".to_string()
        };
        
        let mut images = Vec::new();
        if let Some(spec) = ss.spec.as_ref() {
            for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }
        
        let labels = ss.metadata.labels.clone().unwrap_or_default();
        let annotations = ss.metadata.annotations.clone().unwrap_or_default();
        
        list.push(models::StatefulSetInfo {
            name,
            namespace: namespace_name,
            status,
            replicas,
            age,
            images,
            labels,
            annotations,
        });
    }
    
    Ok(list)
}

pub async fn list_daemonsets(client: &Client, namespace: Option<String>) -> Result<Vec<models::DaemonSetInfo>, kube::Error> {
    let api: Api<DaemonSet> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut list = Vec::new();
    for ds in api.list(&ListParams::default()).await? {
        let name = ds.metadata.name.clone().unwrap_or_default();
        let namespace_name = ds.metadata.namespace.clone().unwrap_or_default();
        let age = format_age(&ds.metadata.creation_timestamp);
        
        let status_ds = ds.status.as_ref();
        let desired = status_ds.map(|s| s.desired_number_scheduled).unwrap_or(0);
        let current = status_ds.map(|s| s.current_number_scheduled).unwrap_or(0);
        let ready = status_ds.map(|s| s.number_ready).unwrap_or(0);
        let up_to_date = status_ds.and_then(|s| s.updated_number_scheduled).unwrap_or(0);
        let available = status_ds.and_then(|s| s.number_available).unwrap_or(0);
        
        let replicas = models::DaemonSetReplicas {
            desired,
            current,
            ready,
            up_to_date,
            available,
        };
        
        let status = if desired == ready {
            "Running".to_string()
        } else {
            "Progressing".to_string()
        };
        
        let mut images = Vec::new();
        if let Some(spec) = ds.spec.as_ref() {
            for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }
        
        let labels = ds.metadata.labels.clone().unwrap_or_default();
        let annotations = ds.metadata.annotations.clone().unwrap_or_default();
        
        list.push(models::DaemonSetInfo {
            name,
            namespace: namespace_name,
            status,
            replicas,
            age,
            images,
            labels,
            annotations,
        });
    }
    
    Ok(list)
}

pub async fn list_replicasets(client: &Client, namespace: Option<String>) -> Result<Vec<models::ReplicaSetInfo>, kube::Error> {
    let api: Api<ReplicaSet> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut list = Vec::new();
    for rs in api.list(&ListParams::default()).await? {
        let name = rs.metadata.name.clone().unwrap_or_default();
        let namespace_name = rs.metadata.namespace.clone().unwrap_or_default();
        let age = format_age(&rs.metadata.creation_timestamp);
        
        let desired = rs.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
        let status_rs = rs.status.as_ref();
        let current = status_rs.and_then(|s| s.ready_replicas).unwrap_or(0);
        let replicas = models::Replicas { current, desired };
        
        let status = if desired == current {
            "Running".to_string()
        } else {
            "Progressing".to_string()
        };
        
        let mut images = Vec::new();
        if let Some(template) = rs.spec.as_ref().and_then(|spec| spec.template.as_ref()) {
            for c in &template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }
        
        let labels = rs.metadata.labels.clone().unwrap_or_default();
        let annotations = rs.metadata.annotations.clone().unwrap_or_default();
        
        list.push(models::ReplicaSetInfo {
            name,
            namespace: namespace_name,
            status,
            replicas,
            age,
            images,
            labels,
            annotations,
        });
    }
    
    Ok(list)
}

pub async fn list_jobs(client: &Client, namespace: Option<String>) -> Result<Vec<models::JobInfo>, kube::Error> {
    let api: Api<Job> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut list = Vec::new();
    for j in api.list(&ListParams::default()).await? {
        let name = j.metadata.name.clone().unwrap_or_default();
        let namespace_name = j.metadata.namespace.clone().unwrap_or_default();
        let age = format_age(&j.metadata.creation_timestamp);
        
        let spec_jobs = j.spec.as_ref();
        let desired_completions = spec_jobs.and_then(|s| s.completions).unwrap_or(1);
        
        let status_j = j.status.as_ref();
        let succeeded = status_j.and_then(|s| s.succeeded).unwrap_or(0);
        let active = status_j.and_then(|s| s.active).unwrap_or(0);
        let failed = status_j.and_then(|s| s.failed).unwrap_or(0);
        
        let completions = format!("{}/{}", succeeded, desired_completions);
        
        let duration = if let Some(st) = status_j.and_then(|s| s.start_time.clone()) {
            let end = status_j.and_then(|s| s.completion_time.clone())
                .or_else(|| {
                    status_j.and_then(|s| s.conditions.as_ref())
                        .and_then(|conds| conds.iter().find(|c| c.type_ == "Failed" && c.status == "True"))
                        .and_then(|c| c.last_transition_time.clone())
                });
                
            if let Some(et) = end {
                let dur = et.0.signed_duration_since(st.0);
                let secs = dur.num_seconds();
                if secs < 60 {
                    Some(format!("{}s", secs))
                } else if secs < 3600 {
                    Some(format!("{}m {}s", dur.num_minutes(), secs % 60))
                } else {
                    Some(format!("{}h {}m", dur.num_hours(), dur.num_minutes() % 60))
                }
            } else {
                let dur = chrono::Utc::now().signed_duration_since(st.0);
                Some(format!("{}s (running)", dur.num_seconds()))
            }
        } else {
            None
        };
        
        let status = if active > 0 {
            "Active".to_string()
        } else if succeeded >= desired_completions {
            "Succeeded".to_string()
        } else if failed > 0 {
            "Failed".to_string()
        } else {
            "Unknown".to_string()
        };
        
        let mut images = Vec::new();
        if let Some(spec) = j.spec.as_ref() {
            for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }
        
        let labels = j.metadata.labels.clone().unwrap_or_default();
        let annotations = j.metadata.annotations.clone().unwrap_or_default();
        
        list.push(models::JobInfo {
            name,
            namespace: namespace_name,
            status,
            completions,
            duration,
            age,
            images,
            labels,
            annotations,
        });
    }
    
    Ok(list)
}

pub async fn list_cronjobs(client: &Client, namespace: Option<String>) -> Result<Vec<models::CronJobInfo>, kube::Error> {
    let api: Api<CronJob> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };
    
    let mut list = Vec::new();
    for cj in api.list(&ListParams::default()).await? {
        let name = cj.metadata.name.clone().unwrap_or_default();
        let namespace_name = cj.metadata.namespace.clone().unwrap_or_default();
        let age = format_age(&cj.metadata.creation_timestamp);
        
        let spec = cj.spec.as_ref();
        let schedule = spec.map(|s| s.schedule.clone()).unwrap_or_default();
        let suspend = spec.and_then(|s| s.suspend).unwrap_or(false);
        
        let status_cj = cj.status.as_ref();
        let active = status_cj.and_then(|s| s.active.as_ref()).map(|a| a.len() as i32).unwrap_or(0);
        
        let last_schedule = if let Some(last) = status_cj.and_then(|s| s.last_schedule_time.clone()) {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(last.0);
            let seconds = duration.num_seconds();
            if seconds < 0 {
                Some("0s".to_string())
            } else if seconds < 60 {
                Some(format!("{}s ago", seconds))
            } else if seconds < 3600 {
                Some(format!("{}m ago", duration.num_minutes()))
            } else if seconds < 86400 {
                Some(format!("{}h ago", duration.num_hours()))
            } else {
                Some(format!("{}d ago", duration.num_days()))
            }
        } else {
            None
        };
        
        let mut images = Vec::new();
        if let Some(job_spec) = cj.spec.as_ref().and_then(|spec| spec.job_template.spec.as_ref()) {
            for c in &job_spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
                if let Some(img) = &c.image {
                    images.push(img.clone());
                }
            }
        }
        
        let labels = cj.metadata.labels.clone().unwrap_or_default();
        let annotations = cj.metadata.annotations.clone().unwrap_or_default();
        
        list.push(models::CronJobInfo {
            name,
            namespace: namespace_name,
            schedule,
            suspend,
            active,
            last_schedule,
            age,
            images,
            labels,
            annotations,
        });
    }
    
    Ok(list)
}

pub async fn list_nodes(client: &Client) -> Result<Vec<models::NodeInfo>, kube::Error> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());
    
    let nodes = nodes_api.list(&ListParams::default()).await?;
    let pods = pods_api.list(&ListParams::default()).await?;
    
    let mut pods_by_node: std::collections::HashMap<String, Vec<Pod>> = std::collections::HashMap::new();
    for pod in pods {
        if let Some(spec) = &pod.spec {
            if let Some(node_name) = &spec.node_name {
                pods_by_node.entry(node_name.clone()).or_default().push(pod);
            }
        }
    }
    
    let mut list = Vec::new();
    for node in nodes {
        let name = node.metadata.name.clone().unwrap_or_default();
        
        let status = if let Some(status_ref) = &node.status {
            if let Some(conditions) = &status_ref.conditions {
                conditions.iter()
                    .find(|c| c.type_ == "Ready")
                    .map(|c| if c.status == "True" { "Ready".to_string() } else { "NotReady".to_string() })
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };
        
        let mut role = "worker".to_string();
        if let Some(labels_map) = &node.metadata.labels {
            if labels_map.contains_key("node-role.kubernetes.io/control-plane") 
                || labels_map.contains_key("node-role.kubernetes.io/master") {
                role = "control-plane".to_string();
            }
        }
        
        let version = node.status.as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|ni| ni.kubelet_version.clone())
            .unwrap_or_else(|| "Unknown".to_string());
            
        let uptime = if let Some(creation) = &node.metadata.creation_timestamp {
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
        };
        
        let mut labels = Vec::new();
        if let Some(map) = &node.metadata.labels {
            for (k, v) in map {
                labels.push(format!("{}={}", k, v));
            }
        }
        labels.sort();
        
        let node_pods = pods_by_node.get(&name).cloned().unwrap_or_default();
        let pods_count = node_pods.len() as i32;
        
        let pods_limit = node.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("pods"))
            .and_then(|q| q.0.parse::<i32>().ok())
            .unwrap_or(110);
            
        let cpu_total_str = node.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("cpu"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());
        let cpu_total_val = parse_cpu_quantity(&cpu_total_str);
        
        let mem_total_str = node.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("memory"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());
        let mem_total_val = parse_memory_quantity(&mem_total_str);
        
        let mut cpu_req = 0.0;
        let mut mem_req = 0.0;
        
        for pod in &node_pods {
            if let Some(spec) = &pod.spec {
                for container in &spec.containers {
                    if let Some(resources) = &container.resources {
                        if let Some(requests) = &resources.requests {
                            if let Some(cpu) = requests.get("cpu") {
                                cpu_req += parse_cpu_quantity(&cpu.0);
                            }
                            if let Some(mem) = requests.get("memory") {
                                mem_req += parse_memory_quantity(&mem.0);
                            }
                        }
                    }
                }
            }
        }
        
        let cpu_pct = if cpu_total_val > 0.0 { (cpu_req / cpu_total_val) * 100.0 } else { 0.0 };
        let mem_pct = if mem_total_val > 0.0 { (mem_req / mem_total_val) * 100.0 } else { 0.0 };
        
        list.push(models::NodeInfo {
            name,
            status,
            role,
            version,
            cpu_pct,
            cpu_used: format!("{:.1}", cpu_req),
            cpu_total: format!("{:.0}", cpu_total_val),
            mem_pct,
            mem_used: format!("{:.1}", mem_req),
            mem_total: format!("{:.0}", mem_total_val),
            pods_count,
            pods_limit,
            uptime,
            labels,
        });
    }
    
    Ok(list)
}

fn parse_cpu_quantity(q: &str) -> f64 {
    let q = q.trim();
    if q.ends_with('m') {
        q[..q.len() - 1].parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        q.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_memory_quantity(q: &str) -> f64 {
    let q = q.trim();
    if q.is_empty() {
        return 0.0;
    }
    
    if q.ends_with("Ki") {
        let val = q[..q.len() - 2].parse::<f64>().unwrap_or(0.0);
        val * 1024.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with("Mi") {
        let val = q[..q.len() - 2].parse::<f64>().unwrap_or(0.0);
        val * 1024.0 * 1024.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with("Gi") {
        q[..q.len() - 2].parse::<f64>().unwrap_or(0.0)
    } else if q.ends_with("Ti") {
        let val = q[..q.len() - 2].parse::<f64>().unwrap_or(0.0);
        val * 1024.0
    } else if q.ends_with("Pi") {
        let val = q[..q.len() - 2].parse::<f64>().unwrap_or(0.0);
        val * 1024.0 * 1024.0
    } else if q.ends_with("Ei") {
        let val = q[..q.len() - 2].parse::<f64>().unwrap_or(0.0);
        val * 1024.0 * 1024.0 * 1024.0
    } else if q.ends_with('k') {
        let val = q[..q.len() - 1].parse::<f64>().unwrap_or(0.0);
        val * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with('M') {
        let val = q[..q.len() - 1].parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with('G') {
        let val = q[..q.len() - 1].parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with('T') {
        let val = q[..q.len() - 1].parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with('P') {
        let val = q[..q.len() - 1].parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else if q.ends_with('E') {
        let val = q[..q.len() - 1].parse::<f64>().unwrap_or(0.0);
        val * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 / (1024.0 * 1024.0 * 1024.0)
    } else {
        let val = q.parse::<f64>().unwrap_or(0.0);
        val / (1024.0 * 1024.0 * 1024.0)
    }
}



