use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{Error, Result};

const DEFAULT_USER_KEY_SALT: &str = "change-me-user-key-salt";
const MAX_OBJECT_KEY_LEN: usize = 1024;

pub fn user_scope_key(user_id: &str, salt: Option<&str>) -> String {
    let salt = salt.unwrap_or(DEFAULT_USER_KEY_SALT);
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(user_id.as_bytes());
    let digest = hasher.finalize();
    let mut scope = String::with_capacity(32);
    for b in digest.iter().take(16) {
        scope.push_str(&format!("{b:02x}"));
    }
    scope
}

pub fn avatar_key(user_scope: &str) -> String {
    format!("avatars/{user_scope}")
}

pub fn image_key(user_scope: &str, ext: &str) -> String {
    format!("images/{user_scope}/{}.{}", object_id(), ext)
}

pub fn document_key(user_scope: &str, ext: &str) -> String {
    format!("docs/{user_scope}/{}.{}", object_id(), ext)
}

pub fn chat_attachment_key(user_scope: &str, conversation_id: &str, ext: &str) -> Result<String> {
    let conversation_id = sanitize_path_segment(conversation_id)?;
    Ok(format!(
        "chat/{user_scope}/{conversation_id}/{}.{}",
        object_id(),
        ext
    ))
}

pub fn validate_object_key(key: &str) -> Result<()> {
    let key = key.trim();
    if key.is_empty() {
        return Err(Error::InvalidKey("object key is empty".to_string()));
    }
    if key.len() > MAX_OBJECT_KEY_LEN {
        return Err(Error::InvalidKey(format!(
            "object key length exceeds {MAX_OBJECT_KEY_LEN}"
        )));
    }
    if key.starts_with('/') {
        return Err(Error::InvalidKey(
            "object key must not start with '/'".to_string(),
        ));
    }
    if key.contains("://") || key.contains('\\') {
        return Err(Error::InvalidKey(
            "object key must not contain URL scheme or backslash".to_string(),
        ));
    }
    if key
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(Error::InvalidKey(
            "object key contains invalid path segment".to_string(),
        ));
    }

    Ok(())
}

fn sanitize_path_segment(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 128 {
        return Err(Error::InvalidKey(
            "path segment length must be between 1 and 128".to_string(),
        ));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(Error::InvalidKey(
            "path segment may only contain letters, numbers, '-' and '_'".to_string(),
        ));
    }
    Ok(value.to_string())
}

fn object_id() -> String {
    Uuid::new_v4().as_simple().to_string()
}
