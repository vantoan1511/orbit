use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent of orbit-core")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn test_legacy_webview_and_tauri_files_removed() {
    let root = repo_root();
    let legacy_artifacts = [
        "package.json",
        "package-lock.json",
        "vite.config.ts",
        "index.html",
        "eslint.config.ts",
        "tsconfig.json",
        "tsconfig.node.json",
        "tsconfig.app.json",
        "src/main.ts",
        "src-tauri/tauri.conf.json",
    ];

    for rel_path in &legacy_artifacts {
        let path = root.join(rel_path);
        assert!(
            !path.exists(),
            "Legacy webview/Tauri artifact should be removed: {:?}",
            path
        );
    }
}

#[test]
fn test_shift_left_scanner_passes_on_pure_rust_workspace() {
    let root = repo_root();
    let script = root.join(".agents/scripts/verify_shift_left.py");
    assert!(script.exists(), "verify_shift_left.py must exist");

    let status = Command::new("python")
        .arg(&script)
        .arg("--all")
        .current_dir(&root)
        .status()
        .expect("execute verify_shift_left.py");

    assert!(
        status.success(),
        "verify_shift_left.py --all must pass cleanly with exit code 0"
    );
}

#[test]
fn test_legacy_iced_gui_crate_removed() {
    let root = repo_root();
    assert!(
        !root.join("crates/orbit-gui").exists(),
        "Legacy Iced GUI crate crates/orbit-gui should be removed"
    );
    assert!(
        root.join("crates/orbit-gpui/Cargo.toml").exists(),
        "Primary GPUI crate crates/orbit-gpui/Cargo.toml must exist"
    );
}
