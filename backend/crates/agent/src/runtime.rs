use std::{sync::Arc, time::Duration};

use model_gateway_rs::llm::chat_completions::ChatCompletionsLlm;
use serde::Deserialize;

use crate::{
    agent_error::{Error, LlmError, Result, SettingsError},
    api::{
        AgentManager, AgentManagerConfig, Cognition, CognitionEngine, RoleCognitionFactory,
        RoleProfile, RolePromptBuilder,
    },
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
}

impl AgentRuntimeCfg {
    pub fn from_llm(llm: Vec<AgentLlmCfg>) -> Self {
        Self {
            llm,
            default_llm: None,
        }
    }
}

pub fn build_agent_manager(config: AgentRuntimeCfg) -> Result<AgentManager> {
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

    Ok(AgentManager::new(manager_config))
}

fn build_default_llm(config: AgentRuntimeCfg) -> Result<AgentLlmClient> {
    if config.llm.is_empty() {
        return Err(SettingsError::missing_llm_usage(
            "agent runtime requires at least one [[llm]] entry",
        )
        .into());
    }

    let selected = match config.default_llm.as_deref() {
        Some(name) => config
            .llm
            .iter()
            .find(|llm| llm.name == name)
            .ok_or_else(|| SettingsError::missing_llm_usage(format!("llm '{name}' not found")))?,
        None => config
            .llm
            .first()
            .expect("checked non-empty llm config above"),
    };

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
