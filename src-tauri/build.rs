use std::fs;
use std::path::{Path, PathBuf};

fn is_non_empty_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.len() > 0)
        .unwrap_or(false)
}

fn try_auto_stage_from_workspace(
    manifest_dir: &Path,
    target_triple: &str,
    profile: &str,
    ext: &str,
    staged_path: &Path,
) -> Result<bool, String> {
    let Some(workspace_root) = manifest_dir.parent() else {
        return Ok(false);
    };

    let cargo_bin_name = format!("orbit{ext}");
    let candidate_sources = [
        workspace_root
            .join("target")
            .join(target_triple)
            .join(profile)
            .join(&cargo_bin_name),
        workspace_root
            .join("target")
            .join(profile)
            .join(&cargo_bin_name),
    ];

    for source_path in candidate_sources {
        if !is_non_empty_file(&source_path) {
            continue;
        }
        if let Some(parent) = staged_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create Tauri sidecar directory {}: {err}",
                    parent.display()
                )
            })?;
        }
        fs::copy(&source_path, staged_path).map_err(|err| {
            format!(
                "Failed to auto-stage GPUI sidecar from {} to {}: {err}",
                source_path.display(),
                staged_path.display()
            )
        })?;
        return Ok(true);
    }

    Ok(false)
}

fn ensure_gpui_sidecar_staged() -> Result<(), String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|err| format!("CARGO_MANIFEST_DIR is not set: {err}"))?;
    let target_triple =
        std::env::var("TARGET").map_err(|err| format!("TARGET is not set: {err}"))?;
    let profile =
        std::env::var("PROFILE").map_err(|err| format!("PROFILE is not set: {err}"))?;
    let ext = if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        ".exe"
    } else {
        ""
    };

    let staged_name = format!("orbit-gpui-{target_triple}{ext}");
    let staged_path = manifest_dir.join("binaries").join(&staged_name);
    println!("cargo:rerun-if-changed=binaries/{staged_name}");

    if is_non_empty_file(&staged_path)
        || try_auto_stage_from_workspace(&manifest_dir, &target_triple, &profile, ext, &staged_path)?
    {
        return Ok(());
    }

    let build_cmd = if profile == "release" {
        "npm run gpui:build:release"
    } else {
        "npm run gpui:build:dev"
    };

    Err(format!(
        "Missing Tauri externalBin GPUI sidecar at {}. Run `{build_cmd}` (or `npm run tauri:dev` / `npm run tauri:build`) to compile and stage `orbit-gpui` before building `orbit-app`.",
        staged_path.display()
    ))
}

fn main() {
    if let Err(err) = ensure_gpui_sidecar_staged() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }

    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // Ensure unit-test binaries (which do not receive tauri_build's bin-only resource.lib)
        // embed the Common Controls v6 manifest required by TaskDialogIndirect in comctl32.dll.
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}
