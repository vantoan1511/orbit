use kube::{
    api::{Api, ListParams, Patch, PatchParams, DeleteParams, PostParams},
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

    let node = p.spec.as_ref().and_then(|s| s.node_name.clone());
    let ip = p.status.as_ref().and_then(|s| s.pod_ip.clone());
    let node_ip = p.status.as_ref().and_then(|s| s.host_ip.clone());
    let qos_class = p.status.as_ref().and_then(|s| s.qos_class.clone());

    let controlled_by = p.metadata.owner_references.as_ref()
        .and_then(|owners| owners.first())
        .map(|o| format!("{}/{}", o.kind, o.name));

    let labels = p.metadata.labels.clone().unwrap_or_default();
    let annotations = p.metadata.annotations.clone().unwrap_or_default();

    let mut containers = Vec::new();
    let mut images = Vec::new();
    let mut total_restarts = 0;

    let container_statuses = p.status.as_ref().and_then(|s| s.container_statuses.as_ref());

    if let Some(spec) = p.spec.as_ref() {
        for c in &spec.containers {
            let c_name = c.name.clone();
            let c_image = c.image.clone().unwrap_or_default();
            if !c_image.is_empty() && !images.contains(&c_image) {
                images.push(c_image.clone());
            }

            let mut status_str = "Waiting".to_string();
            let mut ready_str = "false".to_string();
            let mut restarts = 0;

            if let Some(statuses) = container_statuses
                && let Some(cs) = statuses.iter().find(|s| s.name == c_name)
            {
                ready_str = cs.ready.to_string();
                restarts = cs.restart_count;
                total_restarts += restarts;

                if let Some(state) = &cs.state {
                    if state.running.is_some() {
                        status_str = "Running".to_string();
                    } else if let Some(term) = &state.terminated {
                        let reason = term.reason.as_deref().unwrap_or("Terminated");
                        status_str = format!("Terminated ({})", reason);
                    } else if let Some(waiting) = &state.waiting {
                        status_str = waiting.reason.clone().unwrap_or_else(|| "Waiting".to_string());
                    }
                }
            }

            let mut ports_str = None;
            if let Some(ports) = &c.ports {
                let port_list: Vec<String> = ports.iter()
                    .map(|port| format!("{}/{}", port.container_port, port.protocol.as_deref().unwrap_or("TCP")))
                    .collect();
                if !port_list.is_empty() {
                    ports_str = Some(port_list.join(", "));
                }
            }

            containers.push(models::PodContainer {
                name: c_name,
                image: c_image,
                status: status_str,
                ready: ready_str,
                restarts,
                ports: ports_str,
            });
        }
    }

    models::PodInfo {
        name,
        namespace: namespace_name,
        status,
        age,
        cpu: None,
        memory: None,
        node,
        restarts: total_restarts,
        images,
        labels,
        annotations,
        ip,
        node_ip,
        controlled_by,
        qos_class,
        containers,
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
    let mut containers = Vec::new();
    if let Some(spec) = d.spec.as_ref() {
        for c in &spec.template.spec.as_ref().map(|s| s.containers.clone()).unwrap_or_default() {
            if let Some(img) = &c.image {
                images.push(img.clone());
            }
            containers.push(models::ContainerImageInfo {
                name: c.name.clone(),
                image: c.image.clone().unwrap_or_default(),
            });
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
        containers,
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

pub async fn update_images_resource(
    client: &Client,
    namespace: &str,
    kind: &str,
    name: &str,
    containers: Vec<models::ContainerImageInfo>,
) -> Result<(), kube::Error> {
    let containers_patch: Vec<serde_json::Value> = containers
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "image": c.image
            })
        })
        .collect();

    let patch = serde_json::json!({
        "spec": {
            "template": {
                "spec": {
                    "containers": containers_patch
                }
            }
        }
    });
    let patch_params = PatchParams::default();

    match kind {
        "Deployment" => {
            let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Strategic(&patch)).await?;
        }
        "StatefulSet" => {
            let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Strategic(&patch)).await?;
        }
        "DaemonSet" => {
            let api: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            api.patch(name, &patch_params, &Patch::Strategic(&patch)).await?;
        }
        _ => {
            return Err(kube::Error::Api(kube::error::ErrorResponse {
                status: "Failure".to_string(),
                message: format!("Unsupported update images resource kind: {}", kind),
                reason: "BadRequest".to_string(),
                code: 400,
            }))
        }
    }
    Ok(())
}

