use crate::autostart::backend::RegisteredEntry;

pub const LEGACY_DEV_ENTRY_NAME: &str = "Orbit Dev";
pub const ORBIT_BINARY_PREFIXES: &[&str] = &["orbit_app"];

fn extract_exe_path(command: &str) -> &str {
    let trimmed = command.trim_start();
    if let Some(stripped) = trimmed.strip_prefix('"') {
        if let Some(end_idx) = stripped.find('"') {
            &stripped[..end_idx]
        } else {
            stripped
        }
    } else {
        trimmed.split_whitespace().next().unwrap_or(trimmed)
    }
}

pub fn is_cargo_deps_path(path_or_command: &str) -> bool {
    let normalized = path_or_command.replace('\\', "/").to_lowercase();
    if let Some(pos) = normalized.find("/target/") {
        normalized[pos..].contains("/deps/")
    } else if let Some(stripped) = normalized.strip_prefix("target/") {
        stripped.contains("/deps/")
    } else {
        false
    }
}

pub fn is_stale_entry(entry: &RegisteredEntry) -> bool {
    if entry.name == LEGACY_DEV_ENTRY_NAME {
        return true;
    }

    if !is_cargo_deps_path(&entry.command) {
        return false;
    }

    let exe_token = extract_exe_path(&entry.command);
    let normalized_exe = exe_token.replace('\\', "/").to_lowercase();
    let file_name = normalized_exe.rsplit('/').next().unwrap_or("");
    ORBIT_BINARY_PREFIXES
        .iter()
        .any(|prefix| file_name.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stale_entry_table() {
        let cases = vec![
            (
                "Orbit Dev",
                "\"D:\\Projects\\orbit\\target\\debug\\deps\\orbit_app_lib-f951346a968e7625.exe\" --hidden",
                true,
            ),
            (
                "Orbit Dev",
                "\"C:\\Program Files\\Orbit\\orbit-app.exe\" --hidden",
                true,
            ),
            (
                "Orbit",
                "\"D:\\x\\target\\release\\deps\\orbit_app_lib-abc.exe\" --hidden",
                true,
            ),
            (
                "Anything",
                "D:/x/TARGET/Debug/Deps/orbit_app-1a2b.exe",
                true,
            ),
            (
                "Orbit",
                "\"C:\\Users\\u\\AppData\\Local\\Orbit\\orbit-app.exe\" --hidden",
                false,
            ),
            (
                "Steam",
                "\"D:\\Program Files (x86)\\Steam\\steam.exe\" -silent",
                false,
            ),
            (
                "Other",
                "\"D:\\proj\\target\\debug\\deps\\unrelated-1.exe\"",
                false,
            ),
        ];

        for (name, command, expected) in cases {
            let entry = RegisteredEntry {
                name: name.to_string(),
                command: command.to_string(),
            };
            assert_eq!(
                is_stale_entry(&entry),
                expected,
                "Failed assertion for entry: name={}, command={}",
                name,
                command
            );
        }
    }
}
