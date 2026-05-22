use std::{fs, path::Path};

use serde::Deserialize;

use crate::{
    agent_error::{Result, SettingsError},
    llm_routing::AgentLlmUsageCfg,
};

const DEFAULT_CONFIG_PATH: &str = "config/json/agent/system-agents/vision-preprocessor.json";

#[derive(Debug, Clone, Deserialize)]
pub struct VisionPreprocessAgentConfig {
    pub id: String,
    pub kind: String,
    pub visibility: String,
    pub user_selectable: bool,
    pub auto_route: bool,
    pub llm_usage: String,
    pub prompt: String,
    pub analysis_kind: String,
    pub prompt_version: String,
}

impl VisionPreprocessAgentConfig {
    pub fn load_default() -> Result<Self> {
        Self::load(DEFAULT_CONFIG_PATH)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|err| {
            SettingsError::invalid(format!(
                "failed to read vision preprocess config '{}': {err}",
                path.display()
            ))
        })?;
        let config = serde_json::from_str::<Self>(&content).map_err(|err| {
            SettingsError::invalid(format!(
                "invalid vision preprocess config '{}': {err}",
                path.display()
            ))
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn usage_name(&self, usage: &AgentLlmUsageCfg) -> Result<String> {
        match self.llm_usage.as_str() {
            "vision" => usage.vision.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage(
                    "vision llm is not configured for image preprocess",
                )
                .into()
            }),
            "fast" => usage.fast.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage("fast llm is not configured for image preprocess")
                    .into()
            }),
            "thinking" => usage.thinking.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage(
                    "thinking llm is not configured for image preprocess",
                )
                .into()
            }),
            "default" => usage.default.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage(
                    "default llm is not configured for image preprocess",
                )
                .into()
            }),
            other => Err(SettingsError::invalid(format!(
                "unsupported vision preprocess llm_usage '{other}'"
            ))
            .into()),
        }
    }

    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(SettingsError::invalid("vision preprocess id is empty").into());
        }
        if self.kind != "system_agent" {
            return Err(
                SettingsError::invalid("vision preprocess kind must be system_agent").into(),
            );
        }
        if self.visibility != "system" || self.user_selectable || self.auto_route {
            return Err(SettingsError::invalid(
                "vision preprocess agent must be system-only and not user selectable",
            )
            .into());
        }
        if self.prompt.trim().is_empty() {
            return Err(SettingsError::invalid("vision preprocess prompt is empty").into());
        }
        if self.analysis_kind.trim().is_empty() {
            return Err(SettingsError::invalid("vision preprocess analysis_kind is empty").into());
        }
        if self.prompt_version.trim().is_empty() {
            return Err(SettingsError::invalid("vision preprocess prompt_version is empty").into());
        }
        Ok(())
    }
}
