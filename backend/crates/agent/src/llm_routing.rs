use std::{collections::HashMap, path::Path, sync::Arc};

use model_gateway_rs::{
    llm::{
        Llm,
        chat_completions::ChatCompletionsLlm,
        doubao::{DoubaoLlm, DoubaoThinking},
        ollama::OllamaLlm,
    },
    model::{
        llm::{ChatContentPart, ChatMessage, LlmInput},
        role::Role,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utils::llm_json;

use crate::{
    Cognition, CognitionDecoder, CognitionFailure, CognitionInput, CognitionOutput,
    CognitionResult, FailureReason, JsonTemplateDecoder,
    agent_error::{Error, LlmError, Result, SettingsError},
    agent_utils::prompt::Prompt,
};

pub type AgentLlmClient = dyn Llm + Send + Sync;

const ATTACHMENTS_IMAGES_JSON: &str = "attachments.images_json";
const REQUEST_ATTACHMENTS_IMAGES_JSON: &str = "request.attachments.images_json";
const ATTACHMENTS_IMAGE_SUMMARIES_JSON: &str = "attachments.image_summaries_json";
const REQUEST_ATTACHMENTS_IMAGE_SUMMARIES_JSON: &str = "request.attachments.image_summaries_json";
pub const LLM_ACCESS_DENIED_MARKER: &str = "llm_access_denied";
pub const LLM_PROVIDER_ERROR_MARKER: &str = "llm_provider_error";
const LLM_ACCESS_DENIED_USER_MESSAGE: &str = r#"## 当前状态

模型服务配置不可用，模型服务返回权限错误，所以本轮没有形成可靠回答。

## 对本轮回答的影响

- 已收到你的问题和本轮选择的资料。
- 需要模型生成的分析、总结和协作结果暂时无法完成。
- 这不是商品、活动或资料本身的判断结果。

## 下一步

- 请稍后重试同一个问题。
- 如果连续出现，请联系管理员检查模型服务配置和权限。"#;
const LLM_PROVIDER_ERROR_USER_MESSAGE: &str = r#"## 当前状态

模型服务临时不可用，上游模型服务返回服务端错误，所以本轮没有形成可靠回答。

## 对本轮回答的影响

- 已收到你的问题和本轮选择的资料。
- 需要模型生成的分析、总结和协作结果暂时无法完成。
- 这不是商品、活动或资料本身的判断结果。

## 下一步

- 可以稍后重试同一个问题。
- 如果连续出现，请联系管理员检查模型服务状态。"#;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentLlmCfg {
    pub name: String,
    #[serde(default)]
    pub provider: AgentLlmProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub thinking: Option<bool>,
    pub chat_completions_endpoint: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentLlmProvider {
    #[default]
    ChatCompletions,
    Ollama,
    Doubao,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AgentLlmUsageCfg {
    pub default: Option<String>,
    pub fast: Option<String>,
    pub thinking: Option<String>,
    pub vision: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCognitionKind {
    Commander,
    ChatWorker,
    Worker,
}

#[derive(Clone)]
pub struct AgentLlmRegistry {
    clients: HashMap<String, Arc<AgentLlmClient>>,
    models: HashMap<String, String>,
    usage: ResolvedAgentLlmUsage,
}

#[derive(Debug, Clone)]
struct ResolvedAgentLlmUsage {
    default: String,
    fast: String,
    thinking: String,
    vision: Option<String>,
}

impl AgentLlmRegistry {
    pub fn new(llm_configs: &[AgentLlmCfg], usage: &AgentLlmUsageCfg) -> Result<Self> {
        if llm_configs.is_empty() {
            return Err(SettingsError::missing_llm_usage(
                "agent runtime requires at least one [[llm]] entry",
            )
            .into());
        }

        let default = resolve_usage_name(llm_configs, usage.default.as_deref())
            .unwrap_or_else(|| select_first_llm(llm_configs).name.clone());
        ensure_llm_exists(llm_configs, &default)?;

        let fast = resolve_usage_name(llm_configs, usage.fast.as_deref())
            .unwrap_or_else(|| default.clone());
        let thinking = resolve_usage_name(llm_configs, usage.thinking.as_deref())
            .unwrap_or_else(|| default.clone());
        let vision = resolve_usage_name(llm_configs, usage.vision.as_deref());

        ensure_llm_exists(llm_configs, &fast)?;
        ensure_llm_exists(llm_configs, &thinking)?;
        if let Some(vision) = &vision {
            ensure_llm_exists(llm_configs, vision)?;
        }

        let mut clients = HashMap::new();
        let mut models = HashMap::new();
        for config in llm_configs {
            tracing::info!(
                llm_name = %config.name,
                model = %config.model,
                base_url = %config.base_url,
                provider = ?config.provider,
                max_tokens = ?config.max_tokens,
                temperature = ?config.temperature,
                reasoning_effort = ?config.reasoning_effort,
                thinking = ?config.thinking,
                chat_completions_endpoint = ?config.chat_completions_endpoint,
                has_api_key = config.api_key.as_ref().is_some_and(|key| !key.trim().is_empty()),
                "agent llm client configured"
            );
            clients.insert(config.name.clone(), build_llm_client(config)?);
            models.insert(config.name.clone(), config.model.clone());
        }

        Ok(Self {
            clients,
            models,
            usage: ResolvedAgentLlmUsage {
                default,
                fast,
                thinking,
                vision,
            },
        })
    }

    pub fn usage(&self) -> AgentLlmUsageCfg {
        AgentLlmUsageCfg {
            default: Some(self.usage.default.clone()),
            fast: Some(self.usage.fast.clone()),
            thinking: Some(self.usage.thinking.clone()),
            vision: self.usage.vision.clone(),
        }
    }

    pub(crate) fn client_by_name(&self, name: &str) -> Result<Arc<AgentLlmClient>> {
        self.clients.get(name).cloned().ok_or_else(|| {
            SettingsError::missing_llm_usage(format!("llm '{name}' not found")).into()
        })
    }

    pub(crate) fn model_by_name(&self, name: &str) -> Option<&str> {
        self.models.get(name).map(String::as_str)
    }

    fn client_for_kind(
        &self,
        kind: AgentCognitionKind,
        requires_vision: bool,
    ) -> Result<Arc<AgentLlmClient>> {
        let name = self.usage_name_for_kind(kind, requires_vision)?;

        self.client_by_name(name)
    }

    fn usage_name_for_kind(&self, kind: AgentCognitionKind, requires_vision: bool) -> Result<&str> {
        if requires_vision {
            return self.usage.vision.as_deref().ok_or_else(|| {
                SettingsError::missing_llm_usage("vision llm is not configured for image input")
                    .into()
            });
        }

        Ok(match kind {
            AgentCognitionKind::Commander => &self.usage.fast,
            AgentCognitionKind::ChatWorker => &self.usage.fast,
            AgentCognitionKind::Worker => &self.usage.thinking,
        })
    }

    fn usage_name_for_input(
        &self,
        kind: AgentCognitionKind,
        requires_vision: bool,
        input: &CognitionInput,
    ) -> Result<&str> {
        if requires_vision {
            return self.usage_name_for_kind(kind, requires_vision);
        }

        match input
            .context
            .metadata
            .get("assignment.llm_usage")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some("fast") => Ok(&self.usage.fast),
            Some("thinking") => Ok(&self.usage.thinking),
            Some("default") => Ok(&self.usage.default),
            _ => self.usage_name_for_kind(kind, requires_vision),
        }
    }
}

pub struct RoutingCognitionEngine<D: ?Sized = JsonTemplateDecoder> {
    llms: AgentLlmRegistry,
    kind: AgentCognitionKind,
    system_messages: Vec<ChatMessage>,
    decoder: Arc<D>,
    retry_policy: RoutingCognitionRetryPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutingCognitionRetryPolicy {
    pub max_attempts: usize,
    pub retry_empty: bool,
    pub retry_llm_error: bool,
    pub repair_decode_error: bool,
}

impl Default for RoutingCognitionRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_empty: true,
            retry_llm_error: true,
            repair_decode_error: true,
        }
    }
}

impl RoutingCognitionRetryPolicy {
    fn max_attempts(self) -> usize {
        self.max_attempts.max(1)
    }
}

impl RoutingCognitionEngine<JsonTemplateDecoder> {
    pub fn new(
        llms: AgentLlmRegistry,
        kind: AgentCognitionKind,
        prompt_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let prompt = Prompt::load_from_repo(prompt_path)?;
        let system_messages = vec![prompt.to_message(Role::System)];
        let decoder = Arc::new(JsonTemplateDecoder::new(prompt.extract_json_schema()?));

        Ok(Self {
            llms,
            kind,
            system_messages,
            decoder,
            retry_policy: RoutingCognitionRetryPolicy::default(),
        })
    }

    pub fn new_with_system_append(
        llms: AgentLlmRegistry,
        kind: AgentCognitionKind,
        prompt_path: impl AsRef<Path>,
        system_append: impl Into<String>,
    ) -> Result<Self> {
        let prompt = Prompt::load_from_repo(prompt_path)?;
        let decoder = Arc::new(JsonTemplateDecoder::new(prompt.extract_json_schema()?));
        let content = format!("{}\n\n{}", prompt.content, system_append.into());
        let system_messages = vec![ChatMessage {
            role: Role::System,
            content: content.into(),
        }];

        Ok(Self {
            llms,
            kind,
            system_messages,
            decoder,
            retry_policy: RoutingCognitionRetryPolicy::default(),
        })
    }
}

#[async_trait::async_trait]
impl<D> Cognition for RoutingCognitionEngine<D>
where
    D: CognitionDecoder + Sync + Send + 'static + ?Sized,
{
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        let images = match parse_image_inputs(&input) {
            Ok(images) => images,
            Err(failure) => return CognitionResult::Failure(failure),
        };
        if !images.is_empty() {
            return CognitionResult::Failure(CognitionFailure {
                reason: FailureReason::InternalError,
                description: "raw image inputs are restricted to system vision preprocess"
                    .to_string(),
            });
        }
        let image_summaries = match parse_image_summary_inputs(&input) {
            Ok(summaries) => summaries,
            Err(failure) => return CognitionResult::Failure(failure),
        };
        let requires_vision = false;
        tracing::debug!(
            kind = ?self.kind,
            image_count = images.len(),
            image_summary_count = image_summaries.len(),
            requires_vision,
            "llm routing image input parsed"
        );
        let usage_name = match self
            .llms
            .usage_name_for_input(self.kind, requires_vision, &input)
        {
            Ok(name) => name,
            Err(err) => {
                return CognitionResult::Failure(CognitionFailure {
                    reason: FailureReason::InternalError,
                    description: err.to_string(),
                });
            }
        };
        tracing::debug!(
            kind = ?self.kind,
            image_count = images.len(),
            usage_name,
            model = %self.llms.model_by_name(usage_name).unwrap_or("unknown"),
            assignment_llm_usage = ?input
                .context
                .metadata
                .get("assignment.llm_usage"),
            "llm routing selected llm"
        );
        let llm = match self.llms.clients.get(usage_name).cloned() {
            Some(llm) => llm,
            None => {
                return CognitionResult::Failure(CognitionFailure {
                    reason: FailureReason::InternalError,
                    description: format!("llm '{usage_name}' not found"),
                });
            }
        };
        let model_name = self.llms.model_by_name(usage_name).unwrap_or("unknown");
        let mut base_messages = self.system_messages.clone();
        base_messages.push(render_user_message(&input, &images, &image_summaries));

        let max_attempts = self.retry_policy.max_attempts();
        let mut attempts_run = 0;
        let mut messages = base_messages.clone();
        let mut last_failure = CognitionFailure {
            reason: FailureReason::InternalError,
            description: "LLM cognition did not run".to_string(),
        };

        for attempt in 1..=max_attempts {
            attempts_run = attempt;
            tracing::debug!(
                kind = ?self.kind,
                attempt,
                max_attempts,
                image_count = images.len(),
                usage_name,
                "llm routing request starting"
            );
            let raw = match llm.chat_once(LlmInput { messages }).await {
                Ok(output) => output.get_content().to_string(),
                Err(err) => {
                    crate::log_error!(err);
                    let err_text = err.to_string();
                    tracing::error!(
                        kind = ?self.kind,
                        usage_name,
                        model = model_name,
                        attempt,
                        error = %err_text,
                        "llm provider request failed"
                    );
                    if is_llm_access_denied_message(&err_text) {
                        tracing::warn!(
                            kind = ?self.kind,
                            usage_name,
                            model = model_name,
                            "llm provider denied access to selected model"
                        );
                        last_failure = llm_access_denied_failure();
                        break;
                    }
                    if is_llm_provider_error_message(&err_text) {
                        last_failure =
                            llm_provider_error_failure(attempt, usage_name, model_name, &err_text);
                    } else {
                        last_failure = CognitionFailure {
                            reason: FailureReason::InternalError,
                            description: format!(
                                "LLM request failed on attempt {attempt}: {err_text}"
                            ),
                        };
                    }
                    if attempt < max_attempts && self.retry_policy.retry_llm_error {
                        messages = retry_messages(
                            &base_messages,
                            "上一轮 LLM 请求失败。请重新完成任务，只返回合法 JSON。",
                            attempt + 1,
                        );
                        continue;
                    }
                    break;
                }
            };
            tracing::debug!(
                kind = ?self.kind,
                attempt,
                output_chars = raw.chars().count(),
                usage_name,
                "llm routing response received"
            );

            if raw.trim().is_empty() {
                crate::log_error_msg!("LLM returned empty response");
                last_failure = CognitionFailure {
                    reason: FailureReason::InternalError,
                    description: format!("LLM returned empty response on attempt {attempt}"),
                };
                if attempt < max_attempts && self.retry_policy.retry_empty {
                    messages = retry_messages(
                        &base_messages,
                        "上一轮输出为空。请重新完成任务，只返回合法 JSON，不要 \
                         Markdown，不要解释。",
                        attempt + 1,
                    );
                    continue;
                }
                break;
            }

            match self.decoder.decode(&raw) {
                Ok(output) => return CognitionResult::Success(output),
                Err(failure) => {
                    if matches!(
                        self.kind,
                        AgentCognitionKind::ChatWorker | AgentCognitionKind::Worker
                    ) && let Some(output) = fallback_chat_output(&raw)
                    {
                        return CognitionResult::Success(output);
                    }

                    let snippet = raw_snippet(&raw);
                    let message = format!(
                        "cognition decode failure on attempt {attempt}/{max_attempts}: \
                         reason={:?}, description={}, raw={}",
                        failure.reason, failure.description, snippet
                    );
                    crate::log_error_msg!(&message);
                    last_failure = CognitionFailure {
                        reason: failure.reason.clone(),
                        description: format!(
                            "decode failed on attempt {attempt}/{max_attempts}: {}; raw={snippet}",
                            failure.description
                        ),
                    };
                    if attempt < max_attempts && self.retry_policy.repair_decode_error {
                        messages = repair_messages(&base_messages, &raw, &failure, attempt + 1);
                        continue;
                    }
                    break;
                }
            }
        }

        if is_llm_access_denied_message(&last_failure.description)
            || is_llm_provider_error_message(&last_failure.description)
        {
            return CognitionResult::Failure(last_failure);
        }

        CognitionResult::Failure(CognitionFailure {
            reason: last_failure.reason,
            description: format!(
                "cognition failed after {} attempt(s): {}",
                attempts_run.max(1),
                last_failure.description
            ),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentImageInput {
    url: String,
    detail: Option<String>,
    source_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentImageSummaryInput {
    source_ref: String,
    object_key: String,
    summary: String,
    agent_context: Option<String>,
    analysis_id: String,
    prompt_version: Option<String>,
    llm_name: Option<String>,
}

fn build_llm_client(config: &AgentLlmCfg) -> Result<Arc<AgentLlmClient>> {
    match config.provider {
        AgentLlmProvider::ChatCompletions => build_chat_completions_client(config),
        AgentLlmProvider::Ollama => build_ollama_client(config),
        AgentLlmProvider::Doubao => build_doubao_client(config),
    }
}

fn build_chat_completions_client(config: &AgentLlmCfg) -> Result<Arc<AgentLlmClient>> {
    let mut client =
        ChatCompletionsLlm::new(&config.base_url, &config.model, config.api_key.as_deref())
            .map_err(|err| llm_init_error(config, err))?
            .with_max_tokens(config.max_tokens)
            .with_temperature(config.temperature)
            .with_reasoning_effort(config.reasoning_effort.as_deref());

    if let Some(thinking) = json_thinking_config(config.thinking) {
        client = client
            .with_extra_body_param("thinking", thinking)
            .map_err(|err| llm_init_field_error(config, "thinking", err))?;
    }

    if let Some(endpoint) = &config.chat_completions_endpoint {
        client = client.with_chat_completions_endpoint(endpoint.clone());
    }

    Ok(Arc::new(client))
}

fn build_ollama_client(config: &AgentLlmCfg) -> Result<Arc<AgentLlmClient>> {
    if config
        .api_key
        .as_ref()
        .is_some_and(|key| !key.trim().is_empty())
    {
        tracing::warn!(
            llm_name = %config.name,
            "ollama provider ignores configured api_key"
        );
    }
    if config.reasoning_effort.is_some() {
        tracing::warn!(
            llm_name = %config.name,
            "ollama provider ignores configured reasoning_effort"
        );
    }

    let mut client = OllamaLlm::new(&config.base_url, &config.model)
        .map_err(|err| llm_init_error(config, err))?
        .with_max_tokens(config.max_tokens)
        .map_err(|err| llm_init_field_error(config, "max_tokens", err))?
        .with_temperature(config.temperature)
        .map_err(|err| llm_init_field_error(config, "temperature", err))?;

    if matches!(config.thinking, Some(false)) {
        client = client.without_thinking();
    }

    if let Some(endpoint) = &config.chat_completions_endpoint {
        client = client.with_chat_endpoint(endpoint.clone());
    }

    Ok(Arc::new(client))
}

fn build_doubao_client(config: &AgentLlmCfg) -> Result<Arc<AgentLlmClient>> {
    let api_key = config
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .ok_or_else(|| {
            Error::from(SettingsError::missing_llm_usage(format!(
                "llm '{}' provider 'doubao' requires api_key",
                config.name
            )))
        })?;

    if config.reasoning_effort.is_some() {
        tracing::warn!(
            llm_name = %config.name,
            "doubao provider ignores configured reasoning_effort"
        );
    }

    let mut client = DoubaoLlm::new(&config.base_url, &config.model, api_key)
        .map_err(|err| llm_init_error(config, err))?
        .with_max_tokens(config.max_tokens)
        .with_temperature(config.temperature);

    if let Some(thinking) = doubao_thinking_config(config.thinking) {
        client = client
            .with_thinking(Some(thinking))
            .map_err(|err| llm_init_field_error(config, "thinking", err))?;
    }

    if let Some(endpoint) = &config.chat_completions_endpoint {
        client = client.with_chat_endpoint(endpoint.clone());
    }

    Ok(Arc::new(client))
}

fn llm_init_error(config: &AgentLlmCfg, err: impl std::fmt::Display) -> Error {
    Error::from(LlmError::message(format!(
        "init llm client '{}' failed: {err}",
        config.name
    )))
}

fn llm_init_field_error(config: &AgentLlmCfg, field: &str, err: impl std::fmt::Display) -> Error {
    Error::from(LlmError::message(format!(
        "init llm client '{}' {field} config failed: {err}",
        config.name
    )))
}

fn json_thinking_config(thinking: Option<bool>) -> Option<Value> {
    thinking.map(|enabled| {
        json!({
            "type": if enabled { "enabled" } else { "disabled" },
        })
    })
}

fn doubao_thinking_config(thinking: Option<bool>) -> Option<DoubaoThinking> {
    thinking.map(|enabled| {
        if enabled {
            DoubaoThinking::enabled()
        } else {
            DoubaoThinking::disabled()
        }
    })
}

fn resolve_usage_name(llm_configs: &[AgentLlmCfg], name: Option<&str>) -> Option<String> {
    name.map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .or_else(|| llm_configs.first().map(|llm| llm.name.clone()))
}

fn ensure_llm_exists(llm_configs: &[AgentLlmCfg], name: &str) -> Result<()> {
    if llm_configs.iter().any(|llm| llm.name == name) {
        return Ok(());
    }
    Err(SettingsError::missing_llm_usage(format!("llm '{name}' not found")).into())
}

fn select_first_llm(llm_configs: &[AgentLlmCfg]) -> &AgentLlmCfg {
    llm_configs
        .first()
        .expect("checked non-empty llm config above")
}

fn is_doubao_llm(llm: &AgentLlmCfg) -> bool {
    let name = llm.name.to_ascii_lowercase();
    let model = llm.model.to_ascii_lowercase();
    name.contains("doubao") || model.contains("doubao")
}

pub fn is_llm_access_denied_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains(LLM_ACCESS_DENIED_MARKER)
        || lower.contains("accessdenied")
        || lower.contains("access denied")
        || lower.contains("do not have access to the requested resource")
        || lower.contains("status=403")
        || lower.contains("status: 403")
        || (lower.contains("forbidden")
            && (lower.contains("api error")
                || lower.contains("requested resource")
                || lower.contains("llm")
                || lower.contains("model")))
        || ((lower.contains("status=401")
            || lower.contains("status: 401")
            || lower.contains("unauthorized"))
            && lower.contains("api"))
}

pub fn llm_access_denied_user_message() -> &'static str {
    LLM_ACCESS_DENIED_USER_MESSAGE
}

pub fn is_llm_provider_error_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains(LLM_PROVIDER_ERROR_MARKER)
        || ((lower.contains("api error")
            || lower.contains("llm request failed")
            || lower.contains("provider request failed"))
            && (contains_status_code(&lower, "500")
                || contains_status_code(&lower, "502")
                || contains_status_code(&lower, "503")
                || contains_status_code(&lower, "504")
                || lower.contains("internal server error")
                || lower.contains("bad gateway")
                || lower.contains("service unavailable")
                || lower.contains("gateway timeout")))
}

pub fn llm_provider_error_user_message() -> &'static str {
    LLM_PROVIDER_ERROR_USER_MESSAGE
}

fn contains_status_code(message: &str, code: &str) -> bool {
    [
        format!("status={code}"),
        format!("status: {code}"),
        format!("status:{code}"),
        format!("status {code}"),
    ]
    .iter()
    .any(|pattern| message.contains(pattern))
}

fn llm_access_denied_failure() -> CognitionFailure {
    CognitionFailure {
        reason: FailureReason::InternalError,
        description: format!(
            "{LLM_ACCESS_DENIED_MARKER}: {}",
            llm_access_denied_user_message()
        ),
    }
}

fn llm_provider_error_failure(
    attempt: usize,
    usage_name: &str,
    model_name: &str,
    err_text: &str,
) -> CognitionFailure {
    CognitionFailure {
        reason: FailureReason::InternalError,
        description: format!(
            "{LLM_PROVIDER_ERROR_MARKER}: attempt={attempt}; llm_usage={usage_name}; model={model_name}; error={}",
            compact_provider_error(err_text)
        ),
    }
}

fn compact_provider_error(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_image_inputs(
    input: &CognitionInput,
) -> std::result::Result<Vec<AgentImageInput>, CognitionFailure> {
    let Some(raw) = input
        .context
        .metadata
        .get(ATTACHMENTS_IMAGES_JSON)
        .or_else(|| input.context.metadata.get(REQUEST_ATTACHMENTS_IMAGES_JSON))
    else {
        return Ok(Vec::new());
    };
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str::<Vec<AgentImageInput>>(raw).map_err(|err| CognitionFailure {
        reason: FailureReason::InternalError,
        description: format!("invalid image attachment metadata: {err}"),
    })
}

fn parse_image_summary_inputs(
    input: &CognitionInput,
) -> std::result::Result<Vec<AgentImageSummaryInput>, CognitionFailure> {
    let Some(raw) = input
        .context
        .metadata
        .get(ATTACHMENTS_IMAGE_SUMMARIES_JSON)
        .or_else(|| {
            input
                .context
                .metadata
                .get(REQUEST_ATTACHMENTS_IMAGE_SUMMARIES_JSON)
        })
    else {
        return Ok(Vec::new());
    };
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str::<Vec<AgentImageSummaryInput>>(raw).map_err(|err| CognitionFailure {
        reason: FailureReason::InternalError,
        description: format!("invalid image summary attachment metadata: {err}"),
    })
}

fn render_user_message(
    input: &CognitionInput,
    images: &[AgentImageInput],
    image_summaries: &[AgentImageSummaryInput],
) -> ChatMessage {
    let text = render_input(input, image_summaries);
    if images.is_empty() {
        return ChatMessage::user(text);
    }

    let mut parts = vec![ChatContentPart::text(text)];
    for image in images {
        let _ = &image.source_ref;
        parts.push(match image.detail.as_deref() {
            Some(detail) => ChatContentPart::image_url_detail(image.url.clone(), detail),
            None => ChatContentPart::image_url(image.url.clone()),
        });
    }
    ChatMessage::user_with_parts(parts)
}

fn retry_messages(
    base_messages: &[ChatMessage],
    instruction: &str,
    next_attempt: usize,
) -> Vec<ChatMessage> {
    let mut messages = base_messages.to_vec();
    messages.push(ChatMessage::user(format!(
        "Retry attempt {next_attempt}: {instruction}"
    )));
    messages
}

fn repair_messages(
    base_messages: &[ChatMessage],
    raw: &str,
    failure: &CognitionFailure,
    next_attempt: usize,
) -> Vec<ChatMessage> {
    let mut messages = base_messages.to_vec();
    messages.push(ChatMessage::assistant(raw.to_string()));
    messages.push(ChatMessage::user(format!(
        "Repair attempt {next_attempt}: \
         上一次输出解析失败：{}\n现在只修复输出格式，不改变业务含义。\n必须返回完整 JSON，包含 \
         decision、summary、role_output。\n不要输出 Markdown、代码块或解释文字。\n原始输出片段: {}",
        failure.description,
        raw_snippet(raw)
    )));
    messages
}

fn fallback_chat_output(raw: &str) -> Option<CognitionOutput> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let stripped = llm_json::strip_code_fences(trimmed).trim();
    if let Some(json_text) =
        llm_json::extract_json_value(stripped).or_else(|| llm_json::extract_json_value(trimmed))
    {
        if let Some(output) = fallback_chat_output_from_json(json_text) {
            return Some(output);
        }
        if looks_json_like(stripped) || looks_json_like(trimmed) {
            return None;
        }
    }

    if looks_json_like(stripped) || looks_json_like(trimmed) {
        return None;
    }

    build_fallback_chat_output(stripped)
}

fn fallback_chat_output_from_json(json_text: &str) -> Option<CognitionOutput> {
    let value = llm_json::parse_llm_json_value(json_text).ok()?;
    let summary = value
        .pointer("/role_output/summary")
        .and_then(|value| value.as_str())
        .or_else(|| value.pointer("/summary").and_then(|value| value.as_str()))?;

    build_fallback_chat_output(summary)
}

fn build_fallback_chat_output(summary: &str) -> Option<CognitionOutput> {
    let summary = summary.trim();
    if summary.is_empty() {
        return None;
    }

    Some(json!({
        "decision": {
            "kind": "NoAction"
        },
        "summary": summary,
        "role_output": {
            "summary": summary,
            "findings": [],
            "recommendations": [],
            "evidence": [],
            "risks": [],
            "open_questions": []
        }
    }))
}

fn looks_json_like(input: &str) -> bool {
    let trimmed = input.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn raw_snippet(raw: &str) -> String {
    raw.trim().chars().take(500).collect()
}

fn render_input(input: &CognitionInput, image_summaries: &[AgentImageSummaryInput]) -> String {
    let facts = if input.context.facts.is_empty() {
        "- (none)".to_string()
    } else {
        input
            .context
            .facts
            .iter()
            .map(|fact| {
                format!(
                    "- source: {} | reliability: {} | content: {}",
                    fact.source, fact.reliability, fact.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let metadata = if input.context.metadata.is_empty() {
        "- (none)".to_string()
    } else {
        input
            .context
            .metadata
            .iter()
            .map(|(key, value)| format!("- {}: {}", key, value))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let image_summary_context = render_image_summary_context(image_summaries);

    format!(
        "Intent:\n- id: {}\n- kind: {:?}\n- description: {}\n\nContext facts:\n{}\n\nContext \
         metadata:\n{}\n\n{}",
        input.intent.id,
        input.intent.kind,
        input.intent.description,
        facts,
        metadata,
        image_summary_context
    )
}

fn render_image_summary_context(image_summaries: &[AgentImageSummaryInput]) -> String {
    if image_summaries.is_empty() {
        return "Preprocessed image context:\n- (none)".to_string();
    }

    let lines = image_summaries
        .iter()
        .map(|item| {
            let context = item
                .agent_context
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(item.summary.trim());
            format!(
                "- source_ref: {} | object_key: {} | analysis_id: {} | prompt_version: {} | \
                 llm_name: {} | summary: {}",
                item.source_ref,
                item.object_key,
                item.analysis_id,
                item.prompt_version.as_deref().unwrap_or("unknown"),
                item.llm_name.as_deref().unwrap_or("unknown"),
                context
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Preprocessed image context:\n{}\n\nUse this cached image context as the image content \
         grounding. Raw image inputs are reserved for system vision preprocess.",
        lines
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognition::{Context, Intent, IntentKind};

    fn llm(name: &str, model: &str) -> AgentLlmCfg {
        AgentLlmCfg {
            name: name.to_string(),
            provider: AgentLlmProvider::ChatCompletions,
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            model: model.to_string(),
            api_key: None,
            max_tokens: None,
            temperature: None,
            reasoning_effort: None,
            thinking: None,
            chat_completions_endpoint: None,
        }
    }

    fn llm_with_provider(
        name: &str,
        provider: AgentLlmProvider,
        base_url: &str,
        api_key: Option<&str>,
    ) -> AgentLlmCfg {
        AgentLlmCfg {
            name: name.to_string(),
            provider,
            base_url: base_url.to_string(),
            model: "test-model".to_string(),
            api_key: api_key.map(str::to_string),
            max_tokens: Some(128),
            temperature: Some(0.2),
            reasoning_effort: None,
            thinking: Some(false),
            chat_completions_endpoint: None,
        }
    }

    fn input() -> CognitionInput {
        CognitionInput {
            intent: Intent {
                id: "test".to_string(),
                kind: IntentKind::Planning,
                description: "test task".to_string(),
            },
            context: Context::default(),
        }
    }

    #[test]
    fn doubao_detection_matches_name_or_model() {
        assert!(is_doubao_llm(&llm(
            "doubao-external",
            "doubao-seed-2-0-pro-260215"
        )));
        assert!(is_doubao_llm(&llm(
            "ark-main",
            "doubao-seed-2-0-pro-260215"
        )));
        assert!(!is_doubao_llm(&llm("ollama-gpt", "gpt-oss:20b")));
    }

    #[test]
    fn build_llm_client_supports_configured_providers() {
        let chat = llm_with_provider(
            "chat",
            AgentLlmProvider::ChatCompletions,
            "http://127.0.0.1:4000/v1",
            Some("sk-test"),
        );
        build_llm_client(&chat).expect("chat completions client");

        let ollama = llm_with_provider(
            "ollama",
            AgentLlmProvider::Ollama,
            "http://127.0.0.1:11434",
            None,
        );
        build_llm_client(&ollama).expect("ollama client");

        let doubao = llm_with_provider(
            "doubao",
            AgentLlmProvider::Doubao,
            "https://ark.cn-beijing.volces.com/api/v3",
            Some("sk-test"),
        );
        build_llm_client(&doubao).expect("doubao client");
    }

    #[test]
    fn doubao_provider_requires_api_key() {
        let config = llm_with_provider(
            "doubao",
            AgentLlmProvider::Doubao,
            "https://ark.cn-beijing.volces.com/api/v3",
            None,
        );

        let err = match build_llm_client(&config) {
            Ok(_) => panic!("missing api key should fail"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("requires api_key"));
    }

    #[test]
    fn registry_resolves_usage_with_fallbacks() {
        let registry = AgentLlmRegistry::new(
            &[
                llm("llm-fast", "fast-model"),
                llm("llm-thinking", "thinking-model"),
                llm("llm-vision", "vision-model"),
            ],
            &AgentLlmUsageCfg {
                default: Some("llm-thinking".to_string()),
                fast: Some("llm-fast".to_string()),
                thinking: Some("llm-thinking".to_string()),
                vision: Some("llm-vision".to_string()),
            },
        )
        .expect("registry should initialize");

        assert_eq!(registry.usage().default.as_deref(), Some("llm-thinking"));
        assert_eq!(registry.usage().fast.as_deref(), Some("llm-fast"));
        assert_eq!(registry.usage().thinking.as_deref(), Some("llm-thinking"));
        assert_eq!(registry.usage().vision.as_deref(), Some("llm-vision"));
    }

    #[test]
    fn registry_routes_chat_worker_to_fast_usage() {
        let registry = AgentLlmRegistry::new(
            &[
                llm("llm-fast", "fast-model"),
                llm("llm-thinking", "thinking-model"),
            ],
            &AgentLlmUsageCfg {
                fast: Some("llm-fast".to_string()),
                thinking: Some("llm-thinking".to_string()),
                ..AgentLlmUsageCfg::default()
            },
        )
        .expect("registry should initialize");

        assert_eq!(
            registry
                .usage_name_for_kind(AgentCognitionKind::Commander, false)
                .expect("commander usage should resolve"),
            "llm-fast"
        );
        assert_eq!(
            registry
                .usage_name_for_kind(AgentCognitionKind::ChatWorker, false)
                .expect("chat worker usage should resolve"),
            "llm-fast"
        );
        assert_eq!(
            registry
                .usage_name_for_kind(AgentCognitionKind::Worker, false)
                .expect("worker usage should resolve"),
            "llm-thinking"
        );
    }

    #[test]
    fn assignment_llm_usage_overrides_worker_usage() {
        let registry = AgentLlmRegistry::new(
            &[
                llm("llm-default", "default-model"),
                llm("llm-fast", "fast-model"),
                llm("llm-thinking", "thinking-model"),
            ],
            &AgentLlmUsageCfg {
                default: Some("llm-default".to_string()),
                fast: Some("llm-fast".to_string()),
                thinking: Some("llm-thinking".to_string()),
                ..AgentLlmUsageCfg::default()
            },
        )
        .expect("registry should initialize");

        let mut input = input();
        assert_eq!(
            registry
                .usage_name_for_input(AgentCognitionKind::Worker, false, &input)
                .expect("worker usage should resolve"),
            "llm-thinking"
        );

        input
            .context
            .metadata
            .insert("assignment.llm_usage".to_string(), "fast".to_string());
        assert_eq!(
            registry
                .usage_name_for_input(AgentCognitionKind::Worker, false, &input)
                .expect("worker usage should resolve"),
            "llm-fast"
        );

        input
            .context
            .metadata
            .insert("assignment.llm_usage".to_string(), "default".to_string());
        assert_eq!(
            registry
                .usage_name_for_input(AgentCognitionKind::Worker, false, &input)
                .expect("worker usage should resolve"),
            "llm-default"
        );
    }

    #[test]
    fn registry_rejects_missing_usage_target() {
        let err = match AgentLlmRegistry::new(
            &[llm("llm-thinking", "thinking-model")],
            &AgentLlmUsageCfg {
                fast: Some("missing".to_string()),
                ..AgentLlmUsageCfg::default()
            },
        ) {
            Ok(_) => panic!("missing usage target should fail"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("llm 'missing' not found"));
    }

    #[test]
    fn llm_access_denied_classifier_matches_provider_auth_errors() {
        assert!(is_llm_access_denied_message(
            "api error: status=403, message=The request failed because you do not have access to \
             the requested resource., type=Forbidden, code=AccessDenied"
        ));
        assert!(is_llm_access_denied_message(
            "llm_access_denied: 当前 AI 模型服务配置不可用"
        ));
        assert!(is_llm_access_denied_message(
            "api error: status: 401, message=Unauthorized"
        ));
        assert!(!is_llm_access_denied_message(
            "LLM returned empty response on attempt 1"
        ));
        assert!(!is_llm_access_denied_message(
            "decode failed on attempt 1/3: missing key role_output"
        ));
    }

    #[test]
    fn llm_provider_error_classifier_matches_upstream_5xx_errors() {
        assert!(is_llm_provider_error_message(
            "LLM request failed on attempt 3: api error: status=500, body=Internal Server Error"
        ));
        assert!(is_llm_provider_error_message(
            "provider request failed: api error: status: 503, message=Service Unavailable"
        ));
        assert!(is_llm_provider_error_message(
            "llm_provider_error: attempt=3; llm_usage=doubao-thinking; model=DouBao-Seed-2.0"
        ));
        assert!(!is_llm_provider_error_message(
            "api error: status=403, code=AccessDenied"
        ));
        assert!(!is_llm_provider_error_message(
            "decode failed on attempt 1/3: missing key role_output"
        ));
    }

    #[test]
    fn llm_failure_user_messages_keep_readable_layout_without_internal_config() {
        for message in [
            llm_access_denied_user_message(),
            llm_provider_error_user_message(),
        ] {
            assert!(message.starts_with("## 当前状态"));
            assert!(message.contains("\n\n## 对本轮回答的影响\n\n- "));
            assert!(message.contains("\n\n## 下一步\n\n- "));
            assert!(!message.contains("services.toml"));
            assert!(!message.contains("[agent_llm_usage]"));
            assert!(!message.contains("base_url"));
            assert!(!message.contains("api_key"));
            assert!(!message.contains("API Key"));
            assert!(!message.contains("llm_provider_error"));
            assert!(!message.contains("llm_access_denied"));
            assert!(!message.contains("failure_kind"));
            assert!(!message.contains("worker"));
        }
    }

    #[test]
    fn parse_image_inputs_detects_forbidden_worker_request_metadata_key() {
        let mut input = input();
        input.context.metadata.insert(
            REQUEST_ATTACHMENTS_IMAGES_JSON.to_string(),
            r#"[{"url":"data:image/png;base64,AA==","detail":"high","sourceRef":"conversation_file:file-1"}]"#
                .to_string(),
        );

        let images = parse_image_inputs(&input).expect("image metadata should parse");

        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "data:image/png;base64,AA==");
        assert_eq!(images[0].detail.as_deref(), Some("high"));
        assert_eq!(
            images[0].source_ref.as_deref(),
            Some("conversation_file:file-1")
        );
    }

    #[test]
    fn parse_image_summary_inputs_accepts_camel_case_metadata() {
        let mut input = input();
        input.context.metadata.insert(
            ATTACHMENTS_IMAGE_SUMMARIES_JSON.to_string(),
            r#"[{"sourceRef":"conversation_file:file-1","objectKey":"users/u/images/a.jpg","summary":"图片摘要","agentContext":"图片上下文","analysisId":"analysis-1","promptVersion":"v1","llmName":"vision"}]"#
                .to_string(),
        );

        let summaries = parse_image_summary_inputs(&input).expect("summary metadata should parse");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].source_ref, "conversation_file:file-1");
        assert_eq!(summaries[0].object_key, "users/u/images/a.jpg");
        assert_eq!(summaries[0].summary, "图片摘要");
        assert_eq!(summaries[0].agent_context.as_deref(), Some("图片上下文"));
        assert_eq!(summaries[0].analysis_id, "analysis-1");
        assert_eq!(summaries[0].prompt_version.as_deref(), Some("v1"));
        assert_eq!(summaries[0].llm_name.as_deref(), Some("vision"));
    }

    #[test]
    fn fallback_chat_output_wraps_plain_text() {
        let output = fallback_chat_output("普通中文回复").expect("plain text should be wrapped");

        assert_eq!(output["decision"]["kind"], "NoAction");
        assert_eq!(output["summary"], "普通中文回复");
        assert_eq!(output["role_output"]["summary"], "普通中文回复");
        assert!(
            output["role_output"]["findings"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn fallback_chat_output_wraps_json_summary() {
        let output = fallback_chat_output(r#"{"summary":"已有摘要"}"#)
            .expect("json summary should be wrapped");

        assert_eq!(output["decision"]["kind"], "NoAction");
        assert_eq!(output["summary"], "已有摘要");
        assert_eq!(output["role_output"]["summary"], "已有摘要");
    }

    #[test]
    fn fallback_chat_output_extracts_fenced_json_summary() {
        let output = fallback_chat_output(
            r#"```json
{"summary":"代码块摘要"}
```"#,
        )
        .expect("fenced json summary should be wrapped");

        assert_eq!(output["decision"]["kind"], "NoAction");
        assert_eq!(output["summary"], "代码块摘要");
        assert_eq!(output["role_output"]["summary"], "代码块摘要");
    }

    #[test]
    fn fallback_chat_output_extracts_embedded_json_summary() {
        let output = fallback_chat_output(r#"说明：{"summary":"夹杂摘要"}，结束。"#)
            .expect("embedded json summary should be wrapped");

        assert_eq!(output["decision"]["kind"], "NoAction");
        assert_eq!(output["summary"], "夹杂摘要");
        assert_eq!(output["role_output"]["summary"], "夹杂摘要");
    }

    #[test]
    fn fallback_chat_output_rejects_invalid_json_like_text() {
        assert!(fallback_chat_output(r#"{"summary":"缺少结尾""#).is_none());
    }

    #[test]
    fn fallback_chat_output_rejects_empty_text() {
        assert!(fallback_chat_output("  \n\t ").is_none());
    }
}
