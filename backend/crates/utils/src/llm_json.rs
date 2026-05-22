use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmJsonError {
    #[error("LLM output does not contain a JSON object or array")]
    MissingJson,
    #[error("invalid JSON: {source}; repair failed: {repair_source}")]
    InvalidAfterRepair {
        source: serde_json::Error,
        repair_source: serde_json::Error,
    },
}

pub fn parse_llm_json<T>(raw: &str) -> Result<T, LlmJsonError>
where
    T: DeserializeOwned,
{
    let json_text =
        extract_json_value(strip_code_fences(raw.trim())).ok_or(LlmJsonError::MissingJson)?;
    match serde_json::from_str::<T>(json_text) {
        Ok(value) => Ok(value),
        Err(source) => {
            let repaired = repair_invalid_json_string_escapes(json_text);
            serde_json::from_str::<T>(&repaired).map_err(|repair_source| {
                LlmJsonError::InvalidAfterRepair {
                    source,
                    repair_source,
                }
            })
        }
    }
}

pub fn parse_llm_json_value(raw: &str) -> Result<serde_json::Value, LlmJsonError> {
    parse_llm_json(raw)
}

pub fn extract_json_value(input: &str) -> Option<&str> {
    let mut in_string = false;
    let mut escape = false;

    for (index, ch) in input.char_indices() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' | '[' => {
                if let Some(candidate) = extract_balanced_json(&input[index ..], ch) {
                    return Some(candidate);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn strip_code_fences(input: &str) -> &str {
    let start = match input.find("```") {
        Some(idx) => idx,
        None => return input,
    };
    let after_start = &input[start + 3 ..];
    let newline = after_start.find('\n');
    let content_start = match newline {
        Some(idx) => start + 3 + idx + 1,
        None => start + 3,
    };
    if let Some(end_idx) = input[content_start ..].find("```") {
        let end = content_start + end_idx;
        return input[content_start .. end].trim();
    }
    input
}

pub fn repair_invalid_json_string_escapes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut unicode_escape_remaining = 0usize;

    while let Some(ch) = chars.next() {
        if unicode_escape_remaining > 0 {
            if ch.is_ascii_hexdigit() {
                unicode_escape_remaining -= 1;
                output.push(ch);
            } else {
                unicode_escape_remaining = 0;
                output.push(ch);
            }
            continue;
        }

        if !in_string {
            if ch == '"' {
                in_string = true;
            }
            output.push(ch);
            continue;
        }

        match ch {
            '"' => {
                in_string = false;
                output.push(ch);
            }
            '\\' => match chars.peek().copied() {
                Some('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') => {
                    output.push(ch);
                    output.push(chars.next().expect("peeked escape char exists"));
                }
                Some('u') => {
                    output.push(ch);
                    output.push(chars.next().expect("peeked unicode escape exists"));
                    unicode_escape_remaining = 4;
                }
                Some(_) => {
                    output.push_str("\\\\");
                }
                None => output.push_str("\\\\"),
            },
            _ => output.push(ch),
        }
    }

    output
}

fn extract_balanced_json(input: &str, start_char: char) -> Option<&str> {
    let end_char = match start_char {
        '{' => '}',
        '[' => ']',
        _ => return None,
    };
    let mut in_string = false;
    let mut escape = false;
    let mut depth = 0usize;
    let mut start = None;

    for (index, ch) in input.char_indices() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            ch if ch == start_char => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            ch if ch == end_char && depth > 0 => {
                depth -= 1;
                if depth == 0
                    && let Some(start_index) = start
                {
                    return Some(&input[start_index ..= index]);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{extract_json_value, parse_llm_json, repair_invalid_json_string_escapes};

    #[derive(Debug, Deserialize)]
    struct Sample {
        summary: String,
        scene: String,
        ok: String,
    }

    #[test]
    fn repairs_invalid_json_string_escapes() {
        let input = r#"{"summary":"a \* b","scene":"c \ d","ok":"line\nnext"}"#;
        let repaired = repair_invalid_json_string_escapes(input);
        let value: serde_json::Value = serde_json::from_str(&repaired).expect("repaired json");

        assert_eq!(value["summary"], "a \\* b");
        assert_eq!(value["scene"], "c \\ d");
        assert_eq!(value["ok"], "line\nnext");
    }

    #[test]
    fn parses_fenced_llm_json_with_invalid_escape() {
        let parsed: Sample =
            parse_llm_json("```json\n{\"summary\":\"a \\* b\",\"scene\":\"c\",\"ok\":\"d\"}\n```")
                .expect("parsed");

        assert_eq!(parsed.summary, "a \\* b");
        assert_eq!(parsed.scene, "c");
        assert_eq!(parsed.ok, "d");
    }

    #[test]
    fn extracts_json_after_prefix_text() {
        let json_text = extract_json_value("answer:\n{\"summary\":\"ok\"}").expect("json value");

        assert_eq!(json_text, "{\"summary\":\"ok\"}");
    }
}
