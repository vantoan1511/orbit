use std::collections::HashMap;
use std::rc::Rc;

use gpui::component::{
    Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    form::{field, v_form},
    input::InputState,
    switch::Switch,
};
use gpui::{AnyElement, Entity, FontWeight, SharedString, div, prelude::*, px};
use lucide_icons::Icon;
use orbit_core::kubernetes::edit_forms::{
    ContainerEnvEntry, ContainerFormValues, ContainerPortEntry, CronJobFormValues,
    DaemonSetFormValues, DeploymentFormValues, EditFormModel, EnvSourceType, JobFormValues,
    KeyValueEntry, PodSpecFormValues, ReplicaSetFormValues, StatefulSetFormValues,
};

use crate::icons::render_icon;
use crate::theme::{OrbitTheme, apply_text_step};
use crate::views::edit::{
    EditFormMutatorCallback, EditFormSubTab, EditSessionState, EditViewCallbacks, push_kv_specs,
    push_list_specs, render_input_or_placeholder, render_kv_editor_section,
    render_metadata_kv_sections, render_readonly_fields, render_string_list_section,
    render_two_col_section,
};

const IMAGE_PULL_POLICIES: &[&str] = &["Always", "IfNotPresent", "Never"];
const RESTART_POLICIES: &[&str] = &["Always", "OnFailure", "Never"];
const JOB_RESTART_POLICIES: &[&str] = &["OnFailure", "Never"];
const DNS_POLICIES: &[&str] = &["ClusterFirst", "ClusterFirstWithHostNet", "Default", "None"];
const PORT_PROTOCOLS: &[&str] = &["TCP", "UDP", "SCTP"];
const DEPLOYMENT_STRATEGIES: &[&str] = &["RollingUpdate", "Recreate"];
const STATEFULSET_STRATEGIES: &[&str] = &["RollingUpdate", "OnDelete"];
const POD_MANAGEMENT_POLICIES: &[&str] = &["OrderedReady", "Parallel"];
const JOB_COMPLETION_MODES: &[&str] = &["NonIndexed", "Indexed"];
const CONCURRENCY_POLICIES: &[&str] = &["Allow", "Forbid", "Replace"];

const ENV_SOURCE_TYPES: &[(EnvSourceType, &str)] = &[
    (EnvSourceType::Literal, "Literal"),
    (EnvSourceType::ConfigMapKey, "ConfigMap Key"),
    (EnvSourceType::SecretKey, "Secret Key"),
    (EnvSourceType::FieldRef, "Field Ref"),
    (EnvSourceType::ResourceFieldRef, "Resource Ref"),
    (EnvSourceType::ConfigMapEnvFrom, "ConfigMap EnvFrom"),
    (EnvSourceType::SecretEnvFrom, "Secret EnvFrom"),
];

pub fn sub_tabs_for_form(form: &EditFormModel) -> Vec<(EditFormSubTab, &'static str)> {
    match form {
        EditFormModel::Deployment(_) => vec![
            (EditFormSubTab::General, "General & Scaling"),
            (EditFormSubTab::Metadata, "Metadata"),
            (EditFormSubTab::PodSpec, "Pod Spec"),
            (EditFormSubTab::Containers, "Containers"),
        ],
        EditFormModel::StatefulSet(_) => vec![
            (EditFormSubTab::General, "General & Scaling"),
            (EditFormSubTab::Metadata, "Metadata"),
            (EditFormSubTab::PodSpec, "Pod Spec"),
            (EditFormSubTab::Containers, "Containers"),
            (EditFormSubTab::Storage, "Storage & PVCs"),
        ],
        EditFormModel::DaemonSet(_)
        | EditFormModel::ReplicaSet(_)
        | EditFormModel::Job(_)
        | EditFormModel::CronJob(_) => vec![
            (EditFormSubTab::General, "General"),
            (EditFormSubTab::Metadata, "Metadata"),
            (EditFormSubTab::PodSpec, "Pod Spec"),
            (EditFormSubTab::Containers, "Containers"),
        ],
        EditFormModel::Pod(_) => vec![
            (EditFormSubTab::Metadata, "Metadata"),
            (EditFormSubTab::PodSpec, "Pod Spec"),
            (EditFormSubTab::Containers, "Containers"),
        ],
        _ => Vec::new(),
    }
}

pub fn pod_spec_ref(form: &EditFormModel) -> Option<&PodSpecFormValues> {
    match form {
        EditFormModel::Deployment(v) => Some(&v.pod_spec),
        EditFormModel::StatefulSet(v) => Some(&v.pod_spec),
        EditFormModel::DaemonSet(v) => Some(&v.pod_spec),
        EditFormModel::ReplicaSet(v) => Some(&v.pod_spec),
        EditFormModel::Job(v) => Some(&v.pod_spec),
        EditFormModel::CronJob(v) => Some(&v.pod_spec),
        EditFormModel::Pod(v) => Some(&v.pod_spec),
        _ => None,
    }
}

pub fn pod_spec_mut(form: &mut EditFormModel) -> Option<&mut PodSpecFormValues> {
    match form {
        EditFormModel::Deployment(v) => Some(&mut v.pod_spec),
        EditFormModel::StatefulSet(v) => Some(&mut v.pod_spec),
        EditFormModel::DaemonSet(v) => Some(&mut v.pod_spec),
        EditFormModel::ReplicaSet(v) => Some(&mut v.pod_spec),
        EditFormModel::Job(v) => Some(&mut v.pod_spec),
        EditFormModel::CronJob(v) => Some(&mut v.pod_spec),
        EditFormModel::Pod(v) => Some(&mut v.pod_spec),
        _ => None,
    }
}

pub fn workload_kv_vec_mut<'a>(
    model: &'a mut EditFormModel,
    group: &str,
) -> Option<&'a mut Vec<KeyValueEntry>> {
    if group == "node_selector" {
        return pod_spec_mut(model).map(|ps| &mut ps.node_selector);
    }
    match (model, group) {
        (EditFormModel::Deployment(v), "labels") => Some(&mut v.labels),
        (EditFormModel::Deployment(v), "annotations") => Some(&mut v.annotations),
        (EditFormModel::Deployment(v), "pod_labels") => Some(&mut v.pod_labels),
        (EditFormModel::Deployment(v), "pod_annotations") => Some(&mut v.pod_annotations),

        (EditFormModel::StatefulSet(v), "labels") => Some(&mut v.labels),
        (EditFormModel::StatefulSet(v), "annotations") => Some(&mut v.annotations),
        (EditFormModel::StatefulSet(v), "pod_labels") => Some(&mut v.pod_labels),
        (EditFormModel::StatefulSet(v), "pod_annotations") => Some(&mut v.pod_annotations),

        (EditFormModel::DaemonSet(v), "labels") => Some(&mut v.labels),
        (EditFormModel::DaemonSet(v), "annotations") => Some(&mut v.annotations),
        (EditFormModel::DaemonSet(v), "pod_labels") => Some(&mut v.pod_labels),
        (EditFormModel::DaemonSet(v), "pod_annotations") => Some(&mut v.pod_annotations),

        (EditFormModel::ReplicaSet(v), "labels") => Some(&mut v.labels),
        (EditFormModel::ReplicaSet(v), "annotations") => Some(&mut v.annotations),
        (EditFormModel::ReplicaSet(v), "pod_labels") => Some(&mut v.pod_labels),
        (EditFormModel::ReplicaSet(v), "pod_annotations") => Some(&mut v.pod_annotations),

        (EditFormModel::Job(v), "labels") => Some(&mut v.labels),
        (EditFormModel::Job(v), "annotations") => Some(&mut v.annotations),
        (EditFormModel::Job(v), "pod_labels") => Some(&mut v.pod_labels),
        (EditFormModel::Job(v), "pod_annotations") => Some(&mut v.pod_annotations),

        (EditFormModel::CronJob(v), "labels") => Some(&mut v.labels),
        (EditFormModel::CronJob(v), "annotations") => Some(&mut v.annotations),
        (EditFormModel::CronJob(v), "pod_labels") => Some(&mut v.pod_labels),
        (EditFormModel::CronJob(v), "pod_annotations") => Some(&mut v.pod_annotations),

        (EditFormModel::Pod(v), "labels") => Some(&mut v.labels),
        (EditFormModel::Pod(v), "annotations") => Some(&mut v.annotations),
        _ => None,
    }
}

