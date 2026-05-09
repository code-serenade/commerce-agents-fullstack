use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BuildAgentProductContextRequest {
    pub user_id: String,
    pub product_ids: Vec<String>,
    pub context_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildAgentRetrievalContextRequest {
    pub user_id: String,
    pub query: String,
    pub product_ids: Vec<String>,
    pub context_refs: Vec<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct AgentRetrievalContextResponse {
    pub mode: String,
    pub query_terms: Vec<String>,
    pub product_ids: Vec<String>,
    pub knowledge_count: usize,
    pub source_count: usize,
    pub summary: String,
    pub sources: Vec<AgentContextSource>,
    pub degraded: bool,
    pub degraded_reason: Option<String>,
    pub external_search_available: bool,
}
