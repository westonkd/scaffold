use anyhow::{bail, Context, Result};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::primitives::ByteStream;

pub struct S3Client {
    inner: aws_sdk_s3::Client,
    pub bucket: String,
}

impl S3Client {
    pub async fn from_env() -> Result<Self> {
        let bucket = std::env::var("SCAFFOLD_BUCKET")
            .context("SCAFFOLD_BUCKET environment variable is not set")?;
        if bucket.is_empty() {
            bail!("SCAFFOLD_BUCKET environment variable is empty");
        }
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let inner = aws_sdk_s3::Client::new(&config);
        Ok(Self { inner, bucket })
    }

    pub async fn object_exists(&self, key: &str) -> Result<bool> {
        match self.inner.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(_) => Ok(true),
            Err(SdkError::ServiceError(e)) if matches!(e.err(), HeadObjectError::NotFound(_)) => {
                Ok(false)
            }
            Err(e) => Err(e).context(format!("Failed to check existence of s3://{}/{}", self.bucket, key)),
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

        req.send()
            .await
            .context(format!("Failed to upload s3://{}/{}", self.bucket, key))?;

        Ok(())
    }
}
