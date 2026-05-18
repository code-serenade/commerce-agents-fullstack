use crate::agent::worker::runner::util::compact_event_text;

pub(super) const CATEGORY_LEVEL_BOUNDARY_MARKER: &str = "category_level_boundary";

pub(super) fn merge_unique(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    for item in right {
        if item.trim().is_empty() || left.iter().any(|value| value == &item) {
            continue;
        }
        left.push(item);
    }
    left
}

pub(super) fn contains_any(value: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

pub(super) fn compact_task_text(value: &str) -> String {
    compact_event_text(value.trim().replace('\n', " "))
}
