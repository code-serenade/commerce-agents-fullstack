use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("storage config error: {0}")]
    Config(String),

    #[error("invalid object key: {0}")]
    InvalidKey(String),

    #[error("invalid file extension: {0}")]
    InvalidExtension(String),

    #[error("s3 error: {0}")]
    S3(#[from] toolcraft_s3_kit::error::Error),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
