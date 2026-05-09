use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BuildAgentProductContextRequest {
    pub user_id: String,
    pub product_ids: Vec<String>,
    pub context_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentContextSource {
    pub source_type: String,
    pub id: String,
    pub product_id: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentProductContextResponse {
    pub product_ids: Vec<String>,
    pub context_refs: Vec<String>,
    pub product_count: usize,
    pub knowledge_count: usize,
    pub summary: String,
    pub sources: Vec<AgentContextSource>,
}
