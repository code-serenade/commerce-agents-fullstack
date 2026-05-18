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
        numbers_in_text(text)
            .iter()
            .find(|value| **value >= 1.0 && **value <= 365.0)
            .map(|value| value.round() as i64)
            .unwrap_or(30)
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
        if !(1 ..= 365).contains(&value) {
            continue;
        }

        let unit_tail = text[end ..].trim_start();
        let Some(multiplier) = explicit_period_multiplier(unit_tail) else {
            continue;
        };

        let before = text[.. start]
            .chars()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        if contains_any(
            &before,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        ) || unit_tail.starts_with("天内")
            || unit_tail.starts_with("日内")
            || unit_tail.starts_with("周内")
            || unit_tail.starts_with("月内")
        {
            return Some((value * multiplier).clamp(1, 365));
        }
    }
    None
}

fn explicit_period_multiplier(unit_tail: &str) -> Option<i64> {
    let lower = unit_tail.to_ascii_lowercase();
    if unit_tail.starts_with('天') || unit_tail.starts_with('日') || lower.starts_with("day") {
        Some(1)
    } else if unit_tail.starts_with('周') || lower.starts_with("week") {
        Some(7)
    } else if unit_tail.starts_with('月') || lower.starts_with("month") {
        Some(30)
    } else if unit_tail.starts_with('年') || lower.starts_with("year") {
        Some(365)
    } else {
        None
    }
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

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
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
