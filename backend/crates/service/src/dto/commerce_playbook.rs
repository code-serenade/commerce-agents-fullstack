use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommerceAgentPlaybookResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub recommended_strategy: String,
    pub prompt: String,
    pub required_context: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommerceAgentPlaybookListResponse {
    pub playbooks: Vec<CommerceAgentPlaybookResponse>,
}
