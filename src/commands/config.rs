use anyhow::{bail, Result};
use crate::settings::Settings;

const VALID_KEYS: &[&str] = &["bucket", "region", "api_gateway_url"];

pub fn get(key: &str) -> Result<()> {
    validate_key(key)?;
    let settings = Settings::load()?;
    let value = match key {
        "bucket" => settings.bucket,
        "region" => settings.region,
        "api_gateway_url" => settings.api_gateway_url,
        _ => unreachable!(),
    };
    match value {
        Some(v) => println!("{}", v),
        None => bail!("{} is not set", key),
    }
    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<()> {
    validate_key(key)?;
    let mut settings = Settings::load()?;
    match key {
        "bucket" => settings.bucket = Some(value.to_string()),
        "region" => settings.region = Some(value.to_string()),
        "api_gateway_url" => settings.api_gateway_url = Some(value.to_string()),
        _ => unreachable!(),
    }
    settings.save()?;
    println!("Set {} = {}", key, value);
    Ok(())
}

fn validate_key(key: &str) -> Result<()> {
    if !VALID_KEYS.contains(&key) {
        bail!("Unknown key '{}'. Valid keys: {}", key, VALID_KEYS.join(", "));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use tempfile::TempDir;
    use std::path::PathBuf;

    fn run_get(key: &str, path: &PathBuf) -> Result<Option<String>> {
        validate_key(key)?;
        let settings = Settings::load_from(path)?;
        match key {
            "bucket" => match settings.bucket {
                Some(v) => Ok(Some(v)),
                None => bail!("bucket is not set"),
            },
            _ => unreachable!(),
        }
    }

    fn run_set(key: &str, value: &str, path: &PathBuf) -> Result<()> {
        validate_key(key)?;
        let mut settings = Settings::load_from(path)?;
        match key {
            "bucket" => settings.bucket = Some(value.to_string()),
            _ => unreachable!(),
        }
        settings.save_to(path)
    }

    #[test]
    fn get_returns_bucket_value() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"bucket":"test-bucket"}"#).unwrap();
        let result = run_get("bucket", &path).unwrap();
        assert_eq!(result, Some("test-bucket".to_string()));
    }

    #[test]
    fn get_errors_when_bucket_not_set() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let err = run_get("bucket", &path).unwrap_err();
        assert!(err.to_string().contains("bucket is not set"));
    }

    #[test]
    fn get_errors_for_unknown_key() {
        let err = validate_key("unknown").unwrap_err();
        assert!(err.to_string().contains("Unknown key 'unknown'"));
        assert!(err.to_string().contains("bucket"));
    }

    #[test]
    fn set_writes_bucket_to_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        run_set("bucket", "my-org-bucket", &path).unwrap();
        let settings = Settings::load_from(&path).unwrap();
        assert_eq!(settings.bucket.as_deref(), Some("my-org-bucket"));
    }

    #[test]
    fn set_overwrites_existing_bucket() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"bucket":"old-bucket"}"#).unwrap();
        run_set("bucket", "new-bucket", &path).unwrap();
        let settings = Settings::load_from(&path).unwrap();
        assert_eq!(settings.bucket.as_deref(), Some("new-bucket"));
    }

    #[test]
    fn set_errors_for_unknown_key() {
        let err = validate_key("foo").unwrap_err();
        assert!(err.to_string().contains("Unknown key 'foo'"));
    }

    #[test]
    fn set_creates_file_when_absent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        assert!(!path.exists());
        run_set("bucket", "fresh-bucket", &path).unwrap();
        assert!(path.exists());
    }
}
