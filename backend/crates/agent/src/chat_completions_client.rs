use async_trait::async_trait;
use model_gateway_rs::{
    error::{Error as GatewayError, Result as GatewayResult},
    llm::Llm,
    model::{
        llm::{ChatMessage, ChatMessageContent, LlmInput, LlmOutput},
        role::Role,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use toolcraft_request::{ByteStream, HeaderMap, Request, response::Response};

const ROOT_CHAT_COMPLETIONS_ENDPOINT: &str = "v1/chat/completions";
const VERSIONED_CHAT_COMPLETIONS_ENDPOINT: &str = "chat/completions";

pub struct AgentChatCompletionsLlm {
    request: Request,
    model: String,
    chat_completions_endpoint: String,
    default_max_tokens: Option<u32>,
    default_temperature: Option<f32>,
    default_reasoning_effort: Option<String>,
    default_thinking: Option<ThinkingConfig>,
}

impl AgentChatCompletionsLlm {
    pub fn new(base_url: &str, model: &str, api_key: Option<&str>) -> GatewayResult<Self> {
        let mut request = Request::new()?;
        request.set_base_url(base_url)?;

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".to_string())?;
        if let Some(api_key) = api_key {
            headers.insert("Authorization", format!("Bearer {api_key}"))?;
        }
        request.set_default_headers(headers);

        Ok(Self {
            request,
            model: model.to_string(),
            chat_completions_endpoint: default_chat_completions_endpoint(base_url).to_string(),
            default_max_tokens: None,
            default_temperature: None,
            default_reasoning_effort: None,
            default_thinking: None,
        })
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.default_max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.default_temperature = temperature;
        self
    }

    pub fn with_reasoning_effort(mut self, reasoning_effort: Option<&str>) -> Self {
        self.default_reasoning_effort = reasoning_effort.map(str::to_string);
        self
    }

    pub fn with_thinking(mut self, thinking: Option<bool>) -> Self {
        self.default_thinking = thinking.map(ThinkingConfig::from_enabled);
        self
    }

    pub fn with_chat_completions_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.chat_completions_endpoint = endpoint.into().trim_start_matches('/').to_string();
        self
    }
}

#[async_trait]
impl Llm for AgentChatCompletionsLlm {
    async fn chat_once(&self, input: LlmInput) -> GatewayResult<LlmOutput> {
        let payload = self.payload(input.messages, false)?;
        let response = self
            .request
            .post(&self.chat_completions_endpoint, &payload, None)
            .await?;
        parse_chat_response(response).await
    }

    async fn chat_stream(&self, input: LlmInput) -> GatewayResult<ByteStream> {
        let payload = self.payload(input.messages, true)?;
        self.request
            .post_stream(&self.chat_completions_endpoint, &payload, None)
            .await
            .map_err(GatewayError::from)
    }
}

impl AgentChatCompletionsLlm {
    fn payload(&self, messages: Vec<ChatMessage>, stream: bool) -> GatewayResult<Value> {
        serde_json::to_value(OpenAiChatRequest {
            model: self.model.clone(),
            messages,
            temperature: self.default_temperature,
            max_tokens: self.default_max_tokens,
            reasoning_effort: self.default_reasoning_effort.clone(),
            thinking: self.default_thinking.clone(),
            stream: Some(stream),
        })
        .map_err(GatewayError::from)
    }
}

async fn parse_chat_response(response: Response) -> GatewayResult<LlmOutput> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(GatewayError::ApiError(format_error_body(
            status.as_u16(),
            &body,
        )));
    }

    if let Ok(error) = serde_json::from_str::<OpenAiErrorResponse>(&body) {
        return Err(GatewayError::ApiError(error.to_string()));
    }

    let json: OpenAiChatResponse = serde_json::from_str(&body)?;
    Ok(json.into())
}

fn format_error_body(status: u16, body: &str) -> String {
    match serde_json::from_str::<OpenAiErrorResponse>(body) {
        Ok(error) => format!("status={status}, {error}"),
        Err(_) => format!("status={status}, body={body}"),
    }
}

fn default_chat_completions_endpoint(base_url: &str) -> &'static str {
    let base_url = base_url.trim_end_matches('/');

    if base_url.ends_with("/v1") || base_url.ends_with("/api/v1") || base_url.ends_with("/api/v3") {
        VERSIONED_CHAT_COMPLETIONS_ENDPOINT
    } else {
        ROOT_CHAT_COMPLETIONS_ENDPOINT
    }
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    kind: &'static str,
}

impl ThinkingConfig {
    fn from_enabled(enabled: bool) -> Self {
        Self {
            kind: if enabled { "enabled" } else { "disabled" },
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorResponse {
    error: OpenAiError,
}

impl std::fmt::Display for OpenAiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiError {
    message: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    param: Option<String>,
    code: Option<String>,
}

impl std::fmt::Display for OpenAiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message={}", self.message)?;
        if let Some(kind) = &self.kind {
            write!(f, ", type={kind}")?;
        }
        if let Some(param) = &self.param {
            write!(f, ", param={param}")?;
        }
        if let Some(code) = &self.code {
            write!(f, ", code={code}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    role: Role,
    content: Option<ChatMessageContent>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    total_tokens: Option<u32>,
}

impl From<OpenAiChatResponse> for LlmOutput {
    fn from(response: OpenAiChatResponse) -> Self {
        let message = response
            .choices
            .into_iter()
            .next()
            .map(|choice| ChatMessage {
                role: choice.message.role,
                content: choice.message.content.unwrap_or_default(),
            });

        LlmOutput {
            message,
            usage: response.usage.and_then(|usage| usage.total_tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use model_gateway_rs::model::llm::ChatMessage;

    use super::*;

    #[test]
    fn thinking_false_serializes_to_ark_disabled_object() {
        let client =
            AgentChatCompletionsLlm::new("https://ark.cn-beijing.volces.com/api/v3", "m", None)
                .expect("client should build")
                .with_thinking(Some(false));

        let payload = client
            .payload(vec![ChatMessage::user("hello")], false)
            .expect("payload should serialize");

        assert_eq!(
            payload.get("thinking").and_then(|value| value.get("type")),
            Some(&Value::String("disabled".to_string()))
        );
    }

    #[test]
    fn omitted_thinking_does_not_add_provider_specific_field() {
        let client = AgentChatCompletionsLlm::new("https://api.openai.com/v1", "m", None)
            .expect("client should build");

        let payload = client
            .payload(vec![ChatMessage::user("hello")], false)
            .expect("payload should serialize");

        assert!(payload.get("thinking").is_none());
    }
}
