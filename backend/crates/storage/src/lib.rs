pub mod config;
pub mod error;
pub mod key;
pub mod object;
pub mod s3;

pub use config::{BucketKind, StorageConfig};
pub use error::{Error, Result};
pub use object::{
    DeleteObjectRequest, FileKind, ObjectLocation, PresignDownloadRequest, PresignUploadRequest,
    SignedDownload, SignedUpload, UploadObjectRequest,
};
pub use s3::ObjectStorage;
