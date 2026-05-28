use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use agent::api::{
    AgentId, AgentLlmUsageCfg, AgentManager, AgentRequest, AgentRuntimeCfg, SessionEventRx,
    SessionId, UserId, VisionPreprocessAgent, VisionPreprocessAgentConfig, VisionPreprocessInput,
    VisionPreprocessResult, WorkspaceId, build_agent_manager,
};
use uuid::Uuid;

use crate::error::{Error, Result};

static AGENT_MANAGER_REGISTRY: OnceLock<AgentManagerRegistry> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentRuntimeModelSelection {
    pub selected_name: String,
    pub model: String,
    pub selection_status: String,
}

struct AgentManagerRegistry {
    config: AgentRuntimeCfg,
    llm_usage: AgentLlmUsageCfg,
    manager: Arc<AgentManager>,
}

pub fn init_agent_manager(config: AgentRuntimeCfg) -> Result<()> {
    let llm_usage = resolve_llm_usage(&config)?;
    tracing::info!(
        usage_default = ?llm_usage.default,
        usage_fast = ?llm_usage.fast,
        usage_thinking = ?llm_usage.thinking,
        usage_vision = ?llm_usage.vision,
        configured_llms = ?config
            .llm
            .iter()
            .map(|llm| format!("{}:{}", llm.name, llm.model))
            .collect::<Vec<_>>(),
        "agent manager llm usage resolved"
    );
    let manager = build_agent_manager(config.clone())
        .map_err(|err| Error::Custom(format!("agent init failed: {err}")))?;
    AGENT_MANAGER_REGISTRY
        .set(AgentManagerRegistry {
            config,
            llm_usage,
            manager: Arc::new(manager),
        })
        .map_err(|_| Error::Custom("Agent manager already initialized".to_owned()))?;
    Ok(())
}

pub fn agent_runtime_llm_usage() -> Result<AgentLlmUsageCfg> {
    let registry = registry()?;
    Ok(registry.llm_usage.clone())
}

pub fn vision_preprocess_config() -> Result<VisionPreprocessAgentConfig> {
    VisionPreprocessAgentConfig::load_default()
        .map_err(|err| Error::Custom(format!("vision preprocess config failed: {err}")))
}

pub async fn run_vision_preprocess(input: VisionPreprocessInput) -> Result<VisionPreprocessResult> {
    let registry = registry()?;
    let config = vision_preprocess_config()?;
    tracing::debug!(
        system_agent_id = %config.id,
        analysis_kind = %config.analysis_kind,
        prompt = %config.prompt,
        prompt_version = %config.prompt_version,
        object_key = %input.object_key,
        mime_type = ?input.mime_type,
        file_size = ?input.file_size,
        "vision preprocess runtime initializing"
    );
    let agent =
        VisionPreprocessAgent::new(config, &registry.config.llm, &registry.config.llm_usage)
            .map_err(|err| Error::Custom(format!("vision preprocess init failed: {err}")))?;
    let result = agent
        .run(input)
        .await
        .map_err(|err| Error::Custom(format!("vision preprocess failed: {err}")))?;
    tracing::debug!(
        analysis_kind = %result.analysis_kind,
        prompt_key = %result.prompt_key,
        prompt_version = %result.prompt_version,
        llm_name = %result.llm_name,
        llm_model = ?result.llm_model,
        summary_chars = result.summary.chars().count(),
        "vision preprocess runtime finished"
    );
    Ok(result)
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
        let selected_name = requested
            .or(self.llm_usage.default.as_deref())
            .ok_or_else(|| Error::Custom("agent default llm usage is not configured".to_owned()))?;
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

fn resolve_llm_usage(config: &AgentRuntimeCfg) -> Result<AgentLlmUsageCfg> {
    let first_llm = config.llm.first().ok_or_else(|| {
        Error::Custom("agent runtime requires at least one [[llm]] entry".to_string())
    })?;
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

    let default =
        resolve(config.llm_usage.default.as_ref())?.or_else(|| Some(first_llm.name.clone()));
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

    use super::resolve_llm_usage;

    fn llm(name: &str, model: &str) -> AgentLlmCfg {
        AgentLlmCfg {
            name: name.to_string(),
            provider: Default::default(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            model: model.to_string(),
            api_key: None,
            max_tokens: None,
            temperature: None,
            reasoning_effort: None,
            thinking: None,
            chat_completions_endpoint: None,
        }
    }

    #[test]
    fn resolves_llm_usage_with_default_fallback() {
        let mut config = AgentRuntimeCfg::from_llm(vec![
            llm("llm-fast", "fast-model"),
            llm("llm-thinking", "thinking-model"),
            llm("llm-vision", "vision-model"),
        ]);
        config.llm_usage = AgentLlmUsageCfg {
            default: Some("llm-thinking".to_string()),
            fast: Some("llm-fast".to_string()),
            vision: Some("llm-vision".to_string()),
            thinking: None,
        };

        let usage = resolve_llm_usage(&config).expect("usage should resolve");

        assert_eq!(usage.default.as_deref(), Some("llm-thinking"));
        assert_eq!(usage.fast.as_deref(), Some("llm-fast"));
        assert_eq!(usage.thinking.as_deref(), Some("llm-thinking"));
        assert_eq!(usage.vision.as_deref(), Some("llm-vision"));
    }

    #[test]
    fn resolves_llm_usage_default_to_first_llm_when_omitted() {
        let mut config = AgentRuntimeCfg::from_llm(vec![
            llm("first-llm", "first-model"),
            llm("second-llm", "second-model"),
        ]);
        config.llm_usage = AgentLlmUsageCfg {
            fast: Some("second-llm".to_string()),
            ..AgentLlmUsageCfg::default()
        };

        let usage = resolve_llm_usage(&config).expect("usage should resolve");

        assert_eq!(usage.default.as_deref(), Some("first-llm"));
        assert_eq!(usage.fast.as_deref(), Some("second-llm"));
        assert_eq!(usage.thinking.as_deref(), Some("first-llm"));
    }
}
