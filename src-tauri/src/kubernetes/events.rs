use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::Event;
use crate::kubernetes::models::EventInfo;
use crate::kubernetes::format_age;

/// Formats a Kubernetes `Event` into `EventInfo`.
pub fn format_event(e: &Event) -> EventInfo {
    let uid = e.metadata.uid.clone().unwrap_or_default();
    
    // Determine the most representative timestamp for age/time
    let last_seen_time = e.last_timestamp.clone()
        .or_else(|| e.first_timestamp.clone())
        .or_else(|| e.metadata.creation_timestamp.clone());
        
    let time = format_age(&last_seen_time);

    let r#type = e.type_.clone().unwrap_or_else(|| "Normal".to_string());
    let reason = e.reason.clone().unwrap_or_default();
    
    let object_name = e.involved_object.name.clone().unwrap_or_default();
    let object_kind = e.involved_object.kind.clone().unwrap_or_default();
    let message = e.message.clone().unwrap_or_default();
    let namespace = e.metadata.namespace.clone().unwrap_or_default();
    
    let source = e.source.as_ref()
        .and_then(|s| s.component.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let first_seen = e.first_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();
        
    let last_seen = e.last_timestamp.as_ref()
        .map(|t| t.0.format("%b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_default();

    let count = e.count.unwrap_or(1);
    
    let labels = e.metadata.labels.clone().unwrap_or_default();
    let annotations = e.metadata.annotations.clone().unwrap_or_default();

    EventInfo {
        uid,
        time,
        r#type,
        reason,
        object_name,
        object_kind,
        message,
        namespace,
        source,
        first_seen,
        last_seen,
        count,
        labels,
        annotations,
    }
}

/// Lists all Events in the specified namespace or all namespaces.
pub async fn list_events(client: &Client, namespace: Option<String>) -> Result<Vec<EventInfo>, kube::Error> {
    let api: Api<Event> = if let Some(ns) = namespace {
        Api::namespaced(client.clone(), &ns)
    } else {
        Api::all(client.clone())
    };

    let mut list = Vec::new();
    for e in api.list(&ListParams::default()).await? {
        list.push(format_event(&e));
    }
    
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
    use k8s_openapi::api::core::v1::ObjectReference;
    use k8s_openapi::api::core::v1::EventSource;
    use std::collections::BTreeMap;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_event_basic() {
        let first_seen_time = Utc.with_ymd_and_hms(2026, 7, 12, 10, 0, 0).unwrap();
        let last_seen_time = Utc.with_ymd_and_hms(2026, 7, 12, 10, 5, 0).unwrap();

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "orbit-event".to_string());

        let event = Event {
            metadata: ObjectMeta {
                uid: Some("event-uid-123".to_string()),
                namespace: Some("test-namespace".to_string()),
                creation_timestamp: Some(Time(first_seen_time)),
                labels: Some(labels),
                ..Default::default()
            },
            involved_object: ObjectReference {
                kind: Some("Pod".to_string()),
                name: Some("test-pod".to_string()),
                ..Default::default()
            },
            reason: Some("Scheduled".to_string()),
            message: Some("Successfully assigned test-namespace/test-pod to node".to_string()),
            type_: Some("Normal".to_string()),
            first_timestamp: Some(Time(first_seen_time)),
            last_timestamp: Some(Time(last_seen_time)),
            count: Some(5),
            source: Some(EventSource {
                component: Some("default-scheduler".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let info = format_event(&event);

        assert_eq!(info.uid, "event-uid-123");
        assert_eq!(info.r#type, "Normal");
        assert_eq!(info.reason, "Scheduled");
        assert_eq!(info.object_name, "test-pod");
        assert_eq!(info.object_kind, "Pod");
        assert_eq!(info.message, "Successfully assigned test-namespace/test-pod to node");
        assert_eq!(info.namespace, "test-namespace");
        assert_eq!(info.source, "default-scheduler");
        assert_eq!(info.count, 5);
        assert_eq!(info.labels.get("app").map(|s| s.as_str()), Some("orbit-event"));
    }
}
