use std::{collections::BTreeMap, sync::Arc};

use crate::{
    MemoryQuery, MemoryScope, MemoryStore, SessionEventRx, SessionResult, SessionRuntime,
    SessionRuntimeConfig, SessionRuntimeInput, SessionSnapshot,
    agent::commander::plan_configured_non_chat_task_graph,
    agent_error::Result,
    cognition::Cognition,
    core::protocol::{
        AgentId, MessageContext, SessionEvent, SessionId, TaskId, UserId, WorkspaceId,
    },
    roles::{RoleCatalog, RoleProfile, RoleRouteInput, RoleRouter},
};

pub type CognitionFactory =
    Arc<dyn Fn() -> Result<Box<dyn Cognition + Send>> + Send + Sync + 'static>;
pub type RoleCognitionFactory =
    Arc<dyn Fn(&RoleProfile) -> Result<Box<dyn Cognition + Send>> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AgentSessionConfig {
    pub runtime: SessionRuntimeConfig,
    pub roles: RoleCatalog,
    pub commander_cognition: CognitionFactory,
    pub worker_cognition: RoleCognitionFactory,
    pub memory_store: Arc<dyn MemoryStore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionScope {
    pub user_id: Option<UserId>,
    pub workspace_id: Option<WorkspaceId>,
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub metadata: BTreeMap<String, String>,
}

pub struct AgentSession {
    scope: AgentSessionScope,
    config: AgentSessionConfig,
}

impl AgentSession {
    pub fn new(scope: AgentSessionScope, config: AgentSessionConfig) -> Self {
        Self { scope, config }
    }

    pub async fn run_once(&self, input: impl Into<String>) -> Result<SessionResult> {
        self.run_once_with_events(input, None).await
    }

    pub fn run_stream(self, input: impl Into<String>) -> Result<SessionEventRx> {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(self.config.runtime.event_buffer);
        let input = input.into();
        let session_id = self.scope.session_id.clone();

        tokio::spawn(async move {
            let result = self
                .run_once_with_events(input, Some(event_tx.clone()))
                .await;
            match result {
                Ok(_) => {
                    let _ = event_tx.send(SessionEvent::Finished { session_id }).await;
                }
                Err(err) => {
                    let _ = event_tx
                        .send(SessionEvent::Failed {
                            session_id,
                            reason: err.to_string(),
                        })
                        .await;
                    tracing::warn!(error = %err, "streaming agent session failed");
                }
            }
        });

        Ok(event_rx)
    }

    pub(crate) async fn run_once_with_events(
        &self,
        input: impl Into<String>,
        event_tx: Option<tokio::sync::mpsc::Sender<crate::core::protocol::SessionEvent>>,
    ) -> Result<SessionResult> {
        let event_tx_for_persist = event_tx.clone();
        let input = input.into();
        let commander_cognition = (self.config.commander_cognition)()?;
        let selected_roles = select_roles_for_task(
            &self.config.roles,
            &self.config.runtime,
            &input,
            &self.scope.metadata,
        );
        let workers = selected_roles
            .iter()
            .map(|role| {
                Ok(crate::agent_session::SessionRuntimeWorkerInput {
                    role: (*role).clone(),
                    cognition: (self.config.worker_cognition)(role)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let context = self.message_context();
        let mut memory_scope = self.memory_scope(None);
        memory_scope.session_id = None;
        let memory_bundle = self
            .config
            .memory_store
            .load_bundle(MemoryQuery::scoped(memory_scope));

        let mut runtime = SessionRuntime::start(SessionRuntimeInput {
            session_id: self.scope.session_id.clone(),
            context,
            task_description: input,
            memory_bundle,
            commander_cognition,
            workers,
            config: self.config.runtime.clone(),
            event_tx,
        })?;

        let result = runtime.run_until_complete().await;
        let blackboard = runtime.session_context().blackboard().snapshot();
        runtime.shutdown().await;
        let memory_outcome = self.persist_if_ok(&result, blackboard);
        if memory_outcome.written > 0
            && let Some(event_tx) = event_tx_for_persist
            && let Ok(result) = &result
        {
            if memory_outcome.compaction.has_activity() {
                let stats = &memory_outcome.compaction;
                let _ = event_tx
                    .send(SessionEvent::MemoryCompacted {
                        session_id: result.session_id.clone(),
                        input_records: stats.input_records,
                        output_records: stats.output_records,
                        truncated_records: stats.truncated_records,
                        merged_records: stats.merged_records,
                        dropped_records: stats.dropped_records,
                        before_chars: stats.before_chars,
                        after_chars: stats.after_chars,
                    })
                    .await;
            }
            let _ = event_tx
                .send(SessionEvent::MemoryPersisted {
                    session_id: result.session_id.clone(),
                    count: memory_outcome.written,
                })
                .await;
        }
        result
    }

    pub fn scope(&self) -> &AgentSessionScope {
        &self.scope
    }

    fn message_context(&self) -> MessageContext {
        MessageContext {
            user_id: self.scope.user_id.clone(),
            workspace_id: self.scope.workspace_id.clone(),
            agent_id: Some(self.scope.agent_id.clone()),
            session_id: Some(self.scope.session_id.clone()),
            task_id: None,
            metadata: self.scope.metadata.clone(),
        }
    }

    fn memory_scope(&self, session_id: Option<SessionId>) -> MemoryScope {
        MemoryScope {
            user_id: self.scope.user_id.clone(),
            workspace_id: self.scope.workspace_id.clone(),
            agent_id: Some(self.scope.agent_id.clone()),
            session_id: Some(session_id.unwrap_or_else(|| self.scope.session_id.clone())),
            task_id: None,
        }
    }

    fn persist_if_ok(
        &self,
        result: &Result<SessionResult>,
        blackboard: std::collections::HashMap<String, String>,
    ) -> crate::MemoryWriteOutcome {
        if let Ok(result) = result {
            return self.config.memory_store.persist_session(
                &self.memory_scope(Some(result.session_id.clone())),
                SessionSnapshot {
                    summary: result.output.clone(),
                    final_output: result.output.clone(),
                    blackboard: blackboard.into_iter().collect::<BTreeMap<_, _>>(),
                    task_graphs: result.task_graphs.clone(),
                    role_summaries: result.role_summaries.clone(),
                    worker_details: result.worker_details.clone(),
                    risks: result.risks.clone(),
                    open_questions: result.open_questions.clone(),
                    evaluation_summary: result.evaluation_summary.clone(),
                },
            );
        }
        crate::MemoryWriteOutcome::default()
    }
}

fn select_roles_for_task<'a>(
    catalog: &'a RoleCatalog,
    runtime: &SessionRuntimeConfig,
    input: &str,
    metadata: &BTreeMap<String, String>,
) -> Vec<&'a RoleProfile> {
    let mut runtime_roles = Vec::new();
    runtime_roles.extend(selected_runtime_roles(metadata));

    let role_router = RoleRouter::new(catalog.clone());
    let role_route = role_router.route(RoleRouteInput {
        message: input.to_string(),
        domain_id: Some("domain.general".to_string()),
        action: None,
        max_roles: auto_route_max_roles(metadata),
    });

    if runtime_roles.is_empty() && runtime.commander_fast_route {
        let task_id = TaskId("preselect".to_string());
        if let Some(graph) = plan_configured_non_chat_task_graph(&task_id, input, Some(&role_route))
        {
            runtime_roles.extend(graph.nodes.into_iter().map(|node| node.role));
        }
    }

    if runtime_roles.is_empty() {
        runtime_roles.push(role_route.primary_runtime_role);
        runtime_roles.extend(role_route.support_runtime_roles);
    }
    if !runtime_roles.iter().any(|role| role == "chat") {
        runtime_roles.push("chat".to_string());
    }

    let mut selected = Vec::new();
    for runtime_role in runtime_roles {
        if selected
            .iter()
            .any(|role: &&RoleProfile| role.runtime_role == runtime_role)
        {
            continue;
        }
        if let Some(role) = catalog
            .roles()
            .iter()
            .find(|role| role.runtime_role == runtime_role)
        {
            selected.push(role);
        }
    }
    if selected.is_empty()
        && let Some(role) = catalog.roles().first()
    {
        selected.push(role);
    }
    selected
}

fn selected_runtime_roles(metadata: &BTreeMap<String, String>) -> Vec<String> {
    let collaboration_mode = metadata
        .get("strategy.collaboration_mode")
        .map(String::as_str)
        .unwrap_or_default();
    if !matches!(collaboration_mode, "manual" | "single") {
        return Vec::new();
    }
    let roles = metadata
        .get("strategy.selected_roles")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let requested_primary = metadata
        .get("strategy.primary_role")
        .cloned()
        .filter(|role| roles.iter().any(|item| item == role));

    normalize_requested_runtime_roles(collaboration_mode, roles, requested_primary)
}

fn auto_route_max_roles(metadata: &BTreeMap<String, String>) -> usize {
    match runtime_response_mode(metadata) {
        "quick" => 1,
        "execution" => 4,
        _ => 3,
    }
}

fn runtime_response_mode(metadata: &BTreeMap<String, String>) -> &str {
    if let Some(response_mode) = metadata
        .get("strategy.response_mode")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
    {
        return response_mode;
    }
    if let Some(strategy_id) = metadata
        .get("strategy.id")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
    {
        return strategy_id;
    }
    if metadata
        .get("strategy.id")
        .is_some_and(|value| value == "collab")
    {
        return "execution";
    }
    "learning"
}

fn normalize_requested_runtime_roles(
    collaboration_mode: &str,
    roles: Vec<String>,
    requested_primary: Option<String>,
) -> Vec<String> {
    let roles = unique_runtime_roles(roles);
    if roles.is_empty() {
        return Vec::new();
    }

    let non_chat_roles = roles
        .iter()
        .filter(|role| role.as_str() != "chat")
        .cloned()
        .collect::<Vec<_>>();
    let primary = requested_primary
        .filter(|role| role != "chat")
        .or_else(|| {
            non_chat_roles
                .iter()
                .find(|role| role.as_str() == "ops")
                .cloned()
        })
        .or_else(|| non_chat_roles.first().cloned())
        .or_else(|| roles.first().cloned());

    let Some(primary) = primary else {
        return Vec::new();
    };

    let mut selected = vec![primary.clone()];
    if collaboration_mode == "manual" {
        selected.extend(
            roles
                .into_iter()
                .filter(|role| role != "chat" && role != &primary),
        );
    }
    selected
}

fn unique_runtime_roles(roles: Vec<String>) -> Vec<String> {
    roles.into_iter().fold(Vec::new(), |mut unique, role| {
        let role = role.trim().to_string();
        if !role.is_empty() && !unique.contains(&role) {
            unique.push(role);
        }
        unique
    })
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, time::Duration};

    use super::select_roles_for_task;
    use crate::{RoleCatalog, SessionRuntimeConfig};

    #[test]
    fn selects_only_roles_needed_by_task_graph() {
        let roles = RoleCatalog::builtin();
        let selected = select_roles_for_task(
            &roles,
            &SessionRuntimeConfig {
                response_timeout: Duration::from_secs(300),
                event_buffer: 1024,
                commander_fast_route: true,
                commander_fast_route_min_score: 2.0,
                worker_cognition_timeout: Duration::from_secs(45),
                search: Default::default(),
            },
            "我是做咖啡运营的，主要营销平台是抖音，现在马上就五一了，请给提升转化率的方案",
            &BTreeMap::new(),
        )
        .into_iter()
        .map(|role| role.runtime_role.clone())
        .collect::<Vec<_>>();

        assert!(selected.contains(&"ops".to_string()));
        assert!(selected.contains(&"chat".to_string()));
        assert!(!selected.contains(&"engineering".to_string()));
    }

    #[test]
    fn selects_manual_roles_from_request_metadata() {
        let roles = RoleCatalog::builtin();
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "strategy.collaboration_mode".to_string(),
            "manual".to_string(),
        );
        metadata.insert("strategy.primary_role".to_string(), "creative".to_string());
        metadata.insert(
            "strategy.selected_roles".to_string(),
            "data,creative,ops".to_string(),
        );

        let selected = select_roles_for_task(
            &roles,
            &SessionRuntimeConfig::default(),
            "请协同分析直播素材和转化问题",
            &metadata,
        )
        .into_iter()
        .map(|role| role.runtime_role.clone())
        .collect::<Vec<_>>();

        assert_eq!(selected[0], "creative");
        assert!(selected.contains(&"data".to_string()));
        assert!(selected.contains(&"ops".to_string()));
        assert!(selected.contains(&"chat".to_string()));
    }

    #[test]
    fn manual_multi_role_selection_promotes_ops_when_primary_is_chat() {
        let roles = RoleCatalog::builtin();
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "strategy.collaboration_mode".to_string(),
            "manual".to_string(),
        );
        metadata.insert("strategy.primary_role".to_string(), "chat".to_string());
        metadata.insert(
            "strategy.selected_roles".to_string(),
            "chat,ops,data,service,creative,engineering".to_string(),
        );

        let selected = select_roles_for_task(
            &roles,
            &SessionRuntimeConfig::default(),
            "请分析咖啡类商品",
            &metadata,
        )
        .into_iter()
        .map(|role| role.runtime_role.clone())
        .collect::<Vec<_>>();

        assert_eq!(selected[0], "ops");
        assert!(selected.contains(&"data".to_string()));
        assert!(selected.contains(&"service".to_string()));
        assert!(selected.contains(&"creative".to_string()));
        assert!(selected.contains(&"engineering".to_string()));
        assert_eq!(
            selected
                .iter()
                .filter(|role| role.as_str() == "chat")
                .count(),
            1
        );
    }

    #[test]
    fn selects_single_role_from_request_metadata() {
        let roles = RoleCatalog::builtin();
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "strategy.collaboration_mode".to_string(),
            "single".to_string(),
        );
        metadata.insert("strategy.primary_role".to_string(), "data".to_string());
        metadata.insert(
            "strategy.selected_roles".to_string(),
            "data,creative,ops".to_string(),
        );

        let selected = select_roles_for_task(
            &roles,
            &SessionRuntimeConfig::default(),
            "请只让数据角色分析漏斗",
            &metadata,
        )
        .into_iter()
        .map(|role| role.runtime_role.clone())
        .collect::<Vec<_>>();

        assert!(selected.contains(&"data".to_string()));
        assert!(selected.contains(&"chat".to_string()));
        assert!(!selected.contains(&"creative".to_string()));
        assert!(!selected.contains(&"ops".to_string()));
    }
}
