use std::sync::Arc;

use bytes::Bytes;
use toolcraft_s3_kit::{BucketClient, S3Client};

use crate::{
    BucketKind, DeleteObjectRequest, DownloadObjectRequest, ObjectLocation, PresignDownloadRequest,
    PresignUploadRequest, Result, SignedDownload, SignedUpload, StorageConfig, UploadObjectRequest,
    key,
};

#[derive(Clone)]
pub struct ObjectStorage {
    config: Arc<StorageConfig>,
    public_bucket: BucketClient,
    private_bucket: BucketClient,
}

impl ObjectStorage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        let config = Arc::new(config.normalized()?);
        let client = Arc::new(S3Client::new(
            &config.endpoint,
            &config.access_key,
            &config.secret_key,
            config.region(),
        )?);
        let public_bucket = BucketClient::new(Arc::clone(&client), config.public_bucket.clone());
        let private_bucket = BucketClient::new(Arc::clone(&client), config.private_bucket.clone());

        Ok(Self {
            config,
            public_bucket,
            private_bucket,
        })
    }

    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    pub fn user_scope_key(&self, user_id: &str) -> String {
        key::user_scope_key(user_id, self.config.user_key_salt.as_deref())
    }

    pub async fn upload_object(&self, req: UploadObjectRequest) -> Result<ObjectLocation> {
        key::validate_object_key(&req.key)?;
        let bucket_name = self.config.bucket_name(req.bucket).to_string();
        self.bucket(req.bucket)
            .upload_file(&req.key, req.body, req.content_type.as_deref())
            .await?;
        Ok(ObjectLocation {
            bucket: bucket_name,
            key: req.key,
        })
    }

    pub async fn delete_object(&self, req: DeleteObjectRequest) -> Result<ObjectLocation> {
        key::validate_object_key(&req.key)?;
        let bucket_name = self.config.bucket_name(req.bucket).to_string();
        self.bucket(req.bucket).delete_object(&req.key).await?;
        Ok(ObjectLocation {
            bucket: bucket_name,
            key: req.key,
        })
    }

    pub async fn download_object(&self, req: DownloadObjectRequest) -> Result<Bytes> {
        key::validate_object_key(&req.key)?;
        Ok(self.bucket(req.bucket).download_object(&req.key).await?)
    }

    pub fn presign_upload(&self, req: PresignUploadRequest) -> Result<SignedUpload> {
        key::validate_object_key(&req.key)?;
        let bucket_name = self.config.bucket_name(req.bucket).to_string();
        let upload_url = self
            .bucket(req.bucket)
            .presign_upload(&req.key, req.expires_secs);

        Ok(SignedUpload {
            method: "PUT".to_string(),
            upload_url,
            bucket: bucket_name,
            key: req.key,
            content_type: req.content_type,
            expires_secs: req.expires_secs,
        })
    }

    pub fn presign_download(&self, req: PresignDownloadRequest) -> Result<SignedDownload> {
        key::validate_object_key(&req.key)?;
        let bucket_name = self.config.bucket_name(req.bucket).to_string();
        let download_url = toolcraft_utils::presign_get_object(
            &self.config.access_key,
            &self.config.secret_key,
            &bucket_name,
            &req.key,
            self.config.region(),
            &self.config.endpoint,
            req.expires_secs,
        );

        Ok(SignedDownload {
            method: "GET".to_string(),
            download_url,
            bucket: bucket_name,
            key: req.key,
            expires_secs: req.expires_secs,
        })
    }

    pub fn public_url(&self, key: &str) -> Result<String> {
        key::validate_object_key(key)?;
        Ok(format!(
            "{}/{}/{}",
            self.config.endpoint.trim_end_matches('/'),
            self.config.public_bucket,
            key
        ))
    }

    fn bucket(&self, kind: BucketKind) -> &BucketClient {
        match kind {
            BucketKind::Public => &self.public_bucket,
            BucketKind::Private => &self.private_bucket,
        }
    }
}
