use crate::kubernetes::{format_age, models};
use k8s_openapi::api::batch::v1::CronJob;
use kube::{
    Client,
    api::{Api, ListParams},
};

pub use orbit_core::kubernetes::batch::{list_jobs, map_job};

pub fn map_cronjob(cj: &CronJob) -> models::CronJobInfo {
    let name = cj.metadata.name.clone().unwrap_or_default();
    let namespace_name = cj.metadata.namespace.clone().unwrap_or_default();
    let age = format_age(&cj.metadata.creation_timestamp);

    let spec = cj.spec.as_ref();
    let schedule = spec.map(|s| s.schedule.clone()).unwrap_or_default();
    let suspend = spec.and_then(|s| s.suspend).unwrap_or(false);

    let status_cj = cj.status.as_ref();
    let active = status_cj
        .and_then(|s| s.active.as_ref())
        .map(|a| a.len() as i32)
        .unwrap_or(0);

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
    if let Some(job_spec) = cj
        .spec
        .as_ref()
        .and_then(|spec| spec.job_template.spec.as_ref())
    {
        for c in &job_spec
            .template
            .spec
            .as_ref()
            .map(|s| s.containers.clone())
            .unwrap_or_default()
        {
            if let Some(img) = &c.image {
                images.push(img.clone());
            }
        }
    }

    let labels = cj.metadata.labels.clone().unwrap_or_default();
    let annotations = cj.metadata.annotations.clone().unwrap_or_default();

    models::CronJobInfo {
        kind: "CronJob".to_string(),
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
    }
}

pub async fn list_cronjobs(
    client: &Client,
    namespace: Option<String>,
) -> Result<Vec<models::CronJobInfo>, kube::Error> {
    let api: Api<CronJob> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for cj in api.list(&ListParams::default()).await? {
        list.push(map_cronjob(&cj));
    }

    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::batch::v1::Job;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    #[test]
    fn test_map_job_kind() {
        let j = Job {
            metadata: ObjectMeta {
                name: Some("migrate-db".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mapped = map_job(&j);
        assert_eq!(mapped.name, "migrate-db");
        assert_eq!(mapped.kind, "Job");
    }

    #[test]
    fn test_map_cronjob_kind() {
        let cj = CronJob {
            metadata: ObjectMeta {
                name: Some("nightly-backup".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mapped = map_cronjob(&cj);
        assert_eq!(mapped.name, "nightly-backup");
        assert_eq!(mapped.kind, "CronJob");
    }
}
