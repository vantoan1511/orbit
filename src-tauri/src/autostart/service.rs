use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::autostart::backend::{AutostartBackend, AutostartEntry, AutostartError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Dev,
    Prod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    Applied,
    SkippedDevMode,
}

pub struct AutostartService {
    backend: Arc<dyn AutostartBackend>,
    mode: RunMode,
    app_name: String,
    exe_path: PathBuf,
}

fn platform_backend() -> Result<Arc<dyn AutostartBackend>, AutostartError> {
    #[cfg(target_os = "windows")]
    {
        Ok(Arc::new(crate::autostart::windows::WindowsRunKeyBackend::production()))
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        Ok(Arc::new(crate::autostart::file_backend::FileAutostartBackend::for_current_os()?))
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(Arc::new(crate::autostart::memory::InMemoryBackend::new()))
    }
}

impl AutostartService {
    pub fn from_environment() -> Result<Self, AutostartError> {
        let backend = platform_backend()?;
        let mode = if crate::config::OrbitConfig::is_dev_mode() {
            RunMode::Dev
        } else {
            RunMode::Prod
        };
        let app_name = crate::autostart::default_app_name().to_string();
        let exe_path = std::env::current_exe().map_err(|e| {
            AutostartError::ExecutablePathUnavailable(e.to_string())
        })?;
        Ok(Self::new(backend, mode, app_name, exe_path))
    }

    pub fn new(
        backend: Arc<dyn AutostartBackend>,
        mode: RunMode,
        app_name: String,
        exe_path: PathBuf,
    ) -> Self {
        Self {
            backend,
            mode,
            app_name,
            exe_path,
        }
    }

    pub fn mode(&self) -> RunMode {
        self.mode
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    pub fn exe_path(&self) -> &Path {
        &self.exe_path
    }

    pub fn backend(&self) -> &Arc<dyn AutostartBackend> {
        &self.backend
    }

    fn guard_mutation(&self) -> Option<ApplyOutcome> {
        if self.mode == RunMode::Dev {
            tracing::info!("Autostart mutation skipped in development mode");
            Some(ApplyOutcome::SkippedDevMode)
        } else {
            None
        }
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<ApplyOutcome, AutostartError> {
        if let Some(outcome) = self.guard_mutation() {
            return Ok(outcome);
        }

        if enabled {
            if crate::autostart::stale::is_cargo_deps_path(&self.exe_path.to_string_lossy()) {
                return Err(AutostartError::UnsafeExecutable(self.exe_path.clone()));
            }
            self.backend.register(&AutostartEntry {
                name: self.app_name.clone(),
                exe_path: self.exe_path.clone(),
            })?;
        } else {
            self.backend.unregister(&self.app_name)?;
        }

        Ok(ApplyOutcome::Applied)
    }

    pub fn is_enabled(&self) -> Result<bool, AutostartError> {
        self.backend.is_registered(&self.app_name)
    }

    pub fn reconcile(&self, should_autostart: bool) -> Result<ApplyOutcome, AutostartError> {
        if let Some(outcome) = self.guard_mutation() {
            return Ok(outcome);
        }

        let currently_enabled = self.is_enabled()?;
        match (should_autostart, currently_enabled) {
            (true, false) => self.set_enabled(true),
            (false, true) => self.set_enabled(false),
            _ => Ok(ApplyOutcome::Applied),
        }
    }

    pub fn cleanup_stale(&self) -> Result<Vec<String>, AutostartError> {
        let entries = self.backend.list()?;
        let mut removed = Vec::new();

        for entry in entries {
            if crate::autostart::stale::is_stale_entry(&entry) {
                match self.backend.unregister(&entry.name) {
                    Ok(()) => {
                        tracing::info!(
                            name = %entry.name,
                            command = %entry.command,
                            "Cleaned up stale autostart entry"
                        );
                        removed.push(entry.name);
                    }
                    Err(err) => {
                        tracing::warn!(
                            name = %entry.name,
                            error = %err,
                            "Failed to unregister stale autostart entry"
                        );
                    }
                }
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autostart::backend::RegisteredEntry;
    use crate::autostart::memory::InMemoryBackend;

    #[test]
    fn prod_set_enabled_true_registers_with_hidden_flag() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        let outcome = service.set_enabled(true).unwrap();
        assert_eq!(outcome, ApplyOutcome::Applied);
        assert!(service.is_enabled().unwrap());
        let list = backend.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Orbit");
        assert_eq!(
            list[0].command,
            "\"C:\\Program Files\\Orbit\\orbit.exe\" --hidden"
        );
    }

    #[test]
    fn prod_set_enabled_false_unregisters() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        service.set_enabled(true).unwrap();
        let outcome = service.set_enabled(false).unwrap();
        assert_eq!(outcome, ApplyOutcome::Applied);
        assert!(!service.is_enabled().unwrap());
        assert!(backend.list().unwrap().is_empty());
    }

    #[test]
    fn dev_mode_set_enabled_skips_and_backend_untouched() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Dev,
            "Orbit Dev".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        let outcome = service.set_enabled(true).unwrap();
        assert_eq!(outcome, ApplyOutcome::SkippedDevMode);
        assert!(backend.list().unwrap().is_empty());
        assert!(!service.is_enabled().unwrap());
    }

    #[test]
    fn dev_mode_reconcile_skips() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Dev,
            "Orbit Dev".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        let outcome = service.reconcile(true).unwrap();
        assert_eq!(outcome, ApplyOutcome::SkippedDevMode);
        assert!(backend.list().unwrap().is_empty());
    }

    #[test]
    fn register_rejects_cargo_deps_exe() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("D:\\Projects\\orbit\\target\\debug\\deps\\orbit_app_lib-12345.exe"),
        );

