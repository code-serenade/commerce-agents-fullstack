use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use agent::api::{
    AgentId, AgentLlmUsageCfg, AgentManager, AgentRequest, AgentRuntimeCfg, SessionEventRx,
    SessionId, UserId, WorkspaceId, build_agent_manager,
};
use uuid::Uuid;

use crate::error::{Error, Result};

static AGENT_MANAGER_REGISTRY: OnceLock<AgentManagerRegistry> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeModelOption {
    pub name: String,
    pub model: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeModelSelection {
    pub selected_name: String,
    pub model: String,
    pub selection_status: String,
}

struct AgentManagerRegistry {
    config: AgentRuntimeCfg,
    default_model_name: String,
    llm_usage: AgentLlmUsageCfg,
    manager: Arc<AgentManager>,
}

pub fn init_agent_manager(config: AgentRuntimeCfg) -> Result<()> {
    let default_model_name = resolve_default_model_name(&config)?;
    let llm_usage = resolve_llm_usage(&config, &default_model_name)?;
    let manager = build_agent_manager(config.clone())
        .map_err(|err| Error::Custom(format!("agent init failed: {err}")))?;
    AGENT_MANAGER_REGISTRY
        .set(AgentManagerRegistry {
            config,
            default_model_name,
            llm_usage,
            manager: Arc::new(manager),
        })
        .map_err(|_| Error::Custom("Agent manager already initialized".to_owned()))?;
    Ok(())
}

pub fn list_agent_runtime_models() -> Result<Vec<AgentRuntimeModelOption>> {
    let registry = registry()?;
    Ok(registry
        .config
        .llm
        .iter()
        .map(|llm| AgentRuntimeModelOption {
            name: llm.name.clone(),
            model: llm.model.clone(),
            is_default: llm.name == registry.default_model_name,
        })
        .collect())
}

pub fn select_agent_runtime_model(
    requested_model_name: Option<&str>,
) -> Result<AgentRuntimeModelSelection> {
    let registry = registry()?;
    registry.select_model(requested_model_name)
}

pub fn agent_runtime_llm_usage() -> Result<AgentLlmUsageCfg> {
    let registry = registry()?;
    Ok(registry.llm_usage.clone())
}

pub fn run_agent_stream(
    user_id: String,
    workspace_id: Option<String>,
    agent_id: Option<String>,
    session_id: Option<String>,
    input: String,
    mut metadata: BTreeMap<String, String>,
) -> Result<SessionEventRx> {
    let registry = registry()?;
    let selection = registry.select_model(None)?;
    metadata.insert("model.name".to_string(), selection.selected_name);
    metadata.insert("model.model".to_string(), selection.model);
    metadata.insert(
        "model.selection_status".to_string(),
        selection.selection_status,
    );
    let session_id = session_id.unwrap_or_else(new_session_id);
    registry
        .manager
        .run_stream(AgentRequest {
            user_id: Some(UserId(user_id)),
            workspace_id: workspace_id.map(WorkspaceId),
            agent_id: AgentId(agent_id.unwrap_or_else(|| "web-agent".to_string())),
            session_id: SessionId(session_id),
            input,
            metadata,
        })
        .map_err(|err| Error::Custom(format!("agent stream failed: {err}")))
}

impl AgentManagerRegistry {
    fn select_model(
        &self,
        requested_model_name: Option<&str>,
    ) -> Result<AgentRuntimeModelSelection> {
        let requested = requested_model_name
            .map(str::trim)
            .filter(|name| !name.is_empty());
        let selected_name = requested.unwrap_or(&self.default_model_name);
        let llm = self
            .config
            .llm
            .iter()
            .find(|llm| llm.name == selected_name)
            .ok_or_else(|| {
                Error::Custom(format!("llm model '{selected_name}' is not configured"))
            })?;
        Ok(AgentRuntimeModelSelection {
            selected_name: llm.name.clone(),
            model: llm.model.clone(),
            selection_status: if requested.is_some() {
                "requested".to_string()
            } else {
                "default".to_string()
            },
        })
    }
}

fn registry() -> Result<&'static AgentManagerRegistry> {
    AGENT_MANAGER_REGISTRY
        .get()
        .ok_or_else(|| Error::Custom("Agent manager not initialized".to_owned()))
}

