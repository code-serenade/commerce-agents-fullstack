use axum::http::StatusCode;
use db_core::error::BIZ_INTERNAL_ERROR;
use serde_json::{Value, json};
use toolcraft_axum_kit::{ApiError, CommonError};

pub(crate) const FORM_DRAFT_MODE_TEMPLATE_SMART_FILL: &str = "template_smart_fill";
pub(crate) const FORM_DRAFT_MODE_AGENT_LLM: &str = "agent_llm";
pub(crate) const FORM_DRAFT_AGENT_FAILURE_MESSAGE: &str =
    "Form draft generation failed before form fill.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormDraftGenerationMode {
    TemplateSmartFill,
    AgentLlm,
}

pub(crate) fn parse_form_draft_generation_mode(
    value: Option<&str>,
) -> core::result::Result<FormDraftGenerationMode, String> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some(FORM_DRAFT_MODE_TEMPLATE_SMART_FILL) => {
            Ok(FormDraftGenerationMode::TemplateSmartFill)
        }
        Some(FORM_DRAFT_MODE_AGENT_LLM) => Ok(FormDraftGenerationMode::AgentLlm),
        Some(other) => Err(format!(
            "unsupported form draft generationMode '{other}', allowed values: {FORM_DRAFT_MODE_TEMPLATE_SMART_FILL}, {FORM_DRAFT_MODE_AGENT_LLM}"
        )),
    }
}

pub(crate) fn agent_form_draft_context(
    form_type: &str,
    schema: &str,
    persistence_endpoint: &str,
    analysis_kind: &str,
) -> Value {
    json!({
        "type": form_type,
        "generationSource": FORM_DRAFT_MODE_AGENT_LLM,
        "directWrite": false,
        "requiresUserReview": true,
        "llm": {
            "used": true,
            "systemAgentId": "form_draft_generator",
            "route": "web_server_agent_manager",
            "analysisKind": analysis_kind
        },
        "draftQuality": {
            "schema": schema,
            "fieldNormalization": "trim_empty_to_default",
            "invalidModelJsonPolicy": "reject_before_form_fill",
            "manualReviewRequired": true
        },
        "saveBoundary": {
            "directWrite": false,
            "requiresUserReview": true,
            "persistenceEndpoint": persistence_endpoint
        }
    })
}

pub(crate) fn form_draft_agent_failure_error<E: core::fmt::Display>(err: E) -> ApiError {
    tracing::warn!(error = %err, "form draft agent mode failed before form fill");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        CommonError {
            code: BIZ_INTERNAL_ERROR,
            message: FORM_DRAFT_AGENT_FAILURE_MESSAGE.to_string(),
        }
        .to_json(),
    )
}

pub(crate) fn product_form_draft_input(
    req: &crate::dto::app::CreateProductDraftRequest,
) -> agent::api::FormDraftInput {
    agent::api::FormDraftInput {
        kind: agent::api::FormDraftKind::Product,
        user_input: product_form_draft_user_input(req),
        existing_values: json!({
            "name": req.name,
            "platform": req.platform,
            "category": req.category,
            "brand": req.brand,
            "description": req.description,
            "price": req.price,
            "currency": req.currency
        }),
        context_summary: req.prompt.clone(),
    }
}

pub(crate) fn campaign_form_draft_input(
    req: &crate::dto::app::CreateCampaignFormDraftRequest,
) -> agent::api::FormDraftInput {
    agent::api::FormDraftInput {
        kind: agent::api::FormDraftKind::Campaign,
        user_input: campaign_form_draft_user_input(req),
        existing_values: json!({
            "productId": req.product_id,
            "name": req.name,
            "campaignType": req.campaign_type,
            "description": req.description,
            "budget": req.budget,
            "startDate": req.start_date,
            "endDate": req.end_date,
            "goal": req.goal,
            "targetAudience": req.target_audience,
            "channels": req.channels
        }),
        context_summary: req.prompt.clone(),
    }
}

pub(crate) fn map_agent_product_form_draft(
    result: agent::api::FormDraftResult,
) -> crate::dto::app::ProductDraftResponse {
    let draft = result.result_json;
    crate::dto::app::ProductDraftResponse {
        draft: service::dto::product::ProductDraftValues {
            name: json_string(&draft, "name").unwrap_or_else(|| "智能商品草稿".to_string()),
            platform: json_optional_string(&draft, "platform"),
            external_id: json_optional_string(&draft, "externalId"),
            category: json_optional_string(&draft, "category"),
            brand: json_optional_string(&draft, "brand"),
            status: json_string(&draft, "status").unwrap_or_else(|| "active".to_string()),
            description: json_optional_string(&draft, "description"),
            price: json_optional_string(&draft, "price"),
            currency: json_optional_string(&draft, "currency"),
        },
        warnings: Vec::new(),
        generation_mode: FORM_DRAFT_MODE_AGENT_LLM.to_string(),
        agent_context: agent_form_draft_context(
            "product_form_draft",
            "product_form_draft_v1",
            "POST /api/app/products",
            &result.analysis_kind,
        ),
    }
}

