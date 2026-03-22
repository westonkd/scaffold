use anyhow::{bail, Result};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::primitives::ByteStream;

pub struct S3Client {
    inner: aws_sdk_s3::Client,
    pub bucket: String,
    pub region: String,
}

impl S3Client {
    pub async fn from_settings() -> Result<Self> {
        let settings = crate::settings::Settings::load()?;
        let bucket = settings.bucket.ok_or_else(|| {
            anyhow::anyhow!(
                "bucket is not configured. Run: scaffold config set bucket <bucket-arn>"
            )
        })?;
        let region_provider = match settings.region {
            Some(r) => aws_config::meta::region::RegionProviderChain::first_try(
                aws_sdk_s3::config::Region::new(r),
            ),
            None => aws_config::meta::region::RegionProviderChain::default_provider()
                .or_else(aws_sdk_s3::config::Region::new("us-east-1")),
        };
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        let region_str = shared_config
            .region()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let inner = aws_sdk_s3::Client::new(&shared_config);
        Ok(Self { inner, bucket, region: region_str })
    }

    pub async fn object_exists(&self, key: &str) -> Result<bool> {
        match self.inner.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(_) => Ok(true),
            Err(SdkError::ServiceError(e)) => {
                if matches!(e.err(), HeadObjectError::NotFound(_)) {
                    Ok(false)
                } else {
                    bail!(
                        "Failed to check existence of s3://{}/{}: HTTP {}",
                        self.bucket,
                        key,
                        e.raw().status()
                    )
                }
            }
            Err(e) => bail!("Failed to check existence of s3://{}/{}: {}", self.bucket, key, e),
        }
    }

    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
        tags: Option<&str>,
    ) -> Result<()> {
        let mut req = self
            .inner
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body))
            .content_type(content_type);

        if let Some(t) = tags {
            req = req.tagging(t);
        }

        match req.send().await {
            Ok(_) => {}
            Err(SdkError::ServiceError(e)) => {
                bail!(
                    "Failed to upload s3://{}/{}: HTTP {} - {:?}",
                    self.bucket,
                    key,
                    e.raw().status(),
                    e.err()
                )
            }
            Err(e) => bail!("Failed to upload s3://{}/{}: {}", self.bucket, key, e),
        }

        Ok(())
    }

    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self
                .inner
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token {
                req = req.continuation_token(token);
            }

            let resp = req.send().await.map_err(|e| {
                anyhow::anyhow!("Failed to list objects in s3://{}/{}: {}", self.bucket, prefix, e)
            })?;

            for obj in resp.contents() {
                if let Some(key) = obj.key() {
                    keys.push(key.to_string());
                }
            }

            if resp.is_truncated().unwrap_or(false) {
                continuation_token = resp.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(keys)
    }

    pub async fn list_skill_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self
                .inner
                .list_objects_v2()
                .bucket(&self.bucket)
                .delimiter("/");

            if let Some(token) = continuation_token {
                req = req.continuation_token(token);
            }

            let resp = req.send().await.map_err(|e| {
                anyhow::anyhow!("Failed to list skills in s3://{}: {}", self.bucket, e)
            })?;

            for prefix in resp.common_prefixes() {
                if let Some(p) = prefix.prefix() {
                    names.push(p.trim_end_matches('/').to_string());
                }
            }

            if resp.is_truncated().unwrap_or(false) {
                continuation_token = resp.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(names)
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        match self.inner.get_object().bucket(&self.bucket).key(key).send().await {
            Ok(resp) => {
                let bytes = resp.body.collect().await.map_err(|e| {
                    anyhow::anyhow!("Failed to read body of s3://{}/{}: {}", self.bucket, key, e)
                })?;
                Ok(bytes.into_bytes().to_vec())
            }
            Err(SdkError::ServiceError(e)) => {
                bail!(
                    "Failed to download s3://{}/{}: HTTP {} - {:?}",
                    self.bucket,
                    key,
                    e.raw().status(),
                    e.err()
                )
            }
            Err(e) => bail!("Failed to download s3://{}/{}: {}", self.bucket, key, e),
        }
    }
}
