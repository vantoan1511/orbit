# Spec: Bridge Release & Zip-Free Updater Migration (v0.14.7)

## 1. Intent & Problem Statement

- **User Intent**: We migrated Orbit from Neutralino to Tauri. However, we cannot support shipping legacy `.zip` update artifacts forever. Create a solution where users on version `<= 0.14.4` upgrade to a bridge release (`v0.14.7`), and from `v0.14.7` onward, Orbit switches to a modern updater that directly downloads and runs the NSIS installer executable without needing any `.zip` artifact. `update-manifest.json` is frozen at `v0.14.7`, capping legacy clients and eliminating the requirement to package `.zip` archives for future releases (`v0.15.0+`).
- **Problem**:
  1. Older clients (`<= 0.14.4`, Neutralino) have frozen binary logic that polls `https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json`, downloads `orbit-update.zip`, and invokes `orbit-apply.exe` to unzip and relaunch the app.
  2. If `update-manifest.json` continues to be bumped on every release (`v0.15.0`, `v0.16.0`, ...), CI must package and upload `orbit-update.zip` indefinitely.
  3. In `src-tauri/src/ipc/handlers/update.rs`, the current update apply handler still attempts to find `bin/orbit-apply.exe` and download `orbit-update.zip`. On Tauri installations, `orbit-apply.exe` does not exist, causing update installation to fail with file not found.
- **Target Outcome**:
  - `v0.14.7` is designated as the transitional bridge release and the final release that ships `orbit-update.zip`.
  - `update-manifest.json` on `main` is permanently capped at `v0.14.7`. Legacy clients (`<= 0.14.4`) are safely upgraded into `v0.14.7` (Tauri).
  - Orbit `v0.14.7`+ discovers new releases directly via the GitHub Releases API (`/repos/vantoan1511/orbit/releases/latest`), locates the `*_setup.exe` installer asset, downloads it, executes the installer, and cleanly exits Orbit.
  - Releases after `v0.14.7` (`v0.15.0+`) do not package or publish `orbit-update.zip` and do not touch `update-manifest.json`.

---

## 2. User Experience & Interactions

- **Placement**: Orbit `UpdateDialog.vue`, status bar update indicators, and Settings About tab.
- **Interaction Flow**:
  1. **Background / Manual Check**: Orbit queries GitHub Releases API for the latest release.
  2. **Update Dialog**: If a newer version exists (e.g. `v0.15.0`), Orbit displays `UpdateDialog.vue` showing version `v0.15.0` and aggregated release notes.
  3. **Download**: User clicks "Install". Orbit streams the download of `Orbit_0.15.0_x64-setup.exe` into `%TEMP%/orbit_updates/Orbit_0.15.0_setup.exe.part`, verifying byte counts before promoting to the runnable `.exe`.
  4. **Launch & Replace**: Once download finishes, Orbit spawns the NSIS installer GUI. ONLY upon confirmed process spawn does Orbit exit cleanly (`std::process::exit(0)`).
  5. **Upgrade**: The NSIS installer runs, overwrites the installed files, and relaunches Orbit at the new version.

---

## 3. Contract-First Boundaries

### 3.1 GitHub Releases API & Asset Contract

Rust models in `src-tauri/src/updater.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubReleaseItem {
    pub tag_name: Option<String>,
    pub body: Option<String>,
    #[serde(default)]
    pub assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateManifest {
    pub version: String,
    pub url: String,
    #[serde(default)]
    pub release_notes: Option<String>,
}
```

Asset Resolution Rule:
- Target Windows installer asset must match:
  `asset.name.ends_with("_setup.exe") || (asset.name.starts_with("Orbit_") && asset.name.ends_with(".exe"))`
- Excludes `.msi`, `.zip`, `.tar.gz`, `.sig`, `.sha256`.
- Example matching asset: `Orbit_0.15.0_x64-setup.exe`.

### 3.2 IPC Protocol Contract (Preserved)

Communication between Rust core and Vue frontend:

```typescript
// Event: updateCheckFinished
export interface UpdateCheckFinishedPayload {
  has_update: boolean
  manifest: {
    version: string
    url: string
    release_notes?: string
  }
}

// Event: updateDownloadProgress
export interface UpdateDownloadProgressPayload {
  component: string
  progress_percentage: number
}

// Event: updateReady
export interface UpdateReadyPayload {
  component: string
}
```

### 3.3 Bridge Manifest Contract (`update-manifest.json`)

Frozen permanently at `v0.14.7`:

```json
{
  "version": "0.14.7",
  "url": "https://github.com/vantoan1511/orbit/releases/download/v0.14.7/orbit-update.zip"
}
```

### 3.4 CI Release Workflow Contract (`.github/workflows/release.yml`)

- **Step: Package Update Zip**:
  - Gated using Node semver: Only executed when `semver.lte(VERSION, "0.14.7")`.
  - For versions `> 0.14.7`, this step is skipped.
- **Step: Update Manifest**:
  - Gated using Node semver: Only updates and commits `update-manifest.json` when `semver.lte(VERSION, "0.14.7")`.
  - For versions `> 0.14.7`, `update-manifest.json` remains untouched.
- **Step: Create Release Assets**:
  - `dist/*.zip` glob or conditional asset list so missing zip on `v0.15.0+` does not fail the release upload.

---

## 4. Edge Cases & Mitigations

