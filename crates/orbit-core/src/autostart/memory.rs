use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::autostart::backend::{AutostartBackend, AutostartEntry, AutostartError, RegisteredEntry};

#[derive(Debug, Default)]
pub struct InMemoryBackend {
    entries: Mutex<BTreeMap<String, String>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }
}

impl AutostartBackend for InMemoryBackend {
    fn is_registered(&self, name: &str) -> Result<bool, AutostartError> {
        let entries = self.entries.lock().map_err(|e| AutostartError::Backend {
            code: "lock_poisoned",
            detail: e.to_string(),
        })?;
        Ok(entries.contains_key(name))
    }

    fn register(&self, entry: &AutostartEntry) -> Result<(), AutostartError> {
        let mut entries = self.entries.lock().map_err(|e| AutostartError::Backend {
            code: "lock_poisoned",
            detail: e.to_string(),
        })?;
        let cmd = crate::autostart::format_windows_run_value(&entry.exe_path);
        entries.insert(entry.name.clone(), cmd);
        Ok(())
    }

    fn unregister(&self, name: &str) -> Result<(), AutostartError> {
        let mut entries = self.entries.lock().map_err(|e| AutostartError::Backend {
            code: "lock_poisoned",
            detail: e.to_string(),
        })?;
        entries.remove(name);
        Ok(())
    }

    fn list(&self) -> Result<Vec<RegisteredEntry>, AutostartError> {
        let entries = self.entries.lock().map_err(|e| AutostartError::Backend {
            code: "lock_poisoned",
            detail: e.to_string(),
        })?;
        Ok(entries
            .iter()
            .map(|(name, command)| RegisteredEntry {
                name: name.clone(),
                command: command.clone(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autostart::backend::{AutostartBackend, AutostartEntry, AutostartError};
    use std::path::PathBuf;

    #[test]
    fn register_then_is_registered_true() {
        let backend = InMemoryBackend::new();
        assert!(!backend.is_registered("Orbit").unwrap());

        let entry = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("C:\\fake\\orbit.exe"),
        };
        backend.register(&entry).unwrap();
        assert!(backend.is_registered("Orbit").unwrap());
    }

    #[test]
    fn unregister_missing_is_ok_idempotent() {
        let backend = InMemoryBackend::new();
        let res = backend.unregister("NonExistent");
        assert!(res.is_ok());
    }

    #[test]
    fn register_twice_overwrites_command() {
        let backend = InMemoryBackend::new();
        let entry1 = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("C:\\fake\\v1\\orbit.exe"),
        };
        let entry2 = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("C:\\fake\\v2\\orbit.exe"),
        };
        backend.register(&entry1).unwrap();
        backend.register(&entry2).unwrap();

        let list = backend.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Orbit");
        assert_eq!(list[0].command, "\"C:\\fake\\v2\\orbit.exe\" --hidden");
    }

    #[test]
    fn list_returns_registered_entries_sorted() {
        let backend = InMemoryBackend::new();
        backend
            .register(&AutostartEntry {
                name: "Zeta".to_string(),
                exe_path: PathBuf::from("C:\\fake\\zeta.exe"),
            })
            .unwrap();
        backend
            .register(&AutostartEntry {
                name: "Alpha".to_string(),
                exe_path: PathBuf::from("C:\\fake\\alpha.exe"),
            })
            .unwrap();

        let list = backend.list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Alpha");
        assert_eq!(list[1].name, "Zeta");
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(
            AutostartError::HomeDirUnavailable.code(),
            "home_dir_unavailable"
        );
        assert_eq!(
            AutostartError::ExecutablePathUnavailable("err".into()).code(),
            "executable_path_unavailable"
        );
        assert_eq!(
            AutostartError::UnsafeExecutable(PathBuf::from("fake")).code(),
            "unsafe_executable"
        );
        assert_eq!(
            AutostartError::Backend {
                code: "custom_code",
                detail: "detail".into(),
            }
            .code(),
            "custom_code"
        );
    }
}
