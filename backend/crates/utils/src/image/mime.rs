pub fn normalize_image_mime(value: &str) -> Option<&'static str> {
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match mime.as_str() {
        "image/png" => Some("image/png"),
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/webp" => Some("image/webp"),
        _ => None,
    }
}

pub fn is_supported_vision_mime(value: &str) -> bool {
    normalize_image_mime(value).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_image_mimes() {
        assert_eq!(normalize_image_mime("image/png"), Some("image/png"));
        assert_eq!(normalize_image_mime("IMAGE/JPEG"), Some("image/jpeg"));
        assert_eq!(normalize_image_mime("image/jpg"), Some("image/jpeg"));
        assert_eq!(
            normalize_image_mime("image/webp; charset=binary"),
            Some("image/webp")
        );
        assert_eq!(normalize_image_mime("image/gif"), None);
    }
}
