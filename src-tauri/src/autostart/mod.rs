//! Cross-platform OS autostart (launch at login) management for Orbit.

pub mod backend;
pub mod file_backend;
#[cfg(test)]
pub mod hygiene_tests;
pub mod memory;
pub mod provider;
pub mod service;
pub mod stale;
#[cfg(target_os = "windows")]
pub mod windows;

pub use backend::{AutostartBackend, AutostartEntry, AutostartError, RegisteredEntry};
pub use file_backend::FileAutostartBackend;
pub use memory::InMemoryBackend;
pub use provider::{service, set_global_service};
#[cfg(test)]
pub use provider::{AutostartTestGuard, AUTOSTART_TEST_LOCK};
pub use service::{ApplyOutcome, AutostartService, RunMode};
pub use stale::{is_cargo_deps_path, is_stale_entry, LEGACY_DEV_ENTRY_NAME};
#[cfg(target_os = "windows")]
pub use windows::WindowsRunKeyBackend;

/// Returns the autostart registration key name ("Orbit Dev" in dev mode, "Orbit" in production).
pub fn default_app_name() -> &'static str {
    if crate::config::OrbitConfig::is_dev_mode() {
        "Orbit Dev"
    } else {
        "Orbit"
    }
}

/// Formats the Windows registry Run command line for autostart with `--hidden`.
pub fn format_windows_run_value(app_path: &std::path::Path) -> String {
    format!("\"{}\" --hidden", app_path.to_string_lossy())
}

/// Formats the Linux desktop entry Exec command line for autostart with `--hidden`.
pub fn format_linux_desktop_exec(app_path: &std::path::Path) -> String {
    format!("\"{}\" --hidden", app_path.to_string_lossy())
}

/// Runs the startup cleanup and reconciliation sequence.
pub fn run_startup_sequence(service: &AutostartService, should_autostart: bool) {
    if let Err(e) = service.cleanup_stale() {
        tracing::warn!(
            code = e.code(),
            error = %e,
            "Failed to clean up stale autostart entries during startup"
        );
    }

    if let Err(e) = service.reconcile(should_autostart) {
        tracing::warn!(
            code = e.code(),
            error = %e,
            "Failed to reconcile autostart on startup"
        );
    }
}

/// Reconciles OS autostart state with saved application configuration on app startup.
pub fn reconcile_on_startup() {
    let config = crate::config::OrbitConfig::load();
    let should_autostart = config.launch_on_startup();
    match service() {
        Ok(svc) => run_startup_sequence(&svc, should_autostart),
        Err(e) => {
            tracing::warn!(
                code = e.code(),
                error = %e,
                "Failed to initialize autostart service for startup reconciliation"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_format_windows_run_value_contains_hidden() {
        let path = PathBuf::from("C:\\fake\\orbit.exe");
        let cmd = format_windows_run_value(&path);
        assert_eq!(cmd, "\"C:\\fake\\orbit.exe\" --hidden");
    }

    #[test]
    fn test_format_linux_desktop_exec_contains_hidden() {
        let path = PathBuf::from("/usr/bin/orbit");
        let cmd = format_linux_desktop_exec(&path);
        assert_eq!(cmd, "\"/usr/bin/orbit\" --hidden");
    }

    #[test]
    fn test_autostart_default_app_name_dev_vs_prod() {
        let original_orbit_dev = std::env::var("ORBIT_DEV").ok();

        // 1. Dev mode
        unsafe {
            std::env::set_var("ORBIT_DEV", "1");
        }
        assert_eq!(default_app_name(), "Orbit Dev");

        // 2. Prod mode
        unsafe {
            std::env::set_var("ORBIT_DEV", "0");
        }
        assert_eq!(default_app_name(), "Orbit");

        // Restore env
        unsafe {
            match original_orbit_dev {
                Some(val) => std::env::set_var("ORBIT_DEV", val),
                None => std::env::remove_var("ORBIT_DEV"),
            }
        }
    }

    #[test]
    fn startup_sequence_removes_stale_then_reconciles() {
        let backend = Arc::new(InMemoryBackend::new());
        // Seed stale Orbit Dev entry
        backend
            .register(&AutostartEntry {
                name: "Orbit Dev".to_string(),
                exe_path: PathBuf::from("D:\\Projects\\orbit\\target\\debug\\deps\\orbit_app-123.exe"),
            })
            .unwrap();

        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        run_startup_sequence(&service, true);

        // Stale entry is removed
        assert!(!backend.is_registered("Orbit Dev").unwrap());
        // Prod entry is enabled
        assert!(backend.is_registered("Orbit").unwrap());
        let list = backend.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Orbit");
    }

    struct FailingListBackend;

    impl AutostartBackend for FailingListBackend {
        fn is_registered(&self, _name: &str) -> Result<bool, AutostartError> {
            Ok(false)
        }
        fn register(&self, _entry: &AutostartEntry) -> Result<(), AutostartError> {
            Ok(())
        }
        fn unregister(&self, _name: &str) -> Result<(), AutostartError> {
            Ok(())
        }
        fn list(&self) -> Result<Vec<RegisteredEntry>, AutostartError> {
            Err(AutostartError::Backend {
                code: "simulated_list_failure",
                detail: "disk read error".to_string(),
            })
        }
    }

    #[test]
    fn startup_sequence_survives_backend_list_failure() {
        let service = AutostartService::new(
            Arc::new(FailingListBackend),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        // Must log and not panic
        run_startup_sequence(&service, true);
    }
}
