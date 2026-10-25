# Gotcha: Update Bridge for Runtime-Breaking Framework Migrations

## Context

When migrating from one desktop app runtime (e.g. Neutralino → Tauri) while maintaining an in-app updater:

- Old users already have the **old updater binary** (`orbit-apply.exe`) installed on their machines.
- The old updater binary's behavior is **frozen** — you cannot patch or alter it before the update is applied.
- The transitional release MUST satisfy the old updater's exact artifact expectations.
- However, shipping legacy `.zip` payloads indefinitely for all future releases adds unnecessary CI debt and maintenance overhead.

## The Two-Phase Updater Lifecycle

### Phase 1: Legacy Bridge (v0.14.4 → v0.14.8)

1. **Legacy Clients (`<= 0.14.4`)**:
   - Fetch `https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json`.
   - Read `{ "version": "0.14.8", "url": ".../v0.14.8/orbit-update.zip" }`.
   - `orbit-apply.exe` downloads the zip, unpacks `orbit-win_x64.exe` (the renamed NSIS installer), and launches the installer GUI.
   - User machine is upgraded to Orbit `v0.14.8` (Tauri runtime).
2. **Permanent Manifest Capping**:
   - `update-manifest.json` on `main` is **permanently frozen at v0.14.8**.
   - Any legacy client that launches months or years later will only see up to `0.14.8` and upgrade via the bridge zip.

### Phase 2: Native Installer Updater (v0.14.8 → Future Releases v0.15.0+)

1. **Direct GitHub Releases Discovery**:
   - `src-tauri` queries `https://api.github.com/repos/vantoan1511/orbit/releases/latest` directly.
   - Matches the NSIS setup installer asset (`Orbit_<version>_x64-setup.exe`).
   - Ignores `.msi`, `.zip`, `.sig`, and malformed tags.
   - Gracefully handles GitHub API rate limits (HTTP 403 / 429) without crashing or disrupting the user.
2. **Atomic Download & Safe Elevation**:
   - Streams the installer directly to `%TEMP%/orbit_updates/Orbit_<version>_setup.exe.part`.
   - Validates stream completion, flushes, and renames to `.exe`.
   - Spawns the installer via `ShellInstallerLauncher` using Win32 `ShellExecuteExW` (`open` verb).
   - **Critical Safety Guard**: `OrbitEvent::UpdateReady` is emitted strictly after the launcher returns `Ok(())`, and Orbit **only exits** (`std::process::exit(0)`) after a successful launch. If the user declines the Windows UAC elevation prompt or launch fails, Orbit stays running, emits `OrbitEvent::UpdateInstallFailed`, and resets the download state.
3. **Zero Zip Maintenance for Future Releases**:
   - CI `.github/workflows/release.yml` gates `Package Update Zip` and `Update Manifest` to versions `<= 0.14.8` via `semver.lte`.
   - Versions `> 0.14.8` (e.g. `v0.15.0+`) do not package or upload `orbit-update.zip`.

## Elevation & Installer Spawning (os error 740)

### Problem
When the NSIS installer uses `installMode: both` or `perMachine`, its embedded application manifest specifies `requestedExecutionLevel highestAvailable`. Under Windows UAC with an administrator user, launching the installer executable requires elevation.
If launched via `std::process::Command::new(...).spawn()` (which wraps Win32 `CreateProcessW`), Windows fails immediately with:
```
Failed to launch installer: The requested operation requires elevation. (os error 740)
```
`CreateProcessW` cannot display a UAC elevation consent prompt.

### Solution
1. **Spawn via `ShellExecuteExW`**:
   On Windows, use `ShellExecuteExW` with the `open` verb and `SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI`. The Windows Shell properly triggers the UAC elevation dialog when required, while suppressing modal error message boxes so errors are cleanly captured via `GetLastError()`.
2. **Post-Launch `UpdateReady` Invariant**:
   Never emit `UpdateReady` prior to spawning. Only emit `UpdateReady` (and begin shutdown countdown) after the launcher reports `Ok(())`.
3. **Classification of Denial vs Failure**:
   - `ERROR_CANCELLED (1223)` and `ERROR_ACCESS_DENIED (5)` are classified as `ElevationDenied`.
   - Other codes are classified as `LaunchFailed` with detailed OS error messages.
   - On error, emit `OrbitEvent::UpdateInstallFailed` so the frontend UI can notify the user without closing Orbit or leaving the updater locked in downloading state.

### Affected Client Workaround
Clients on versions affected by the elevation issue (such as 0.16.3) cannot self-elevate via in-app update to 0.17.0. Those users must download the installer executable directly from GitHub Releases and run it manually once. Subsequent updates will use the fixed `ShellExecuteExW` launcher.

## Rules & Takeaways

- **Never drop release assets** referenced by existing in-field clients without a permanent stepping-stone bridge.
- **Freeze legacy manifests** at the bridge version so old updaters never encounter unsupported future formats.
- **Always verify spawn success before exiting** the host application when delegating to an external installer.
