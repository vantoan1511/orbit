use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateManifest {
    pub version: String,
    pub url: String,
    #[serde(default)]
    pub release_notes: Option<String>,
}

const GITHUB_REPO: &str = "vantoan1511/orbit";

#[derive(Deserialize)]
struct GithubReleaseItem {
    tag_name: Option<String>,
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
            },
            GithubReleaseItem {
                tag_name: Some("v0.7.1".to_string()),
                body: Some("### Features\n- Fix bug A".to_string()),
            },
            GithubReleaseItem {
                tag_name: Some("v0.6.0".to_string()),
                body: Some("### Features\n- Old feature".to_string()),
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
            },
            GithubReleaseItem {
                tag_name: Some("v0.7.0".to_string()),
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
                tag_name: Some("v0.6.0".to_string()),
                body: Some("### Features\n- Old feature".to_string()),
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
            },
            GithubReleaseItem {
                tag_name: Some("v0.7.0".to_string()),
                body: Some("   ".to_string()),
            },
        ];

        let result = UpdateManifest::aggregate_release_notes(&releases, "v0.7.1", "v0.6.0");
        assert!(result.is_some());
        let notes = result.unwrap();
        assert!(notes.contains("## Release v0.7.1\n\n_No release notes provided._"));
        assert!(notes.contains("## Release v0.7.0\n\n_No release notes provided._"));
    }
}

