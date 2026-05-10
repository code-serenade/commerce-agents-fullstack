use agent::api::AgentLlmCfg;
use db_core::DatabaseConfig;
use serde::Deserialize;
use service::dto::external_search::ExternalSearchConfig;
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
    pub default_llm: Option<String>,
    #[serde(default)]
    pub llm: Vec<AgentLlmCfg>,
    #[serde(default)]
    pub external_search: ExternalSearchConfig,
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
