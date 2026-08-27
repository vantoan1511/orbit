use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateManifest {
    pub version: String,
    pub url: String,
    #[serde(default)]
    pub release_notes: Option<String>,
}

#[derive(Deserialize)]
struct GithubReleaseItem {
    #[serde(default)]
    tag_name: String,
    body: Option<String>,
}

impl UpdateManifest {
    /// Fetch the latest update manifest from the given URL.
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

        let list_api_url = "https://api.github.com/repos/vantoan1511/orbit/releases?per_page=30";
        if let Ok(resp) = client.get(list_api_url).send().await {
            if resp.status().is_success() {
                if let Ok(releases) = resp.json::<Vec<GithubReleaseItem>>().await {
                    if let Some(notes) = Self::aggregate_release_notes(&releases, target_version, current_version) {
                        return Some(notes);
                    }
                }
            }
        }

        // Fallback: try tag endpoint if list fails or is empty
        let tag = if target_version.starts_with('v') {
            target_version.to_string()
        } else {
            format!("v{}", target_version)
        };
        let api_url = format!("https://api.github.com/repos/vantoan1511/orbit/releases/tags/{}", tag);

        let resp = client.get(&api_url).send().await.ok()?;
        if resp.status().is_success() {
            let release = resp.json::<GithubReleaseItem>().await.ok()?;
            release.body
        } else {
            // Fallback: try latest release endpoint if specific tag lookup fails
            let fallback_url = "https://api.github.com/repos/vantoan1511/orbit/releases/latest";
            let resp = client.get(fallback_url).send().await.ok()?;
            if resp.status().is_success() {
                let release = resp.json::<GithubReleaseItem>().await.ok()?;
                release.body
            } else {
                None
            }
        }
    }

    /// Check if an update is available.
    pub fn has_update(&self, current_version: &str) -> Result<bool, semver::Error> {
        let current = semver::Version::parse(current_version)?;
        let remote = semver::Version::parse(&self.version)?;
        Ok(remote > current)
    }

    /// Download a file from the given URL to a temporary directory.
    /// Returns the path to the downloaded file.
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

        let mut combined_notes = String::new();

        for release in releases {
            if let Ok(release_sem) = semver::Version::parse(release.tag_name.trim_start_matches('v')) {
                if release_sem > current_sem && release_sem <= target_sem {
                    if let Some(body) = &release.body {
                        let trimmed_body = body.trim();
                        if !trimmed_body.is_empty() {
                            if !combined_notes.is_empty() {
                                combined_notes.push_str("\n\n");
                            }
                            combined_notes.push_str(&format!("## Release {}\n\n{}", release.tag_name, trimmed_body));
                        }
                    }
                }
            }
        }

        if !combined_notes.is_empty() {
            Some(combined_notes)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_release_notes_multiple_intermediate_versions() {
        let releases = vec![
            GithubReleaseItem {
                tag_name: "v0.7.1".to_string(),
                body: Some("### Features\n- Fix bug A".to_string()),
            },
            GithubReleaseItem {
                tag_name: "v0.7.0".to_string(),
                body: Some("### Features\n- Add feature B".to_string()),
            },
            GithubReleaseItem {
                tag_name: "v0.6.0".to_string(),
                body: Some("### Features\n- Old feature".to_string()),
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "0.7.1", "0.6.0");
        assert!(result.is_some());
        let notes = result.unwrap();
        assert!(notes.contains("## Release v0.7.1\n\n### Features\n- Fix bug A"));
        assert!(notes.contains("## Release v0.7.0\n\n### Features\n- Add feature B"));
        assert!(!notes.contains("v0.6.0"));
    }

    #[test]
    fn test_aggregate_release_notes_single_version() {
        let releases = vec![
            GithubReleaseItem {
                tag_name: "v0.7.1".to_string(),
                body: Some("### Features\n- Fix bug A".to_string()),
            },
            GithubReleaseItem {
                tag_name: "v0.7.0".to_string(),
                body: Some("### Features\n- Add feature B".to_string()),
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
                tag_name: "v0.6.0".to_string(),
                body: Some("### Features\n- Old feature".to_string()),
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "0.7.1", "0.7.0");
        assert!(result.is_none());
    }
}

