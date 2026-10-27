use std::path::PathBuf;

use crate::autostart::backend::{
    AutostartBackend, AutostartEntry, AutostartError, RegisteredEntry,
};

pub trait EntryFormat: Send + Sync {
    fn file_name(&self, app_name: &str) -> String;
    fn render(&self, entry: &AutostartEntry) -> String;
    fn parse(&self, content: &str) -> Option<RegisteredEntry>;
}

pub struct DesktopEntryFormat;

impl EntryFormat for DesktopEntryFormat {
    fn file_name(&self, app_name: &str) -> String {
        format!("{}.desktop", app_name.to_lowercase())
    }

    fn render(&self, entry: &AutostartEntry) -> String {
        let exec_str = crate::autostart::format_linux_desktop_exec(&entry.exe_path);
        format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec={}\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
            entry.name, exec_str
        )
    }

    fn parse(&self, content: &str) -> Option<RegisteredEntry> {
        let mut name = None;
        let mut command = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(val) = trimmed.strip_prefix("Name=") {
                name = Some(val.to_string());
            } else if let Some(val) = trimmed.strip_prefix("Exec=") {
                command = Some(val.to_string());
            }
        }

        match (name, command) {
            (Some(name), Some(command)) => Some(RegisteredEntry { name, command }),
            _ => None,
        }
    }
}

pub struct LaunchAgentFormat;

impl EntryFormat for LaunchAgentFormat {
    fn file_name(&self, app_name: &str) -> String {
        format!("{}.plist", app_name)
    }

    fn render(&self, entry: &AutostartEntry) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--hidden</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
            entry.name,
            entry.exe_path.to_string_lossy()
        )
    }

    fn parse(&self, content: &str) -> Option<RegisteredEntry> {
        let label_key = "<key>Label</key>";
        let label_pos = content.find(label_key)?;
        let label_substr = &content[label_pos + label_key.len()..];
        let name_start = label_substr.find("<string>")? + "<string>".len();
        let name_end = label_substr[name_start..].find("</string>")? + name_start;
        let name = label_substr[name_start..name_end].trim().to_string();

        let args_key = "<key>ProgramArguments</key>";
        let args_pos = content.find(args_key)?;
        let args_substr = &content[args_pos + args_key.len()..];
        let array_start = args_substr.find("<array>")? + "<array>".len();
        let array_end = args_substr[array_start..].find("</array>")? + array_start;
        let array_content = &args_substr[array_start..array_end];

        let mut args = Vec::new();
        let mut rest = array_content;
        while let Some(start_tag) = rest.find("<string>") {
            let after_start = &rest[start_tag + "<string>".len()..];
            if let Some(end_tag) = after_start.find("</string>") {
                args.push(after_start[..end_tag].trim().to_string());
                rest = &after_start[end_tag + "</string>".len()..];
            } else {
                break;
            }
        }

        if args.is_empty() {
            return None;
        }

        let exe = &args[0];
        let command = if args.len() > 1 {
            format!("\"{}\" {}", exe, args[1..].join(" "))
        } else {
            format!("\"{}\"", exe)
        };

        Some(RegisteredEntry { name, command })
    }
}

pub struct FileAutostartBackend {
    dir: PathBuf,
    format: Box<dyn EntryFormat>,
}

impl FileAutostartBackend {
    pub fn new(dir: PathBuf, format: Box<dyn EntryFormat>) -> Self {
        Self { dir, format }
    }

    #[cfg(target_os = "linux")]
    pub fn for_current_os() -> Result<Self, AutostartError> {
        let home = std::env::var_os("HOME").ok_or(AutostartError::HomeDirUnavailable)?;
        let mut dir = PathBuf::from(home);
        dir.push(".config");
        dir.push("autostart");
        Ok(Self::new(dir, Box::new(DesktopEntryFormat)))
    }

    #[cfg(target_os = "macos")]
    pub fn for_current_os() -> Result<Self, AutostartError> {
        let home = std::env::var_os("HOME").ok_or(AutostartError::HomeDirUnavailable)?;
        let mut dir = PathBuf::from(home);
        dir.push("Library");
        dir.push("LaunchAgents");
        Ok(Self::new(dir, Box::new(LaunchAgentFormat)))
    }
}

impl AutostartBackend for FileAutostartBackend {
    fn is_registered(&self, name: &str) -> Result<bool, AutostartError> {
        let file_path = self.dir.join(self.format.file_name(name));
        Ok(file_path.exists())
    }

    fn register(&self, entry: &AutostartEntry) -> Result<(), AutostartError> {
        std::fs::create_dir_all(&self.dir).map_err(|e| AutostartError::Backend {
            code: "io_error",
            detail: e.to_string(),
        })?;
        let file_path = self.dir.join(self.format.file_name(&entry.name));
        let content = self.format.render(entry);
        std::fs::write(&file_path, content).map_err(|e| AutostartError::Backend {
            code: "io_error",
            detail: e.to_string(),
        })?;
        Ok(())
    }

