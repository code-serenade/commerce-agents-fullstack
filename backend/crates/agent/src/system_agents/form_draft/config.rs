use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    agent_error::{Result, SettingsError},
    llm_routing::AgentLlmUsageCfg,
};

const DEFAULT_CONFIG_PATH: &str = "config/json/agent/system-agents/form-draft-generator.json";

#[derive(Debug, Clone, Deserialize)]
pub struct FormDraftPromptConfig {
    pub product: String,
    pub campaign: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormDraftAgentConfig {
    pub id: String,
    pub kind: String,
    pub visibility: String,
    pub user_selectable: bool,
    pub auto_route: bool,
    pub llm_usage: String,
    pub analysis_kind: String,
    pub prompt_version: String,
    pub prompts: FormDraftPromptConfig,
}

impl FormDraftAgentConfig {
    pub fn load_default() -> Result<Self> {
        Self::load(DEFAULT_CONFIG_PATH)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = resolve_config_path(path.as_ref());
        let content = fs::read_to_string(&path).map_err(|err| {
            SettingsError::invalid(format!(
                "failed to read form draft config '{}': {err}",
                path.display()
            ))
        })?;
        let config = serde_json::from_str::<Self>(&content).map_err(|err| {
            SettingsError::invalid(format!(
                "invalid form draft config '{}': {err}",
                path.display()
            ))
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn usage_name(&self, usage: &AgentLlmUsageCfg) -> Result<String> {
        match self.llm_usage.as_str() {
            "fast" => usage.fast.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage("fast llm is not configured for form draft").into()
            }),
            "thinking" => usage.thinking.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage("thinking llm is not configured for form draft")
                    .into()
            }),
            "default" => usage.default.clone().ok_or_else(|| {
                SettingsError::missing_llm_usage("default llm is not configured for form draft")
                    .into()
            }),
            other => Err(SettingsError::invalid(format!(
                "unsupported form draft llm_usage '{other}'"
            ))
            .into()),
        }
    }

    pub fn prompt_for_kind(&self, kind: super::FormDraftKind) -> &str {
        match kind {
            super::FormDraftKind::Product => &self.prompts.product,
            super::FormDraftKind::Campaign => &self.prompts.campaign,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(SettingsError::invalid("form draft id is empty").into());
        }
        if self.kind != "system_agent" {
            return Err(SettingsError::invalid("form draft kind must be system_agent").into());
        }
        if self.visibility != "system" || self.user_selectable || self.auto_route {
            return Err(SettingsError::invalid(
                "form draft agent must be system-only and not user selectable",
            )
            .into());
        }
        if self.analysis_kind != "form_draft" {
            return Err(
                SettingsError::invalid("form draft analysis_kind must be form_draft").into(),
            );
        }
        if self.prompt_version.trim().is_empty() {
            return Err(SettingsError::invalid("form draft prompt_version is empty").into());
        }
        if self.prompts.product.trim().is_empty() {
            return Err(SettingsError::invalid("form draft product prompt is empty").into());
        }
        if self.prompts.campaign.trim().is_empty() {
            return Err(SettingsError::invalid("form draft campaign prompt is empty").into());
        }
        Ok(())
    }
}

fn resolve_config_path(path: &Path) -> PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}
