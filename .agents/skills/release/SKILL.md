---
name: release
description: Bump version in package.json, neutralino.config.json, and Cargo.toml files, update lock files, commit, tag, and push to trigger automated release workflow
---

# Orbit Release Workflow

This skill guides the release process for Orbit. When the user requests a new version release, execute the following steps to bump all version numbers, update lock files, commit, tag, and push.

## Step 1: Identify Target Version
1. Ask the user for the release version (e.g., `1.0.9`) or propose the next logical patch bump based on the version currently defined in [neutralino.config.json](./neutralino.config.json).
2. Validate that the version follows Semantic Versioning (e.g., `MAJOR.MINOR.PATCH`).

## Step 2: Bump Version Numbers
Update the version string in the following files:
1. `"version"` in [neutralino.config.json](./neutralino.config.json)
2. `version` under `[package]` in [core/engine/Cargo.toml](./core/engine/Cargo.toml)
3. `version` under `[package]` in [core/updater/Cargo.toml](./core/updater/Cargo.toml)
5. `"version"` in [package.json](./package.json)

## Step 3: Update Lock Files
Ensure that dependency and lock files are updated to reflect the new version:
1. Run `npm install` in the project root to update `package-lock.json`.
2. Run `cargo check` inside the `core` directory (or run it via workspace) to update `core/Cargo.lock` with the new crate version.

## Step 4: Create Pull Request & Merge to Main
To ensure GitHub automatic release notes (`release.yml`) extract and include changelog items properly, release changes must be merged into `main` via a Pull Request (or tagged PR):
1. Stage all modified files and commit on a release feature branch:
```powershell
git add package.json package-lock.json neutralino.config.json core/engine/Cargo.toml core/updater/Cargo.toml core/Cargo.lock
git commit -m "chore: release v<VERSION>"
```
*(Replace `<VERSION>` with the actual target version, e.g., `1.0.9`)*

2. Push the release branch to remote and create/merge a Pull Request to `main` (ensure the PR title uses conventional prefix like `chore: release v<VERSION>` or includes appropriate labels).

## Step 5: Create and Push Tag
After merging the release PR into `main`, checkout `main`, pull latest, create a git tag, and push it to trigger the automated GitHub Actions release workflow:
```powershell
git checkout main
git pull origin main
git tag v<VERSION>
git push origin v<VERSION>
```
*(Replace `<VERSION>` with the actual target version)*

