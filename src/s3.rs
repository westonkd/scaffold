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
}
