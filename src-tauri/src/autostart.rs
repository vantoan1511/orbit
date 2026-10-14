//! Cross-platform OS autostart (launch at login) management for Orbit.

use std::path::PathBuf;

pub const DEFAULT_APP_NAME: &str = "Orbit";

pub struct AutostartManager {
    pub app_name: String,
    pub app_path: PathBuf,
}

impl AutostartManager {
    /// Creates a new AutostartManager resolving the current running executable path.
    pub fn new(app_name: &str) -> Result<Self, String> {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        Ok(Self {
            app_name: app_name.to_string(),
            app_path: current_exe,
        })
    }

    /// Creates an AutostartManager with an explicit application executable path (useful for testing).
    pub fn with_path(app_name: &str, app_path: PathBuf) -> Self {
        Self {
            app_name: app_name.to_string(),
            app_path,
        }
    }

    /// Checks whether autostart is currently registered with the operating system.
    pub fn is_enabled(&self) -> Result<bool, String> {
        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("reg")
                .args([
                    "query",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    &self.app_name,
                ])
                .output()
                .map_err(|e| format!("Failed to execute reg query: {}", e))?;
            Ok(output.status.success())
        }

        #[cfg(target_os = "macos")]
        {
            let path = Self::macos_plist_path(&self.app_name)?;
            Ok(path.exists())
        }

        #[cfg(target_os = "linux")]
        {
            let path = Self::linux_desktop_path(&self.app_name)?;
            Ok(path.exists())
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(false)
        }
    }

    /// Registers Orbit for automatic startup at login.
    pub fn enable(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let exe_str = format!("\"{}\"", self.app_path.to_string_lossy());
            let status = std::process::Command::new("reg")
                .args([
                    "add",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    &self.app_name,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &exe_str,
                    "/f",
                ])
                .status()
                .map_err(|e| format!("Failed to execute reg add: {}", e))?;
            if !status.success() {
                return Err("Failed to register autostart in Windows Registry".to_string());
            }
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            let path = Self::macos_plist_path(&self.app_name)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                self.app_name,
                self.app_path.to_string_lossy()
            );
            std::fs::write(&path, plist_content).map_err(|e| e.to_string())?;
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            let path = Self::linux_desktop_path(&self.app_name)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let desktop_content = format!(
                "[Desktop Entry]\nType=Application\nName={}\nExec=\"{}\"\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
                self.app_name,
                self.app_path.to_string_lossy()
            );
            std::fs::write(&path, desktop_content).map_err(|e| e.to_string())?;
            Ok(())
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(())
        }
    }

    /// Unregisters Orbit from automatic startup at login.
    pub fn disable(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("reg")
                .args([
                    "delete",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                    "/v",
                    &self.app_name,
                    "/f",
                ])
                .status();
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            let path = Self::macos_plist_path(&self.app_name)?;
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            let path = Self::linux_desktop_path(&self.app_name)?;
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            Ok(())
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    fn macos_plist_path(app_name: &str) -> Result<PathBuf, String> {
        let home = std::env::var_os("HOME").ok_or("Could not find HOME directory")?;
        let mut path = PathBuf::from(home);
        path.push("Library");
        path.push("LaunchAgents");
        path.push(format!("{}.plist", app_name));
        Ok(path)
    }

    #[cfg(target_os = "linux")]
    fn linux_desktop_path(app_name: &str) -> Result<PathBuf, String> {
        let home = std::env::var_os("HOME").ok_or("Could not find HOME directory")?;
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("autostart");
        path.push(format!("{}.desktop", app_name.to_lowercase()));
        Ok(path)
    }
}

/// Reconciles OS autostart state with saved application configuration on app startup.
pub fn reconcile_on_startup() {
    let config = crate::config::OrbitConfig::load();
    let should_autostart = config.launch_on_startup();
    match AutostartManager::new(DEFAULT_APP_NAME) {
        Ok(manager) => match manager.is_enabled() {
            Ok(is_enabled) => {
                if should_autostart && !is_enabled {
                    if let Err(e) = manager.enable() {
                        tracing::warn!(error = %e, "Failed to reconcile autostart on boot (enable)");
                    }
                } else if !should_autostart && is_enabled {
                    if let Err(e) = manager.disable() {
                        tracing::warn!(error = %e, "Failed to reconcile autostart on boot (disable)");
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query system autostart state during boot");
            }
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to resolve executable path for autostart reconciliation");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autostart_manager_creation() {
        let manager = AutostartManager::with_path("OrbitTest", PathBuf::from("C:\\fake\\orbit.exe"));
        assert_eq!(manager.app_name, "OrbitTest");
        assert_eq!(manager.app_path, PathBuf::from("C:\\fake\\orbit.exe"));
    }

    #[test]
    fn test_autostart_manager_new_resolves_current_exe() {
        let manager = AutostartManager::new("OrbitTest");
        assert!(manager.is_ok());
        let m = manager.unwrap();
        assert_eq!(m.app_name, "OrbitTest");
        assert!(m.app_path.is_absolute());
    }

    #[test]
    fn test_autostart_enable_disable_lifecycle() {
        let test_name = "OrbitUnitTestAutostart";
        let manager = AutostartManager::with_path(test_name, PathBuf::from("C:\\fake\\orbit_test.exe"));
        
        // Ensure clean initial state
        let _ = manager.disable();

        // Enable
        let enable_res = manager.enable();
        assert!(enable_res.is_ok(), "enable() must succeed: {:?}", enable_res);

        // Check is_enabled
        let is_en = manager.is_enabled().expect("is_enabled must succeed");
        assert!(is_en, "autostart should be enabled");

        // Disable
        let disable_res = manager.disable();
        assert!(disable_res.is_ok(), "disable() must succeed: {:?}", disable_res);

        // Check is_enabled again
        let is_en_after = manager.is_enabled().expect("is_enabled must succeed");
        assert!(!is_en_after, "autostart should be disabled after disable()");
    }
}
