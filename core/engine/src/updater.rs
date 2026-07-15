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
            
            if let Some(tx) = &progress_tx {
                if total_size > 0 {
                    let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
                    let _ = tx.send(progress).await;
                }
            }
        }
        
        Ok(temp_path)
    }
}