pub fn workload_list_vec_mut<'a>(
    model: &'a mut EditFormModel,
    group: &str,
) -> Option<&'a mut Vec<String>> {
    let rest = group.strip_prefix("container:")?;
    let mut parts = rest.split(':');
    let idx = parts.next()?.parse::<usize>().ok()?;
    let field = parts.next()?;
    let pod_spec = pod_spec_mut(model)?;
    let container = pod_spec.containers.get_mut(idx)?;
    match field {
        "command" => Some(&mut container.command),
        "args" => Some(&mut container.args),
        _ => None,
    }
}

fn push_workload_meta_and_pod_specs(
    specs: &mut Vec<(String, String, &'static str)>,
    labels: &[KeyValueEntry],
    annotations: &[KeyValueEntry],
    pod_meta: Option<(&[KeyValueEntry], &[KeyValueEntry])>,
    pod_spec: &PodSpecFormValues,
) {
    push_kv_specs(specs, "labels", labels, "Label key", "Label value");
    push_kv_specs(
        specs,
        "annotations",
        annotations,
        "Annotation key",
        "Annotation value",
    );
    if let Some((pod_labels, pod_annotations)) = pod_meta {
        push_kv_specs(specs, "pod_labels", pod_labels, "Label key", "Label value");
        push_kv_specs(
            specs,
            "pod_annotations",
            pod_annotations,
            "Annotation key",
            "Annotation value",
        );
    }
    collect_pod_spec_input_specs(pod_spec, specs);
}

pub fn collect_workload_input_specs(
    form: &EditFormModel,
    specs: &mut Vec<(String, String, &'static str)>,
) -> bool {
    match form {
        EditFormModel::Deployment(v) => {
            specs.push(("workload:replicas".to_string(), v.replicas.clone(), "1"));
            specs.push(("workload:max_surge".to_string(), v.max_surge.clone(), "25%"));
            specs.push((
                "workload:max_unavailable".to_string(),
                v.max_unavailable.clone(),
                "25%",
            ));
            specs.push((
                "workload:min_ready_seconds".to_string(),
                v.min_ready_seconds.clone(),
                "0",
            ));
            specs.push((
                "workload:revision_history_limit".to_string(),
                v.revision_history_limit.clone(),
                "10",
            ));
            specs.push((
                "workload:progress_deadline_seconds".to_string(),
                v.progress_deadline_seconds.clone(),
                "600",
            ));
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                &v.pod_spec,
            );
            true
        }
        EditFormModel::StatefulSet(v) => {
            specs.push(("workload:replicas".to_string(), v.replicas.clone(), "1"));
            specs.push((
                "workload:service_name".to_string(),
                v.service_name.clone(),
                "Headless service name",
            ));
            specs.push(("workload:partition".to_string(), v.partition.clone(), "0"));
            specs.push((
                "workload:max_unavailable".to_string(),
                v.max_unavailable.clone(),
                "1",
            ));
            specs.push((
                "workload:min_ready_seconds".to_string(),
                v.min_ready_seconds.clone(),
                "0",
            ));
            specs.push((
                "workload:revision_history_limit".to_string(),
                v.revision_history_limit.clone(),
                "10",
            ));
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                &v.pod_spec,
            );
            true
        }
        EditFormModel::DaemonSet(v) => {
            specs.push((
                "workload:max_unavailable".to_string(),
                v.max_unavailable.clone(),
                "1",
            ));
            specs.push(("workload:max_surge".to_string(), v.max_surge.clone(), "0"));
            specs.push((
                "workload:min_ready_seconds".to_string(),
                v.min_ready_seconds.clone(),
                "0",
            ));
            specs.push((
                "workload:revision_history_limit".to_string(),
                v.revision_history_limit.clone(),
                "10",
            ));
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                &v.pod_spec,
            );
            true
        }
        EditFormModel::ReplicaSet(v) => {
            specs.push(("workload:replicas".to_string(), v.replicas.clone(), "1"));
            specs.push((
                "workload:min_ready_seconds".to_string(),
                v.min_ready_seconds.clone(),
                "0",
            ));
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                &v.pod_spec,
            );
            true
        }
        EditFormModel::Job(v) => {
            specs.push(("workload:completions".to_string(), v.completions.clone(), "1"));
            specs.push(("workload:parallelism".to_string(), v.parallelism.clone(), "1"));
            specs.push((
                "workload:backoff_limit".to_string(),
                v.backoff_limit.clone(),
                "6",
            ));
            specs.push((
                "workload:active_deadline_seconds".to_string(),
                v.active_deadline_seconds.clone(),
                "Optional",
            ));
            specs.push((
                "workload:ttl_seconds_after_finished".to_string(),
                v.ttl_seconds_after_finished.clone(),
                "Optional",
            ));
            specs.push((
                "workload:backoff_limit_per_index".to_string(),
                v.backoff_limit_per_index.clone(),
                "Optional",
            ));
            specs.push((
                "workload:max_failed_indexes".to_string(),
                v.max_failed_indexes.clone(),
                "Optional",
            ));
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                &v.pod_spec,
            );
            true
        }
        EditFormModel::CronJob(v) => {
            specs.push((
                "workload:schedule".to_string(),
                v.schedule.clone(),
                "*/5 * * * *",
            ));
            specs.push((
                "workload:starting_deadline_seconds".to_string(),
                v.starting_deadline_seconds.clone(),
                "Optional",
            ));
            specs.push((
                "workload:successful_jobs_history_limit".to_string(),
                v.successful_jobs_history_limit.clone(),
                "3",
            ));
            specs.push((
                "workload:failed_jobs_history_limit".to_string(),
                v.failed_jobs_history_limit.clone(),
                "1",
            ));
            specs.push(("workload:completions".to_string(), v.completions.clone(), "Optional"));
            specs.push(("workload:parallelism".to_string(), v.parallelism.clone(), "1"));
            specs.push((
                "workload:backoff_limit".to_string(),
                v.backoff_limit.clone(),
                "6",
            ));
            specs.push((
                "workload:active_deadline_seconds".to_string(),
                v.active_deadline_seconds.clone(),
                "Optional",
            ));
            specs.push((
                "workload:ttl_seconds_after_finished".to_string(),
                v.ttl_seconds_after_finished.clone(),
                "Optional",
            ));
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                &v.pod_spec,
            );
            true
        }
        EditFormModel::Pod(v) => {
            push_workload_meta_and_pod_specs(
                specs,
                &v.labels,
                &v.annotations,
                None,
                &v.pod_spec,
            );
            true
        }
        _ => false,
    }
}

