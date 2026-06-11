use std::path::PathBuf;

use agent::api::{AgentLlmCfg, AgentLlmUsageCfg, SearchAdapterConfig};
use db_core::DatabaseConfig;
use serde::Deserialize;
use storage::StorageConfig;
use toolcraft_config::load_settings;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub http: HttpCfg,
    pub jwt_verify: JwtVerifyRemoteCfg,
    pub db: Vec<DatabaseConfig>,
    pub s3: StorageConfig,
    #[serde(default)]
    pub agent_llm_usage: AgentLlmUsageCfg,
    #[serde(default)]
    pub llm: Vec<AgentLlmCfg>,
    #[serde(default)]
    pub external_search: SearchAdapterConfig,
    #[serde(default)]
    pub agent_runtime: AgentRuntimeSettings,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AgentRuntimeSettings {
    #[serde(default)]
    pub worker_cognition_timeout_sec: Option<u64>,
    #[serde(default = "default_agent_memory_path")]
    pub memory_path: PathBuf,
}

fn default_agent_memory_path() -> PathBuf {
    PathBuf::from("data/agent-memory.json")
}

#[derive(Debug, Deserialize)]
pub struct HttpCfg {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtVerifyRemoteCfg {
    pub url: String,
    pub header: String,
    pub token: String,
}

impl Settings {
    pub fn load(config_path: &str) -> Result<Self> {
        let r = load_settings(config_path)?;
        Ok(r)
    }
}
