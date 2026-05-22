use base64::{Engine as _, engine::general_purpose::STANDARD};
use thiserror::Error;

use super::mime::normalize_image_mime;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImageDataUrlError {
    #[error("unsupported image mime type: {0}")]
    UnsupportedMime(String),

    #[error("image decode failed: {0}")]
    Decode(String),

    #[error("image encode failed: {0}")]
    Encode(String),
}

pub type Result<T, E = ImageDataUrlError> = core::result::Result<T, E>;

pub fn image_data_url(mime: &str, bytes: &[u8]) -> Result<String> {
    let mime = normalize_image_mime(mime)
        .ok_or_else(|| ImageDataUrlError::UnsupportedMime(mime.to_string()))?;
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_data_url() {
        let data_url = image_data_url("image/png", &[0, 1, 2]).unwrap();
        assert_eq!(data_url, "data:image/png;base64,AAEC");
    }

    #[test]
    fn rejects_unsupported_mime() {
        let err = image_data_url("application/pdf", &[0]).unwrap_err();
        assert_eq!(
            err,
            ImageDataUrlError::UnsupportedMime("application/pdf".to_string())
        );
    }
}
