use std::{sync::Arc, time::Duration};

use model_gateway_rs::llm::chat_completions::ChatCompletionsLlm;
use serde::Deserialize;

use crate::{
    agent_error::{Error, LlmError, Result, SettingsError},
    api::{
        AgentManager, AgentManagerConfig, Cognition, CognitionEngine, RoleCognitionFactory,
        RoleProfile, RolePromptBuilder,
    },
    skills::SearchAdapterConfig,
};

pub type AgentLlmClient = ChatCompletionsLlm;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentLlmCfg {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub chat_completions_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentRuntimeCfg {
    #[serde(default)]
    pub llm: Vec<AgentLlmCfg>,
    #[serde(default)]
    pub default_llm: Option<String>,
    #[serde(default)]
    pub search: SearchAdapterConfig,
}

impl AgentRuntimeCfg {
    pub fn from_llm(llm: Vec<AgentLlmCfg>) -> Self {
        Self {
            llm,
            default_llm: None,
            search: SearchAdapterConfig::default(),
        }
    }
}

pub fn build_agent_manager(config: AgentRuntimeCfg) -> Result<AgentManager> {
    let search = config.search.clone();
    let llm = Arc::new(build_default_llm(config)?);

    let commander_llm = Arc::clone(&llm);
    let commander = Arc::new(move || {
        Ok(Box::new(CognitionEngine::new(
            Arc::clone(&commander_llm),
            "config/prompts/zh/cognition/commander_routing.md",
        )?) as Box<dyn Cognition + Send>)
    });

    let worker_llm = Arc::clone(&llm);
    let worker: RoleCognitionFactory = Arc::new(move |role: &RoleProfile| {
        if role.runtime_role == "chat" {
            return Ok(Box::new(CognitionEngine::new(
                Arc::clone(&worker_llm),
                "config/prompts/zh/cognition/chat_worker.md",
            )?) as Box<dyn Cognition + Send>);
        }

        let role_prompt = RolePromptBuilder::build_worker_system_prompt(role);
        Ok(Box::new(CognitionEngine::new_with_system_append(
            Arc::clone(&worker_llm),
            "config/prompts/zh/cognition/worker_execution.md",
            role_prompt,
        )?) as Box<dyn Cognition + Send>)
    });

    let mut manager_config = AgentManagerConfig::new_role_aware(commander, worker);
    manager_config.runtime.response_timeout = Duration::from_secs(900);
    manager_config.runtime.commander_fast_route = true;
    manager_config.runtime.commander_fast_route_min_score = 2.0;
    manager_config.runtime.search = search;

    Ok(AgentManager::new(manager_config))
}

fn build_default_llm(config: AgentRuntimeCfg) -> Result<AgentLlmClient> {
    if config.llm.is_empty() {
        return Err(SettingsError::missing_llm_usage(
            "agent runtime requires at least one [[llm]] entry",
        )
        .into());
    }

    let selected = select_default_llm_config(&config)?;

    let mut client = ChatCompletionsLlm::new(
        &selected.base_url,
        &selected.model,
        selected.api_key.as_deref(),
    )
    .map_err(|err| {
        Error::from(LlmError::message(format!(
            "init llm client '{}' failed: {err}",
            selected.name
        )))
    })?
    .with_max_tokens(selected.max_tokens)
    .with_temperature(selected.temperature)
    .with_reasoning_effort(selected.reasoning_effort.as_deref());

    if let Some(endpoint) = &selected.chat_completions_endpoint {
        client = client.with_chat_completions_endpoint(endpoint.clone());
    }

    Ok(client)
}

fn select_default_llm_config(config: &AgentRuntimeCfg) -> Result<&AgentLlmCfg> {
    if config.llm.is_empty() {
        return Err(SettingsError::missing_llm_usage(
            "agent runtime requires at least one [[llm]] entry",
        )
        .into());
    }

    match config.default_llm.as_deref() {
        Some(name) => config
            .llm
            .iter()
            .find(|llm| llm.name == name)
            .ok_or_else(|| {
                SettingsError::missing_llm_usage(format!("llm '{name}' not found")).into()
            }),
        None => Ok(config
            .llm
            .iter()
            .find(|llm| is_doubao_llm(llm))
            .or_else(|| config.llm.first())
            .expect("checked non-empty llm config above")),
    }
}

fn is_doubao_llm(llm: &AgentLlmCfg) -> bool {
    let name = llm.name.to_ascii_lowercase();
    let model = llm.model.to_ascii_lowercase();
    name.contains("doubao") || model.contains("doubao")
}

#[cfg(test)]
mod tests {
    use super::{AgentLlmCfg, AgentRuntimeCfg, is_doubao_llm, select_default_llm_config};

    fn llm(name: &str, model: &str) -> AgentLlmCfg {
        AgentLlmCfg {
            name: name.to_string(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            model: model.to_string(),
            api_key: None,
            max_tokens: None,
            temperature: None,
            reasoning_effort: None,
            chat_completions_endpoint: None,
        }
    }

    #[test]
    fn doubao_detection_matches_name_or_model() {
        assert!(is_doubao_llm(&llm(
            "doubao-external",
            "doubao-seed-2-0-pro-260215"
        )));
        assert!(is_doubao_llm(&llm(
            "ark-main",
            "doubao-seed-2-0-pro-260215"
        )));
        assert!(!is_doubao_llm(&llm("ollama-gpt", "gpt-oss:20b")));
    }

    #[test]
    fn runtime_config_prefers_doubao_without_default_llm() {
        let config = AgentRuntimeCfg::from_llm(vec![
            llm("ollama-gpt", "gpt-oss:20b"),
            llm("doubao-external", "doubao-seed-2-0-pro-260215"),
        ]);

        let preferred = select_default_llm_config(&config).expect("selected llm");

        assert_eq!(preferred.name, "doubao-external");
    }

    #[test]
    fn runtime_config_respects_explicit_default_llm() {
        let mut config = AgentRuntimeCfg::from_llm(vec![
            llm("doubao-external", "doubao-seed-2-0-pro-260215"),
            llm("ollama-gpt", "gpt-oss:20b"),
        ]);
        config.default_llm = Some("ollama-gpt".to_string());

        let preferred = select_default_llm_config(&config).expect("selected llm");

        assert_eq!(preferred.name, "ollama-gpt");
    }
}