pub async fn clone_deployment(
    client: &Client,
    source_namespace: &str,
    source_name: &str,
    new_name: &str,
    new_namespace: &str,
) -> Result<(), kube::Error> {
    let source_api: Api<Deployment> = Api::namespaced(client.clone(), source_namespace);
    let mut cloned = source_api.get(source_name).await?;

    // Strip server-managed metadata
    cloned.metadata.uid = None;
    cloned.metadata.resource_version = None;
    cloned.metadata.creation_timestamp = None;
    cloned.metadata.generation = None;
    cloned.metadata.managed_fields = None;
    cloned.metadata.owner_references = None;
    cloned.metadata.finalizers = None;
    cloned.status = None;

    cloned.metadata.name = Some(new_name.to_string());
    cloned.metadata.namespace = Some(new_namespace.to_string());

    // Update metadata labels if any match source_name
    if let Some(labels) = cloned.metadata.labels.as_mut() {
        for (_, val) in labels.iter_mut() {
            if val == source_name {
                *val = new_name.to_string();
            }
        }
    }

    // Update spec.selector.match_labels and spec.template.metadata.labels
    if let Some(spec) = cloned.spec.as_mut() {
        if let Some(match_labels) = spec.selector.match_labels.as_mut() {
            for (_, val) in match_labels.iter_mut() {
                if val == source_name {
                    *val = new_name.to_string();
                }
            }
        }

        if let Some(template_labels) = spec.template.metadata.as_mut().and_then(|m| m.labels.as_mut()) {
            for (_, val) in template_labels.iter_mut() {
                if val == source_name {
                    *val = new_name.to_string();
                }
            }
        }
    }

    let target_api: Api<Deployment> = Api::namespaced(client.clone(), new_namespace);
    target_api.create(&PostParams::default(), &cloned).await?;
    Ok(())
}

pub const REVISION_ANNOTATION: &str = "deployment.kubernetes.io/revision";

pub fn find_rollback_replicaset<'a>(
    deployment: &Deployment,
    replicasets: &'a [ReplicaSet],
    target_revision: Option<i64>,
) -> Result<&'a ReplicaSet, String> {
    let deploy_name = deployment.metadata.name.as_deref().unwrap_or_default();
    let deploy_uid = deployment.metadata.uid.as_deref().unwrap_or_default();

    let mut matching_rs: Vec<(&'a ReplicaSet, i64)> = replicasets
        .iter()
        .filter(|rs| {
            rs.metadata.owner_references.as_ref().map_or(false, |owners| {
                owners.iter().any(|owner| {
                    owner.kind == "Deployment"
                        && (owner.name == deploy_name || (!deploy_uid.is_empty() && owner.uid == deploy_uid))
                })
            })
        })
        .filter_map(|rs| {
            let rev = rs
                .metadata
                .annotations
                .as_ref()
                .and_then(|a| a.get(REVISION_ANNOTATION))
                .and_then(|r| r.parse::<i64>().ok())?;
            Some((rs, rev))
        })
        .collect();

    if matching_rs.is_empty() {
        return Err("No matching ReplicaSets with revision annotations found".to_string());
    }

    matching_rs.sort_by(|a, b| b.1.cmp(&a.1));

    if let Some(target) = target_revision {
        if target > 0 {
            return matching_rs
                .into_iter()
                .find(|(_, rev)| *rev == target)
                .map(|(rs, _)| rs)
                .ok_or_else(|| format!("ReplicaSet with revision {} not found", target));
        }
    }

    let current_revision = deployment
        .metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get(REVISION_ANNOTATION))
        .and_then(|r| r.parse::<i64>().ok());

    if let Some(curr) = current_revision {
        if let Some((rs, _)) = matching_rs.iter().find(|(_, rev)| *rev < curr) {
            return Ok(rs);
        }
    }

    if matching_rs.len() > 1 {
        return Ok(matching_rs[1].0);
    }

    Err("No previous revision available to rollback to".to_string())
}

pub fn clean_template_for_rollback(template: &mut k8s_openapi::api::core::v1::PodTemplateSpec) {
    if let Some(labels) = template.metadata.as_mut().and_then(|m| m.labels.as_mut()) {
        labels.remove("pod-template-hash");
    }
}

