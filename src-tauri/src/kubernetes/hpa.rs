use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
use kube::{
    Client,
    api::{Api, Patch, PatchParams},
};
use serde_json::json;

pub use orbit_core::kubernetes::hpa::{list_hpas, map_hpa};

pub fn build_hpa_scale_patch(
    min_replicas: i32,
    max_replicas: i32,
) -> Result<serde_json::Value, String> {
    if min_replicas < 1 {
        return Err(format!(
            "min_replicas must be at least 1, got {}",
            min_replicas
        ));
    }
    if max_replicas < min_replicas {
        return Err(format!(
            "max_replicas ({}) cannot be less than min_replicas ({})",
            max_replicas, min_replicas
        ));
    }

    Ok(json!({
        "spec": {
            "minReplicas": min_replicas,
            "maxReplicas": max_replicas
        }
    }))
}

pub async fn update_hpa_replicas(
    client: &Client,
    namespace: &str,
    name: &str,
    min_replicas: i32,
    max_replicas: i32,
) -> Result<(), kube::Error> {
    let patch_val = build_hpa_scale_patch(min_replicas, max_replicas).map_err(|e| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: e,
            reason: "BadRequest".to_string(),
            code: 400,
        })
    })?;

    let api: Api<HorizontalPodAutoscaler> = Api::namespaced(client.clone(), namespace);
    let patch = Patch::Merge(&patch_val);
    let params = PatchParams::default();

    api.patch(name, &params, &patch).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_hpa_scale_patch_validation() {
        let valid = build_hpa_scale_patch(2, 10).unwrap();
        assert_eq!(valid["spec"]["minReplicas"], 2);
        assert_eq!(valid["spec"]["maxReplicas"], 10);

        assert!(build_hpa_scale_patch(0, 10).is_err());
        assert!(build_hpa_scale_patch(5, 3).is_err());
    }
}
