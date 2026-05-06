use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use agent::api::{
    AgentId, AgentManager, AgentRequest, AgentRuntimeCfg, SessionEventRx, SessionId, UserId,
    WorkspaceId, build_agent_manager,
};
use uuid::Uuid;

use crate::error::{Error, Result};

static AGENT_MANAGERS: OnceLock<AgentManagers> = OnceLock::new();
static AGENT_RUNTIME_CONFIG: OnceLock<AgentRuntimeCfg> = OnceLock::new();

struct AgentManagers {
    default_name: String,
    by_name: HashMap<String, Arc<AgentManager>>,
}

pub fn init_agent_manager(config: AgentRuntimeCfg) -> Result<()> {
    let default_name = selected_llm_name(&config)?;
    let mut by_name = HashMap::new();

    for llm in &config.llm {
        let manager = build_agent_manager(AgentRuntimeCfg {
            llm: config.llm.clone(),
            default_llm: Some(llm.name.clone()),
        })
        .map_err(|err| Error::Custom(format!("agent init failed for '{}': {err}", llm.name)))?;
        by_name.insert(llm.name.clone(), Arc::new(manager));
    }

    AGENT_MANAGERS
        .set(AgentManagers {
            default_name,
            by_name,
        })
        .map_err(|_| Error::Custom("Agent manager already initialized".to_owned()))?;
    AGENT_RUNTIME_CONFIG
        .set(config)
        .map_err(|_| Error::Custom("Agent runtime config already initialized".to_owned()))?;
    Ok(())
}

pub fn get_agent_runtime_config() -> Result<&'static AgentRuntimeCfg> {
    AGENT_RUNTIME_CONFIG
        .get()
        .ok_or_else(|| Error::Custom("Agent runtime config not initialized".to_owned()))
}

pub fn run_agent_stream(
    user_id: String,
    workspace_id: Option<String>,
    agent_id: Option<String>,
    session_id: Option<String>,
    model_name: Option<String>,
    input: String,
) -> Result<SessionEventRx> {
    let managers = AGENT_MANAGERS
        .get()
        .ok_or_else(|| Error::Custom("Agent manager not initialized".to_owned()))?;
    let manager = resolve_manager(managers, model_name.as_deref())?;
    let session_id = session_id.unwrap_or_else(new_session_id);
    manager
        .run_stream(AgentRequest {
            user_id: Some(UserId(user_id)),
            workspace_id: workspace_id.map(WorkspaceId),
            agent_id: AgentId(agent_id.unwrap_or_else(|| "web-agent".to_string())),
            session_id: SessionId(session_id),
            input,
        })
        .map_err(|err| Error::Custom(format!("agent stream failed: {err}")))
}

fn resolve_manager(managers: &AgentManagers, model_name: Option<&str>) -> Result<Arc<AgentManager>> {
    let selected_name = model_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(&managers.default_name);
    managers
        .by_name
        .get(selected_name)
        .cloned()
        .ok_or_else(|| Error::Custom(format!("llm '{selected_name}' is not configured")))
}

fn selected_llm_name(config: &AgentRuntimeCfg) -> Result<String> {
    if config.llm.is_empty() {
        return Err(Error::Custom(
            "agent runtime requires at least one [[llm]] entry".to_owned(),
        ));
    }

    let selected = config
        .default_llm
        .as_deref()
        .unwrap_or(&config.llm[0].name);
    if config.llm.iter().any(|llm| llm.name == selected) {
        return Ok(selected.to_owned());
    }

    Err(Error::Custom(format!("default llm '{selected}' is not configured")))
}

fn new_session_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("web-agent-{millis}-{}", Uuid::new_v4())
}
