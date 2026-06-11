use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dto::app::{CreateCampaignFormDraftRequest, CreateProductDraftRequest};

const MAX_IMAGE_ATTACHMENTS: usize = 4;
const MAX_IMAGE_URL_LEN: usize = 4096;
const MAX_IMAGE_DATA_URL_LEN: usize = 8 * 1024 * 1024;
const MAX_CONTEXT_REFS: usize = 20;
const MAX_SELECTED_RUNTIME_ROLES: usize = 8;

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

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct FormDraftStreamRequest {
    #[validate(length(min = 1, max = 32))]
    pub form_type: String,

    pub product: Option<CreateProductDraftRequest>,

    pub campaign: Option<CreateCampaignFormDraftRequest>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamMetadata {
    pub attachments: Option<AgentStreamAttachmentMetadata>,
    pub context: Option<AgentStreamContextMetadata>,
    pub strategy: Option<AgentStreamStrategyMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamAttachmentMetadata {
    #[serde(default)]
    pub images: Vec<AgentStreamImageAttachmentMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamImageAttachmentMetadata {
    pub url: String,
    pub detail: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamContextMetadata {
    pub conversation_id: Option<String>,

    #[serde(default)]
    pub campaign_ids: Vec<String>,

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
    pub routing_mode: Option<String>,
    pub collaboration_mode: Option<String>,
    pub learning_level: Option<String>,
    pub primary_role: Option<String>,
    #[serde(default)]
    pub selected_roles: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentStreamContextSelection {
    pub campaign_ids: Vec<String>,
    pub conversation_id: Option<String>,
    pub product_ids: Vec<String>,
    pub context_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentStreamImageAttachmentSelection {
    pub url: String,
    pub detail: Option<String>,
    pub source_ref: Option<String>,
}

impl AgentStreamMetadata {
    pub fn into_parts(
        self,
    ) -> Result<(BTreeMap<String, String>, AgentStreamContextSelection), String> {
        let mut metadata = BTreeMap::new();
        let mut context = self.context_selection()?;
        let image_attachments = self.image_attachments()?;
        for image in &image_attachments {
            if let Some(source_ref) = image.source_ref.as_ref() {
                push_unique_context_ref(&mut context.context_refs, source_ref)?;
            }
        }
        let image_count = image_attachments.len();

        if image_count > 0 {
            metadata.insert(
                "attachments.image_count".to_string(),
                image_count.to_string(),
            );
        }

        if let Some(strategy) = self.strategy {
            if let Some(id) = validate_strategy_value("strategy.id", strategy.id)? {
                metadata.insert("strategy.id".to_string(), id);
            }
            if let Some(response_mode) =
                validate_strategy_value("strategy.responseMode", strategy.response_mode)?
            {
                metadata.insert("strategy.response_mode".to_string(), response_mode);
            }
            if let Some(routing_mode) =
                validate_strategy_value("strategy.routingMode", strategy.routing_mode)?
            {
                if !matches!(routing_mode.as_str(), "fast" | "commander") {
                    return Err("strategy.routingMode must be either fast or commander".to_string());
                }
                metadata.insert("strategy.routing_mode".to_string(), routing_mode);
            }
            if let Some(collaboration_mode) =
                validate_strategy_value("strategy.collaborationMode", strategy.collaboration_mode)?
            {
                metadata.insert(
                    "strategy.collaboration_mode".to_string(),
                    collaboration_mode,
                );
            }
            if let Some(learning_level) =
                validate_strategy_value("strategy.learningLevel", strategy.learning_level)?
            {
                metadata.insert("strategy.learning_level".to_string(), learning_level);
            }
            if let Some(primary_role) =
                validate_strategy_value("strategy.primaryRole", strategy.primary_role)?
            {
                metadata.insert("strategy.primary_role".to_string(), primary_role);
            }
            let selected_roles = validate_strategy_values(
                "strategy.selectedRoles",
                &strategy.selected_roles,
                MAX_SELECTED_RUNTIME_ROLES,
            )?;
            if !selected_roles.is_empty() {
                metadata.insert(
                    "strategy.selected_roles".to_string(),
                    selected_roles.join(","),
                );
            }
        }

        tracing::debug!(
            routing_mode = metadata
                .get("strategy.routing_mode")
                .map(String::as_str)
                .unwrap_or("<missing>"),
            response_mode = metadata
                .get("strategy.response_mode")
                .map(String::as_str)
                .unwrap_or("<missing>"),
            strategy_id = metadata
                .get("strategy.id")
                .map(String::as_str)
                .unwrap_or("<missing>"),
            collaboration_mode = metadata
                .get("strategy.collaboration_mode")
                .map(String::as_str)
                .unwrap_or("<missing>"),
            selected_roles = metadata
                .get("strategy.selected_roles")
                .map(String::as_str)
                .unwrap_or("<none>"),
            image_count,
            "agent stream metadata parsed"
        );

        Ok((metadata, context))
    }

    fn context_selection(&self) -> Result<AgentStreamContextSelection, String> {
        let Some(context) = &self.context else {
            return Ok(AgentStreamContextSelection::default());
        };

        Ok(AgentStreamContextSelection {
            campaign_ids: validate_context_values(
                "context.campaignIds",
                &context.campaign_ids,
                5,
                64,
            )?,
            conversation_id: validate_optional_context_value(
                "context.conversationId",
                &context.conversation_id,
                64,
            )?,
            product_ids: validate_context_values(
                "context.productIds",
                &context.product_ids,
                5,
                64,
            )?,
            context_refs: validate_context_values(
                "context.contextRefs",
                &context.context_refs,
                MAX_CONTEXT_REFS,
                160,
            )?,
        })
    }

    fn image_attachments(&self) -> Result<Vec<AgentStreamImageAttachmentSelection>, String> {
        let Some(attachments) = &self.attachments else {
            return Ok(Vec::new());
        };
        validate_image_attachments(&attachments.images)
    }
}

impl AgentStreamContextSelection {
    pub fn has_product_context(&self) -> bool {
        !self.product_ids.is_empty()
            || self
                .context_refs
                .iter()
                .any(|context_ref| is_product_context_ref(context_ref))
    }

    pub fn product_context_refs(&self) -> Vec<String> {
        self.context_refs
            .iter()
            .filter(|context_ref| is_product_context_ref(context_ref))
            .cloned()
            .collect()
    }

    pub fn first_product_ref_id(&self) -> Option<String> {
        self.context_refs
            .iter()
            .find_map(|context_ref| product_id_from_context_ref(context_ref).map(str::to_string))
    }

    pub fn campaign_context_refs(&self) -> Vec<String> {
        self.context_refs
            .iter()
            .filter(|context_ref| is_campaign_context_ref(context_ref))
            .cloned()
            .collect()
    }
}

fn is_campaign_context_ref(value: &str) -> bool {
    campaign_id_from_context_ref(value).is_some()
}

fn campaign_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("campaign:")
        .or_else(|| value.strip_prefix("campaign/"))
}

fn is_product_context_ref(value: &str) -> bool {
    product_id_from_context_ref(value).is_some()
}

fn product_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("product:")
        .or_else(|| value.strip_prefix("product/"))
}

fn push_unique_context_ref(values: &mut Vec<String>, value: &str) -> Result<(), String> {
    if values.iter().any(|item| item == value) {
        return Ok(());
    }
    if values.len() >= MAX_CONTEXT_REFS {
        return Err(format!(
            "context.contextRefs must contain at most {MAX_CONTEXT_REFS} items"
        ));
    }
    values.push(value.to_string());
    Ok(())
}

fn validate_optional_context_value(
    field: &str,
    value: &Option<String>,
    max_len: usize,
) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max_len {
        return Err(format!("{field} must be at most {max_len} characters"));
    }
    Ok(Some(value.to_string()))
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

fn validate_strategy_values(
    field: &str,
    values: &[String],
    max_count: usize,
) -> Result<Vec<String>, String> {
    if values.len() > max_count {
        return Err(format!("{field} must contain at most {max_count} items"));
    }

    let mut normalized = Vec::new();
    for value in values {
        if let Some(value) = validate_strategy_value(field, Some(value.clone()))?
            && !normalized.iter().any(|item| item == &value)
        {
            normalized.push(value);
        }
    }
    Ok(normalized)
}

fn validate_image_attachments(
    values: &[AgentStreamImageAttachmentMetadata],
) -> Result<Vec<AgentStreamImageAttachmentSelection>, String> {
    if values.len() > MAX_IMAGE_ATTACHMENTS {
        return Err(format!(
            "metadata.attachments.images must contain at most {MAX_IMAGE_ATTACHMENTS} items"
        ));
    }

    let mut normalized = Vec::new();
    for value in values {
        let url = validate_image_url(&value.url)?;
        let detail = validate_image_detail(value.detail.as_deref())?;
        let source_ref = validate_optional_context_value(
            "metadata.attachments.images.sourceRef",
            &value.source_ref,
            160,
        )?;
        if !normalized
            .iter()
            .any(|item: &AgentStreamImageAttachmentSelection| item.url == url)
        {
            normalized.push(AgentStreamImageAttachmentSelection {
                url,
                detail,
                source_ref,
            });
        }
    }
    Ok(normalized)
}

fn validate_image_url(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("metadata.attachments.images.url cannot be empty".to_string());
    }
    if value.starts_with("https://") {
        if value.len() > MAX_IMAGE_URL_LEN {
            return Err(format!(
                "metadata.attachments.images.url must be at most {MAX_IMAGE_URL_LEN} characters"
            ));
        }
        return Ok(value.to_string());
    }
    if is_allowed_image_data_url(value) {
        if value.len() > MAX_IMAGE_DATA_URL_LEN {
            return Err("metadata.attachments.images data URL is too large".to_string());
        }
        return Ok(value.to_string());
    }
    Err("metadata.attachments.images.url must be https or png/jpeg/webp data URL".to_string())
}

fn is_allowed_image_data_url(value: &str) -> bool {
    value.starts_with("data:image/png;base64,")
        || value.starts_with("data:image/jpeg;base64,")
        || value.starts_with("data:image/webp;base64,")
}

fn validate_image_detail(value: Option<&str>) -> Result<Option<String>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    match value {
        "auto" | "low" | "high" => Ok(Some(value.to_string())),
        _ => Err("metadata.attachments.images.detail must be auto, low or high".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgentStreamAttachmentMetadata, AgentStreamContextMetadata,
        AgentStreamImageAttachmentMetadata, AgentStreamMetadata, AgentStreamStrategyMetadata,
    };

    #[test]
    fn metadata_parts_include_strategy_and_context_selection() {
        let product_id = "11111111-1111-1111-1111-111111111111".to_string();
        let metadata = AgentStreamMetadata {
            attachments: Some(AgentStreamAttachmentMetadata {
                images: vec![AgentStreamImageAttachmentMetadata {
                    url: "https://example.com/image.png".to_string(),
                    detail: Some("high".to_string()),
                    source_ref: Some("conversation_file:file-1".to_string()),
                }],
            }),
            context: Some(AgentStreamContextMetadata {
                campaign_ids: vec!["22222222-2222-2222-2222-222222222222".to_string()],
                conversation_id: Some("conversation-1".to_string()),
                product_ids: vec![product_id.clone(), product_id.clone()],
                context_refs: vec![format!("product:{product_id}")],
            }),
            strategy: Some(AgentStreamStrategyMetadata {
                id: Some("collab".to_string()),
                response_mode: Some("execution".to_string()),
                routing_mode: Some("commander".to_string()),
                collaboration_mode: Some("manual".to_string()),
                learning_level: Some("higher_vocational".to_string()),
                primary_role: Some("data".to_string()),
                selected_roles: vec!["data".to_string(), "ops".to_string(), "data".to_string()],
            }),
        };

        let (agent_metadata, context) = metadata.into_parts().expect("metadata should parse");

        assert_eq!(
            agent_metadata.get("strategy.id").map(String::as_str),
            Some("collab")
        );
        assert_eq!(
            agent_metadata
                .get("strategy.selected_roles")
                .map(String::as_str),
            Some("data,ops")
        );
        assert_eq!(
            agent_metadata
                .get("attachments.image_count")
                .map(String::as_str),
            Some("1")
        );
        assert!(!agent_metadata.contains_key("attachments.images_json"));
        assert_eq!(context.conversation_id.as_deref(), Some("conversation-1"));
        assert_eq!(
            context.campaign_ids,
            vec!["22222222-2222-2222-2222-222222222222".to_string()]
        );
        assert_eq!(context.product_ids, vec![product_id.clone()]);
        assert_eq!(
            context.context_refs,
            vec![
                format!("product:{product_id}"),
                "conversation_file:file-1".to_string(),
            ]
        );
    }

    #[test]
    fn metadata_rejects_too_many_product_ids() {
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: Some(AgentStreamContextMetadata {
                campaign_ids: Vec::new(),
                conversation_id: None,
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

    #[test]
    fn metadata_accepts_eight_selected_runtime_roles() {
        let roles = [
            "data",
            "ops",
            "creative",
            "search",
            "service",
            "accounting",
            "engineering",
            "web",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: None,
            strategy: Some(AgentStreamStrategyMetadata {
                id: None,
                response_mode: None,
                routing_mode: None,
                collaboration_mode: Some("manual".to_string()),
                learning_level: None,
                primary_role: Some("data".to_string()),
                selected_roles: roles.clone(),
            }),
        };

        let (agent_metadata, _) = metadata.into_parts().expect("8 roles should parse");

        assert_eq!(
            agent_metadata
                .get("strategy.selected_roles")
                .map(String::as_str),
            Some(roles.join(",").as_str())
        );
    }

    #[test]
    fn metadata_rejects_more_than_eight_selected_runtime_roles() {
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: None,
            strategy: Some(AgentStreamStrategyMetadata {
                id: None,
                response_mode: None,
                routing_mode: None,
                collaboration_mode: Some("manual".to_string()),
                learning_level: None,
                primary_role: Some("data".to_string()),
                selected_roles: [
                    "data",
                    "ops",
                    "creative",
                    "search",
                    "service",
                    "accounting",
                    "engineering",
                    "web",
                    "inventory",
                ]
                .into_iter()
                .map(str::to_string)
                .collect(),
            }),
        };

        let err = metadata
            .into_parts()
            .expect_err("9 roles should exceed backend SSE limit");

        assert!(err.contains("strategy.selectedRoles"));
        assert!(err.contains("8"));
    }

    #[test]
    fn metadata_parses_conversation_id_context_selection() {
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: Some(AgentStreamContextMetadata {
                campaign_ids: Vec::new(),
                conversation_id: Some("  conversation-uuid  ".to_string()),
                product_ids: Vec::new(),
                context_refs: Vec::new(),
            }),
            strategy: None,
        };

        let (_, context) = metadata.into_parts().expect("metadata should parse");

        assert_eq!(
            context.conversation_id.as_deref(),
            Some("conversation-uuid")
        );
        assert!(context.product_ids.is_empty());
        assert!(context.context_refs.is_empty());
    }

    #[test]
    fn metadata_accepts_workspace_artifact_context_refs() {
        let artifact_ref = "workspace_artifact:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string();
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: Some(AgentStreamContextMetadata {
                campaign_ids: Vec::new(),
                conversation_id: None,
                product_ids: Vec::new(),
                context_refs: vec![artifact_ref.clone(), artifact_ref.clone()],
            }),
            strategy: None,
        };

        let (_, context) = metadata.into_parts().expect("metadata should parse");

        assert_eq!(context.context_refs, vec![artifact_ref]);
        assert!(!context.context_refs.is_empty());
        assert!(!context.has_product_context());
        assert!(context.product_context_refs().is_empty());
    }

    #[test]
    fn metadata_adds_image_source_ref_to_context_refs() {
        let metadata = AgentStreamMetadata {
            attachments: Some(AgentStreamAttachmentMetadata {
                images: vec![AgentStreamImageAttachmentMetadata {
                    url: "data:image/png;base64,aW1hZ2U=".to_string(),
                    detail: Some("high".to_string()),
                    source_ref: Some("conversation_file:file-image".to_string()),
                }],
            }),
            context: Some(AgentStreamContextMetadata {
                campaign_ids: Vec::new(),
                conversation_id: Some("conversation-1".to_string()),
                product_ids: Vec::new(),
                context_refs: Vec::new(),
            }),
            strategy: None,
        };

        let (agent_metadata, context) = metadata.into_parts().expect("metadata should parse");

        assert_eq!(
            agent_metadata
                .get("attachments.image_count")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            context.context_refs,
            vec!["conversation_file:file-image".to_string()]
        );
    }

    #[test]
    fn metadata_separates_product_context_refs_from_artifacts() {
        let product_ref = "product:11111111-1111-1111-1111-111111111111".to_string();
        let artifact_ref = "workspace_artifact:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string();
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: Some(AgentStreamContextMetadata {
                campaign_ids: Vec::new(),
                conversation_id: None,
                product_ids: Vec::new(),
                context_refs: vec![product_ref.clone(), artifact_ref],
            }),
            strategy: None,
        };

        let (_, context) = metadata.into_parts().expect("metadata should parse");

        assert!(context.has_product_context());
        assert_eq!(context.product_context_refs(), vec![product_ref]);
        assert_eq!(
            context.first_product_ref_id().as_deref(),
            Some("11111111-1111-1111-1111-111111111111")
        );
    }

    #[test]
    fn metadata_accepts_campaign_context_refs() {
        let campaign_id = "22222222-2222-2222-2222-222222222222".to_string();
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: Some(AgentStreamContextMetadata {
                campaign_ids: vec![campaign_id.clone()],
                conversation_id: None,
                product_ids: Vec::new(),
                context_refs: vec![format!("campaign:{campaign_id}")],
            }),
            strategy: None,
        };

        let (_, context) = metadata.into_parts().expect("metadata should parse");

        assert_eq!(context.campaign_ids, vec![campaign_id.clone()]);
        assert_eq!(
            context.campaign_context_refs(),
            vec![format!("campaign:{campaign_id}")]
        );
        assert!(!context.has_product_context());
    }

    #[test]
    fn metadata_rejects_too_long_conversation_id() {
        let metadata = AgentStreamMetadata {
            attachments: None,
            context: Some(AgentStreamContextMetadata {
                campaign_ids: Vec::new(),
                conversation_id: Some("x".repeat(65)),
                product_ids: Vec::new(),
                context_refs: Vec::new(),
            }),
            strategy: None,
        };

        let err = metadata
            .into_parts()
            .expect_err("too long conversation id should fail");

        assert!(err.contains("context.conversationId"));
    }

    #[test]
    fn metadata_rejects_invalid_image_url() {
        let metadata = AgentStreamMetadata {
            attachments: Some(AgentStreamAttachmentMetadata {
                images: vec![AgentStreamImageAttachmentMetadata {
                    url: "http://example.com/image.png".to_string(),
                    detail: Some("high".to_string()),
                    source_ref: None,
                }],
            }),
            context: None,
            strategy: None,
        };

        let err = metadata
            .into_parts()
            .expect_err("http image url should fail");

        assert!(err.contains("metadata.attachments.images.url"));
    }
}
