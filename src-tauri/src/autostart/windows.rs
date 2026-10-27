#![cfg(target_os = "windows")]

#[cfg(test)]
use std::sync::Arc;

use windows_sys::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS,
};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_EXPAND_SZ, REG_OPTION_NON_VOLATILE, REG_SZ,
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegEnumValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
#[cfg(test)]
use windows_sys::Win32::System::Registry::{RegDeleteKeyW, RegDeleteTreeW};

use crate::autostart::backend::{
    AutostartBackend, AutostartEntry, AutostartError, RegisteredEntry,
};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

struct RegKey(HKEY);

impl RegKey {
    fn open_read_write(subkey: &str) -> Result<Self, AutostartError> {
        let wide_subkey = to_wide(subkey);
        let mut hkey: HKEY = std::ptr::null_mut();
        // SAFETY: RegCreateKeyExW is called with valid pointers to open or create subkey under HKCU.
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide_subkey.as_ptr(),
                0,
                std::ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_READ | KEY_WRITE,
                std::ptr::null(),
                &mut hkey,
                std::ptr::null_mut(),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(AutostartError::Backend {
                code: "registry_open_failed",
                detail: format!("RegCreateKeyExW failed with code {status}"),
            });
        }
        Ok(Self(hkey))
    }

    fn open_read_only(subkey: &str) -> Result<Option<Self>, AutostartError> {
        let wide_subkey = to_wide(subkey);
        let mut hkey: HKEY = std::ptr::null_mut();
        // SAFETY: RegOpenKeyExW is called with valid pointers to open subkey read-only under HKCU.
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide_subkey.as_ptr(),
                0,
                KEY_READ,
                &mut hkey,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(AutostartError::Backend {
                code: "registry_open_failed",
                detail: format!("RegOpenKeyExW failed with code {status}"),
            });
        }
        Ok(Some(Self(hkey)))
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != HKEY_CURRENT_USER {
            // SAFETY: self.0 is a valid open HKEY and is closed exactly once when dropped.
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowsRunKeyBackend {
    subkey: String,
}

impl WindowsRunKeyBackend {
    pub(crate) fn production() -> Self {
        Self {
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
        }
    }

    pub fn sandbox(subkey: String) -> Self {
        Self { subkey }
    }
}

impl AutostartBackend for WindowsRunKeyBackend {
    fn is_registered(&self, name: &str) -> Result<bool, AutostartError> {
        let key = match RegKey::open_read_only(&self.subkey)? {
            Some(k) => k,
            None => return Ok(false),
        };
        let wide_name = to_wide(name);
        // SAFETY: RegQueryValueExW called with NULL buffer to check existence.
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                wide_name.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if status == ERROR_SUCCESS {
            Ok(true)
        } else if status == ERROR_FILE_NOT_FOUND {
            Ok(false)
        } else {
            Err(AutostartError::Backend {
                code: "registry_query_failed",
                detail: format!("RegQueryValueExW failed with code {status}"),
            })
        }
    }

    fn register(&self, entry: &AutostartEntry) -> Result<(), AutostartError> {
        let key = RegKey::open_read_write(&self.subkey)?;
        let cmd = crate::autostart::format_windows_run_value(&entry.exe_path);
        let wide_name = to_wide(&entry.name);
        let wide_val = to_wide(&cmd);
        // SAFETY: RegSetValueExW receives valid pointers and byte count for REG_SZ.
        let status = unsafe {
            RegSetValueExW(
                key.0,
                wide_name.as_ptr(),
                0,
                REG_SZ,
                wide_val.as_ptr() as *const u8,
                (wide_val.len() * std::mem::size_of::<u16>()) as u32,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(AutostartError::Backend {
                code: "registry_set_value_failed",
                detail: format!("RegSetValueExW failed with code {status}"),
            });
        }
        Ok(())
    }

    fn unregister(&self, name: &str) -> Result<(), AutostartError> {
        let wide_subkey = to_wide(&self.subkey);
        let mut hkey: HKEY = std::ptr::null_mut();
        // SAFETY: RegOpenKeyExW called with KEY_WRITE to delete a value if key exists.
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide_subkey.as_ptr(),
                0,
                KEY_WRITE,
                &mut hkey,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if status != ERROR_SUCCESS {
            return Err(AutostartError::Backend {
                code: "registry_open_failed",
                detail: format!("RegOpenKeyExW failed with code {status}"),
            });
        }
        let key = RegKey(hkey);
        let wide_name = to_wide(name);
        // SAFETY: RegDeleteValueW deletes named value from valid open key.
        let status = unsafe { RegDeleteValueW(key.0, wide_name.as_ptr()) };
        if status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND {
            Ok(())
        } else {
            Err(AutostartError::Backend {
                code: "registry_delete_value_failed",
                detail: format!("RegDeleteValueW failed with code {status}"),
            })
        }
    }

    fn list(&self) -> Result<Vec<RegisteredEntry>, AutostartError> {
        let key = match RegKey::open_read_only(&self.subkey)? {
            Some(k) => k,
            None => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        let mut index = 0u32;

        loop {
            let mut name_buf: Vec<u16> = vec![0u16; 512];
            let mut name_len: u32 = name_buf.len() as u32;
            let mut val_type: u32 = 0;
            let mut val_buf: Vec<u8> = vec![0u8; 1024];
            let mut val_len: u32 = val_buf.len() as u32;

            // SAFETY: RegEnumValueW called with valid pointers to buffers.
            let status = unsafe {
                RegEnumValueW(
                    key.0,
                    index,
                    name_buf.as_mut_ptr(),
                    &mut name_len,
                    std::ptr::null_mut(),
                    &mut val_type,
                    val_buf.as_mut_ptr(),
                    &mut val_len,
                )
            };

            if status == ERROR_NO_MORE_ITEMS {
                break;
            }

            if status == ERROR_MORE_DATA {
                name_buf.resize(name_len as usize + 256, 0);
                name_len = name_buf.len() as u32;
                val_buf.resize(val_len as usize + 1024, 0);
                val_len = val_buf.len() as u32;
                // SAFETY: Retry RegEnumValueW with enlarged buffers.
                let retry_status = unsafe {
                    RegEnumValueW(
                        key.0,
                        index,
                        name_buf.as_mut_ptr(),
                        &mut name_len,
                        std::ptr::null_mut(),
                        &mut val_type,
                        val_buf.as_mut_ptr(),
                        &mut val_len,
                    )
                };
                if retry_status != ERROR_SUCCESS {
                    return Err(AutostartError::Backend {
                        code: "registry_enum_failed",
                        detail: format!("RegEnumValueW retry failed with code {retry_status}"),
                    });
                }
            } else if status != ERROR_SUCCESS {
                return Err(AutostartError::Backend {
                    code: "registry_enum_failed",
                    detail: format!("RegEnumValueW failed with code {status}"),
                });
            }

            if val_type == REG_SZ || val_type == REG_EXPAND_SZ {
                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                let u16_count = (val_len as usize) / std::mem::size_of::<u16>();
                if u16_count > 0 {
                    let mut u16_vec = Vec::with_capacity(u16_count);
                    for i in 0..u16_count {
                        let low = val_buf[i * 2];
                        let high = val_buf[i * 2 + 1];
                        u16_vec.push(u16::from_le_bytes([low, high]));
                    }
                    let trimmed_u16 = if u16_vec.last() == Some(&0) {
                        &u16_vec[..u16_vec.len() - 1]
                    } else {
                        &u16_vec[..]
                    };
                    let command = String::from_utf16_lossy(trimmed_u16);
                    entries.push(RegisteredEntry { name, command });
                }
            }

            index += 1;
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }
}

#[cfg(test)]
pub struct SandboxRunKey {
    root_subkey: String,
    run_subkey: String,
    backend: Arc<WindowsRunKeyBackend>,
}

#[cfg(test)]
impl SandboxRunKey {
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4();
        let root_subkey = format!("Software\\OrbitTest\\{}", id);
        let run_subkey = format!("{}\\Run", root_subkey);
        let backend = Arc::new(WindowsRunKeyBackend::sandbox(run_subkey.clone()));
        Self {
            root_subkey,
            run_subkey,
            backend,
        }
    }

    pub fn backend(&self) -> Arc<WindowsRunKeyBackend> {
        self.backend.clone()
    }

    pub fn subkey(&self) -> &str {
        &self.run_subkey
    }
}

#[cfg(test)]
impl Drop for SandboxRunKey {
    fn drop(&mut self) {
        let wide_root = to_wide(&self.root_subkey);
        // SAFETY: RegDeleteTreeW recursively removes the test-specific subkey under HKCU.
        unsafe {
            RegDeleteTreeW(HKEY_CURRENT_USER, wide_root.as_ptr());
            let wide_parent = to_wide("Software\\OrbitTest");
            RegDeleteKeyW(HKEY_CURRENT_USER, wide_parent.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn sandbox_register_list_unregister_roundtrip() {
        let sandbox = SandboxRunKey::new();
        let backend = sandbox.backend();

        assert!(!backend.is_registered("OrbitAppTest").unwrap());

        let entry = AutostartEntry {
            name: "OrbitAppTest".to_string(),
            exe_path: PathBuf::from("C:\\fake\\orbit.exe"),
        };
        backend.register(&entry).unwrap();
        assert!(backend.is_registered("OrbitAppTest").unwrap());

        let list = backend.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "OrbitAppTest");
        assert_eq!(list[0].command, "\"C:\\fake\\orbit.exe\" --hidden");

        backend.unregister("OrbitAppTest").unwrap();
        assert!(!backend.is_registered("OrbitAppTest").unwrap());
    }

    #[test]
    fn sandbox_value_is_stored_as_reg_sz_with_exact_command() {
        let sandbox = SandboxRunKey::new();
        let backend = sandbox.backend();

        let entry = AutostartEntry {
            name: "ExactCmd".to_string(),
            exe_path: PathBuf::from("D:\\orbit\\test.exe"),
        };
        backend.register(&entry).unwrap();

        let list = backend.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].command, "\"D:\\orbit\\test.exe\" --hidden");
    }

    #[test]
    fn sandbox_unregister_missing_is_ok() {
        let sandbox = SandboxRunKey::new();
        let backend = sandbox.backend();
        assert!(backend.unregister("NeverRegistered").is_ok());
    }

    #[test]
    fn sandbox_guard_cleans_up_when_test_panics() {
        let root_key_to_check = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let root_clone = root_key_to_check.clone();

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let sandbox = SandboxRunKey::new();
            *root_clone.lock().unwrap() = sandbox.root_subkey.clone();
            sandbox
                .backend()
                .register(&AutostartEntry {
                    name: "PanicTest".to_string(),
                    exe_path: PathBuf::from("C:\\fake\\panic.exe"),
                })
                .unwrap();
            panic!("Simulated panic inside test");
        }));

        let root_key = root_key_to_check.lock().unwrap().clone();
        assert!(!root_key.is_empty());
        let wide_root = to_wide(&root_key);
        let mut hkey = std::ptr::null_mut();
        // SAFETY: Check if key exists after drop
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                wide_root.as_ptr(),
                0,
                KEY_READ,
                &mut hkey,
            )
        };
        if status == ERROR_SUCCESS {
            unsafe {
                RegCloseKey(hkey);
            }
            panic!("Registry subkey should have been deleted by SandboxRunKey Drop guard!");
        }
        assert_eq!(status, ERROR_FILE_NOT_FOUND);
    }

    #[test]
    fn sandbox_isolated_from_production_run_key() {
        let prod = WindowsRunKeyBackend::production();
        let prod_before = prod.list().unwrap();

        let sandbox = SandboxRunKey::new();
        sandbox
            .backend()
            .register(&AutostartEntry {
                name: "OrbitTestEntry".to_string(),
                exe_path: PathBuf::from("C:\\fake\\orbit.exe"),
            })
            .unwrap();

        let prod_after = prod.list().unwrap();
        assert_eq!(prod_before, prod_after, "Production Run key was modified by sandbox!");
    }
}