fn collect_pod_spec_input_specs(
    pod_spec: &PodSpecFormValues,
    specs: &mut Vec<(String, String, &'static str)>,
) {
    specs.push((
        "podspec:service_account_name".to_string(),
        pod_spec.service_account_name.clone(),
        "default",
    ));
    specs.push((
        "podspec:termination_grace_period_seconds".to_string(),
        pod_spec.termination_grace_period_seconds.clone(),
        "30",
    ));
    push_kv_specs(
        specs,
        "node_selector",
        &pod_spec.node_selector,
        "Node label key",
        "Node label value",
    );

    for (c_idx, c) in pod_spec.containers.iter().enumerate() {
        specs.push((
            format!("container:{c_idx}:name"),
            c.name.clone(),
            "Container name",
        ));
        specs.push((
            format!("container:{c_idx}:image"),
            c.image.clone(),
            "e.g. nginx:latest",
        ));
        specs.push((
            format!("container:{c_idx}:working_dir"),
            c.working_dir.clone(),
            "/app",
        ));
        specs.push((
            format!("container:{c_idx}:cpu_request"),
            c.resources.cpu_request.clone(),
            "e.g. 100m",
        ));
        specs.push((
            format!("container:{c_idx}:memory_request"),
            c.resources.memory_request.clone(),
            "e.g. 128Mi",
        ));
        specs.push((
            format!("container:{c_idx}:cpu_limit"),
            c.resources.cpu_limit.clone(),
            "e.g. 500m",
        ));
        specs.push((
            format!("container:{c_idx}:memory_limit"),
            c.resources.memory_limit.clone(),
            "e.g. 512Mi",
        ));

        let cmd_group = format!("container:{c_idx}:command");
        push_list_specs(specs, &cmd_group, &c.command, "e.g. /bin/sh");
        let args_group = format!("container:{c_idx}:args");
        push_list_specs(specs, &args_group, &c.args, "e.g. -c");

        for (e_idx, env) in c.environments.iter().enumerate() {
            specs.push((
                format!("env:{c_idx}:{e_idx}:name"),
                env.name.clone(),
                "ENV_VAR_NAME",
            ));
            specs.push((
                format!("env:{c_idx}:{e_idx}:value"),
                env.value.clone(),
                "Value",
            ));
            specs.push((
                format!("env:{c_idx}:{e_idx}:ref_name"),
                env.ref_name.clone(),
                "Resource / container name",
            ));
            specs.push((
                format!("env:{c_idx}:{e_idx}:ref_key"),
                env.ref_key.clone(),
                "Key / resource",
            ));
            specs.push((
                format!("env:{c_idx}:{e_idx}:field_path"),
                env.field_path.clone(),
                "e.g. metadata.name",
            ));
            specs.push((
                format!("env:{c_idx}:{e_idx}:prefix"),
                env.prefix.clone(),
                "Optional prefix",
            ));
        }

        for (p_idx, port) in c.ports.iter().enumerate() {
            specs.push((
                format!("port:{c_idx}:{p_idx}:name"),
                port.name.clone(),
                "e.g. http",
            ));
            specs.push((
                format!("port:{c_idx}:{p_idx}:container_port"),
                port.container_port.clone(),
                "80",
            ));
        }
    }
}

pub fn apply_workload_input_value(
    form: &mut EditFormModel,
    field_id: &str,
    value: String,
) -> bool {
    if let Some(rest) = field_id.strip_prefix("workload:") {
        return match (form, rest) {
            (EditFormModel::Deployment(v), "replicas") => {
                v.replicas = value;
                true
            }
            (EditFormModel::Deployment(v), "max_surge") => {
                v.max_surge = value;
                true
            }
            (EditFormModel::Deployment(v), "max_unavailable") => {
                v.max_unavailable = value;
                true
            }
            (EditFormModel::Deployment(v), "min_ready_seconds") => {
                v.min_ready_seconds = value;
                true
            }
            (EditFormModel::Deployment(v), "revision_history_limit") => {
                v.revision_history_limit = value;
                true
            }
            (EditFormModel::Deployment(v), "progress_deadline_seconds") => {
                v.progress_deadline_seconds = value;
                true
            }

            (EditFormModel::StatefulSet(v), "replicas") => {
                v.replicas = value;
                true
            }
            (EditFormModel::StatefulSet(v), "service_name") => {
                v.service_name = value;
                true
            }
            (EditFormModel::StatefulSet(v), "partition") => {
                v.partition = value;
                true
            }
            (EditFormModel::StatefulSet(v), "max_unavailable") => {
                v.max_unavailable = value;
                true
            }
            (EditFormModel::StatefulSet(v), "min_ready_seconds") => {
                v.min_ready_seconds = value;
                true
            }
            (EditFormModel::StatefulSet(v), "revision_history_limit") => {
                v.revision_history_limit = value;
                true
            }

            (EditFormModel::DaemonSet(v), "max_unavailable") => {
                v.max_unavailable = value;
                true
            }
            (EditFormModel::DaemonSet(v), "max_surge") => {
                v.max_surge = value;
                true
            }
            (EditFormModel::DaemonSet(v), "min_ready_seconds") => {
                v.min_ready_seconds = value;
                true
            }
            (EditFormModel::DaemonSet(v), "revision_history_limit") => {
                v.revision_history_limit = value;
                true
            }

            (EditFormModel::ReplicaSet(v), "replicas") => {
                v.replicas = value;
                true
            }
            (EditFormModel::ReplicaSet(v), "min_ready_seconds") => {
                v.min_ready_seconds = value;
                true
            }

            (EditFormModel::Job(v), "completions") => {
                v.completions = value;
                true
            }
            (EditFormModel::Job(v), "parallelism") => {
                v.parallelism = value;
                true
            }
            (EditFormModel::Job(v), "backoff_limit") => {
                v.backoff_limit = value;
                true
            }
            (EditFormModel::Job(v), "active_deadline_seconds") => {
                v.active_deadline_seconds = value;
                true
            }
            (EditFormModel::Job(v), "ttl_seconds_after_finished") => {
                v.ttl_seconds_after_finished = value;
                true
            }
            (EditFormModel::Job(v), "backoff_limit_per_index") => {
                v.backoff_limit_per_index = value;
                true
            }
            (EditFormModel::Job(v), "max_failed_indexes") => {
                v.max_failed_indexes = value;
                true
            }

            (EditFormModel::CronJob(v), "schedule") => {
                v.schedule = value;
                true
            }
            (EditFormModel::CronJob(v), "starting_deadline_seconds") => {
                v.starting_deadline_seconds = value;
                true
            }
            (EditFormModel::CronJob(v), "successful_jobs_history_limit") => {
                v.successful_jobs_history_limit = value;
                true
            }
            (EditFormModel::CronJob(v), "failed_jobs_history_limit") => {
                v.failed_jobs_history_limit = value;
                true
            }
            (EditFormModel::CronJob(v), "completions") => {
                v.completions = value;
                true
            }
            (EditFormModel::CronJob(v), "parallelism") => {
                v.parallelism = value;
                true
            }
            (EditFormModel::CronJob(v), "backoff_limit") => {
                v.backoff_limit = value;
                true
            }
            (EditFormModel::CronJob(v), "active_deadline_seconds") => {
                v.active_deadline_seconds = value;
                true
            }
            (EditFormModel::CronJob(v), "ttl_seconds_after_finished") => {
                v.ttl_seconds_after_finished = value;
                true
            }
            _ => false,
        };
    }

    let Some(pod_spec) = pod_spec_mut(form) else {
        return false;
    };

    match field_id {
        "podspec:service_account_name" => {
            pod_spec.service_account_name = value;
            return true;
        }
        "podspec:termination_grace_period_seconds" => {
            pod_spec.termination_grace_period_seconds = value;
            return true;
        }
        _ => {}
    }

    if let Some(rest) = field_id.strip_prefix("container:") {
        let mut parts = rest.split(':');
        if let (Some(c_idx), Some(field)) = (
            parts.next().and_then(|s| s.parse::<usize>().ok()),
            parts.next(),
        ) && let Some(c) = pod_spec.containers.get_mut(c_idx)
        {
            match field {
                "name" => c.name = value,
                "image" => c.image = value,
                "working_dir" => c.working_dir = value,
                "cpu_request" => c.resources.cpu_request = value,
                "memory_request" => c.resources.memory_request = value,
                "cpu_limit" => c.resources.cpu_limit = value,
                "memory_limit" => c.resources.memory_limit = value,
                _ => return false,
            }
            return true;
        }
        return false;
    }

    if let Some(rest) = field_id.strip_prefix("env:") {
        let mut parts = rest.split(':');
        if let (Some(c_idx), Some(e_idx), Some(field)) = (
            parts.next().and_then(|s| s.parse::<usize>().ok()),
            parts.next().and_then(|s| s.parse::<usize>().ok()),
            parts.next(),
        ) && let Some(c) = pod_spec.containers.get_mut(c_idx)
            && let Some(env) = c.environments.get_mut(e_idx)
        {
            match field {
                "name" => env.name = value,
                "value" => env.value = value,
                "ref_name" => env.ref_name = value,
                "ref_key" => env.ref_key = value,
                "field_path" => env.field_path = value,
                "prefix" => env.prefix = value,
                _ => return false,
            }
            return true;
        }
        return false;
    }

    if let Some(rest) = field_id.strip_prefix("port:") {
        let mut parts = rest.split(':');
        if let (Some(c_idx), Some(p_idx), Some(field)) = (
            parts.next().and_then(|s| s.parse::<usize>().ok()),
            parts.next().and_then(|s| s.parse::<usize>().ok()),
            parts.next(),
        ) && let Some(c) = pod_spec.containers.get_mut(c_idx)
            && let Some(port) = c.ports.get_mut(p_idx)
        {
            match field {
                "name" => port.name = value,
                "container_port" => port.container_port = value,
                _ => return false,
            }
            return true;
        }
        return false;
    }

    false
}

pub fn render_workload_form_body(
    session: &EditSessionState,
    form_model: &EditFormModel,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> Option<AnyElement> {
    let tabs = sub_tabs_for_form(form_model);
    if tabs.is_empty() {
        return None;
    }

    let active_tab = if tabs.iter().any(|(t, _)| *t == session.active_sub_tab) {
        session.active_sub_tab
    } else {
        tabs[0].0
    };

    let tab_bar = render_workload_sub_tab_bar(active_tab, &tabs, theme, callbacks);
    let body = match active_tab {
        EditFormSubTab::General => match form_model {
            EditFormModel::Deployment(v) => {
                render_deployment_general(v, inputs, disabled, theme, callbacks)
            }
            EditFormModel::StatefulSet(v) => {
                render_statefulset_general(v, inputs, disabled, theme, callbacks)
            }
            EditFormModel::DaemonSet(v) => {
                render_daemonset_general(v, inputs, disabled, theme, callbacks)
            }
            EditFormModel::ReplicaSet(v) => render_replicaset_general(v, inputs, disabled, theme),
            EditFormModel::Job(v) => render_job_general(v, inputs, disabled, theme, callbacks),
            EditFormModel::CronJob(v) => {
                render_cronjob_general(v, inputs, disabled, theme, callbacks)
            }
            _ => div().into_any_element(),
        },
        EditFormSubTab::Storage => match form_model {
            EditFormModel::StatefulSet(v) => render_statefulset_storage(v, theme),
            _ => div().into_any_element(),
        },
        EditFormSubTab::Metadata => match form_model {
            EditFormModel::Deployment(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            EditFormModel::StatefulSet(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            EditFormModel::DaemonSet(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            EditFormModel::ReplicaSet(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            EditFormModel::Job(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            EditFormModel::CronJob(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                Some((&v.pod_labels, &v.pod_annotations)),
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            EditFormModel::Pod(v) => render_workload_metadata(
                &v.labels,
                &v.annotations,
                None,
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            _ => div().into_any_element(),
        },
        EditFormSubTab::PodSpec => {
            let restart_policies = match form_model {
                EditFormModel::Job(_) | EditFormModel::CronJob(_) => JOB_RESTART_POLICIES,
                _ => RESTART_POLICIES,
            };
            match pod_spec_ref(form_model) {
                Some(ps) => {
                    render_pod_spec_panel(ps, restart_policies, inputs, disabled, theme, callbacks)
                }
                None => div().into_any_element(),
            }
        }
        EditFormSubTab::Containers => match pod_spec_ref(form_model) {
            Some(ps) => render_containers_panel(
                session.active_container_index,
                &ps.containers,
                inputs,
                disabled,
                theme,
                callbacks,
            ),
            None => div().into_any_element(),
        },
        _ => div().into_any_element(),
    };

    Some(
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(tab_bar)
            .child(body)
            .into_any_element(),
    )
}

fn render_workload_sub_tab_bar(
    active: EditFormSubTab,
    tabs: &[(EditFormSubTab, &'static str)],
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let mut bar = div()
        .flex()
        .items_center()
        .flex_wrap()
        .gap_2()
        .pb_2()
        .border_b_1()
        .border_color(theme.border.default);

    for (idx, (tab, label)) in tabs.iter().enumerate() {
        let tab_val = *tab;
        let is_active = active == tab_val;
        let cb = Rc::clone(&callbacks.on_select_sub_tab);
        let mut btn = Button::new(SharedString::from(format!("workload-subtab-{idx}")))
            .small()
            .on_click(move |_, window, cx| cb(tab_val, window, cx))
            .child(*label);
        if is_active {
            btn = btn.primary();
        } else {
            btn = btn.ghost();
        }
        bar = bar.child(btn);
    }

    bar.into_any_element()
}

fn render_option_pills<F>(
    id_prefix: &'static str,
    options: &[&'static str],
    current: &str,
    disabled: bool,
    on_mutate: &EditFormMutatorCallback,
    make_updater: F,
) -> AnyElement
where
    F: Fn(&'static str, &mut EditFormModel) + 'static,
{
    let updater = Rc::new(make_updater);
    let mut row = div().flex().items_center().flex_wrap().gap_1p5();
    for (idx, &opt) in options.iter().enumerate() {
        let is_active = current == opt;
        let cb = Rc::clone(on_mutate);
        let upd = Rc::clone(&updater);
        let mut btn = Button::new(SharedString::from(format!("{id_prefix}-{idx}")))
            .xsmall()
            .disabled(disabled)
            .on_click(move |_, window, cx| {
                let upd = Rc::clone(&upd);
                cb(
                    Box::new(move |m| upd(opt, m)),
                    false,
                    window,
                    cx,
                );
            })
            .child(opt);
        if is_active {
            btn = btn.primary();
        } else {
            btn = btn.outline();
        }
        row = row.child(btn);
    }
    row.into_any_element()
}

fn render_deployment_general(
    v: &DeploymentFormValues,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let pause_cb = Rc::clone(&callbacks.on_mutate_form);
    let scaling = render_two_col_section(
        "Scaling & Rollout",
        "Replica count, rollout pause state, and update strategy.",
        theme,
        v_form()
            .child(
                field().label("Replicas").child(render_input_or_placeholder(
                    inputs,
                    "workload:replicas",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Paused").child(
                    Switch::new("deployment-paused-switch")
                        .checked(v.paused)
                        .disabled(disabled)
                        .on_click(move |checked, window, cx| {
                            let next = *checked;
                            pause_cb(
                                Box::new(move |m| {
                                    if let EditFormModel::Deployment(dep) = m {
                                        dep.paused = next;
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        }),
                ),
            )
            .child(
                field().label("Strategy Type").child(render_option_pills(
                    "dep-strategy",
                    DEPLOYMENT_STRATEGIES,
                    &v.strategy_type,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let EditFormModel::Deployment(dep) = m {
                            dep.strategy_type = opt.to_string();
                        }
                    },
                )),
            )
            .when(v.strategy_type == "RollingUpdate", |form| {
                form.child(
                    field().label("Max Surge").child(render_input_or_placeholder(
                        inputs,
                        "workload:max_surge",
                        disabled,
                        theme,
                    )),
                )
                .child(
                    field().label("Max Unavailable").child(render_input_or_placeholder(
                        inputs,
                        "workload:max_unavailable",
                        disabled,
                        theme,
                    )),
                )
            })
            .child(
                field().label("Min Ready Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:min_ready_seconds",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Revision History Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:revision_history_limit",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Progress Deadline Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:progress_deadline_seconds",
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    );

    let selector_rows: Vec<(String, String)> = v
        .selector_labels
        .iter()
        .map(|kv| (kv.key.clone(), kv.value.clone()))
        .collect();
    let selector_refs: Vec<(&str, &str)> = selector_rows
        .iter()
        .map(|(k, val)| (k.as_str(), val.as_str()))
        .collect();

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(scaling)
        .when(!selector_refs.is_empty(), |el| {
            el.child(render_two_col_section(
                "Label Selector",
                "Immutable matchLabels selector for this Deployment.",
                theme,
                render_readonly_fields(&selector_refs, theme),
            ))
        })
        .into_any_element()
}

fn render_statefulset_general(
    v: &StatefulSetFormValues,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    render_two_col_section(
        "Scaling & Update Strategy",
        "StatefulSet replicas, headless service name, and update strategy.",
        theme,
        v_form()
            .child(
                field().label("Replicas").child(render_input_or_placeholder(
                    inputs,
                    "workload:replicas",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Service Name").required(true).child(
                    render_input_or_placeholder(inputs, "workload:service_name", disabled, theme),
                ),
            )
            .child(
                field().label("Pod Management Policy").child(render_option_pills(
                    "sts-pod-mgmt",
                    POD_MANAGEMENT_POLICIES,
                    &v.pod_management_policy,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let EditFormModel::StatefulSet(sts) = m {
                            sts.pod_management_policy = opt.to_string();
                        }
                    },
                )),
            )
            .child(
                field().label("Update Strategy").child(render_option_pills(
                    "sts-strategy",
                    STATEFULSET_STRATEGIES,
                    &v.strategy_type,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let EditFormModel::StatefulSet(sts) = m {
                            sts.strategy_type = opt.to_string();
                        }
                    },
                )),
            )
            .when(v.strategy_type == "RollingUpdate", |form| {
                form.child(
                    field().label("Partition").child(render_input_or_placeholder(
                        inputs,
                        "workload:partition",
                        disabled,
                        theme,
                    )),
                )
                .child(
                    field().label("Max Unavailable").child(render_input_or_placeholder(
                        inputs,
                        "workload:max_unavailable",
                        disabled,
                        theme,
                    )),
                )
            })
            .child(
                field().label("Min Ready Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:min_ready_seconds",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Revision History Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:revision_history_limit",
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    )
}

fn render_statefulset_storage(v: &StatefulSetFormValues, theme: &OrbitTheme) -> AnyElement {
    let mut rows: Vec<(&str, &str)> = Vec::new();
    if !v.pvc_retention_when_deleted.is_empty() {
        rows.push(("When Deleted", v.pvc_retention_when_deleted.as_str()));
    }
    if !v.pvc_retention_when_scaled.is_empty() {
        rows.push(("When Scaled", v.pvc_retention_when_scaled.as_str()));
    }

    let claims_list = if v.volume_claim_summaries.is_empty() {
        apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No volumeClaimTemplates configured on this StatefulSet.")
            .into_any_element()
    } else {
        let mut col = div().flex().flex_col().gap_2();
        for summary in &v.volume_claim_summaries {
            col = col.child(
                apply_text_step(div(), theme.typography.size_xs)
                    .px_3()
                    .py_2()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.bg_panel)
                    .font(theme.typography.mono_font(FontWeight::NORMAL))
                    .text_color(theme.text.primary)
                    .child(summary.clone()),
            );
        }
        col.into_any_element()
    };

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(render_two_col_section(
            "Volume Claim Templates",
            "Immutable PVC templates provisioned for each StatefulSet pod ordinal.",
            theme,
            claims_list,
        ))
        .when(!rows.is_empty(), |el| {
            el.child(render_two_col_section(
                "PVC Retention Policy",
                "Lifecycle retention policy when pods are scaled down or deleted.",
                theme,
                render_readonly_fields(&rows, theme),
            ))
        })
        .into_any_element()
}

fn render_daemonset_general(
    v: &DaemonSetFormValues,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    render_two_col_section(
        "Update Strategy",
        "DaemonSet node update strategy and rollout limits.",
        theme,
        v_form()
            .child(
                field().label("Update Strategy").child(render_option_pills(
                    "ds-strategy",
                    STATEFULSET_STRATEGIES,
                    &v.strategy_type,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let EditFormModel::DaemonSet(ds) = m {
                            ds.strategy_type = opt.to_string();
                        }
                    },
                )),
            )
            .when(v.strategy_type == "RollingUpdate", |form| {
                form.child(
                    field().label("Max Unavailable").child(render_input_or_placeholder(
                        inputs,
                        "workload:max_unavailable",
                        disabled,
                        theme,
                    )),
                )
                .child(
                    field().label("Max Surge").child(render_input_or_placeholder(
                        inputs,
                        "workload:max_surge",
                        disabled,
                        theme,
                    )),
                )
            })
            .child(
                field().label("Min Ready Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:min_ready_seconds",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Revision History Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:revision_history_limit",
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    )
}

fn render_replicaset_general(
    _v: &ReplicaSetFormValues,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_two_col_section(
        "Scaling",
        "Replica count and readiness delay for this ReplicaSet.",
        theme,
        v_form()
            .child(
                field().label("Replicas").child(render_input_or_placeholder(
                    inputs,
                    "workload:replicas",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Min Ready Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:min_ready_seconds",
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    )
}

fn render_job_general(
    v: &JobFormValues,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let suspend_cb = Rc::clone(&callbacks.on_mutate_form);
    render_two_col_section(
        "Job Execution",
        "Parallelism, completion mode, retry limits, and TTL.",
        theme,
        v_form()
            .child(
                field().label("Completion Mode").child(render_option_pills(
                    "job-completion-mode",
                    JOB_COMPLETION_MODES,
                    &v.completion_mode,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let EditFormModel::Job(job) = m {
                            job.completion_mode = opt.to_string();
                        }
                    },
                )),
            )
            .child(
                field().label("Completions").child(render_input_or_placeholder(
                    inputs,
                    "workload:completions",
                    disabled,
                    theme,
                )),
            )
            .when(v.completion_mode == "Indexed", |form| {
                form.child(
                    field().label("Backoff Limit Per Index").child(render_input_or_placeholder(
                        inputs,
                        "workload:backoff_limit_per_index",
                        disabled,
                        theme,
                    )),
                )
                .child(
                    field().label("Max Failed Indexes").child(render_input_or_placeholder(
                        inputs,
                        "workload:max_failed_indexes",
                        disabled,
                        theme,
                    )),
                )
            })
            .child(
                field().label("Parallelism").child(render_input_or_placeholder(
                    inputs,
                    "workload:parallelism",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Backoff Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:backoff_limit",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Active Deadline Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:active_deadline_seconds",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("TTL Seconds After Finished").child(render_input_or_placeholder(
                    inputs,
                    "workload:ttl_seconds_after_finished",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Suspend").child(
                    Switch::new("job-suspend-switch")
                        .checked(v.suspend)
                        .disabled(disabled)
                        .on_click(move |checked, window, cx| {
                            let next = *checked;
                            suspend_cb(
                                Box::new(move |m| {
                                    if let EditFormModel::Job(job) = m {
                                        job.suspend = next;
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        }),
                ),
            )
            .into_any_element(),
    )
}

fn render_cronjob_general(
    v: &CronJobFormValues,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let suspend_cb = Rc::clone(&callbacks.on_mutate_form);
    let schedule_sec = render_two_col_section(
        "Schedule & Concurrency",
        "Cron schedule, concurrency policy, and job history limits.",
        theme,
        v_form()
            .child(
                field().label("Schedule").required(true).child(
                    render_input_or_placeholder(inputs, "workload:schedule", disabled, theme),
                ),
            )
            .child(
                field().label("Concurrency Policy").child(render_option_pills(
                    "cj-concurrency",
                    CONCURRENCY_POLICIES,
                    &v.concurrency_policy,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let EditFormModel::CronJob(cj) = m {
                            cj.concurrency_policy = opt.to_string();
                        }
                    },
                )),
            )
            .child(
                field().label("Suspend").child(
                    Switch::new("cronjob-suspend-switch")
                        .checked(v.suspend)
                        .disabled(disabled)
                        .on_click(move |checked, window, cx| {
                            let next = *checked;
                            suspend_cb(
                                Box::new(move |m| {
                                    if let EditFormModel::CronJob(cj) = m {
                                        cj.suspend = next;
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        }),
                ),
            )
            .child(
                field().label("Starting Deadline Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:starting_deadline_seconds",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Successful Jobs History Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:successful_jobs_history_limit",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Failed Jobs History Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:failed_jobs_history_limit",
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    );

    let job_template_sec = render_two_col_section(
        "Job Template Execution",
        "Execution limits applied to each scheduled Job.",
        theme,
        v_form()
            .child(
                field().label("Completions").child(render_input_or_placeholder(
                    inputs,
                    "workload:completions",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Parallelism").child(render_input_or_placeholder(
                    inputs,
                    "workload:parallelism",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Backoff Limit").child(render_input_or_placeholder(
                    inputs,
                    "workload:backoff_limit",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Active Deadline Seconds").child(render_input_or_placeholder(
                    inputs,
                    "workload:active_deadline_seconds",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("TTL Seconds After Finished").child(render_input_or_placeholder(
                    inputs,
                    "workload:ttl_seconds_after_finished",
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    );

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(schedule_sec)
        .child(job_template_sec)
        .into_any_element()
}

fn render_workload_metadata(
    labels: &[KeyValueEntry],
    annotations: &[KeyValueEntry],
    pod_template_meta: Option<(&[KeyValueEntry], &[KeyValueEntry])>,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let resource_meta = render_metadata_kv_sections(
        labels,
        annotations,
        inputs,
        disabled,
        theme,
        &callbacks.on_mutate_form,
    );

    let Some((pod_labels, pod_annotations)) = pod_template_meta else {
        return resource_meta;
    };

    let template_meta = render_two_col_section(
        "Pod Template Metadata",
        "Labels and annotations applied to pods created by this workload.",
        theme,
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(render_kv_editor_section(
                "pod_labels",
                "Pod Labels",
                "Add Pod Label",
                pod_labels,
                inputs,
                disabled,
                theme,
                &callbacks.on_mutate_form,
            ))
            .child(render_kv_editor_section(
                "pod_annotations",
                "Pod Annotations",
                "Add Pod Annotation",
                pod_annotations,
                inputs,
                disabled,
                theme,
                &callbacks.on_mutate_form,
            ))
            .into_any_element(),
    );

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(resource_meta)
        .child(template_meta)
        .into_any_element()
}

fn render_pod_spec_panel(
    pod_spec: &PodSpecFormValues,
    restart_policies: &[&'static str],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    let host_net_cb = Rc::clone(&callbacks.on_mutate_form);
    let host_pid_cb = Rc::clone(&callbacks.on_mutate_form);
    let host_ipc_cb = Rc::clone(&callbacks.on_mutate_form);

    let identity = render_two_col_section(
        "Pod Identity & Networking",
        "Service account, restart policy, DNS policy, and host namespaces.",
        theme,
        v_form()
            .child(
                field().label("Service Account").child(render_input_or_placeholder(
                    inputs,
                    "podspec:service_account_name",
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Restart Policy").child(render_option_pills(
                    "podspec-restart",
                    restart_policies,
                    &pod_spec.restart_policy,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let Some(ps) = pod_spec_mut(m) {
                            ps.restart_policy = opt.to_string();
                        }
                    },
                )),
            )
            .child(
                field()
                    .label("Termination Grace Period (s)")
                    .child(render_input_or_placeholder(
                        inputs,
                        "podspec:termination_grace_period_seconds",
                        disabled,
                        theme,
                    )),
            )
            .child(
                field().label("DNS Policy").child(render_option_pills(
                    "podspec-dns",
                    DNS_POLICIES,
                    &pod_spec.dns_policy,
                    disabled,
                    &callbacks.on_mutate_form,
                    |opt, m| {
                        if let Some(ps) = pod_spec_mut(m) {
                            ps.dns_policy = opt.to_string();
                        }
                    },
                )),
            )
            .child(
                field().label("Host Network").child(
                    Switch::new("podspec-host-net")
                        .checked(pod_spec.host_network)
                        .disabled(disabled)
                        .on_click(move |checked, window, cx| {
                            let next = *checked;
                            host_net_cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m) {
                                        ps.host_network = next;
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        }),
                ),
            )
            .child(
                field().label("Host PID").child(
                    Switch::new("podspec-host-pid")
                        .checked(pod_spec.host_pid)
                        .disabled(disabled)
                        .on_click(move |checked, window, cx| {
                            let next = *checked;
                            host_pid_cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m) {
                                        ps.host_pid = next;
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        }),
                ),
            )
            .child(
                field().label("Host IPC").child(
                    Switch::new("podspec-host-ipc")
                        .checked(pod_spec.host_ipc)
                        .disabled(disabled)
                        .on_click(move |checked, window, cx| {
                            let next = *checked;
                            host_ipc_cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m) {
                                        ps.host_ipc = next;
                                    }
                                }),
                                false,
                                window,
                                cx,
                            );
                        }),
                ),
            )
            .into_any_element(),
    );

    let node_selector = render_two_col_section(
        "Node Selector",
        "Key-value node labels required to schedule pods.",
        theme,
        render_kv_editor_section(
            "node_selector",
            "Node Selector Labels",
            "Add Selector",
            &pod_spec.node_selector,
            inputs,
            disabled,
            theme,
            &callbacks.on_mutate_form,
        ),
    );

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(identity)
        .child(node_selector)
        .into_any_element()
}

fn render_containers_panel(
    active_container_index: usize,
    containers: &[ContainerFormValues],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    callbacks: &EditViewCallbacks,
) -> AnyElement {
    if containers.is_empty() {
        return apply_text_step(div(), theme.typography.size_xs)
            .text_color(theme.text.muted)
            .child("No containers defined in this pod spec.")
            .into_any_element();
    }

    let c_idx = active_container_index.min(containers.len().saturating_sub(1));
    let container = &containers[c_idx];

    let mut selector = div().flex().items_center().flex_wrap().gap_2();
    for (idx, c) in containers.iter().enumerate() {
        let is_active = idx == c_idx;
        let cb = Rc::clone(&callbacks.on_select_container);
        let label = if c.name.trim().is_empty() {
            format!("Container {}", idx + 1)
        } else {
            c.name.clone()
        };
        let mut btn = Button::new(SharedString::from(format!("container-select-{idx}")))
            .small()
            .on_click(move |_, window, cx| cb(idx, window, cx))
            .child(label);
        if is_active {
            btn = btn.primary();
        } else {
            btn = btn.outline();
        }
        selector = selector.child(btn);
    }

    let cmd_group = format!("container:{c_idx}:command");
    let args_group = format!("container:{c_idx}:args");

    let general = render_two_col_section(
        "Container Configuration",
        "Container name, image, pull policy, working directory, and entrypoint.",
        theme,
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                v_form()
                    .child(
                        field().label("Container Name").required(true).child(
                            render_input_or_placeholder(
                                inputs,
                                &format!("container:{c_idx}:name"),
                                disabled,
                                theme,
                            ),
                        ),
                    )
                    .child(
                        field().label("Image").required(true).child(
                            render_input_or_placeholder(
                                inputs,
                                &format!("container:{c_idx}:image"),
                                disabled,
                                theme,
                            ),
                        ),
                    )
                    .child(
                        field().label("Image Pull Policy").child(render_option_pills(
                            "container-pull-policy",
                            IMAGE_PULL_POLICIES,
                            &container.image_pull_policy,
                            disabled,
                            &callbacks.on_mutate_form,
                            move |opt, m| {
                                if let Some(ps) = pod_spec_mut(m)
                                    && let Some(c) = ps.containers.get_mut(c_idx)
                                {
                                    c.image_pull_policy = opt.to_string();
                                }
                            },
                        )),
                    )
                    .child(field().label("Working Directory").child(
                        render_input_or_placeholder(
                            inputs,
                            &format!("container:{c_idx}:working_dir"),
                            disabled,
                            theme,
                        ),
                    )),
            )
            .child(render_string_list_section(
                &cmd_group,
                "Command",
                "Add Command Arg",
                &container.command,
                inputs,
                disabled,
                theme,
                &callbacks.on_mutate_form,
            ))
            .child(render_string_list_section(
                &args_group,
                "Args",
                "Add Arg",
                &container.args,
                inputs,
                disabled,
                theme,
                &callbacks.on_mutate_form,
            ))
            .into_any_element(),
    );

    let resources = render_container_resources_editor(c_idx, inputs, disabled, theme);
    let env_editor = render_container_env_editor(
        c_idx,
        &container.environments,
        inputs,
        disabled,
        theme,
        &callbacks.on_mutate_form,
    );
    let ports_editor = render_container_ports_editor(
        c_idx,
        &container.ports,
        inputs,
        disabled,
        theme,
        &callbacks.on_mutate_form,
    );

    div()
        .flex()
        .flex_col()
        .gap_6()
        .when(containers.len() > 1, |el| el.child(selector))
        .child(general)
        .child(resources)
        .child(env_editor)
        .child(ports_editor)
        .into_any_element()
}

pub fn render_container_resources_editor(
    c_idx: usize,
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
) -> AnyElement {
    render_two_col_section(
        "Compute Resources",
        "CPU and memory requests and limits for this container.",
        theme,
        v_form()
            .child(
                field().label("CPU Request").child(render_input_or_placeholder(
                    inputs,
                    &format!("container:{c_idx}:cpu_request"),
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Memory Request").child(render_input_or_placeholder(
                    inputs,
                    &format!("container:{c_idx}:memory_request"),
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("CPU Limit").child(render_input_or_placeholder(
                    inputs,
                    &format!("container:{c_idx}:cpu_limit"),
                    disabled,
                    theme,
                )),
            )
            .child(
                field().label("Memory Limit").child(render_input_or_placeholder(
                    inputs,
                    &format!("container:{c_idx}:memory_limit"),
                    disabled,
                    theme,
                )),
            )
            .into_any_element(),
    )
}

pub fn render_container_env_editor(
    c_idx: usize,
    environments: &[ContainerEnvEntry],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    on_mutate_form: &EditFormMutatorCallback,
) -> AnyElement {
    let mut rows = div().flex().flex_col().gap_3();
    if environments.is_empty() {
        rows = rows.child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No environment variables configured."),
        );
    } else {
        for (e_idx, env) in environments.iter().enumerate() {
            let mut source_pills = div().flex().items_center().flex_wrap().gap_1();
            for (src_idx, (src_type, src_label)) in ENV_SOURCE_TYPES.iter().enumerate() {
                let is_active = env.source_type == *src_type;
                let next_src = *src_type;
                let cb = Rc::clone(on_mutate_form);
                let mut btn =
                    Button::new(SharedString::from(format!("env-src-{c_idx}-{e_idx}-{src_idx}")))
                        .xsmall()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m)
                                        && let Some(c) = ps.containers.get_mut(c_idx)
                                        && let Some(entry) = c.environments.get_mut(e_idx)
                                    {
                                        entry.source_type = next_src;
                                    }
                                }),
                                true,
                                window,
                                cx,
                            );
                        })
                        .child(*src_label);
                if is_active {
                    btn = btn.primary();
                } else {
                    btn = btn.ghost();
                }
                source_pills = source_pills.child(btn);
            }

            let remove_cb = Rc::clone(on_mutate_form);
            let header_row = div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(source_pills)
                .child(
                    Button::new(SharedString::from(format!("env-remove-{c_idx}-{e_idx}")))
                        .ghost()
                        .small()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            remove_cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m)
                                        && let Some(c) = ps.containers.get_mut(c_idx)
                                        && e_idx < c.environments.len()
                                    {
                                        c.environments.remove(e_idx);
                                    }
                                }),
                                true,
                                window,
                                cx,
                            );
                        })
                        .child(render_icon(Icon::Trash2, px(13.0), theme.text.muted)),
                );

            let fields_row = match env.source_type {
                EnvSourceType::Literal => div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:name"),
                        disabled,
                        theme,
                    )))
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:value"),
                        disabled,
                        theme,
                    ))),
                EnvSourceType::ConfigMapKey | EnvSourceType::SecretKey | EnvSourceType::ResourceFieldRef => div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:name"),
                        disabled,
                        theme,
                    )))
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:ref_name"),
                        disabled,
                        theme,
                    )))
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:ref_key"),
                        disabled,
                        theme,
                    ))),
                EnvSourceType::FieldRef => div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:name"),
                        disabled,
                        theme,
                    )))
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:field_path"),
                        disabled,
                        theme,
                    ))),
                EnvSourceType::ConfigMapEnvFrom | EnvSourceType::SecretEnvFrom => div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:ref_name"),
                        disabled,
                        theme,
                    )))
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("env:{c_idx}:{e_idx}:prefix"),
                        disabled,
                        theme,
                    ))),
            };

            rows = rows.child(
                div()
                    .w_full()
                    .p_3()
                    .rounded(theme.style.radius_control)
                    .bg(theme.surface.bg_app)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(header_row)
                    .child(fields_row),
            );
        }
    }

    let add_cb = Rc::clone(on_mutate_form);
    let panel = div()
        .w_full()
        .p_4()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child("Environment Variables"),
                )
                .child(
                    Button::new(SharedString::from(format!("env-add-{c_idx}")))
                        .outline()
                        .small()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            add_cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m)
                                        && let Some(c) = ps.containers.get_mut(c_idx)
                                    {
                                        c.environments.push(ContainerEnvEntry::default());
                                    }
                                }),
                                true,
                                window,
                                cx,
                            );
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(render_icon(Icon::Plus, px(12.0), theme.text.secondary))
                                .child("Add Env"),
                        ),
                ),
        )
        .child(rows)
        .into_any_element();

    render_two_col_section(
        "Environment",
        "Literal values, ConfigMap/Secret references, Downward API, and envFrom sources.",
        theme,
        panel,
    )
}

