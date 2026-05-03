use std::{
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use agent::api::{
    AgentId, AgentManager, AgentRequest, AgentRuntimeCfg, SessionEventRx, SessionId, UserId,
    WorkspaceId, build_agent_manager,
};
use uuid::Uuid;

use crate::error::{Error, Result};

static AGENT_MANAGER: OnceLock<Arc<AgentManager>> = OnceLock::new();

pub fn init_agent_manager(config: AgentRuntimeCfg) -> Result<()> {
    let manager = build_agent_manager(config)
        .map_err(|err| Error::Custom(format!("agent init failed: {err}")))?;
    AGENT_MANAGER
        .set(Arc::new(manager))
        .map_err(|_| Error::Custom("Agent manager already initialized".to_owned()))?;
    Ok(())
}

pub fn run_agent_stream(
    user_id: String,
    workspace_id: Option<String>,
    agent_id: Option<String>,
    session_id: Option<String>,
    input: String,
) -> Result<SessionEventRx> {
    let manager = AGENT_MANAGER
        .get()
        .ok_or_else(|| Error::Custom("Agent manager not initialized".to_owned()))?;
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

fn new_session_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("web-agent-{millis}-{}", Uuid::new_v4())
}