pub(crate) fn map_agent_campaign_form_draft(
    result: agent::api::FormDraftResult,
) -> crate::dto::app::CampaignFormDraftResponse {
    let draft = result.result_json;
    crate::dto::app::CampaignFormDraftResponse {
        draft: service::dto::campaign::CampaignFormDraftValues {
            product_id: json_optional_string(&draft, "productId"),
            name: json_string(&draft, "name").unwrap_or_else(|| "活动草稿".to_string()),
            campaign_type: json_optional_string(&draft, "campaignType"),
            description: json_optional_string(&draft, "description"),
            budget: draft.get("budget").and_then(Value::as_f64),
            start_date: json_optional_string(&draft, "startDate"),
            end_date: json_optional_string(&draft, "endDate"),
            status: json_string(&draft, "status").unwrap_or_else(|| "draft".to_string()),
        },
        warnings: Vec::new(),
        generation_mode: FORM_DRAFT_MODE_AGENT_LLM.to_string(),
        agent_context: agent_form_draft_context(
            "campaign_form_draft",
            "campaign_form_draft_v1",
            "POST /api/app/campaigns",
            &result.analysis_kind,
        ),
    }
}

fn product_form_draft_user_input(req: &crate::dto::app::CreateProductDraftRequest) -> String {
    compact_form_draft_user_input(
        [
            req.name.as_deref(),
            req.category.as_deref(),
            req.brand.as_deref(),
            req.platform.as_deref(),
            req.description.as_deref(),
            req.price.as_deref(),
            req.currency.as_deref(),
            req.prompt.as_deref(),
        ]
        .into_iter(),
    )
}

fn campaign_form_draft_user_input(req: &crate::dto::app::CreateCampaignFormDraftRequest) -> String {
    let mut parts = vec![
        req.product_id.as_deref(),
        req.name.as_deref(),
        req.campaign_type.as_deref(),
        req.description.as_deref(),
        req.start_date.as_deref(),
        req.end_date.as_deref(),
        req.goal.as_deref(),
        req.target_audience.as_deref(),
        req.prompt.as_deref(),
    ]
    .into_iter()
    .filter_map(|part| part.map(str::trim).filter(|value| !value.is_empty()))
    .map(str::to_string)
    .collect::<Vec<_>>();
    if let Some(budget) = req.budget {
        parts.push(format!("budget: {budget}"));
    }
    if let Some(channels) = req.channels.as_ref() {
        let channels = channels
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !channels.is_empty() {
            parts.push(format!("channels: {}", channels.join(",")));
        }
    }
    if parts.is_empty() {
        "请生成一个保守的活动表单草稿。".to_string()
    } else {
        parts.join("\n")
    }
}

fn compact_form_draft_user_input<'a>(parts: impl Iterator<Item = Option<&'a str>>) -> String {
    let values = parts
        .filter_map(|part| part.map(str::trim).filter(|value| !value.is_empty()))
        .collect::<Vec<_>>();
    if values.is_empty() {
        "请生成一个保守的商品表单草稿。".to_string()
    } else {
        values.join("\n")
    }
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn json_optional_string(value: &Value, key: &str) -> Option<String> {
    json_string(value, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent::api::FormDraftResult;
    use serde_json::json;

    #[test]
    fn form_draft_generation_mode_defaults_to_template() {
        assert_eq!(
            parse_form_draft_generation_mode(None).unwrap(),
            FormDraftGenerationMode::TemplateSmartFill
        );
        assert_eq!(
            parse_form_draft_generation_mode(Some("")).unwrap(),
            FormDraftGenerationMode::TemplateSmartFill
        );
    }

    #[test]
    fn form_draft_generation_mode_accepts_only_stable_codes() {
        assert_eq!(
            parse_form_draft_generation_mode(Some(FORM_DRAFT_MODE_AGENT_LLM)).unwrap(),
            FormDraftGenerationMode::AgentLlm
        );

        let err = parse_form_draft_generation_mode(Some("智能补全")).unwrap_err();

        assert!(err.contains("unsupported form draft generationMode"));
    }

    #[test]
    fn agent_product_form_draft_mapping_hides_model_details() {
        let response = map_agent_product_form_draft(FormDraftResult {
            analysis_kind: "form_draft".to_string(),
            prompt_key: "zh/form_drafts/product_form_draft.md".to_string(),
            prompt_version: "v1".to_string(),
            llm_name: "internal-fast".to_string(),
            llm_model: Some("hidden-model".to_string()),
            result_json: json!({
                "name": "温显杯",
                "status": "active",
                "platform": "douyin"
            }),
        });

        assert_eq!(response.generation_mode, FORM_DRAFT_MODE_AGENT_LLM);
        assert_eq!(response.draft.name, "温显杯");
        assert_eq!(response.agent_context["llm"]["used"].as_bool(), Some(true));
        assert_eq!(
            response.agent_context["llm"]["systemAgentId"].as_str(),
            Some("form_draft_generator")
        );
        assert!(response.agent_context.get("llmName").is_none());
        assert!(response.agent_context.get("llmModel").is_none());
    }

    #[test]
    fn form_draft_agent_failure_error_uses_safe_message() {
        let raw_error = "provider raw error model=hidden-model base_url=http://127.0.0.1:11434 api_key=secret prompt=zh/form_drafts/product.md";
        let (status, axum::Json(body)) = form_draft_agent_failure_error(raw_error);

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.message, FORM_DRAFT_AGENT_FAILURE_MESSAGE);
        assert!(!body.message.contains("hidden-model"));
        assert!(!body.message.contains("127.0.0.1"));
        assert!(!body.message.contains("api_key"));
        assert!(!body.message.contains("form_drafts"));
        assert!(!body.message.contains("provider raw error"));
    }
}
