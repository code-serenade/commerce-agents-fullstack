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
    numbers_in_text(text)
        .iter()
        .find(|value| **value >= 1.0 && **value <= 365.0)
        .map(|value| value.round() as i64)
        .unwrap_or(30)
}

pub(super) fn infer_integer_or_default(text: &str, default: i64) -> i64 {
    numbers_in_text(text)
        .first()
        .copied()
        .map(|value| value.round() as i64)
        .unwrap_or(default)
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
