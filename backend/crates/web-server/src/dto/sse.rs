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
}
