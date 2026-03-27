use anyhow::{bail, Result};
use serde::Deserialize;

use crate::credentials;
use crate::s3::S3Client;
use crate::settings::Settings;

pub enum StorageClient {
    S3(S3Client),
    Gw(GwClient),
}

pub struct GwClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct ListResponse {
    keys: Vec<String>,
}

impl StorageClient {
    pub async fn from_settings() -> Result<Self> {
        let settings = Settings::load()?;
        if let Some(url) = settings.api_gateway_url {
            if !url.starts_with("https://") {
                bail!("api_gateway_url must use HTTPS (got: {})", url);
            }
            let token = credentials::load_token()?;
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?;
            Ok(StorageClient::Gw(GwClient { base_url: url, token, client }))
        } else {
            Ok(StorageClient::S3(S3Client::from_settings().await?))
        }
    }

    pub fn describe(&self) -> String {
        match self {
            StorageClient::S3(c) => format!("bucket: {}, region: {}", c.bucket, c.region),
            StorageClient::Gw(c) => format!("api_gateway_url: {}", c.base_url),
        }
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        match self {
            StorageClient::S3(c) => c.get_object(key).await,
            StorageClient::Gw(c) => c.get_object(key).await,
        }
    }

    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
        tags: Option<&str>,
    ) -> Result<()> {
        match self {
            StorageClient::S3(c) => c.put_object(key, body, content_type, tags).await,
            StorageClient::Gw(c) => c.put_object(key, body, content_type, tags).await,
        }
    }

    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        match self {
            StorageClient::S3(c) => c.list_objects(prefix).await,
            StorageClient::Gw(c) => c.list_objects(prefix).await,
        }
    }

    pub async fn list_skill_names(&self) -> Result<Vec<String>> {
        match self {
            StorageClient::S3(c) => c.list_skill_names().await,
            StorageClient::Gw(c) => c.list_skill_names().await,
        }
    }

    pub async fn object_exists(&self, key: &str) -> Result<bool> {
        match self {
            StorageClient::S3(c) => c.object_exists(key).await,
            StorageClient::Gw(c) => c.object_exists(key).await,
        }
    }
}

impl GwClient {
    fn object_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), key)
    }

    fn list_url(&self) -> String {
        format!("{}/", self.base_url.trim_end_matches('/'))
    }

    fn handle_status(&self, status: reqwest::StatusCode, key: &str) -> Result<()> {
        if status.is_success() {
            return Ok(());
        }
        match status.as_u16() {
            401 => bail!("Unauthorized. Run: scaffold login to refresh your token"),
            403 => bail!(
                "Forbidden. Ensure you are connected to the VPN and your token is valid. \
                 Run: scaffold login to refresh your token."
            ),
            _ => bail!("API Gateway error {} for {}", status, key),
        }
    }

    async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get(self.object_url(key))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Request failed for {}: {}", key, e))?;

        let status = resp.status();
        if status.as_u16() == 404 {
            bail!("Not found: {}", key);
        }
        self.handle_status(status, key)?;

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| anyhow::anyhow!("Failed to read response body for {}: {}", key, e))
    }

    async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
        tags: Option<&str>,
    ) -> Result<()> {
        let mut req = self
            .client
            .put(self.object_url(key))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", content_type)
            .body(body);

        if let Some(t) = tags {
            req = req.header("x-amz-tagging", t);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Upload failed for {}: {}", key, e))?;

        self.handle_status(resp.status(), key)
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        let resp = self
            .client
            .get(self.list_url())
            .query(&[("prefix", prefix)])
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("List request failed: {}", e))?;

        self.handle_status(resp.status(), "/")?;

        let list: ListResponse = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse list response: {}", e))?;

        Ok(list.keys)
    }

    async fn list_skill_names(&self) -> Result<Vec<String>> {
        let keys = self.list_objects("").await?;
        let mut names: Vec<String> = keys
            .iter()
            .filter_map(|k| k.split('/').next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        names.sort();
        Ok(names)
    }

    async fn object_exists(&self, key: &str) -> Result<bool> {
        let resp = self
            .client
            .get(self.object_url(key))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Request failed for {}: {}", key, e))?;

        let status = resp.status();
        if status.as_u16() == 404 {
            return Ok(false);
        }
        self.handle_status(status, key)?;
        Ok(true)
    }
}
