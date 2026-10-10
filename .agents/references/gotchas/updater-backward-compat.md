# Gotcha: Update Bridge for Runtime-Breaking Framework Migrations

## Context

When migrating from one desktop app runtime (e.g. Neutralino → Tauri) while maintaining an in-app updater:

- Old users already have the **old updater binary** (`orbit-apply.exe`) installed on their machines.
- The old updater binary's behavior is **frozen** — you cannot patch or alter it before the update is applied.
- The transitional release MUST satisfy the old updater's exact artifact expectations.
- However, shipping legacy `.zip` payloads indefinitely for all future releases adds unnecessary CI debt and maintenance overhead.

## The Two-Phase Updater Lifecycle

### Phase 1: Legacy Bridge (v0.14.4 → v0.14.7)

1. **Legacy Clients (`<= 0.14.4`)**:
   - Fetch `https://raw.githubusercontent.com/vantoan1511/orbit/main/update-manifest.json`.
   - Read `{ "version": "0.14.7", "url": ".../v0.14.7/orbit-update.zip" }`.
   - `orbit-apply.exe` downloads the zip, unpacks `orbit-win_x64.exe` (the renamed NSIS installer), and launches the installer GUI.
   - User machine is upgraded to Orbit `v0.14.7` (Tauri runtime).
2. **Permanent Manifest Capping**:
   - `update-manifest.json` on `main` is **permanently frozen at v0.14.7**.
   - Any legacy client that launches months or years later will only see up to `0.14.7` and upgrade via the bridge zip.

### Phase 2: Native Installer Updater (v0.14.7 → Future Releases v0.15.0+)

1. **Direct GitHub Releases Discovery**:
   - `src-tauri` queries `https://api.github.com/repos/vantoan1511/orbit/releases/latest` directly.
   - Matches the NSIS setup installer asset (`Orbit_<version>_x64-setup.exe`).
   - Ignores `.msi`, `.zip`, `.sig`, and malformed tags.
   - Gracefully handles GitHub API rate limits (HTTP 403 / 429) without crashing or disrupting the user.
2. **Atomic Download & Safe Elevation**:
   - Streams the installer directly to `%TEMP%/orbit_updates/Orbit_<version>_setup.exe.part`.
   - Validates stream completion, flushes, and renames to `.exe`.
   - Spawns the installer via `std::process::Command::new(&installer_path).spawn()`.
   - **Critical Safety Guard**: Orbit **only exits** (`std::process::exit(0)`) if `spawn()` succeeds (`Ok(child)`). If the user cancels the Windows UAC elevation prompt or OS permissions fail, Orbit stays running and emits `OrbitEvent::ErrorOccurred`.
3. **Zero Zip Maintenance for Future Releases**:
   - CI `.github/workflows/release.yml` gates `Package Update Zip` and `Update Manifest` to versions `<= 0.14.7` via `semver.lte`.
   - Versions `> 0.14.7` (e.g. `v0.15.0+`) do not package or upload `orbit-update.zip`.

## Rules & Takeaways

- **Never drop release assets** referenced by existing in-field clients without a permanent stepping-stone bridge.
- **Freeze legacy manifests** at the bridge version so old updaters never encounter unsupported future formats.
- **Always verify spawn success before exiting** the host application when delegating to an external installer.
