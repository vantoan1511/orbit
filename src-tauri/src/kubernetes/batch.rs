pub use orbit_core::kubernetes::batch::{list_cronjobs, list_jobs, map_cronjob, map_job};

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::batch::v1::{CronJob, Job};
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
