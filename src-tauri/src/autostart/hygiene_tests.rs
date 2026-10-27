#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

fn src_rs_files() -> Vec<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);
    files
}

#[test]
fn run_key_literal_only_in_windows_backend() {
    let part1 = "Software\\Microsoft\\Windows\\CurrentVersion\\";
    let part2 = "Run";
    let needle = format!("{}{}", part1, part2);

    let files = src_rs_files();
    for file in files {
        let path_str = file.to_string_lossy().replace('\\', "/");
        if path_str.ends_with("src/autostart/windows.rs") || path_str.ends_with("src/autostart/hygiene_tests.rs") {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        assert!(
            !content.contains(&needle),
            "Forbidden production Run key literal found in: {}",
            file.display()
        );
    }
}

#[test]
fn current_exe_only_in_allowed_files() {
    let needle = "current_exe";
    let allowed_suffixes = [
        "src/autostart/service.rs",
        "src/autostart/provider.rs",
        "src/autostart/hygiene_tests.rs",
    ];

    let files = src_rs_files();
    for file in files {
        let path_str = file.to_string_lossy().replace('\\', "/");
        if allowed_suffixes.iter().any(|allowed| path_str.ends_with(allowed)) {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        assert!(
            !content.contains(needle),
            "Forbidden current_exe() call found in: {}",
            file.display()
        );
    }
}

#[test]
fn no_test_constructs_platform_backend_for_current_os() {
    let needle = "for_current_os(";
    let allowed_suffixes = [
        "src/autostart/file_backend.rs",
        "src/autostart/service.rs",
        "src/autostart/hygiene_tests.rs",
    ];

    let files = src_rs_files();
    for file in files {
        let path_str = file.to_string_lossy().replace('\\', "/");
        if allowed_suffixes.iter().any(|allowed| path_str.ends_with(allowed)) {
            continue;
        }

        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        assert!(
            !content.contains(needle),
            "Forbidden for_current_os() call found in: {}",
            file.display()
        );
    }
}
