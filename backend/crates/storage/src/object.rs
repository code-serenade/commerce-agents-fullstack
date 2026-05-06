use bytes::Bytes;

use crate::BucketKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Avatar,
    Image,
    Document,
    ChatAttachment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectLocation {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct UploadObjectRequest {
    pub bucket: BucketKind,
    pub key: String,
    pub body: Bytes,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteObjectRequest {
    pub bucket: BucketKind,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct PresignUploadRequest {
    pub bucket: BucketKind,
    pub key: String,
    pub content_type: Option<String>,
    pub expires_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PresignDownloadRequest {
    pub bucket: BucketKind,
    pub key: String,
    pub expires_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedUpload {
    pub method: String,
    pub upload_url: String,
    pub bucket: String,
    pub key: String,
    pub content_type: Option<String>,
    pub expires_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedDownload {
    pub method: String,
    pub download_url: String,
    pub bucket: String,
    pub key: String,
    pub expires_secs: Option<u64>,
}

impl FileKind {
    pub fn normalize_ext(self, ext: &str) -> crate::Result<String> {
        let ext = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        if ext.is_empty() {
            return Err(crate::Error::InvalidExtension(
                "file extension is empty".to_string(),
            ));
        }

        let allowed = match self {
            Self::Avatar | Self::Image => IMAGE_ALLOWED_EXTS,
            Self::Document | Self::ChatAttachment => GENERAL_ALLOWED_EXTS,
        };

        if allowed.contains(&ext.as_str()) {
            Ok(ext)
        } else {
            Err(crate::Error::InvalidExtension(ext))
        }
    }

    pub fn content_type_for_ext(self, ext: &str) -> &'static str {
        match ext {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "txt" => "text/plain; charset=utf-8",
            "md" => "text/markdown; charset=utf-8",
            "json" => "application/json",
            _ => match self {
                Self::Avatar | Self::Image => "application/octet-stream",
                Self::Document | Self::ChatAttachment => "application/octet-stream",
            },
        }
    }
}

const IMAGE_ALLOWED_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];
const GENERAL_ALLOWED_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "gif", "pdf", "doc", "docx", "xls", "xlsx", "txt", "md", "json",
];
