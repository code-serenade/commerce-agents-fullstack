use model_gateway_rs::model::llm::ChatMessage;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LlmChatStreamRequest {
    #[validate(length(min = 1, max = 64))]
    pub llm_name: Option<String>,
    pub messages: Vec<ChatMessage>,
}