        let res = service.set_enabled(true);
        assert!(matches!(res, Err(AutostartError::UnsafeExecutable(_))));
        assert!(backend.list().unwrap().is_empty());
    }

    #[test]
    fn reconcile_enables_when_config_true_and_missing() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        let outcome = service.reconcile(true).unwrap();
        assert_eq!(outcome, ApplyOutcome::Applied);
        assert!(service.is_enabled().unwrap());
    }

    #[test]
    fn reconcile_disables_when_config_false_and_present() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        service.set_enabled(true).unwrap();
        let outcome = service.reconcile(false).unwrap();
        assert_eq!(outcome, ApplyOutcome::Applied);
        assert!(!service.is_enabled().unwrap());
    }

    #[test]
    fn reconcile_noop_when_in_sync() {
        let backend = Arc::new(InMemoryBackend::new());
        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Program Files\\Orbit\\orbit.exe"),
        );

        service.set_enabled(true).unwrap();
        let outcome = service.reconcile(true).unwrap();
        assert_eq!(outcome, ApplyOutcome::Applied);
        assert!(service.is_enabled().unwrap());
    }

    #[test]
    fn cleanup_stale_removes_orbit_dev_and_deps_entries_but_keeps_others() {
        let backend = Arc::new(InMemoryBackend::new());
        // Seed entries
        backend
            .register(&AutostartEntry {
                name: "Steam".to_string(),
                exe_path: PathBuf::from("C:\\Program Files\\Steam\\steam.exe"),
            })
            .unwrap();
        backend
            .register(&AutostartEntry {
                name: "Orbit".to_string(),
                exe_path: PathBuf::from("C:\\Users\\user\\AppData\\Local\\Orbit\\orbit-app.exe"),
            })
            .unwrap();
        backend
            .register(&AutostartEntry {
                name: "Orbit Dev".to_string(),
                exe_path: PathBuf::from("C:\\Users\\user\\AppData\\Local\\Orbit\\orbit-app.exe"),
            })
            .unwrap();
        backend
            .register(&AutostartEntry {
                name: "Orbit Deps".to_string(),
                exe_path: PathBuf::from("D:\\Projects\\orbit\\target\\release\\deps\\orbit_app-test.exe"),
            })
            .unwrap();

        let service = AutostartService::new(
            backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Users\\user\\AppData\\Local\\Orbit\\orbit-app.exe"),
        );

        let removed = service.cleanup_stale().unwrap();
        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&"Orbit Dev".to_string()));
        assert!(removed.contains(&"Orbit Deps".to_string()));

        let remaining = backend.list().unwrap();
        let remaining_names: Vec<String> = remaining.into_iter().map(|e| e.name).collect();
        assert_eq!(remaining_names, vec!["Orbit".to_string(), "Steam".to_string()]);
    }

    struct FailingBackend {
        inner: InMemoryBackend,
        fail_name: String,
    }

    impl AutostartBackend for FailingBackend {
        fn is_registered(&self, name: &str) -> Result<bool, AutostartError> {
            self.inner.is_registered(name)
        }
        fn register(&self, entry: &AutostartEntry) -> Result<(), AutostartError> {
            self.inner.register(entry)
        }
        fn unregister(&self, name: &str) -> Result<(), AutostartError> {
            if name == self.fail_name {
                Err(AutostartError::Backend {
                    code: "mock_failure",
                    detail: "simulated failure".to_string(),
                })
            } else {
                self.inner.unregister(name)
            }
        }
        fn list(&self) -> Result<Vec<RegisteredEntry>, AutostartError> {
            self.inner.list()
        }
    }

    #[test]
    fn cleanup_stale_continues_after_single_unregister_failure() {
        let failing_backend = Arc::new(FailingBackend {
            inner: InMemoryBackend::new(),
            fail_name: "Orbit Dev".to_string(),
        });

        failing_backend
            .register(&AutostartEntry {
                name: "Orbit Dev".to_string(),
                exe_path: PathBuf::from("C:\\Users\\user\\AppData\\Local\\Orbit\\orbit-app.exe"),
            })
            .unwrap();
        failing_backend
            .register(&AutostartEntry {
                name: "Orbit Stale 2".to_string(),
                exe_path: PathBuf::from("D:\\Projects\\orbit\\target\\release\\deps\\orbit_app-stale.exe"),
            })
            .unwrap();

        let service = AutostartService::new(
            failing_backend.clone(),
            RunMode::Prod,
            "Orbit".to_string(),
            PathBuf::from("C:\\Users\\user\\AppData\\Local\\Orbit\\orbit-app.exe"),
        );

        let removed = service.cleanup_stale().unwrap();
        assert_eq!(removed, vec!["Orbit Stale 2".to_string()]);
        assert!(failing_backend.is_registered("Orbit Dev").unwrap());
        assert!(!failing_backend.is_registered("Orbit Stale 2").unwrap());
    }
}
