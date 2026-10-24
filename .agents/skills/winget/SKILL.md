---
name: winget
description: Publish and update Orbit package manifests on Windows Package Manager (microsoft/winget-pkgs)
---

# Orbit WinGet Package Distribution

This workflow guide covers publishing Orbit releases to the Windows Package Manager (WinGet) community repository (`microsoft/winget-pkgs`).

---

## 1. Architecture & Automation Flow

```
[Developer merges Release PR to main]
                 │
                 ▼
     [.github/workflows/release.yml]
     - Builds binaries & installers
     - Updates update-manifest.json
     - Pushes Git tag (e.g. v0.9.0)
     - Publishes GitHub Release with Orbit-Setup-<version>.exe
                 │
                 ▼ (workflow_run: Release completed)
     [.github/workflows/winget.yml]
     - Resolves release tag (or uses manual input)
     - Validates WINGET_TOKEN secret
     - Syncs fork (vantoan1511/winget-pkgs) from upstream (microsoft/winget-pkgs)
     - Submits PR to microsoft/winget-pkgs via winget-releaser / komac
```

---

## 2. Prerequisites Setup

Before the workflow can successfully submit manifests to `microsoft/winget-pkgs`:

### A. Fork `microsoft/winget-pkgs`
1. Go to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).
2. Click **Fork** to create a fork under your account (`vantoan1511/winget-pkgs`).

### B. Create Personal Access Token (Classic)
1. Go to GitHub **Settings** -> **Developer settings** -> **Personal access tokens** -> **Tokens (classic)**.
2. Generate a new token with:
   - Name: `WinGet Releaser`
   - Scope: `public_repo` (required to push branches and open PRs to `microsoft/winget-pkgs`).
3. Copy the generated token.

### C. Add Repository Secret
1. In the `vantoan1511/orbit` repository, navigate to:
   **Settings** -> **Secrets and variables** -> **Actions**.
2. Click **New repository secret**.
3. Name: `WINGET_TOKEN`.
4. Value: `<paste your Personal Access Token>`.
5. Click **Add secret**.

---

## 3. One-Time Initial Onboarding

WinGet automated releasers (`komac`, `wingetcreate update`) require that the package already exists in `microsoft/winget-pkgs`.

To submit Orbit for the very first time (version `0.9.0`):

1. Clone or navigate to your fork `vantoan1511/winget-pkgs`.
2. Sync the fork to the latest `master`.
3. Create a branch:
   ```bash
   git checkout -b vantoan1511.Orbit-0.9.0
   ```
4. Copy the manifest files from `scripts/winget/` in the Orbit repository into:
   `manifests/v/vantoan1511/Orbit/0.9.0/`
   - `vantoan1511.Orbit.yaml`
   - `vantoan1511.Orbit.installer.yaml`
   - `vantoan1511.Orbit.locale.en-US.yaml`
5. Validate locally using the WinGet CLI (optional):
   ```powershell
   winget validate --manifest manifests/v/vantoan1511/Orbit/0.9.0
   ```
6. Commit, push to your fork, and open a Pull Request against `microsoft/winget-pkgs:master`.
7. Once the WinGet review bots approve and merge the initial PR, Orbit will be available via:
   ```powershell
   winget install vantoan1511.Orbit
   ```
8. **All future releases** will be automatically updated by `.github/workflows/winget.yml` without any manual steps!

---

## 4. Triggering the Workflow

### Automatic Trigger
- When `.github/workflows/release.yml` completes successfully on `main`, `winget.yml` triggers automatically via `workflow_run`.
- It can also trigger when a release is manually published via GitHub Releases (`release: published`).

### Manual Trigger
1. Go to **Actions** -> **Publish to WinGet**.
2. Click **Run workflow**.
3. (Optional) Provide a release tag (e.g. `v0.9.0`). If left blank, it automatically discovers and submits the latest GitHub release.

---

## 5. Fork Synchronization

The workflow includes an explicit synchronization step:
```powershell
gh repo sync "${{ github.repository_owner }}/winget-pkgs" --source "microsoft/winget-pkgs" --branch master --force
```
This guarantees:
- The fork is never out of date with upstream `microsoft/winget-pkgs`.
- Stale or merged branches do not cause merge conflicts.
- Fast-forward or force reset aligns the fork's `master` branch with upstream before generating new version branches.