pub fn render_container_ports_editor(
    c_idx: usize,
    ports: &[ContainerPortEntry],
    inputs: &HashMap<String, Entity<InputState>>,
    disabled: bool,
    theme: &OrbitTheme,
    on_mutate_form: &EditFormMutatorCallback,
) -> AnyElement {
    let mut rows = div().flex().flex_col().gap_2();
    if ports.is_empty() {
        rows = rows.child(
            apply_text_step(div(), theme.typography.size_xs)
                .text_color(theme.text.muted)
                .child("No container ports exposed."),
        );
    } else {
        for (p_idx, port) in ports.iter().enumerate() {
            let mut proto_pills = div().flex().items_center().gap_1();
            for (proto_idx, &proto) in PORT_PROTOCOLS.iter().enumerate() {
                let is_active = port.protocol == proto;
                let cb = Rc::clone(on_mutate_form);
                let mut btn = Button::new(SharedString::from(format!(
                    "port-proto-{c_idx}-{p_idx}-{proto_idx}"
                )))
                .xsmall()
                .disabled(disabled)
                .on_click(move |_, window, cx| {
                    cb(
                        Box::new(move |m| {
                            if let Some(ps) = pod_spec_mut(m)
                                && let Some(c) = ps.containers.get_mut(c_idx)
                                && let Some(p) = c.ports.get_mut(p_idx)
                            {
                                p.protocol = proto.to_string();
                            }
                        }),
                        false,
                        window,
                        cx,
                    );
                })
                .child(proto);
                if is_active {
                    btn = btn.primary();
                } else {
                    btn = btn.outline();
                }
                proto_pills = proto_pills.child(btn);
            }

            let remove_cb = Rc::clone(on_mutate_form);
            rows = rows.child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(render_input_or_placeholder(
                        inputs,
                        &format!("port:{c_idx}:{p_idx}:name"),
                        disabled,
                        theme,
                    )))
                    .child(div().w(px(100.0)).child(render_input_or_placeholder(
                        inputs,
                        &format!("port:{c_idx}:{p_idx}:container_port"),
                        disabled,
                        theme,
                    )))
                    .child(proto_pills)
                    .child(
                        Button::new(SharedString::from(format!("port-remove-{c_idx}-{p_idx}")))
                            .ghost()
                            .small()
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                remove_cb(
                                    Box::new(move |m| {
                                        if let Some(ps) = pod_spec_mut(m)
                                            && let Some(c) = ps.containers.get_mut(c_idx)
                                            && p_idx < c.ports.len()
                                        {
                                            c.ports.remove(p_idx);
                                        }
                                    }),
                                    true,
                                    window,
                                    cx,
                                );
                            })
                            .child(render_icon(Icon::Trash2, px(13.0), theme.text.muted)),
                    ),
            );
        }
    }

    let add_cb = Rc::clone(on_mutate_form);
    let panel = div()
        .w_full()
        .p_4()
        .rounded(theme.style.radius_card)
        .bg(theme.surface.bg_panel)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    apply_text_step(div(), theme.typography.size_xs)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text.primary)
                        .child("Container Ports"),
                )
                .child(
                    Button::new(SharedString::from(format!("port-add-{c_idx}")))
                        .outline()
                        .small()
                        .disabled(disabled)
                        .on_click(move |_, window, cx| {
                            add_cb(
                                Box::new(move |m| {
                                    if let Some(ps) = pod_spec_mut(m)
                                        && let Some(c) = ps.containers.get_mut(c_idx)
                                    {
                                        c.ports.push(ContainerPortEntry::default());
                                    }
                                }),
                                true,
                                window,
                                cx,
                            );
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(render_icon(Icon::Plus, px(12.0), theme.text.secondary))
                                .child("Add Port"),
                        ),
                ),
        )
        .child(rows)
        .into_any_element();

    render_two_col_section(
        "Ports",
        "Container ports and protocols exposed by this container.",
        theme,
        panel,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_core::kubernetes::edit_forms::extract_edit_form;
    use serde_json::json;

    #[test]
    fn test_workload_sub_tabs_match_kind_parity() {
        let dep = extract_edit_form("Deployment", &json!({ "kind": "Deployment" }));
        assert_eq!(
            sub_tabs_for_form(&dep)
                .into_iter()
                .map(|(t, _)| t)
                .collect::<Vec<_>>(),
            vec![
                EditFormSubTab::General,
                EditFormSubTab::Metadata,
                EditFormSubTab::PodSpec,
                EditFormSubTab::Containers,
            ]
        );

        let sts = extract_edit_form("StatefulSet", &json!({ "kind": "StatefulSet" }));
        assert_eq!(
            sub_tabs_for_form(&sts)
                .into_iter()
                .map(|(t, _)| t)
                .collect::<Vec<_>>(),
            vec![
                EditFormSubTab::General,
                EditFormSubTab::Metadata,
                EditFormSubTab::PodSpec,
                EditFormSubTab::Containers,
                EditFormSubTab::Storage,
            ]
        );

        for kind in ["DaemonSet", "ReplicaSet", "Job", "CronJob"] {
            let model = extract_edit_form(kind, &json!({ "kind": kind }));
            assert_eq!(
                sub_tabs_for_form(&model)
                    .into_iter()
                    .map(|(t, _)| t)
                    .collect::<Vec<_>>(),
                vec![
                    EditFormSubTab::General,
                    EditFormSubTab::Metadata,
                    EditFormSubTab::PodSpec,
                    EditFormSubTab::Containers,
                ],
                "unexpected tabs for {kind}"
            );
        }

        let pod = extract_edit_form("Pod", &json!({ "kind": "Pod" }));
        assert_eq!(
            sub_tabs_for_form(&pod)
                .into_iter()
                .map(|(t, _)| t)
                .collect::<Vec<_>>(),
            vec![
                EditFormSubTab::Metadata,
                EditFormSubTab::PodSpec,
                EditFormSubTab::Containers,
            ]
        );
    }
}
