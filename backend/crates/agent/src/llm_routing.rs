use std::{collections::HashMap, path::Path, sync::Arc};

use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::{
        llm::{ChatContentPart, ChatMessage, LlmInput},
        role::Role,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    Cognition, CognitionDecoder, CognitionFailure, CognitionInput, CognitionOutput,
    CognitionResult, FailureReason, JsonTemplateDecoder,
    agent_error::{Error, LlmError, Result, SettingsError},
    agent_utils::prompt::Prompt,
};

pub type AgentLlmClient = ChatCompletionsLlm;

const ATTACHMENTS_IMAGES_JSON: &str = "attachments.images_json";

#[derive(Debug, Clone, Deserialize)]
pub struct AgentLlmCfg {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub chat_completions_endpoint: Option<String>,
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
    pub fn new(
        llm_configs: &[AgentLlmCfg],
        default_llm: Option<&str>,
        usage: &AgentLlmUsageCfg,
    ) -> Result<Self> {
        if llm_configs.is_empty() {
            return Err(SettingsError::missing_llm_usage(
                "agent runtime requires at least one [[llm]] entry",
            )
            .into());
        }

        let default = resolve_usage_name(llm_configs, usage.default.as_deref().or(default_llm))
            .unwrap_or_else(|| select_implicit_default_llm(llm_configs).name.clone());
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
        for config in llm_configs {
            clients.insert(config.name.clone(), Arc::new(build_llm_client(config)?));
        }

        Ok(Self {
            clients,
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

    fn client_for_kind(
        &self,
        kind: AgentCognitionKind,
        requires_vision: bool,
    ) -> Result<Arc<AgentLlmClient>> {
        let name = if requires_vision {
            self.usage.vision.as_deref().ok_or_else(|| {
                SettingsError::missing_llm_usage("vision llm is not configured for image input")
            })?
        } else {
            match kind {
                AgentCognitionKind::Commander | AgentCognitionKind::ChatWorker => &self.usage.fast,
                AgentCognitionKind::Worker => &self.usage.thinking,
            }
        };

        self.clients.get(name).cloned().ok_or_else(|| {
            SettingsError::missing_llm_usage(format!("llm '{name}' not found")).into()
        })
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
        let requires_vision = !images.is_empty();
        let llm = match self.llms.client_for_kind(self.kind, requires_vision) {
            Ok(llm) => llm,
            Err(err) => {
                return CognitionResult::Failure(CognitionFailure {
                    reason: FailureReason::InternalError,
                    description: err.to_string(),
                });
            }
        };
        let mut base_messages = self.system_messages.clone();
        base_messages.push(render_user_message(&input, &images));

        let max_attempts = self.retry_policy.max_attempts();
        let mut messages = base_messages.clone();
        let mut last_failure = CognitionFailure {
            reason: FailureReason::InternalError,
            description: "LLM cognition did not run".to_string(),
        };

        for attempt in 1 ..= max_attempts {
            let raw = match llm.chat_once(LlmInput { messages }).await {
                Ok(output) => output.get_content().to_string(),
                Err(err) => {
                    crate::log_error!(err);
                    last_failure = CognitionFailure {
                        reason: FailureReason::InternalError,
                        description: format!("LLM request failed on attempt {attempt}: {err}"),
                    };
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

        CognitionResult::Failure(CognitionFailure {
            reason: last_failure.reason,
            description: format!(
                "cognition failed after {max_attempts} attempt(s): {}",
                last_failure.description
            ),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AgentImageInput {
    url: String,
    detail: Option<String>,
    source_ref: Option<String>,
}

fn build_llm_client(config: &AgentLlmCfg) -> Result<AgentLlmClient> {
    let mut client =
        ChatCompletionsLlm::new(&config.base_url, &config.model, config.api_key.as_deref())
            .map_err(|err| {
                Error::from(LlmError::message(format!(
                    "init llm client '{}' failed: {err}",
                    config.name
                )))
            })?
            .with_max_tokens(config.max_tokens)
            .with_temperature(config.temperature)
            .with_reasoning_effort(config.reasoning_effort.as_deref());

    if let Some(endpoint) = &config.chat_completions_endpoint {
        client = client.with_chat_completions_endpoint(endpoint.clone());
    }

    Ok(client)
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

fn select_implicit_default_llm(llm_configs: &[AgentLlmCfg]) -> &AgentLlmCfg {
    llm_configs
        .iter()
        .find(|llm| is_doubao_llm(llm))
        .or_else(|| llm_configs.first())
        .expect("checked non-empty llm config above")
}

fn is_doubao_llm(llm: &AgentLlmCfg) -> bool {
    let name = llm.name.to_ascii_lowercase();
    let model = llm.model.to_ascii_lowercase();
    name.contains("doubao") || model.contains("doubao")
}

fn parse_image_inputs(
    input: &CognitionInput,
) -> std::result::Result<Vec<AgentImageInput>, CognitionFailure> {
    let Some(raw) = input.context.metadata.get(ATTACHMENTS_IMAGES_JSON) else {
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

fn render_user_message(input: &CognitionInput, images: &[AgentImageInput]) -> ChatMessage {
    let text = render_input(input);
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
        "Repair attempt {next_attempt}: 上一条输出无法被系统解析为目标 JSON。\n请只返回合法 \
         JSON，不要 Markdown，不要解释。\n必须满足系统提示词中的 JSON schema \
         和字段结构。\n解析错误: {}\n原始输出片段: {}",
        failure.description,
        raw_snippet(raw)
    )));
    messages
}

fn raw_snippet(raw: &str) -> String {
    raw.trim().chars().take(500).collect()
}

fn render_input(input: &CognitionInput) -> String {
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

    format!(
        "Intent:\n- id: {}\n- kind: {:?}\n- description: {}\n\nContext facts:\n{}\n\nContext \
         metadata:\n{}",
        input.intent.id, input.intent.kind, input.intent.description, facts, metadata
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn llm(name: &str, model: &str) -> AgentLlmCfg {
        AgentLlmCfg {
            name: name.to_string(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            model: model.to_string(),
            api_key: None,
            max_tokens: None,
            temperature: None,
            reasoning_effort: None,
            chat_completions_endpoint: None,
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
    fn registry_resolves_usage_with_fallbacks() {
        let registry = AgentLlmRegistry::new(
            &[
                llm("llm-fast", "fast-model"),
                llm("llm-thinking", "thinking-model"),
                llm("llm-vision", "vision-model"),
            ],
            Some("llm-thinking"),
            &AgentLlmUsageCfg {
                fast: Some("llm-fast".to_string()),
                thinking: Some("llm-thinking".to_string()),
                vision: Some("llm-vision".to_string()),
                default: None,
            },
        )
        .expect("registry should initialize");

        assert_eq!(registry.usage().default.as_deref(), Some("llm-thinking"));
        assert_eq!(registry.usage().fast.as_deref(), Some("llm-fast"));
        assert_eq!(registry.usage().thinking.as_deref(), Some("llm-thinking"));
        assert_eq!(registry.usage().vision.as_deref(), Some("llm-vision"));
    }

    #[test]
    fn registry_rejects_missing_usage_target() {
        let err = match AgentLlmRegistry::new(
            &[llm("llm-thinking", "thinking-model")],
            Some("llm-thinking"),
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
}