    fn unregister(&self, name: &str) -> Result<(), AutostartError> {
        let file_path = self.dir.join(self.format.file_name(name));
        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(|e| AutostartError::Backend {
                code: "io_error",
                detail: e.to_string(),
            })?;
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<RegisteredEntry>, AutostartError> {
        if !self.dir.exists() {
            return Ok(Vec::new());
        }

        let read_dir = std::fs::read_dir(&self.dir).map_err(|e| AutostartError::Backend {
            code: "io_error",
            detail: e.to_string(),
        })?;

        let mut list = Vec::new();
        for entry_res in read_dir {
            let entry = entry_res.map_err(|e| AutostartError::Backend {
                code: "io_error",
                detail: e.to_string(),
            })?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(registered) = self.format.parse(&content) {
                        list.push(registered);
                    }
                }
            }
        }

        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("orbit_autostart_test_{}_{}", std::process::id(), id));
            let _ = std::fs::remove_dir_all(&path);
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn desktop_format_renders_exec_with_hidden_and_lowercase_filename() {
        let fmt = DesktopEntryFormat;
        assert_eq!(fmt.file_name("Orbit Dev"), "orbit dev.desktop");

        let entry = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("/usr/bin/orbit"),
        };
        let rendered = fmt.render(&entry);
        assert!(rendered.contains("Name=Orbit\n"));
        assert!(rendered.contains("Exec=\"/usr/bin/orbit\" --hidden\n"));

        let parsed = fmt.parse(&rendered).unwrap();
        assert_eq!(parsed.name, "Orbit");
        assert_eq!(parsed.command, "\"/usr/bin/orbit\" --hidden");
    }

    #[test]
    fn launch_agent_format_renders_plist_with_label_and_hidden_arg() {
        let fmt = LaunchAgentFormat;
        assert_eq!(fmt.file_name("Orbit"), "Orbit.plist");

        let entry = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("/Applications/Orbit.app/Contents/MacOS/orbit"),
        };
        let rendered = fmt.render(&entry);
        assert!(rendered.contains("<string>Orbit</string>"));
        assert!(rendered.contains("<string>/Applications/Orbit.app/Contents/MacOS/orbit</string>"));
        assert!(rendered.contains("<string>--hidden</string>"));

        let parsed = fmt.parse(&rendered).unwrap();
        assert_eq!(parsed.name, "Orbit");
        assert_eq!(
            parsed.command,
            "\"/Applications/Orbit.app/Contents/MacOS/orbit\" --hidden"
        );
    }

    #[test]
    fn register_creates_parent_dir_and_file() {
        let temp = TempDirGuard::new();
        let target_dir = temp.path.join("nested").join("autostart");
        let backend = FileAutostartBackend::new(target_dir.clone(), Box::new(DesktopEntryFormat));

        let entry = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("/opt/orbit/bin"),
        };
        backend.register(&entry).unwrap();

        assert!(backend.is_registered("Orbit").unwrap());
        let expected_file = target_dir.join("orbit.desktop");
        assert!(expected_file.is_file());
    }

    #[test]
    fn unregister_removes_file_and_missing_is_ok() {
        let temp = TempDirGuard::new();
        let backend = FileAutostartBackend::new(temp.path.clone(), Box::new(DesktopEntryFormat));

        // Missing is Ok
        assert!(backend.unregister("Missing").is_ok());

        let entry = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("/opt/orbit/bin"),
        };
        backend.register(&entry).unwrap();
        assert!(backend.is_registered("Orbit").unwrap());

        backend.unregister("Orbit").unwrap();
        assert!(!backend.is_registered("Orbit").unwrap());
    }

    #[test]
    fn list_parses_registered_files_and_skips_unparsable() {
        let temp = TempDirGuard::new();
        std::fs::create_dir_all(&temp.path).unwrap();
        let backend = FileAutostartBackend::new(temp.path.clone(), Box::new(DesktopEntryFormat));

        let entry = AutostartEntry {
            name: "Orbit".to_string(),
            exe_path: PathBuf::from("/opt/orbit/bin"),
        };
        backend.register(&entry).unwrap();

        // Write an unparsable file
        std::fs::write(temp.path.join("garbage.desktop"), "invalid content").unwrap();

        let list = backend.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Orbit");
    }

    #[test]
    fn missing_dir_list_returns_empty() {
        let temp = TempDirGuard::new();
        let non_existent = temp.path.join("does_not_exist");
        let backend = FileAutostartBackend::new(non_existent, Box::new(DesktopEntryFormat));

        let list = backend.list().unwrap();
        assert!(list.is_empty());
    }
}
