use serde::Deserialize;

use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub endpoint: String,
    pub public_bucket: String,
    pub private_bucket: String,
    #[serde(default)]
    pub region: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub user_key_salt: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketKind {
    Public,
    Private,
}

impl StorageConfig {
    pub fn normalized(self) -> Result<Self> {
        let endpoint = self.endpoint.trim().trim_end_matches('/').to_string();
        let public_bucket = self.public_bucket.trim().trim_matches('/').to_string();
        let private_bucket = self.private_bucket.trim().trim_matches('/').to_string();
        let access_key = self.access_key.trim().to_string();
        let secret_key = self.secret_key.trim().to_string();
        let region = self
            .region
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let user_key_salt = self
            .user_key_salt
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        if endpoint.is_empty() {
            return Err(Error::Config("missing s3.endpoint".to_string()));
        }
        if public_bucket.is_empty() {
            return Err(Error::Config("missing s3.public_bucket".to_string()));
        }
        if private_bucket.is_empty() {
            return Err(Error::Config("missing s3.private_bucket".to_string()));
        }
        if access_key.is_empty() {
            return Err(Error::Config("missing s3.access_key".to_string()));
        }
        if secret_key.is_empty() {
            return Err(Error::Config("missing s3.secret_key".to_string()));
        }

        Ok(Self {
            endpoint,
            public_bucket,
            private_bucket,
            region,
            access_key,
            secret_key,
            user_key_salt,
        })
    }

    pub fn bucket_name(&self, kind: BucketKind) -> &str {
        match kind {
            BucketKind::Public => &self.public_bucket,
            BucketKind::Private => &self.private_bucket,
        }
    }

    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}
