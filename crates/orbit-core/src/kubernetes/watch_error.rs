use kube::runtime::watcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatcherFailureClass {
    /// Transport/server problems: connect, timeout, TLS, HTTP 5xx. Counts toward "unreachable".
    Unreachable,
    /// The API server answered but refuses this resource (HTTP 401/403/404). Never affects cluster health.
    Rejected,
}

pub const REJECTED_STATUS_CODES: [u16; 3] = [401, 403, 404];

#[inline]
pub fn classify_status_code(code: u16) -> WatcherFailureClass {
    if REJECTED_STATUS_CODES.contains(&code) {
        WatcherFailureClass::Rejected
    } else {
        WatcherFailureClass::Unreachable
    }
}

pub fn classify_watcher_error(err: &watcher::Error) -> WatcherFailureClass {
    let status_code: Option<u16> = match err {
        watcher::Error::InitialListFailed(kube::Error::Api(resp))
        | watcher::Error::WatchStartFailed(kube::Error::Api(resp))
        | watcher::Error::WatchFailed(kube::Error::Api(resp)) => Some(resp.code),
        watcher::Error::WatchError(resp) => Some(resp.code),
        _ => None,
    };

    match status_code {
        Some(code) => classify_status_code(code),
        None => WatcherFailureClass::Unreachable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kube::core::ErrorResponse;

    #[test]
    fn status_401_403_404_are_rejected() {
        for &code in &REJECTED_STATUS_CODES {
            assert_eq!(
                classify_status_code(code),
                WatcherFailureClass::Rejected,
                "Expected {code} to be classified as Rejected"
            );
        }
    }

    #[test]
    fn status_500_502_503_are_unreachable() {
        for &code in &[500, 502, 503] {
            assert_eq!(
                classify_status_code(code),
                WatcherFailureClass::Unreachable,
                "Expected {code} to be classified as Unreachable"
            );
        }
    }

    #[test]
    fn api_error_403_in_initial_list_is_rejected() {
        let resp = ErrorResponse {
            status: "Failure".into(),
            message: "forbidden".into(),
            reason: "Forbidden".into(),
            code: 403,
        };
        let err = watcher::Error::InitialListFailed(kube::Error::Api(resp));
        assert_eq!(classify_watcher_error(&err), WatcherFailureClass::Rejected);
    }

    #[test]
    fn watch_error_response_404_is_rejected() {
        let resp = ErrorResponse {
            status: "Failure".into(),
            message: "not found".into(),
            reason: "NotFound".into(),
            code: 404,
        };
        let err = watcher::Error::WatchError(resp);
        assert_eq!(classify_watcher_error(&err), WatcherFailureClass::Rejected);
    }

    #[test]
    fn no_resource_version_is_unreachable() {
        let err = watcher::Error::NoResourceVersion;
        assert_eq!(classify_watcher_error(&err), WatcherFailureClass::Unreachable);
    }
}