fn resolve_default_model_name(config: &AgentRuntimeCfg) -> Result<String> {
    if config.llm.is_empty() {
        return Err(Error::Custom(
            "agent runtime requires at least one [[llm]] entry".to_string(),
        ));
    }
    match config.default_llm.as_deref() {
        Some(name) if config.llm.iter().any(|llm| llm.name == name) => Ok(name.to_string()),
        Some(name) => Err(Error::Custom(format!("llm '{name}' not found"))),
        None => Ok(config
            .llm
            .iter()
            .find(|llm| {
                llm.name.to_ascii_lowercase().contains("doubao")
                    || llm.model.to_ascii_lowercase().contains("doubao")
            })
            .or_else(|| config.llm.first())
            .expect("checked non-empty llm config above")
            .name
            .clone()),
    }
}

fn resolve_llm_usage(
    config: &AgentRuntimeCfg,
    default_model_name: &str,
) -> Result<AgentLlmUsageCfg> {
    let resolve = |value: Option<&String>| -> Result<Option<String>> {
        let Some(name) = value
            .map(String::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
        else {
            return Ok(None);
        };
        if config.llm.iter().any(|llm| llm.name == name) {
            return Ok(Some(name.to_string()));
        }
        Err(Error::Custom(format!("llm '{name}' not found")))
    };

    let default = resolve(config.llm_usage.default.as_ref())?
        .or_else(|| Some(default_model_name.to_string()));
    let fast = resolve(config.llm_usage.fast.as_ref())?.or_else(|| default.clone());
    let thinking = resolve(config.llm_usage.thinking.as_ref())?.or_else(|| default.clone());
    let vision = resolve(config.llm_usage.vision.as_ref())?;

    Ok(AgentLlmUsageCfg {
        default,
        fast,
        thinking,
        vision,
    })
}

fn new_session_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("web-agent-{millis}-{}", Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use agent::api::{AgentLlmCfg, AgentLlmUsageCfg, AgentRuntimeCfg};

    use super::{resolve_default_model_name, resolve_llm_usage};

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
    fn default_model_prefers_configured_default_llm() {
        let mut config = AgentRuntimeCfg::from_llm(vec![
            llm("doubao-external", "doubao-seed-2-0-pro-260215"),
            llm("ollama-gpt", "gpt-oss:20b"),
        ]);
        config.default_llm = Some("ollama-gpt".to_string());

        assert_eq!(
            resolve_default_model_name(&config).expect("default model"),
            "ollama-gpt"
        );
    }

    #[test]
    fn default_model_prefers_doubao_when_no_default_is_set() {
        let config = AgentRuntimeCfg::from_llm(vec![
            llm("ollama-gpt", "gpt-oss:20b"),
            llm("doubao-external", "doubao-seed-2-0-pro-260215"),
        ]);

        assert_eq!(
            resolve_default_model_name(&config).expect("default model"),
            "doubao-external"
        );
    }

    #[test]
    fn resolves_llm_usage_with_default_fallback() {
        let mut config = AgentRuntimeCfg::from_llm(vec![
            llm("llm-fast", "fast-model"),
            llm("llm-thinking", "thinking-model"),
            llm("llm-vision", "vision-model"),
        ]);
        config.default_llm = Some("llm-thinking".to_string());
        config.llm_usage = AgentLlmUsageCfg {
            fast: Some("llm-fast".to_string()),
            vision: Some("llm-vision".to_string()),
            ..AgentLlmUsageCfg::default()
        };

        let usage = resolve_llm_usage(&config, "llm-thinking").expect("usage should resolve");

        assert_eq!(usage.default.as_deref(), Some("llm-thinking"));
        assert_eq!(usage.fast.as_deref(), Some("llm-fast"));
        assert_eq!(usage.thinking.as_deref(), Some("llm-thinking"));
        assert_eq!(usage.vision.as_deref(), Some("llm-vision"));
    }
}
