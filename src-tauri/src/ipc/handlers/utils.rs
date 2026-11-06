use serde_json::Value;

pub fn get_string(data: &Option<Value>, key: &str) -> Option<String> {
    data.as_ref()
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn get_i64(data: &Option<Value>, key: &str) -> Option<i64> {
    data.as_ref()
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_i64())
}

pub fn get_bool(data: &Option<Value>, key: &str) -> Option<bool> {
    data.as_ref()
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_bool())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_string() {
        let data = Some(json!({
            "name": "orbit",
            "number": 42
        }));

        assert_eq!(get_string(&data, "name"), Some("orbit".to_string()));
        assert_eq!(get_string(&data, "number"), None);
        assert_eq!(get_string(&data, "nonexistent"), None);
        assert_eq!(get_string(&None, "name"), None);
    }

    #[test]
    fn test_get_i64() {
        let data = Some(json!({
            "count": 100,
            "text": "100"
        }));

        assert_eq!(get_i64(&data, "count"), Some(100));
        assert_eq!(get_i64(&data, "text"), None);
        assert_eq!(get_i64(&data, "nonexistent"), None);
        assert_eq!(get_i64(&None, "count"), None);
    }

    #[test]
    fn test_get_bool() {
        let data = Some(json!({
            "is_active": true,
            "is_suspended": false,
            "text": "true",
            "number": 1
        }));

        assert_eq!(get_bool(&data, "is_active"), Some(true));
        assert_eq!(get_bool(&data, "is_suspended"), Some(false));
        assert_eq!(get_bool(&data, "text"), None);
        assert_eq!(get_bool(&data, "number"), None);
        assert_eq!(get_bool(&data, "nonexistent"), None);
        assert_eq!(get_bool(&None, "is_active"), None);
    }
}
