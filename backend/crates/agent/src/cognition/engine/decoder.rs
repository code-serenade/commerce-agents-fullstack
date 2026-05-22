use serde_json::Value;
use utils::llm_json;

use crate::{CognitionFailure, CognitionOutput, FailureReason};

/// 将 LLM 输出解析为结构化 JSON。
pub trait CognitionDecoder {
    fn decode(&self, raw: &str) -> Result<CognitionOutput, CognitionFailure>;
}

impl<T> CognitionDecoder for Box<T>
where
    T: CognitionDecoder + ?Sized,
{
    fn decode(&self, raw: &str) -> Result<CognitionOutput, CognitionFailure> {
        (**self).decode(raw)
    }
}

#[derive(Debug, Clone)]
pub struct JsonTemplateDecoder {
    template: Value,
}

impl JsonTemplateDecoder {
    pub fn new(template: Value) -> Self {
        Self { template }
    }
}

impl CognitionDecoder for JsonTemplateDecoder {
    fn decode(&self, raw: &str) -> Result<CognitionOutput, CognitionFailure> {
        let value = llm_json::parse_llm_json_value(raw).map_err(|err| CognitionFailure {
            reason: FailureReason::InternalError,
            description: format!("invalid json: {err}"),
        })?;

        validate_shape(&value, &self.template)?;
        Ok(value)
    }
}

fn validate_shape(value: &Value, template: &Value) -> Result<(), CognitionFailure> {
    validate_shape_at(value, template, "$")
}

fn validate_shape_at(value: &Value, template: &Value, path: &str) -> Result<(), CognitionFailure> {
    match template {
        Value::Object(template_map) => {
            let value_map = value
                .as_object()
                .ok_or_else(|| shape_failure(path, "expected object"))?;
            for (key, child_template) in template_map {
                let child_value = value_map
                    .get(key)
                    .ok_or_else(|| shape_failure(path, &format!("missing key '{key}'")))?;
                let child_path = format!("{path}.{key}");
                validate_shape_at(child_value, child_template, &child_path)?;
            }
            Ok(())
        }
        Value::Array(template_items) => {
            let value_items = value
                .as_array()
                .ok_or_else(|| shape_failure(path, "expected array"))?;
            if let Some(item_template) = template_items.first() {
                for (index, item) in value_items.iter().enumerate() {
                    let child_path = format!("{path}[{index}]");
                    validate_shape_at(item, item_template, &child_path)?;
                }
            }
            Ok(())
        }
        _ => {
            if value.is_array() || value.is_object() {
                Err(shape_failure(path, "expected primitive"))
            } else {
                Ok(())
            }
        }
    }
}

fn shape_failure(path: &str, message: &str) -> CognitionFailure {
    CognitionFailure {
        reason: FailureReason::InternalError,
        description: format!("json template validation failed at {path}: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CognitionDecoder, JsonTemplateDecoder};

    #[test]
    fn validates_nested_object_shape() {
        let decoder = JsonTemplateDecoder::new(json!({
            "route": {"agent": "", "message": ""}
        }));

        let value = decoder
            .decode(r#"{"route":{"agent":"worker","message":"go"}}"#)
            .expect("decode should succeed");

        assert_eq!(value["route"]["agent"], "worker");
    }

    #[test]
    fn validates_array_element_shape() {
        let decoder = JsonTemplateDecoder::new(json!([
            {"key1": "", "val1": ""}
        ]));

        decoder
            .decode(r#"[{"key1":"a","val1":"b"},{"key1":"c","val1":"d"}]"#)
            .expect("decode should succeed");
    }

    #[test]
    fn rejects_missing_required_key() {
        let decoder = JsonTemplateDecoder::new(json!({
            "action": "",
            "worker": "",
            "task": ""
        }));

        let err = decoder
            .decode(r#"{"action":"dispatch","worker":"w1"}"#)
            .expect_err("decode should fail");

        assert!(err.description.contains("missing key 'task'"));
    }
}
