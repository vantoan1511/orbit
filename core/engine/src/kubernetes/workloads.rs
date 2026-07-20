use kube::{
    api::{Api, ListParams, Patch, PatchParams, DeleteParams},
    Client,
};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use crate::kubernetes::{models, format_age};

pub fn map_pod(p: &Pod) -> models::PodInfo {
    let name = p.metadata.name.clone().unwrap_or_default();
    let namespace_name = p.metadata.namespace.clone().unwrap_or_default();

    let status = p.status.as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let age = format_age(&p.metadata.creation_timestamp);

    models::PodInfo {
        name,
        namespace: namespace_name,
        status,
        age,
        cpu: None,
        memory: None,
    }
}

pub async fn list_pods(client: &Client, namespace: Option<String>) -> Result<Vec<models::PodInfo>, kube::Error> {
    let pods: Api<Pod> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut pod_list = Vec::new();
    for p in pods.list(&ListParams::default()).await? {
        pod_list.push(map_pod(&p));
    }

    Ok(pod_list)
}

pub fn map_deployment(d: &Deployment) -> models::DeploymentInfo {
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

    models::DeploymentInfo {
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
        list.push(map_deployment(&d));
    }

    Ok(list)
}

pub fn map_statefulset(ss: &StatefulSet) -> models::StatefulSetInfo {
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

    models::StatefulSetInfo {
        name,
        namespace: namespace_name,
        status,
        replicas,
        age,
        images,
        labels,
        annotations,
    }
}

pub async fn list_statefulsets(client: &Client, namespace: Option<String>) -> Result<Vec<models::StatefulSetInfo>, kube::Error> {
    let api: Api<StatefulSet> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for ss in api.list(&ListParams::default()).await? {
        list.push(map_statefulset(&ss));
    }

    Ok(list)
}

pub fn map_daemonset(ds: &DaemonSet) -> models::DaemonSetInfo {
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

    models::DaemonSetInfo {
        name,
        namespace: namespace_name,
        status,
        replicas,
        age,
        images,
        labels,
        annotations,
    }
}

pub async fn list_daemonsets(client: &Client, namespace: Option<String>) -> Result<Vec<models::DaemonSetInfo>, kube::Error> {
    let api: Api<DaemonSet> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for ds in api.list(&ListParams::default()).await? {
        list.push(map_daemonset(&ds));
    }

    Ok(list)
}

pub fn map_replicaset(rs: &ReplicaSet) -> models::ReplicaSetInfo {
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

    models::ReplicaSetInfo {
        name,
        namespace: namespace_name,
        status,
        replicas,
        age,
        images,
        labels,
        annotations,
    }
}

pub async fn list_replicasets(client: &Client, namespace: Option<String>) -> Result<Vec<models::ReplicaSetInfo>, kube::Error> {
    let api: Api<ReplicaSet> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for rs in api.list(&ListParams::default()).await? {
        list.push(map_replicaset(&rs));
    }

    Ok(list)
}

pub async fn scale_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
    replicas: i32,
) -> Result<(), kube::Error> {
    let patch = serde_json::json!({
        "spec": {
            "replicas": replicas
        }
    });
    let patch_params = PatchParams::default();

    match kind {
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;
        }
        "ReplicaSet" => {
            let api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;
        }
        _ => return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported scale resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
    Ok(())
}

pub async fn redeploy_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
) -> Result<(), kube::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/restartedAt": now
                    }
                }
            }
        }
    });
    let patch_params = PatchParams::default();

    match kind {
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;
        }
        "DaemonSet" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;
        }
        _ => return Err(kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: format!("Unsupported redeploy resource kind: {}", kind),
            reason: "BadRequest".to_string(),
            code: 400,
        })),
    }
    Ok(())
}

pub async fn delete_pod(
    client: &Client,
    namespace: &str,
    name: &str,
) -> Result<(), kube::Error> {
    let api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    api.delete(name, &DeleteParams::default()).await?;
    Ok(())
}
