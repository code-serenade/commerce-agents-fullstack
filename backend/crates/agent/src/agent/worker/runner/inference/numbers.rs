use crate::agent::worker::runner::policy::worker_policies;

pub(super) fn numbers_in_text(text: &str) -> Vec<f64> {
    let mut values = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
            continue;
        }
        push_parsed_number(&mut values, &mut current);
    }
    push_parsed_number(&mut values, &mut current);
    values
}

pub(in crate::agent::worker::runner) fn infer_period_days(text: &str) -> i64 {
    infer_explicit_period_days(text).unwrap_or_else(|| {
        let min = config_i64("/period_days/min", 1) as f64;
        let max = config_i64("/period_days/max", 365) as f64;
        numbers_in_text(text)
            .iter()
            .find(|value| **value >= min && **value <= max)
            .map(|value| value.round() as i64)
            .unwrap_or_else(|| config_i64("/period_days/default", 30))
    })
}

pub(super) fn infer_explicit_period_days(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[.. start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start ..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start .. end].parse::<i64>() else {
            continue;
        };
        if !(config_i64("/period_days/min", 1) ..= config_i64("/period_days/max", 365))
            .contains(&value)
        {
            continue;
        }

        let unit_tail = text[end ..].trim_start();
        let Some(multiplier) = explicit_period_multiplier(unit_tail) else {
            continue;
        };

        let before = text[.. start]
            .chars()
            .rev()
            .take(config_i64("/explicit_period/scan_before_chars", 10).max(0) as usize)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        if contains_any(
            &before,
            &config_string_list("/explicit_period/before_keywords"),
        ) || starts_with_any(
            unit_tail,
            &config_string_list("/explicit_period/tail_keywords"),
        ) {
            return Some((value * multiplier).clamp(
                config_i64("/period_days/min", 1),
                config_i64("/period_days/max", 365),
            ));
        }
    }
    None
}

fn explicit_period_multiplier(unit_tail: &str) -> Option<i64> {
    let lower = unit_tail.to_ascii_lowercase();
    config_array("/explicit_period/units")
        .into_iter()
        .find_map(|unit| {
            let multiplier = unit.get("multiplier")?.as_i64()?;
            let prefixes = unit.get("prefixes")?.as_array()?;
            prefixes
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|prefix| unit_tail.starts_with(prefix) || lower.starts_with(prefix))
                .then_some(multiplier)
        })
}

pub(in crate::agent::worker::runner) fn infer_integer_or_default(text: &str, default: i64) -> i64 {
    numbers_in_text(text)
        .first()
        .copied()
        .map(|value| value.round() as i64)
        .unwrap_or(default)
}

pub(in crate::agent::worker::runner) fn infer_percent_number(text: &str, default: f64) -> f64 {
    numbers_in_text(text).first().copied().unwrap_or(default)
}

pub(in crate::agent::worker::runner) fn infer_second_percent_number(
    text: &str,
    default: f64,
) -> f64 {
    numbers_in_text(text).get(1).copied().unwrap_or(default)
}

fn contains_any<T: AsRef<str>>(text: &str, needles: &[T]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles.iter().any(|needle| {
        let needle = needle.as_ref();
        text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
    })
}

fn starts_with_any<T: AsRef<str>>(text: &str, prefixes: &[T]) -> bool {
    prefixes
        .iter()
        .any(|prefix| text.starts_with(prefix.as_ref()))
}

fn config_i64(pointer: &str, fallback: i64) -> i64 {
    worker_policies()
        .inference
        .i64_or("numbers", pointer, fallback)
}

fn config_string_list(pointer: &str) -> Vec<String> {
    worker_policies().inference.string_list("numbers", pointer)
}

fn config_array(pointer: &str) -> Vec<serde_json::Value> {
    worker_policies()
        .inference
        .section("numbers")
        .and_then(|value| value.pointer(pointer))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn push_parsed_number(values: &mut Vec<f64>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    if let Ok(value) = current.parse::<f64>()
        && value.is_finite()
    {
        values.push(value);
    }
    current.clear();
}