| Edge Case | Risk | Mitigation |
|-----------|------|------------|
| **1. GitHub API Rate Limiting (HTTP 403/429)** | Check fails if user checks repeatedly or shares an unauthenticated corporate IP NAT. | Handle HTTP 403/429 status explicitly in `updater.rs`. Log a warning and fallback gracefully without crashing or showing disruptive error dialogs. |
| **2. Release Missing NSIS Asset** | CI release published, but NSIS asset failed to upload or only MSI exists. | `find_windows_installer` returns `None`. `has_update` is set to `false`, logging that no compatible installer asset is available. Never download an incompatible or corrupt file. |
| **3. Partial / Interrupted Download** | Network drops mid-download (e.g. at 50%); user tries to run truncated file. | Download to temporary `.part` file: `%TEMP%/orbit_updates/Orbit_{version}_setup.exe.part`. Check bytes received vs `Content-Length`. Only rename to `.exe` upon full, verified completion. Delete `.part` on error. |
| **4. UAC Elevation / Spawn Failure Before Exit** | User declines UAC prompt or OS refuses execution. If Orbit exits before spawn succeeds, the app closes with nothing updated. | Orbit calls `Command::new(&installer_path).spawn()`. ONLY if `spawn()` returns `Ok(child)` does Orbit call `std::process::exit(0)`. If `spawn()` returns `Err(e)`, Orbit remains open and emits `OrbitEvent::ErrorOccurred`. |
| **5. Semver Tags with Prereleases or 'v' prefix** | Tags like `v0.15.0`, `0.15.0`, `v0.15.0-rc.1`, or invalid tags like `nightly`. | Strip leading `v` and use `semver::Version::parse`. If parsing fails, safely ignore the release rather than panicking. Support standard semver precedence. |
| **6. PowerShell Semver Gating Failure on Prereleases** | PowerShell `[version]"0.14.7-rc.1"` throws exception because `System.Version` cannot parse hyphenated prereleases. | Use Node.js script in CI step: `node -e "const semver = require('semver'); process.exit(semver.lte(process.env.VERSION, '0.14.7') ? 0 : 1)"` which handles all semver formats natively. |
| **7. Missing Zip in Future Release Uploads (v0.15.0+)** | GitHub release action fails if `dist/orbit-update.zip` is hardcoded in `files:` but absent on v0.15.0+. | Use glob pattern `dist/*.zip` or conditional inclusion in `release.yml` so that absence of zip on v0.15.0+ does not cause release workflow errors. |
| **8. Stale / Conflicting Temp Files** | A previous aborted download leaves a locked or corrupted file in `%TEMP%/orbit_updates/`. | Use version-specific names (`Orbit_{version}_setup.exe`). Truncate existing file on open (`File::create`). |
| **9. Equal or Downgrade Versions** | Remote version is equal or lower than currently running version. | Enforce strict `remote_semver > current_semver`. Never signal update available for equal or lower releases. |

---

## 5. Scope Boundaries

### In Scope (v1 — v0.14.7 Bridge & Installer Updater)
1. **GitHub Releases Discovery**: Update `src-tauri/src/updater.rs` to fetch the latest release, handle rate limits, and locate the Windows `*_setup.exe` asset.
2. **Atomic Download & Direct Installer Execution**: Update `src-tauri/src/ipc/handlers/update.rs` to download with atomic `.part` staging, spawn installer, and only exit on successful spawn.
3. **Legacy Manifest Freeze**: Pin `update-manifest.json` to `v0.14.7` as the permanent bridge endpoint for legacy clients.
4. **CI Release Workflow Gating**: Condition `orbit-update.zip` packaging and manifest commits in `.github/workflows/release.yml` using Node semver so that versions `> 0.14.7` do not ship `.zip` files.
5. **Contract & Edge Case Tests**:
   - Rust unit tests in `src-tauri/src/updater.rs` for asset filtering, rate limit handling, and semver comparisons.
   - Node unit tests in `src/utils/__tests__/updateManifest.test.ts` for bridge manifest capping and workflow gating logic.

### Out of Scope
- Code signing with minisign / `tauri-plugin-updater` private keys (direct NSIS installer execution is used).
- Frontend UI modifications (`UpdateDialog.vue` already seamlessly handles download progress and triggers).
- macOS / Linux auto-update launcher (Orbit distribution is Windows-focused).

---

## 6. Acceptance Criteria

- [ ] **AC-1**: `UpdateManifest::fetch_from_github` successfully retrieves the latest release from GitHub API, matches the Windows `*_setup.exe` asset, and handles missing assets gracefully.
- [ ] **AC-2**: GitHub API rate limits (HTTP 403/429) are handled without panicking or displaying breaking error dialogs.
- [ ] **AC-3**: `apply_update` downloads installer atomically using `.part` staging and only exits Orbit if `spawn()` succeeds.
- [ ] **AC-4**: `update-manifest.json` on `main` contains `version: "0.14.7"` and `url: "...v0.14.7/orbit-update.zip"`.
- [ ] **AC-5**: `.github/workflows/release.yml` only packages and uploads `orbit-update.zip` for versions `<= 0.14.7`, using robust Node semver evaluation.
- [ ] **AC-6**: Contract tests in `src/utils/__tests__/updateManifest.test.ts` pass, asserting manifest pinning and release workflow gating.
- [ ] **AC-7**: Rust tests in `src-tauri/src/updater.rs` pass, asserting asset resolution, prerelease semver handling, and rate-limit fallbacks.
- [ ] **AC-8**: Shift-left pre-flight check (`python .agents/scripts/verify_shift_left.py --changed`) passes with zero violations.
