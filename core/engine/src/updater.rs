use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateManifest {
    pub version: String,
    pub url: String,
}

impl UpdateManifest {
    /// Fetch the latest update manifest from the given URL.
    pub async fn fetch(url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let manifest = reqwest::get(url)
            .await?
            .json::<UpdateManifest>()
            .await?;
        Ok(manifest)
    }

    /// Check if an update is available.
    pub fn has_update(&self, current_version: &str) -> Result<bool, semver::Error> {
        let current = semver::Version::parse(current_version)?;
        let remote = semver::Version::parse(&self.version)?;
        Ok(remote > current)
    }

    /// Download a file from the given URL to a temporary directory.
    /// Returns the path to the downloaded file.
    pub async fn download(url: &str, filename: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;
        
        let mut temp_path = std::env::temp_dir();
        temp_path.push("orbit_updates");
        tokio::fs::create_dir_all(&temp_path).await?;
        
        temp_path.push(filename);
        tokio::fs::write(&temp_path, bytes).await?;
        
        Ok(temp_path)
    }
}