pub async fn rollback_deployment(
    client: &Client,
    namespace: &str,
    name: &str,
    target_revision: Option<i64>,
) -> Result<(), kube::Error> {
    let deploy_api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deployment = deploy_api.get(name).await?;

    let rs_api: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
    let rs_list = rs_api.list(&ListParams::default()).await?;

    let target_rs = find_rollback_replicaset(&deployment, &rs_list.items, target_revision).map_err(|msg| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: msg,
            reason: "BadRequest".to_string(),
            code: 400,
        })
    })?;

    let mut template = match target_rs.spec.as_ref().and_then(|s| s.template.clone()) {
        Some(t) => t,
        None => {
            return Err(kube::Error::Api(kube::error::ErrorResponse {
                status: "Failure".to_string(),
                message: "Target ReplicaSet has no pod template".to_string(),
                reason: "InternalError".to_string(),
                code: 500,
            }))
        }
    };

    clean_template_for_rollback(&mut template);

    let patch = serde_json::json!({
        "spec": {
            "template": template
        }
    });

    let patch_params = PatchParams::default();
    deploy_api.patch(name, &patch_params, &Patch::Merge(&patch)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec, ReplicaSet, ReplicaSetSpec};
    use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta, OwnerReference};
    use std::collections::BTreeMap;

    #[test]
    fn test_clone_deployment_label_patching() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "my-service".to_string());
        labels.insert("tier".to_string(), "backend".to_string());

        let mut template_labels = BTreeMap::new();
        template_labels.insert("app".to_string(), "my-service".to_string());
        template_labels.insert("tier".to_string(), "backend".to_string());

        let mut dep = Deployment {
            metadata: ObjectMeta {
                name: Some("my-service".to_string()),
                namespace: Some("default".to_string()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                selector: LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(template_labels),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            status: None,
        };

        let source_name = "my-service";
        let new_name = "my-service-copy";

        // Apply metadata updates as in clone_deployment
        dep.metadata.name = Some(new_name.to_string());
        if let Some(meta_labels) = dep.metadata.labels.as_mut() {
            for (_, val) in meta_labels.iter_mut() {
                if val == source_name {
                    *val = new_name.to_string();
                }
            }
        }

        if let Some(spec) = dep.spec.as_mut() {
            if let Some(match_labels) = spec.selector.match_labels.as_mut() {
                for (_, val) in match_labels.iter_mut() {
                    if val == source_name {
                        *val = new_name.to_string();
                    }
                }
            }

            if let Some(t_labels) = spec.template.metadata.as_mut().and_then(|m| m.labels.as_mut()) {
                for (_, val) in t_labels.iter_mut() {
                    if val == source_name {
                        *val = new_name.to_string();
                    }
                }
            }
        }

        assert_eq!(dep.metadata.name.as_deref(), Some("my-service-copy"));
        assert_eq!(dep.metadata.labels.as_ref().unwrap().get("app").unwrap(), "my-service-copy");
        assert_eq!(dep.metadata.labels.as_ref().unwrap().get("tier").unwrap(), "backend");
        assert_eq!(
            dep.spec.as_ref().unwrap().selector.match_labels.as_ref().unwrap().get("app").unwrap(),
            "my-service-copy"
        );
        assert_eq!(
            dep.spec.as_ref().unwrap().template.metadata.as_ref().unwrap().labels.as_ref().unwrap().get("app").unwrap(),
            "my-service-copy"
        );
    }

    fn make_replicaset(name: &str, deploy_name: &str, revision: &str, image: &str) -> ReplicaSet {
        let mut annotations = BTreeMap::new();
        annotations.insert(REVISION_ANNOTATION.to_string(), revision.to_string());

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), deploy_name.to_string());
        labels.insert("pod-template-hash".to_string(), "hash123".to_string());

        ReplicaSet {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some("default".to_string()),
                annotations: Some(annotations),
                owner_references: Some(vec![OwnerReference {
                    api_version: "apps/v1".to_string(),
                    kind: "Deployment".to_string(),
                    name: deploy_name.to_string(),
                    uid: "deploy-uid-1".to_string(),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            spec: Some(ReplicaSetSpec {
                template: Some(PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "app".to_string(),
                            image: Some(image.to_string()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            }),
            status: None,
        }
    }

    #[test]
    fn test_find_rollback_replicaset_previous() {
        let mut dep_annotations = BTreeMap::new();
        dep_annotations.insert(REVISION_ANNOTATION.to_string(), "3".to_string());

        let dep = Deployment {
            metadata: ObjectMeta {
                name: Some("web".to_string()),
                uid: Some("deploy-uid-1".to_string()),
                namespace: Some("default".to_string()),
                annotations: Some(dep_annotations),
                ..Default::default()
            },
            ..Default::default()
        };

        let rs1 = make_replicaset("web-rs-1", "web", "1", "nginx:1.18");
        let rs2 = make_replicaset("web-rs-2", "web", "2", "nginx:1.19");
        let rs3 = make_replicaset("web-rs-3", "web", "3", "nginx:1.20");
        let rs_other = make_replicaset("other-rs-1", "other", "2", "redis:6");

        let replicasets = vec![rs1, rs2, rs3, rs_other];

        // Should find revision 2 when target_revision is None
        let target = find_rollback_replicaset(&dep, &replicasets, None).expect("Should find target RS");
        assert_eq!(target.metadata.name.as_deref(), Some("web-rs-2"));

        // Should find revision 1 when target_revision is Some(1)
        let target_rev1 = find_rollback_replicaset(&dep, &replicasets, Some(1)).expect("Should find target RS rev 1");
        assert_eq!(target_rev1.metadata.name.as_deref(), Some("web-rs-1"));

        // Should fail when target_revision does not exist
        let err = find_rollback_replicaset(&dep, &replicasets, Some(99)).unwrap_err();
        assert!(err.contains("revision 99 not found"));
    }

    #[test]
    fn test_clean_template_for_rollback() {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "web".to_string());
        labels.insert("pod-template-hash".to_string(), "abcxyz".to_string());

        let mut template = PodTemplateSpec {
            metadata: Some(ObjectMeta {
                labels: Some(labels),
                ..Default::default()
            }),
            spec: None,
        };

        clean_template_for_rollback(&mut template);

        let labels = template.metadata.unwrap().labels.unwrap();
        assert_eq!(labels.get("app").map(|s| s.as_str()), Some("web"));
        assert!(!labels.contains_key("pod-template-hash"));
    }
}

