use std::{path::PathBuf, sync::Arc, time::Duration};

use serde::Deserialize;

use crate::{
    agent_error::Result,
    api::{
        AgentManager, AgentManagerConfig, Cognition, FileMemoryStore, RoleCognitionFactory,
        RoleProfile, RolePromptBuilder,
    },
    llm_routing::{
        AgentCognitionKind, AgentLlmCfg, AgentLlmRegistry, AgentLlmUsageCfg, RoutingCognitionEngine,
    },
    skills::SearchAdapterConfig,
};

#[derive(Debug, Clone, Deserialize)]
pub struct AgentRuntimeCfg {
    #[serde(default)]
    pub llm: Vec<AgentLlmCfg>,
    #[serde(default)]
    pub llm_usage: AgentLlmUsageCfg,
    #[serde(default)]
    pub search: SearchAdapterConfig,
    #[serde(default = "default_worker_cognition_timeout_sec")]
    pub worker_cognition_timeout_sec: u64,
    #[serde(default = "default_memory_path")]
    pub memory_path: PathBuf,
}

impl AgentRuntimeCfg {
    pub fn from_llm(llm: Vec<AgentLlmCfg>) -> Self {
        Self {
            llm,
            llm_usage: AgentLlmUsageCfg::default(),
            search: SearchAdapterConfig::default(),
            worker_cognition_timeout_sec: default_worker_cognition_timeout_sec(),
            memory_path: default_memory_path(),
        }
    }
}

fn default_worker_cognition_timeout_sec() -> u64 {
    45
}

fn default_memory_path() -> PathBuf {
    PathBuf::from("data/agent-memory.json")
}

pub fn build_agent_manager(config: AgentRuntimeCfg) -> Result<AgentManager> {
    let search = config.search.clone();
    let llms = AgentLlmRegistry::new(&config.llm, &config.llm_usage)?;

    let commander_llms = llms.clone();
    let commander = Arc::new(move || {
        Ok(Box::new(RoutingCognitionEngine::new(
            commander_llms.clone(),
            AgentCognitionKind::Commander,
            "config/prompts/zh/cognition/commander_routing.md",
        )?) as Box<dyn Cognition + Send>)
    });

    let worker_llms = llms.clone();
    let worker: RoleCognitionFactory = Arc::new(move |role: &RoleProfile| {
        if role.runtime_role == "chat" {
            return Ok(Box::new(RoutingCognitionEngine::new(
                worker_llms.clone(),
                AgentCognitionKind::ChatWorker,
                "config/prompts/zh/cognition/chat_worker.md",
            )?) as Box<dyn Cognition + Send>);
        }

        let role_prompt = RolePromptBuilder::build_worker_system_prompt(role);
        Ok(Box::new(RoutingCognitionEngine::new_with_system_append(
            worker_llms.clone(),
            AgentCognitionKind::Worker,
            "config/prompts/zh/cognition/worker_execution.md",
            role_prompt,
        )?) as Box<dyn Cognition + Send>)
    });

    let mut manager_config = AgentManagerConfig::new_role_aware(commander, worker);
    manager_config.runtime.response_timeout = Duration::from_secs(900);
    manager_config.runtime.worker_cognition_timeout =
        Duration::from_secs(config.worker_cognition_timeout_sec.max(1));
    manager_config.runtime.commander_fast_route = true;
    manager_config.runtime.commander_fast_route_min_score = 2.0;
    manager_config.runtime.search = search;
    manager_config.memory_store = Arc::new(FileMemoryStore::open(&config.memory_path)?);

    Ok(AgentManager::new(manager_config))
}
