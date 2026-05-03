use db_core::DatabaseConfig;
use serde::Deserialize;
use toolcraft_config::load_settings;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub http: HttpCfg,
    pub jwt_verify: JwtVerifyRemoteCfg,
    pub db: Vec<DatabaseConfig>,
    #[serde(default)]
    pub llm: Vec<LlmCfg>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct LlmCfg {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub chat_completions_endpoint: Option<String>,
}

impl Settings {
    pub fn load(config_path: &str) -> Result<Self> {
        let r = load_settings(config_path)?;
        Ok(r)
    }
}
