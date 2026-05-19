pub(super) fn contains_keyword(value: &str, keyword: &str) -> bool {
    if keyword.trim().is_empty() {
        return false;
    }
    value.contains(keyword) || value.to_lowercase().contains(&keyword.to_lowercase())
}

pub(super) fn contains_string_any(value: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| contains_keyword(value, needle))
}
