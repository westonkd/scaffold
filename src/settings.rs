use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub bucket: Option<String>,
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_gateway_url: Option<String>,
}

impl Settings {
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".scaffold").join("settings.json"))
    }

    pub fn load() -> Result<Self> {
        Self::load_from(&Self::path()?)
    }

    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::path()?)
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let contents = serde_json::to_string(self).context("Failed to serialize settings")?;
        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn settings_path(dir: &TempDir) -> PathBuf {
        dir.path().join("settings.json")
    }

    #[test]
    fn load_returns_defaults_when_file_absent() {
        let dir = TempDir::new().unwrap();
        let path = settings_path(&dir);
        let settings = Settings::load_from(&path).unwrap();
        assert!(settings.bucket.is_none());
    }

    #[test]
    fn load_reads_bucket_from_file() {
        let dir = TempDir::new().unwrap();
        let path = settings_path(&dir);
        std::fs::write(&path, r#"{"bucket":"my-bucket"}"#).unwrap();
        let settings = Settings::load_from(&path).unwrap();
        assert_eq!(settings.bucket.as_deref(), Some("my-bucket"));
    }

    #[test]
    fn save_writes_bucket_to_file() {
        let dir = TempDir::new().unwrap();
        let path = settings_path(&dir);
        let settings = Settings { bucket: Some("my-bucket".to_string()), region: None, api_gateway_url: None };
        settings.save_to(&path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, r#"{"bucket":"my-bucket","region":null}"#);
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("dir").join("settings.json");
        let settings = Settings { bucket: Some("my-bucket".to_string()), region: None, api_gateway_url: None };
        settings.save_to(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn load_handles_null_bucket() {
        let dir = TempDir::new().unwrap();
        let path = settings_path(&dir);
        std::fs::write(&path, r#"{"bucket":null}"#).unwrap();
        let settings = Settings::load_from(&path).unwrap();
        assert!(settings.bucket.is_none());
    }

    #[test]
    fn roundtrip_preserves_bucket() {
        let dir = TempDir::new().unwrap();
        let path = settings_path(&dir);
        let original = Settings { bucket: Some("roundtrip-bucket".to_string()), region: None, api_gateway_url: None };
        original.save_to(&path).unwrap();
        let loaded = Settings::load_from(&path).unwrap();
        assert_eq!(loaded.bucket, original.bucket);
    }
}
