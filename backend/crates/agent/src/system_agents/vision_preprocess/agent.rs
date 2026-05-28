use model_gateway_rs::{
    llm::Llm,
    model::llm::{ChatContentPart, ChatMessage, LlmInput},
};
use serde_json::Value;

use super::{VisionPreprocessAgentConfig, VisionPreprocessOutput};
use crate::{
    agent_error::{CognitionError, LlmError, Result},
    agent_utils::prompt::Prompt,
    llm_routing::{AgentLlmCfg, AgentLlmRegistry, AgentLlmUsageCfg},
};

#[derive(Debug, Clone)]
pub struct VisionPreprocessInput {
    pub data_url: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub object_key: String,
    pub bucket: String,
    pub file_size: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct VisionPreprocessResult {
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
    pub summary: String,
    pub result_json: Value,
}

pub struct VisionPreprocessAgent {
    config: VisionPreprocessAgentConfig,
    llms: AgentLlmRegistry,
}

impl VisionPreprocessAgent {
    pub fn new(
        config: VisionPreprocessAgentConfig,
        llm_configs: &[AgentLlmCfg],
        usage: &AgentLlmUsageCfg,
    ) -> Result<Self> {
        Ok(Self {
            config,
            llms: AgentLlmRegistry::new(llm_configs, usage)?,
        })
    }

    pub async fn run(&self, input: VisionPreprocessInput) -> Result<VisionPreprocessResult> {
        let usage = self.llms.usage();
        let llm_name = self.config.usage_name(&usage)?;
        let llm_model = self.llms.model_by_name(&llm_name).map(str::to_string);
        let llm = self.llms.client_by_name(&llm_name)?;
        let prompt = Prompt::load_from_repo(format!("config/prompts/{}", self.config.prompt))?;
        tracing::debug!(
            system_agent_id = %self.config.id,
            object_key = %input.object_key,
            llm_name = %llm_name,
            llm_model = ?llm_model,
            data_url_bytes = input.data_url.len(),
            "vision preprocess llm request starting"
        );
        let messages = vec![
            prompt.to_message(model_gateway_rs::model::role::Role::System),
            ChatMessage::user_with_parts(vec![
                ChatContentPart::text(render_user_prompt(&input)),
                ChatContentPart::image_url_detail(input.data_url, "high"),
            ]),
        ];

        let raw = llm
            .chat_once(LlmInput { messages })
            .await
            .map_err(|err| {
                LlmError::message(format!("vision preprocess llm request failed: {err}"))
            })?
            .get_content()
            .to_string();
        tracing::debug!(
            system_agent_id = %self.config.id,
            object_key = %input.object_key,
            llm_name = %llm_name,
            output_chars = raw.chars().count(),
            output_preview = %preview_text(&raw, 2000),
            "vision preprocess llm response received"
        );
        let output = decode_output(&raw).map_err(|err| {
            tracing::debug!(
                system_agent_id = %self.config.id,
                object_key = %input.object_key,
                llm_name = %llm_name,
                output_chars = raw.chars().count(),
                output_preview = %preview_text(&raw, 4000),
                error = %err,
                "vision preprocess llm response decode failed"
            );
            err
        })?;
        let (summary, result_json) = output.into_result_parts()?;
        tracing::debug!(
            system_agent_id = %self.config.id,
            object_key = %input.object_key,
            summary_chars = summary.chars().count(),
            "vision preprocess output decoded"
        );

        Ok(VisionPreprocessResult {
            analysis_kind: self.config.analysis_kind.clone(),
            prompt_key: self.config.prompt.clone(),
            prompt_version: self.config.prompt_version.clone(),
            llm_name,
            llm_model,
            summary,
            result_json,
        })
    }

    pub fn config(&self) -> &VisionPreprocessAgentConfig {
        &self.config
    }
}

fn render_user_prompt(input: &VisionPreprocessInput) -> String {
    format!(
        "请对这张会话上传图片做一次预识别，并只返回符合 system prompt 要求的 \
         JSON。\n\n文件信息：\n- bucket: {}\n- object_key: {}\n- file_name: {}\n- mime_type: \
         {}\n- file_size: {}\n\n注意：不要输出 Markdown，不要输出 JSON 之外的解释。",
        input.bucket,
        input.object_key,
        input.file_name.as_deref().unwrap_or("unknown"),
        input.mime_type.as_deref().unwrap_or("unknown"),
        input
            .file_size
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    )
}

fn decode_output(raw: &str) -> Result<VisionPreprocessOutput> {
    utils::llm_json::parse_llm_json::<VisionPreprocessOutput>(raw).map_err(|err| {
        CognitionError::decode(format!("invalid vision preprocess JSON: {err}")).into()
    })
}

fn preview_text(value: &str, max_chars: usize) -> String {
    let mut preview = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        preview.push_str("...");
    }
    preview
}
