use std::path::PathBuf;

use agent::api::{AgentLlmCfg, AgentLlmUsageCfg, SearchAdapterConfig};
use db_core::DatabaseConfig;
use serde::Deserialize;
use service::api::LearningQuestionDeliveryMode;
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
    pub learning: LearningRuntimeSettings,
    #[serde(default)]
    pub agent_runtime: AgentRuntimeSettings,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LearningRuntimeSettings {
    #[serde(default)]
    pub question_delivery_mode: LearningQuestionDeliveryMode,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learning_runtime_defaults_to_disabled() {
        let settings: LearningRuntimeSettings =
            serde_json::from_str("{}").expect("default learning runtime settings");
        assert_eq!(
            settings.question_delivery_mode,
            LearningQuestionDeliveryMode::Disabled
        );
    }

    #[test]
    fn learning_runtime_accepts_only_stable_delivery_mode_codes() {
        let internal: LearningRuntimeSettings =
            serde_json::from_str(r#"{"question_delivery_mode":"internal_pilot"}"#)
                .expect("internal pilot settings");
        assert_eq!(
            internal.question_delivery_mode,
            LearningQuestionDeliveryMode::InternalPilot
        );

        assert!(
            serde_json::from_str::<LearningRuntimeSettings>(
                r#"{"question_delivery_mode":"内部灰度"}"#
            )
            .is_err()
        );
    }
}
