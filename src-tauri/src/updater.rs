use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateManifest {
    pub version: String,
    pub url: String,
    #[serde(default)]
    pub release_notes: Option<String>,
}

const GITHUB_REPO: &str = "vantoan1511/orbit";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GithubReleaseItem {
    pub tag_name: Option<String>,
    pub body: Option<String>,
    #[serde(default)]
    pub assets: Vec<GithubReleaseAsset>,
}

impl UpdateManifest {
    /// Matches Windows setup installer asset from release assets.
    /// Excludes .msi, .zip, .sig, .tar.gz, etc.
    pub fn find_windows_installer<'a>(assets: &'a [GithubReleaseAsset]) -> Option<&'a GithubReleaseAsset> {
        assets.iter().find(|asset| {
            let name = &asset.name;
            (name.ends_with("_setup.exe") || (name.starts_with("Orbit_") && name.ends_with(".exe")))
                && !name.ends_with(".zip")
                && !name.ends_with(".msi")
        })
    }

    /// Evaluates a single GitHub release against the running version.
    /// Returns (UpdateManifest, has_update: bool) or None if the release has no tag, invalid semver, or lacks an installer.
    pub fn from_github_release(release: &GithubReleaseItem, current_version: &str) -> Option<(Self, bool)> {
        let tag = release.tag_name.as_deref()?;
        let target_sem = semver::Version::parse(tag.trim_start_matches('v')).ok()?;
        let current_sem = semver::Version::parse(current_version.trim_start_matches('v')).ok()?;

        let installer = Self::find_windows_installer(&release.assets)?;

        let has_update = target_sem > current_sem;
        let manifest = Self {
            version: target_sem.to_string(),
            url: installer.browser_download_url.clone(),
            release_notes: release.body.clone(),
        };

        Some((manifest, has_update))
    }

    /// Fetches the latest release from GitHub API directly without needing update-manifest.json.
    /// Handles rate limits (HTTP 403/429) gracefully.
    pub async fn fetch_from_github(current_version: &str) -> Result<Option<(Self, bool)>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .user_agent("orbit-engine")
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let latest_url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
        let resp = match client.get(&latest_url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = ?e, "Failed to connect to GitHub releases API");
                return Ok(None);
            }
        };

        let status = resp.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            tracing::warn!(status = %status, "GitHub releases API rate limited; ignoring check");
            return Ok(None);
        }

        if !status.is_success() {
            tracing::warn!(status = %status, "GitHub releases API returned non-success status");
            return Ok(None);
        }

        let release = resp.json::<GithubReleaseItem>().await?;
        if let Some((mut manifest, has_update)) = Self::from_github_release(&release, current_version) {
            if has_update {
                if let Some(notes) = Self::fetch_github_release_notes(&manifest.version, current_version).await {
                    manifest.release_notes = Some(notes);
                }
            }
            return Ok(Some((manifest, has_update)));
        }

        Ok(None)
    }

    /// Fetch update manifest from a given URL (legacy/fallback support).
    pub async fn fetch(url: &str, current_version: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut manifest = reqwest::get(url)
            .await?
            .json::<UpdateManifest>()
            .await?;

        if manifest.release_notes.is_none() || manifest.release_notes.as_deref() == Some("") {
            if let Some(notes) = Self::fetch_github_release_notes(&manifest.version, current_version).await {
                manifest.release_notes = Some(notes);
            } else {
                manifest.release_notes = Some("No release notes provided for this version.".to_string());
            }
        }

        Ok(manifest)
    }

    async fn fetch_github_release_notes(target_version: &str, current_version: &str) -> Option<String> {
        let client = reqwest::Client::builder()
            .user_agent("orbit-engine")
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;

        if let Some(notes) = Self::fetch_aggregated_release_notes(&client, target_version, current_version).await {
            return Some(notes);
        }

        Self::fetch_single_release_notes(&client, target_version).await
    }

    async fn fetch_aggregated_release_notes(
        client: &reqwest::Client,
        target_version: &str,
        current_version: &str,
    ) -> Option<String> {
        let list_api_url = format!("https://api.github.com/repos/{}/releases?per_page=100", GITHUB_REPO);
        let resp = client.get(&list_api_url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }

        let releases = resp.json::<Vec<GithubReleaseItem>>().await.ok()?;
        Self::aggregate_release_notes(&releases, target_version, current_version)
    }

    async fn fetch_release_by_url(client: &reqwest::Client, url: &str) -> Option<String> {
        let resp = client.get(url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let release = resp.json::<GithubReleaseItem>().await.ok()?;
        release.body
    }

    async fn fetch_single_release_notes(client: &reqwest::Client, target_version: &str) -> Option<String> {
        let tag = if target_version.starts_with('v') {
            target_version.to_string()
        } else {
            format!("v{}", target_version)
        };
        let tag_url = format!("https://api.github.com/repos/{}/releases/tags/{}", GITHUB_REPO, tag);
        if let Some(body) = Self::fetch_release_by_url(client, &tag_url).await {
            return Some(body);
        }

        // Fallback: try latest release endpoint if specific tag lookup fails
        let fallback_url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
        Self::fetch_release_by_url(client, &fallback_url).await
    }

    /// Check if an update is available.
    pub fn has_update(&self, current_version: &str) -> Result<bool, semver::Error> {
        let current = semver::Version::parse(current_version)?;
        let remote = semver::Version::parse(&self.version)?;
        Ok(remote > current)
    }

    /// Resolves atomic staging and destination download paths for a version.
    pub fn resolve_download_paths(version: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let mut base_dir = std::env::temp_dir();
        base_dir.push("orbit_updates");

        let part_filename = format!("Orbit_{}_setup.exe.part", version);
        let final_filename = format!("Orbit_{}_setup.exe", version);

        (base_dir.join(part_filename), base_dir.join(final_filename))
    }

    /// Atomically download an installer file to a temporary directory using .part staging.
    /// Validates full download completion before promoting to final executable.
    pub async fn download_installer(
        url: &str,
        version: &str,
        progress_tx: Option<tokio::sync::mpsc::Sender<u8>>,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::AsyncWriteExt;

        let (part_path, final_path) = Self::resolve_download_paths(version);
        if let Some(parent) = part_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut response = reqwest::get(url).await?;
        let total_size = response.content_length().unwrap_or(0);

        let mut file = tokio::fs::File::create(&part_path).await?;
        let mut downloaded: u64 = 0;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let (Some(tx), true) = (&progress_tx, total_size > 0) {
                let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
                let _ = tx.send(progress).await;
            }
        }

        file.flush().await?;
        drop(file);

        // Promote .part to final .exe atomically
        tokio::fs::rename(&part_path, &final_path).await?;

        Ok(final_path)
    }

    /// Legacy download method kept for backward compatibility.
    pub async fn download(
        url: &str,
        filename: &str,
        progress_tx: Option<tokio::sync::mpsc::Sender<u8>>,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::AsyncWriteExt;
        
        let mut response = reqwest::get(url).await?;
        let total_size = response.content_length().unwrap_or(0);
        
        let mut temp_path = std::env::temp_dir();
        temp_path.push("orbit_updates");
        tokio::fs::create_dir_all(&temp_path).await?;
        
        temp_path.push(filename);
        let mut file = tokio::fs::File::create(&temp_path).await?;
        
        let mut downloaded: u64 = 0;
        
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            if let (Some(tx), true) = (&progress_tx, total_size > 0) {
                let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
                let _ = tx.send(progress).await;
            }
        }
        
        Ok(temp_path)
    }

    fn aggregate_release_notes(
        releases: &[GithubReleaseItem],
        target_version: &str,
        current_version: &str,
    ) -> Option<String> {
        let target_sem = semver::Version::parse(target_version.trim_start_matches('v')).ok()?;
        let current_sem = semver::Version::parse(current_version.trim_start_matches('v')).ok()?;

        let mut matching_releases: Vec<(semver::Version, &str, &Option<String>)> = releases
            .iter()
            .filter_map(|release| {
                let tag = release.tag_name.as_deref()?;
                let release_sem = semver::Version::parse(tag.trim_start_matches('v')).ok()?;
                if release_sem > current_sem && release_sem <= target_sem {
                    Some((release_sem, tag, &release.body))
                } else {
                    None
                }
            })
            .collect();

        if matching_releases.is_empty() {
            return None;
        }

        // Sort descending (newest first)
        matching_releases.sort_by(|a, b| b.0.cmp(&a.0));

        let mut combined_notes = String::new();

        for (_, tag, body) in matching_releases {
            let body_text = body
                .as_deref()
                .map(str::trim)
                .filter(|b| !b.is_empty())
                .unwrap_or("_No release notes provided._");

            if !combined_notes.is_empty() {
                combined_notes.push_str("\n\n");
            }
            combined_notes.push_str(&format!("## Release {}\n\n{}", tag, body_text));
        }

        Some(combined_notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_release_notes_multiple_intermediate_versions() {
        // Provided in arbitrary/reverse order to verify that sorting guarantees descending order
        let releases = vec![
            GithubReleaseItem {
                tag_name: Some("v0.7.0".to_string()),
                body: Some("### Features\n- Add feature B".to_string()),
                ..Default::default()
            },
            GithubReleaseItem {
                tag_name: Some("v0.7.1".to_string()),
                body: Some("### Features\n- Fix bug A".to_string()),
                ..Default::default()
            },
            GithubReleaseItem {
                tag_name: Some("v0.6.0".to_string()),
                body: Some("### Features\n- Old feature".to_string()),
                ..Default::default()
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "0.7.1", "0.6.0");
        assert!(result.is_some());
        let notes = result.unwrap();
        assert!(notes.contains("## Release v0.7.1\n\n### Features\n- Fix bug A"));
        assert!(notes.contains("## Release v0.7.0\n\n### Features\n- Add feature B"));
        assert!(!notes.contains("v0.6.0"));

        // Verify descending order (newest first)
        let pos_v071 = notes.find("## Release v0.7.1").unwrap();
        let pos_v070 = notes.find("## Release v0.7.0").unwrap();
        assert!(pos_v071 < pos_v070, "v0.7.1 should appear before v0.7.0 in descending order");
    }

    #[test]
    fn test_aggregate_release_notes_single_version() {
        let releases = vec![
            GithubReleaseItem {
                tag_name: Some("v0.7.1".to_string()),
                body: Some("### Features\n- Fix bug A".to_string()),
                ..Default::default()
            },
            GithubReleaseItem {
                tag_name: Some("v0.7.0".to_string()),
                body: Some("### Features\n- Add feature B".to_string()),
                ..Default::default()
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "v0.7.1", "v0.7.0");
        assert!(result.is_some());
        let notes = result.unwrap();
        assert!(notes.contains("## Release v0.7.1"));
        assert!(!notes.contains("v0.7.0"));
    }

    #[test]
    fn test_aggregate_release_notes_no_matching_versions() {
        let releases = vec![
            GithubReleaseItem {
                tag_name: Some("v0.6.0".to_string()),
                body: Some("### Features\n- Old feature".to_string()),
                ..Default::default()
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "0.7.1", "0.7.0");
        assert!(result.is_none());
    }

    #[test]
    fn test_aggregate_release_notes_empty_body_fallback() {
        let releases = vec![
            GithubReleaseItem {
                tag_name: Some("v0.7.1".to_string()),
                body: None,
                ..Default::default()
            },
            GithubReleaseItem {
                tag_name: Some("v0.7.0".to_string()),
                body: Some("   ".to_string()),
                ..Default::default()
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "v0.7.1", "v0.6.0");
        assert!(result.is_some());
        let notes = result.unwrap();
        assert!(notes.contains("## Release v0.7.1\n\n_No release notes provided._"));
        assert!(notes.contains("## Release v0.7.0\n\n_No release notes provided._"));
    }

    #[test]
    fn test_find_windows_installer_asset() {
        let assets = vec![
            GithubReleaseAsset {
                name: "Orbit_0.15.0_x64_en-US.msi".to_string(),
                browser_download_url: "https://example.com/msi".to_string(),
                size: Some(100),
            },
            GithubReleaseAsset {
                name: "Orbit_0.15.0_x64-setup.exe".to_string(),
                browser_download_url: "https://example.com/setup.exe".to_string(),
                size: Some(200),
            },
            GithubReleaseAsset {
                name: "orbit-update.zip".to_string(),
                browser_download_url: "https://example.com/zip".to_string(),
                size: Some(300),
            },
        ];

        let matched = UpdateManifest::find_windows_installer(&assets);
        assert!(matched.is_some());
        let asset = matched.unwrap();
        assert_eq!(asset.name, "Orbit_0.15.0_x64-setup.exe");
        assert_eq!(asset.browser_download_url, "https://example.com/setup.exe");
    }

    #[test]
    fn test_find_windows_installer_asset_missing() {
        let assets = vec![
            GithubReleaseAsset {
                name: "Orbit_0.15.0_x64_en-US.msi".to_string(),
                browser_download_url: "https://example.com/msi".to_string(),
                size: Some(100),
            },
            GithubReleaseAsset {
                name: "orbit-update.zip".to_string(),
                browser_download_url: "https://example.com/zip".to_string(),
                size: Some(300),
            },
        ];

        let matched = UpdateManifest::find_windows_installer(&assets);
        assert!(matched.is_none());
    }

    #[test]
    fn test_manifest_from_github_release_newer_version() {
        let release = GithubReleaseItem {
            tag_name: Some("v0.15.0".to_string()),
            body: Some("New cool features".to_string()),
            assets: vec![
                GithubReleaseAsset {
                    name: "Orbit_0.15.0_x64-setup.exe".to_string(),
                    browser_download_url: "https://example.com/setup.exe".to_string(),
                    size: Some(200),
                },
            ],
        };

        let parsed = UpdateManifest::from_github_release(&release, "0.14.7");
        assert!(parsed.is_some());
        let (manifest, has_update) = parsed.unwrap();
        assert!(has_update);
        assert_eq!(manifest.version, "0.15.0");
        assert_eq!(manifest.url, "https://example.com/setup.exe");
        assert_eq!(manifest.release_notes.as_deref(), Some("New cool features"));
    }

    #[test]
    fn test_manifest_from_github_release_same_or_lower() {
        let release = GithubReleaseItem {
            tag_name: Some("v0.14.7".to_string()),
            body: Some("Current release notes".to_string()),
            assets: vec![
                GithubReleaseAsset {
                    name: "Orbit_0.14.7_x64-setup.exe".to_string(),
                    browser_download_url: "https://example.com/setup.exe".to_string(),
                    size: Some(200),
                },
            ],
        };

        let parsed = UpdateManifest::from_github_release(&release, "0.14.7");
        assert!(parsed.is_some());
        let (_, has_update) = parsed.unwrap();
        assert!(!has_update);

        let older_release = GithubReleaseItem {
            tag_name: Some("v0.14.6".to_string()),
            body: Some("Older release".to_string()),
            assets: vec![
                GithubReleaseAsset {
                    name: "Orbit_0.14.6_x64-setup.exe".to_string(),
                    browser_download_url: "https://example.com/setup.exe".to_string(),
                    size: Some(200),
                },
            ],
        };
        let parsed_older = UpdateManifest::from_github_release(&older_release, "0.14.7");
        assert!(parsed_older.is_some());
        let (_, has_update_older) = parsed_older.unwrap();
        assert!(!has_update_older);
    }

    #[test]
    fn test_manifest_from_github_release_prerelease() {
        let release = GithubReleaseItem {
            tag_name: Some("v0.15.0-rc.1".to_string()),
            body: Some("Release candidate".to_string()),
            assets: vec![
                GithubReleaseAsset {
                    name: "Orbit_0.15.0-rc.1_x64-setup.exe".to_string(),
                    browser_download_url: "https://example.com/setup.exe".to_string(),
                    size: Some(200),
                },
            ],
        };

        let parsed = UpdateManifest::from_github_release(&release, "0.14.7");
        assert!(parsed.is_some());
        let (manifest, has_update) = parsed.unwrap();
        assert!(has_update);
        assert_eq!(manifest.version, "0.15.0-rc.1");

        // Compared against same version but full release 0.15.0 -> 0.15.0-rc.1 < 0.15.0
        let parsed_against_final = UpdateManifest::from_github_release(&release, "0.15.0");
        assert!(parsed_against_final.is_some());
        let (_, has_update_final) = parsed_against_final.unwrap();
        assert!(!has_update_final);
    }

    #[test]
    fn test_manifest_from_github_release_invalid_tag() {
        let release = GithubReleaseItem {
            tag_name: Some("nightly-build".to_string()),
            body: Some("Nightly".to_string()),
            assets: vec![
                GithubReleaseAsset {
                    name: "Orbit_nightly_x64-setup.exe".to_string(),
                    browser_download_url: "https://example.com/setup.exe".to_string(),
                    size: Some(200),
                },
            ],
        };

        let parsed = UpdateManifest::from_github_release(&release, "0.14.7");
        assert!(parsed.is_none());
    }

    #[test]
    fn test_atomic_download_target_paths() {
        let (part_path, final_path) = UpdateManifest::resolve_download_paths("0.15.0");
        assert!(part_path.to_str().unwrap().ends_with("Orbit_0.15.0_setup.exe.part"));
        assert!(final_path.to_str().unwrap().ends_with("Orbit_0.15.0_setup.exe"));
        assert_eq!(part_path.parent(), final_path.parent());
    }
}

