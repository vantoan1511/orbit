# Plan: Bridge Release & Zip-Free Updater Migration (v0.14.7)

**Source PRD**: [`.agents/PRPs/prds/bridge-release-updater-migration.prd.md`](file:///d:/Projects/orbit/.agents/PRPs/prds/bridge-release-updater-migration.prd.md)

---

## Architecture & Strategy Overview

```
┌────────────────────────────────────────────────────────────────────────┐
│                      UPDATER TRANSITION ARCHITECTURE                   │
│                                                                        │
│  [Legacy Clients <= 0.14.4]                                            │
│        │ Queries frozen manifest                                       │
│        ▼                                                               │
│  update-manifest.json (on main branch)                                 │
│        │ Permanently pinned to v0.14.7                                 │
│        ▼                                                               │
│  v0.14.7/orbit-update.zip (Final Zip Ever Released)                    │
│        │ orbit-apply.exe unpacks and runs NSIS installer               │
│        ▼                                                               │
│  Upgraded to Orbit v0.14.7 (Tauri Desktop App)                         │
│        │                                                               │
│        ▼ (Future Updates: v0.15.0+)                                    │
│  GitHub Releases API (/repos/vantoan1511/orbit/releases/latest)        │
│        │ Resolves Orbit_0.15.0_x64-setup.exe directly                  │
│        │ Handles: Rate limits (403/429), missing assets, prereleases   │
│        ▼                                                               │
│  Atomic Download (.part staging) & Safe Spawn (Only exit on success)   │
│  (No Zip, No orbit-apply.exe)                                          │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Step-by-Step Tasks

### Task 1: Legacy Bridge Manifest Pinning & Release Workflow Gating with Node Semver

- **ACTION**:
  1. Pin `update-manifest.json` permanently to version `0.14.7` pointing to `https://github.com/vantoan1511/orbit/releases/download/v0.14.7/orbit-update.zip`.
  2. Update `.github/workflows/release.yml` using Node semver (avoiding PowerShell `[version]` limitations on prereleases):
     - Step `Package Update Zip`: only executes if `node -e "const s = require('semver'); process.exit(s.lte(process.env.VERSION, '0.14.7') ? 0 : 1)"` succeeds.
     - Step `Update Manifest`: only executes if version `<= 0.14.7`.
     - Step `Create Release`: uses `dist/*.zip` glob or conditional asset staging so absent zip on `v0.15.0+` does not fail the release action.
  3. Update `src/utils/__tests__/updateManifest.test.ts` to assert the `v0.14.7` bridge contract, semver gating, and optional zip glob upload.
- **TEST_FIRST**:
  - Test file: `src/utils/__tests__/updateManifest.test.ts`
  - Tests:
    - `assert.equal(manifest.version, '0.14.7')`
    - `assert.match(manifest.url, /releases\/download\/v0\.14\.7\/orbit-update\.zip$/)`
    - `assert.ok(content.includes('0.14.7'), 'release.yml must gate zip packaging to versions <= 0.14.7')`
    - `assert.ok(content.includes('semver.lte') || content.includes('s.lte'), 'release.yml must use semver for version comparison')`
- **IMPLEMENT**:
  - Update `update-manifest.json`:
    ```json
    {
      "version": "0.14.7",
      "url": "https://github.com/vantoan1511/orbit/releases/download/v0.14.7/orbit-update.zip"
    }
    ```
  - In `.github/workflows/release.yml`, update `Package Update Zip` and `Update Manifest` steps with Node semver gating and change release files pattern to include `dist/*.zip`.
- **MICRO_VALIDATE**:
  - `node --test src/utils/__tests__/updateManifest.test.ts`
  - `python .agents/scripts/verify_shift_left.py --changed`

---

### Task 2: GitHub Releases Discovery, Asset Matching, Semver & Rate Limit Handling in Rust

- **ACTION**:
  Enhance `src-tauri/src/updater.rs` to fetch updates via the GitHub Releases API (`/repos/vantoan1511/orbit/releases/latest`) with explicit handling for:
  - Finding Windows setup asset (`*_setup.exe` or `Orbit_*.exe`, excluding `.msi`, `.zip`, `.sig`).
  - Graceful fallback on missing NSIS asset (returns `has_update = false` instead of panicking or wrong download).
  - Robust semver parsing (handles leading `v`, prereleases like `-rc.1`, ignores malformed tags).
  - Rate limiting (HTTP 403/429 logged as warning and returns `None` without crashing).
- **TEST_FIRST**:
  - Test file: `src-tauri/src/updater.rs`
  - Test names:
    - `test_find_windows_installer_asset`: Verifies selecting `Orbit_0.15.0_x64-setup.exe` while ignoring `.msi` and `.zip`.
    - `test_find_windows_installer_asset_missing`: Verifies that if release only contains `.msi` or `.zip`, returns `None`.
    - `test_manifest_from_github_release_newer_version`: Verifies release tag `v0.15.0` against `0.14.7` yields `has_update = true` with direct setup URL.
    - `test_manifest_from_github_release_same_or_lower`: Verifies equal or lower versions yield `has_update = false`.
    - `test_manifest_from_github_release_prerelease`: Verifies semver precedence on prerelease tags.
    - `test_manifest_from_github_release_invalid_tag`: Verifies non-semver tags (e.g. `nightly`) are safely ignored without panic.
- **IMPLEMENT**:
  - Add `GithubReleaseAsset` struct to `src-tauri/src/updater.rs`:
    ```rust
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct GithubReleaseAsset {
        pub name: String,
        pub browser_download_url: String,
        pub size: Option<u64>,
    }
    ```
  - Implement `find_windows_installer(assets: &[GithubReleaseAsset]) -> Option<&GithubReleaseAsset>`.
  - Implement `UpdateManifest::fetch_from_github(current_version: &str) -> Result<Option<UpdateManifest>, ...>` with 403/429 rate limit guard.
- **MICRO_VALIDATE**:
  - `cargo test --manifest-path src-tauri/Cargo.toml updater::tests`
  - `python .agents/scripts/verify_shift_left.py --changed`

---

### Task 3: Atomic Download & Safe Process Elevation/Spawn in Tauri Update Handler

- **ACTION**:
  Refactor `src-tauri/src/ipc/handlers/update.rs` (`check_for_updates` and `apply_update`):
  1. `check_for_updates`: Uses `UpdateManifest::fetch_from_github` with fallback.
  2. `apply_update`:
     - Downloads into atomic staging file: `%TEMP%/orbit_updates/Orbit_{version}_setup.exe.part`.
     - Validates download size/completion, then renames to `%TEMP%/orbit_updates/Orbit_{version}_setup.exe`.
     - Spawns installer via `Command::new(&installer_path).spawn()`.
     - **Elevation/Spawn Safety**: ONLY if `spawn()` returns `Ok(child)` does the handler trigger `std::process::exit(0)`. If `spawn()` fails (e.g. UAC rejected, permission denied), logs error, sends `OrbitEvent::ErrorOccurred`, and keeps Orbit running!
     - Completely removes obsolete `orbit-apply.exe` invocation and `.zip` archive logic.
- **TEST_FIRST**:
  - Test file: `src-tauri/src/updater.rs`
  - Tests:
    - `test_atomic_download_target_paths`: Verifies generated `.part` and final `.exe` path names include version and correct temp directory.
- **IMPLEMENT**:
  - Implement atomic file staging and safe spawn handling in `src-tauri/src/ipc/handlers/update.rs`.
- **MICRO_VALIDATE**:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `python .agents/scripts/verify_shift_left.py --changed`

---

### Task 4: Documentation & Gotcha Update

- **ACTION**:
  Update `.agents/references/gotchas/updater-backward-compat.md` to document the complete two-phase updater lifecycle and all edge case mitigations:
  - Phase 1: Neutralino v0.14.4 → Tauri v0.14.7 Bridge via `orbit-update.zip`.
  - Phase 2: Tauri v0.14.7 → Future Releases via direct NSIS setup installer without `.zip` archives.
  - Edge cases covered: Rate limits, missing assets, atomic `.part` downloads, UAC spawn verification, semver prereleases, and CI workflow gating.
- **TEST_FIRST**:
  - Inspect document for accuracy and cross-link to PRD and Plan.
- **IMPLEMENT**:
  - Update `.agents/references/gotchas/updater-backward-compat.md`.
- **MICRO_VALIDATE**:
  - `python .agents/scripts/verify_shift_left.py --changed`

---

### Task 5: Full Shift-Left Verification Suite

- **ACTION**:
  Run all project test runners, type checking, and linters across Rust and TypeScript.
- **TEST_FIRST**:
  - Node tests: `npm test` (all 304+ tests must pass).
  - Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml` (all tests must pass).
- **MICRO_VALIDATE**:
  - `npm run type-check`
  - `npm run lint`
  - `python .agents/scripts/verify_shift_left.py --all-checks`
