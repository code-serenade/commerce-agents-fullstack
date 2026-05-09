use std::collections::BTreeMap;

use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamRequest {
    #[validate(length(min = 1, max = 64))]
    pub agent_id: Option<String>,

    #[validate(length(min = 1, max = 128))]
    pub session_id: Option<String>,

    #[validate(length(min = 1, max = 128))]
    pub workspace_id: Option<String>,

    #[validate(length(min = 1, max = 20000))]
    pub input: String,

    #[serde(default)]
    pub metadata: AgentStreamMetadata,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamMetadata {
    pub context: Option<AgentStreamContextMetadata>,
    pub strategy: Option<AgentStreamStrategyMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamContextMetadata {
    #[serde(default)]
    pub product_ids: Vec<String>,
    #[serde(default)]
    pub context_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamStrategyMetadata {
    pub id: Option<String>,
    pub response_mode: Option<String>,
    pub collaboration_mode: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentStreamContextSelection {
    pub product_ids: Vec<String>,
    pub context_refs: Vec<String>,
}

impl AgentStreamMetadata {
    pub fn into_parts(
        self,
    ) -> Result<(BTreeMap<String, String>, AgentStreamContextSelection), String> {
        let mut metadata = BTreeMap::new();
        let context = self.context_selection()?;

        if let Some(strategy) = self.strategy {
            if let Some(id) = validate_strategy_value("strategy.id", strategy.id)? {
                metadata.insert("strategy.id".to_string(), id);
            }
            if let Some(response_mode) =
                validate_strategy_value("strategy.responseMode", strategy.response_mode)?
            {
                metadata.insert("strategy.response_mode".to_string(), response_mode);
            }
            if let Some(collaboration_mode) =
                validate_strategy_value("strategy.collaborationMode", strategy.collaboration_mode)?
            {
                metadata.insert(
                    "strategy.collaboration_mode".to_string(),
                    collaboration_mode,
                );
            }
        }

        Ok((metadata, context))
    }

    fn context_selection(&self) -> Result<AgentStreamContextSelection, String> {
        let Some(context) = &self.context else {
            return Ok(AgentStreamContextSelection::default());
        };

        Ok(AgentStreamContextSelection {
            product_ids: validate_context_values(
                "context.productIds",
                &context.product_ids,
                5,
                64,
            )?,
            context_refs: validate_context_values(
                "context.contextRefs",
                &context.context_refs,
                20,
                160,
            )?,
        })
    }
}

impl AgentStreamContextSelection {
    pub fn is_empty(&self) -> bool {
        self.product_ids.is_empty() && self.context_refs.is_empty()
    }
}

fn validate_context_values(
    field: &str,
    values: &[String],
    max_count: usize,
    max_len: usize,
) -> Result<Vec<String>, String> {
    if values.len() > max_count {
        return Err(format!("{field} must contain at most {max_count} items"));
    }

    let mut normalized = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > max_len {
            return Err(format!(
                "{field} items must be at most {max_len} characters"
            ));
        }
        if !normalized.iter().any(|item| item == value) {
            normalized.push(value.to_string());
        }
    }
    Ok(normalized)
}

fn validate_strategy_value(field: &str, value: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() > 64 {
        return Err(format!("{field} must be at most 64 characters"));
    }
    if !value.chars().all(|item| {
        item.is_ascii_lowercase() || item.is_ascii_digit() || item == '_' || item == '-'
    }) {
        return Err(format!(
            "{field} only allows lowercase letters, digits, '_' or '-'"
        ));
    }
    Ok(Some(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{AgentStreamContextMetadata, AgentStreamMetadata, AgentStreamStrategyMetadata};

    #[test]
    fn metadata_parts_include_strategy_and_context_selection() {
        let product_id = "11111111-1111-1111-1111-111111111111".to_string();
        let metadata = AgentStreamMetadata {
            context: Some(AgentStreamContextMetadata {
                product_ids: vec![product_id.clone(), product_id.clone()],
                context_refs: vec![format!("product:{product_id}")],
            }),
            strategy: Some(AgentStreamStrategyMetadata {
                id: Some("collab".to_string()),
                response_mode: Some("execution".to_string()),
                collaboration_mode: Some("manual".to_string()),
            }),
        };

        let (agent_metadata, context) = metadata.into_parts().expect("metadata should parse");

        assert_eq!(
            agent_metadata.get("strategy.id").map(String::as_str),
            Some("collab")
        );
        assert_eq!(context.product_ids, vec![product_id.clone()]);
        assert_eq!(context.context_refs, vec![format!("product:{product_id}")]);
    }

    #[test]
    fn metadata_rejects_too_many_product_ids() {
        let metadata = AgentStreamMetadata {
            context: Some(AgentStreamContextMetadata {
                product_ids: (0..6).map(|index| format!("product-{index}")).collect(),
                context_refs: Vec::new(),
            }),
            strategy: None,
        };

        let err = metadata
            .into_parts()
            .expect_err("too many product ids should fail");

        assert!(err.contains("context.productIds"));
    }
}
