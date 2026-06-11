use model_gateway_rs::{
    llm::Llm,
    model::{
        llm::{ChatMessage, LlmInput},
        role::Role,
    },
};
use serde_json::Value;

use super::{CampaignFormDraftOutput, FormDraftAgentConfig, ProductFormDraftOutput};
use crate::{
    agent_error::{CognitionError, LlmError, Result},
    agent_utils::prompt::Prompt,
    llm_routing::{AgentLlmCfg, AgentLlmRegistry, AgentLlmUsageCfg},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormDraftKind {
    Product,
    Campaign,
}

impl FormDraftKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Product => "product",
            Self::Campaign => "campaign",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormDraftInput {
    pub kind: FormDraftKind,
    pub user_input: String,
    pub existing_values: Value,
    pub context_summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FormDraftMessagePlan {
    pub prompt_key: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

#[derive(Debug, Clone)]
pub struct FormDraftResult {
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
    pub result_json: Value,
}

pub struct FormDraftAgent {
    config: FormDraftAgentConfig,
    llms: AgentLlmRegistry,
}

impl FormDraftAgent {
    pub fn new(
        config: FormDraftAgentConfig,
        llm_configs: &[AgentLlmCfg],
        usage: &AgentLlmUsageCfg,
    ) -> Result<Self> {
        Ok(Self {
            config,
            llms: AgentLlmRegistry::new(llm_configs, usage)?,
        })
    }

    pub fn build_message_plan(&self, input: &FormDraftInput) -> Result<FormDraftMessagePlan> {
        build_message_plan(&self.config, input)
    }

    pub async fn run(&self, input: FormDraftInput) -> Result<FormDraftResult> {
        let usage = self.llms.usage();
        let llm_name = self.config.usage_name(&usage)?;
        let llm_model = self.llms.model_by_name(&llm_name).map(str::to_string);
        let llm = self.llms.client_by_name(&llm_name)?;
        let plan = self.build_message_plan(&input)?;
        tracing::debug!(
            system_agent_id = %self.config.id,
            draft_kind = input.kind.as_str(),
            llm_name = %llm_name,
            llm_model = ?llm_model,
            "form draft llm request starting"
        );

        let raw = llm
            .chat_once(LlmInput {
                messages: vec![
                    ChatMessage {
                        role: Role::System,
                        content: plan.system_prompt.into(),
                    },
                    ChatMessage::user(plan.user_prompt),
                ],
            })
            .await
            .map_err(|err| LlmError::message(format!("form draft llm request failed: {err}")))?
            .get_content()
            .to_string();
        let result_json = decode_output(input.kind, &raw)?;

        Ok(FormDraftResult {
            analysis_kind: self.config.analysis_kind.clone(),
            prompt_key: plan.prompt_key,
            prompt_version: self.config.prompt_version.clone(),
            llm_name,
            llm_model,
            result_json,
        })
    }

    pub fn config(&self) -> &FormDraftAgentConfig {
        &self.config
    }
}

pub fn build_message_plan(
    config: &FormDraftAgentConfig,
    input: &FormDraftInput,
) -> Result<FormDraftMessagePlan> {
    let prompt_key = config.prompt_for_kind(input.kind).to_string();
    let prompt = Prompt::load_from_repo(format!("config/prompts/{prompt_key}"))?;
    Ok(FormDraftMessagePlan {
        prompt_key,
        system_prompt: prompt.content,
        user_prompt: render_user_prompt(input),
    })
}

pub fn render_user_prompt(input: &FormDraftInput) -> String {
    let existing_values =
        serde_json::to_string_pretty(&input.existing_values).unwrap_or_else(|_| "{}".to_string());
    format!(
        "请为 {kind} 表单生成一个可回填草稿，并只返回符合 system prompt 的 JSON。\n\n\
         用户输入：\n{user_input}\n\n\
         已有表单值 JSON：\n{existing_values}\n\n\
         上下文摘要：\n{context_summary}\n\n\
         注意：该输出只能回填表单，不得表示已经保存、创建或提交。",
        kind = input.kind.as_str(),
        user_input = input.user_input.trim(),
        existing_values = existing_values,
        context_summary = input.context_summary.as_deref().unwrap_or("无")
    )
}

pub fn decode_output(kind: FormDraftKind, raw: &str) -> Result<Value> {
    match kind {
        FormDraftKind::Product => {
            let output =
                utils::llm_json::parse_llm_json::<ProductFormDraftOutput>(raw).map_err(|err| {
                    CognitionError::decode(format!("invalid product form draft JSON: {err}"))
                })?;
            if output.name.trim().is_empty() || output.status != "active" {
                return Err(CognitionError::decode(
                    "product form draft requires non-empty name and status=active",
                )
                .into());
            }
            serde_json::to_value(output).map_err(|err| {
                CognitionError::decode(format!("product form draft serialization failed: {err}"))
                    .into()
            })
        }
        FormDraftKind::Campaign => {
            let output =
                utils::llm_json::parse_llm_json::<CampaignFormDraftOutput>(raw).map_err(|err| {
                    CognitionError::decode(format!("invalid campaign form draft JSON: {err}"))
                })?;
            if output.name.trim().is_empty() || output.status != "draft" {
                return Err(CognitionError::decode(
                    "campaign form draft requires non-empty name and status=draft",
                )
                .into());
            }
            if let Some(product_id) = output.product_id.as_deref()
                && !product_id.trim().is_empty()
                && uuid::Uuid::parse_str(product_id).is_err()
            {
                return Err(
                    CognitionError::decode("campaign form draft productId must be uuid").into(),
                );
            }
            serde_json::to_value(output).map_err(|err| {
                CognitionError::decode(format!("campaign form draft serialization failed: {err}"))
                    .into()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_input(kind: FormDraftKind) -> FormDraftInput {
        FormDraftInput {
            kind,
            user_input: "帮我补全温显杯商品".to_string(),
            existing_values: json!({"name": "温显杯"}),
            context_summary: Some("学生实训商品，主打防漏和温度显示。".to_string()),
        }
    }

    #[test]
    fn form_draft_config_loads_default_and_resolves_prompt() {
        let config = FormDraftAgentConfig::load_default().expect("config loads");

        assert_eq!(config.id, "form_draft_generator");
        assert_eq!(config.llm_usage, "fast");
        assert_eq!(
            config.prompt_for_kind(FormDraftKind::Product),
            "zh/form_drafts/product_form_draft.md"
        );
        assert_eq!(
            config.prompt_for_kind(FormDraftKind::Campaign),
            "zh/form_drafts/campaign_form_draft.md"
        );
    }

    #[test]
    fn form_draft_message_plan_uses_prompt_and_user_context() {
        let config = FormDraftAgentConfig::load_default().expect("config loads");
        let plan =
            build_message_plan(&config, &test_input(FormDraftKind::Product)).expect("message plan");

        assert_eq!(plan.prompt_key, "zh/form_drafts/product_form_draft.md");
        assert!(plan.system_prompt.contains("JSON schema:"));
        assert!(plan.system_prompt.contains("\"status\": \"active\""));
        assert!(plan.user_prompt.contains("帮我补全温显杯商品"));
        assert!(plan.user_prompt.contains("\"name\": \"温显杯\""));
        assert!(plan.user_prompt.contains("不得表示已经保存"));
    }

    #[test]
    fn form_draft_decode_accepts_product_fenced_json() {
        let value = decode_output(
            FormDraftKind::Product,
            "```json\n{\"name\":\"温显杯\",\"status\":\"active\",\"platform\":\"douyin\"}\n```",
        )
        .expect("product decoded");

        assert_eq!(value["name"], "温显杯");
        assert_eq!(value["status"], "active");
        assert_eq!(value["platform"], "douyin");
    }

    #[test]
    fn form_draft_decode_rejects_campaign_invalid_product_id() {
        let err = decode_output(
            FormDraftKind::Campaign,
            r#"{"productId":"bad-id","name":"直播测款","status":"draft"}"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("productId must be uuid"));
    }

    #[test]
    fn form_draft_decode_rejects_wrong_status() {
        let err = decode_output(
            FormDraftKind::Product,
            r#"{"name":"温显杯","status":"archived"}"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("status=active"));
    }
}
