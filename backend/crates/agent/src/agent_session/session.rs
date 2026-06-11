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
        let worker_roles = session_worker_roles(&self.config.roles, selected_roles);
        let workers = worker_roles
            .iter()
            .map(|(role, routable)| {
                Ok(crate::agent_session::SessionRuntimeWorkerInput {
                    role: (*role).clone(),
                    routable: *routable,
                    cognition: (self.config.worker_cognition)(role)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let context = self.message_context();
        let mut memory_scope = self.memory_scope(None);
        memory_scope.session_id = None;
        let memory_bundle = self.config.memory_store.load_bundle(
            MemoryQuery::scoped(memory_scope)
                .for_text(agent_memory_query_text(&input, &self.scope.metadata)),
        );

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
    _runtime: &SessionRuntimeConfig,
    input: &str,
    metadata: &BTreeMap<String, String>,
) -> Vec<&'a RoleProfile> {
    let collaboration_mode = metadata
        .get("strategy.collaboration_mode")
        .map(String::as_str)
        .unwrap_or_default();
    let mut runtime_roles = Vec::new();
    runtime_roles.extend(selected_runtime_roles(metadata));

    let role_router = RoleRouter::new(catalog.clone());
    let role_route = role_router.route(RoleRouteInput {
        message: input.to_string(),
        domain_id: Some("domain.general".to_string()),
        action: None,
        max_roles: auto_route_max_roles(metadata),
    });

    let task_id = TaskId("preselect".to_string());
    let configured_graph_roles =
        plan_configured_non_chat_task_graph(&task_id, input, Some(&role_route))
            .map(|graph| graph.nodes.into_iter().map(|node| node.role).collect())
            .unwrap_or_default();

    if runtime_roles.is_empty() {
        runtime_roles.extend(configured_graph_roles);
    } else if collaboration_mode == "manual" {
        append_missing_runtime_roles(&mut runtime_roles, configured_graph_roles);
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

fn session_worker_roles<'a>(
    catalog: &'a RoleCatalog,
    selected_roles: Vec<&'a RoleProfile>,
) -> Vec<(&'a RoleProfile, bool)> {
    let mut roles: Vec<(&'a RoleProfile, bool)> = Vec::new();
    for role in selected_roles {
        if roles
            .iter()
            .any(|(selected, _)| selected.runtime_role == role.runtime_role)
        {
            continue;
        }
        roles.push((role, true));
    }
    for role in catalog.roles() {
        if roles
            .iter()
            .any(|(selected, _)| selected.runtime_role == role.runtime_role)
        {
            continue;
        }
        roles.push((role, false));
    }
    if roles.is_empty()
        && let Some(role) = catalog.roles().first()
    {
        roles.push((role, true));
    }
    roles
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

fn append_missing_runtime_roles(runtime_roles: &mut Vec<String>, additional_roles: Vec<String>) {
    for role in additional_roles {
        let role = role.trim().to_string();
        if !role.is_empty() && !runtime_roles.contains(&role) {
            runtime_roles.push(role);
        }
    }
}

pub fn agent_memory_query_text(input: &str, metadata: &BTreeMap<String, String>) -> String {
    const MAX_METADATA_CHARS: usize = 1_200;

    let mut parts = Vec::new();
    push_unique_query_part(&mut parts, input);
    for key in [
        "context.product_ids",
        "request.context.product_ids",
        "context.productIds",
        "request.context.productIds",
        "context.product_id",
        "request.context.product_id",
        "context.productId",
        "request.context.productId",
        "rag.product_ids",
        "request.rag.product_ids",
        "rag.productIds",
        "request.rag.productIds",
        "rag.product_id",
        "request.rag.product_id",
        "rag.productId",
        "request.rag.productId",
    ] {
        if let Some(value) = metadata.get(key) {
            push_unique_query_part(&mut parts, value);
            push_context_ref_query_parts(&mut parts, value, "product");
        }
    }
    for key in [
        "context.campaign_ids",
        "request.context.campaign_ids",
        "context.campaignIds",
        "request.context.campaignIds",
        "context.campaign_id",
        "request.context.campaign_id",
        "context.campaignId",
        "request.context.campaignId",
    ] {
        if let Some(value) = metadata.get(key) {
            push_unique_query_part(&mut parts, value);
            push_context_ref_query_parts(&mut parts, value, "campaign");
        }
    }
    for key in [
        "context.refs",
        "request.context.refs",
        "context.context_refs",
        "request.context.context_refs",
        "context.contextRefs",
        "request.context.contextRefs",
        "context.combined_refs",
        "request.context.combined_refs",
        "context.combinedRefs",
        "request.context.combinedRefs",
        "context.source_titles",
        "request.context.source_titles",
        "context.sourceTitles",
        "request.context.sourceTitles",
        "context.summary",
        "request.context.summary",
        "context.selected_summary",
        "request.context.selected_summary",
        "context.selectedSummary",
        "request.context.selectedSummary",
        "context.combined_summary",
        "request.context.combined_summary",
        "context.combinedSummary",
        "request.context.combinedSummary",
        "rag.query_terms",
        "request.rag.query_terms",
        "rag.queryTerms",
        "request.rag.queryTerms",
        "rag.summary",
        "request.rag.summary",
        "rag.source_titles",
        "request.rag.source_titles",
        "rag.sourceTitles",
        "request.rag.sourceTitles",
        "conversation.history_summary",
        "request.conversation.history_summary",
        "conversation.historySummary",
        "request.conversation.historySummary",
        "conversation.history_older_summary",
        "request.conversation.history_older_summary",
        "conversation.historyOlderSummary",
        "request.conversation.historyOlderSummary",
        "workspaceContext.refs",
        "request.workspaceContext.refs",
        "workspaceContext.summary",
        "request.workspaceContext.summary",
        "workspaceContext.source_titles",
        "request.workspaceContext.source_titles",
        "workspaceContext.sourceTitles",
        "request.workspaceContext.sourceTitles",
        "qualityLearning.learningSummary",
        "request.qualityLearning.learningSummary",
        "workspaceContextPack.refs",
        "request.workspaceContextPack.refs",
        "workspaceContextPack.contentRefs",
        "request.workspaceContextPack.contentRefs",
        "workspaceContextPack.summary",
        "request.workspaceContextPack.summary",
        "attachments.image_refs",
        "request.attachments.image_refs",
        "attachments.imageRefs",
        "request.attachments.imageRefs",
        "attachments.image_titles",
        "request.attachments.image_titles",
        "attachments.imageTitles",
        "request.attachments.imageTitles",
        "attachments.ocr_text",
        "request.attachments.ocr_text",
        "attachments.ocrText",
        "request.attachments.ocrText",
        "context.ocr_text",
        "request.context.ocr_text",
        "context.ocrText",
        "request.context.ocrText",
        "request.ocr_text",
        "request.ocrText",
        "context.vision_summary",
        "request.context.vision_summary",
        "context.visionSummary",
        "request.context.visionSummary",
        "context.image_notes",
        "request.context.image_notes",
        "context.imageNotes",
        "request.context.imageNotes",
        "request.image_notes",
        "request.imageNotes",
    ] {
        if let Some(value) = metadata.get(key) {
            push_unique_query_part(&mut parts, value);
        }
    }
    for key in [
        "attachments.image_summaries_json",
        "request.attachments.image_summaries_json",
        "attachments.imageSummariesJson",
        "request.attachments.imageSummariesJson",
        "context.image_summaries_json",
        "request.context.image_summaries_json",
        "context.imageSummariesJson",
        "request.context.imageSummariesJson",
    ] {
        if let Some(value) = metadata.get(key) {
            push_image_summary_query_parts(&mut parts, value);
        }
    }
    for key in [
        "context.source_summaries",
        "request.context.source_summaries",
        "context.sourceSummaries",
        "request.context.sourceSummaries",
        "rag.source_summaries",
        "request.rag.source_summaries",
        "rag.sourceSummaries",
        "request.rag.sourceSummaries",
        "workspaceContext.source_summaries",
        "request.workspaceContext.source_summaries",
        "workspaceContext.sourceSummaries",
        "request.workspaceContext.sourceSummaries",
        "workspaceContextPack.source_summaries",
        "request.workspaceContextPack.source_summaries",
        "workspaceContextPack.sourceSummaries",
        "request.workspaceContextPack.sourceSummaries",
    ] {
        if let Some(value) = metadata.get(key) {
            push_source_summary_query_parts(&mut parts, value);
        }
    }
    for key in [
        "context.sources_json",
        "request.context.sources_json",
        "context.sourcesJson",
        "request.context.sourcesJson",
        "rag.sources_json",
        "request.rag.sources_json",
        "rag.sourcesJson",
        "request.rag.sourcesJson",
        "workspaceContext.sources_json",
        "request.workspaceContext.sources_json",
        "workspaceContext.sourcesJson",
        "request.workspaceContext.sourcesJson",
    ] {
        if let Some(value) = metadata.get(key) {
            push_source_index_query_parts(&mut parts, value);
        }
    }
    for key in [
        "conversation.memory_summary_json",
        "request.conversation.memory_summary_json",
        "conversation.memorySummaryJson",
        "request.conversation.memorySummaryJson",
    ] {
        if let Some(value) = metadata.get(key) {
            push_conversation_memory_summary_query_parts(&mut parts, value);
        }
    }

    let mut query = String::new();
    let mut metadata_chars = 0usize;
    for (index, part) in parts.into_iter().enumerate() {
        if index > 0 {
            query.push('\n');
        }
        if index == 0 {
            query.push_str(&part);
            continue;
        }
        let remaining = MAX_METADATA_CHARS.saturating_sub(metadata_chars);
        if remaining == 0 {
            break;
        }
        let part = truncate_query_part(&part, remaining);
        metadata_chars += part.chars().count();
        query.push_str(&part);
    }
    query
}

fn push_context_ref_query_parts(parts: &mut Vec<String>, value: &str, target_prefix: &str) {
    value
        .split(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    ',' | '，' | ';' | '；' | '/' | '\\' | '|' | '(' | ')' | '[' | ']' | '{' | '}'
                )
        })
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .filter_map(|id| normalize_metadata_context_ref_from_id(id, target_prefix))
        .for_each(|context_ref| push_unique_query_part(parts, &context_ref));
}

fn normalize_metadata_context_ref_from_id(id: &str, target_prefix: &str) -> Option<String> {
    let id = if let Some((id_prefix, id)) = id.split_once(':') {
        if !id_prefix.eq_ignore_ascii_case(target_prefix) {
            return None;
        }
        id
    } else {
        id
    };
    if !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        Some(format!(
            "{}:{}",
            target_prefix.to_ascii_lowercase(),
            id.to_ascii_lowercase()
        ))
    } else {
        None
    }
}

fn push_image_summary_query_parts(parts: &mut Vec<String>, value: &str) {
    const MAX_IMAGE_SUMMARY_ITEMS: usize = 4;
    const MAX_IMAGE_SUMMARY_CHARS: usize = 240;

    let Ok(value) = serde_json::from_str::<serde_json::Value>(value) else {
        return;
    };
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![value],
        _ => return,
    };
    for item in items.iter().take(MAX_IMAGE_SUMMARY_ITEMS) {
        let source_ref = image_summary_text_field(item, &["sourceRef", "source_ref"]);
        let summary = image_summary_text_field(item, &["agentContext", "agent_context"])
            .or_else(|| image_summary_text_field(item, &["summary"]));
        let mut query_part = String::new();
        if let Some(source_ref) = source_ref {
            query_part.push_str(source_ref);
        }
        if let Some(summary) = summary {
            if !query_part.is_empty() {
                query_part.push(' ');
            }
            query_part.push_str(&truncate_query_part(summary, MAX_IMAGE_SUMMARY_CHARS));
        }
        push_unique_query_part(parts, &query_part);
    }
}

fn image_summary_text_field<'a>(item: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        item.get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn push_source_summary_query_parts(parts: &mut Vec<String>, value: &str) {
    const MAX_SOURCE_SUMMARY_ITEMS: usize = 6;
    const MAX_SOURCE_SUMMARY_CHARS: usize = 220;

    let Ok(value) = serde_json::from_str::<serde_json::Value>(value) else {
        push_unique_query_part(parts, value);
        return;
    };
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![value],
        serde_json::Value::String(text) => {
            push_unique_query_part(parts, &text);
            return;
        }
        _ => return,
    };
    for item in items.iter().take(MAX_SOURCE_SUMMARY_ITEMS) {
        let source_ref = summary_text_field(
            item,
            &[
                "ref",
                "sourceRef",
                "source_ref",
                "id",
                "sourceId",
                "source_id",
                "fileId",
                "file_id",
            ],
        );
        let product_id = summary_text_field(item, &["product_id", "productId"]);
        let title = summary_text_field(item, &["title", "name", "fileName", "file_name"]);
        let text = summary_text_field(item, &["summary", "text", "content", "ocr_text", "ocrText"]);
        let mut query_part = String::new();
        for value in [source_ref, product_id, title, text].into_iter().flatten() {
            if !query_part.is_empty() {
                query_part.push(' ');
            }
            query_part.push_str(&truncate_query_part(value, MAX_SOURCE_SUMMARY_CHARS));
        }
        push_unique_query_part(parts, &query_part);
    }
}

fn summary_text_field<'a>(item: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        item.get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn push_source_index_query_parts(parts: &mut Vec<String>, value: &str) {
    const MAX_SOURCE_INDEX_ITEMS: usize = 8;
    const MAX_SOURCE_INDEX_FIELD_CHARS: usize = 120;

    let Ok(value) = serde_json::from_str::<serde_json::Value>(value) else {
        return;
    };
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![value],
        _ => return,
    };
    for item in items.iter().take(MAX_SOURCE_INDEX_ITEMS) {
        let fields = [
            summary_text_field(item, &["source_type", "sourceType", "type"]),
            summary_text_field(item, &["id", "source_id", "sourceId"]),
            summary_text_field(
                item,
                &["ref_key", "refKey", "ref", "sourceRef", "source_ref"],
            ),
            summary_text_field(item, &["product_id", "productId"]),
            summary_text_field(item, &["title", "name", "fileName", "file_name"]),
        ];
        let mut query_part = String::new();
        for value in fields.into_iter().flatten() {
            if !query_part.is_empty() {
                query_part.push(' ');
            }
            query_part.push_str(&truncate_query_part(value, MAX_SOURCE_INDEX_FIELD_CHARS));
        }
        push_unique_query_part(parts, &query_part);
    }
}

fn push_conversation_memory_summary_query_parts(parts: &mut Vec<String>, value: &str) {
    const MAX_CONVERSATION_MEMORY_SECTION_ITEMS: usize = 4;
    const MAX_CONVERSATION_MEMORY_SUMMARY_CHARS: usize = 360;
    const MAX_CONVERSATION_MEMORY_ITEM_CHARS: usize = 200;

    let Ok(value) = serde_json::from_str::<serde_json::Value>(value) else {
        return;
    };
    if let serde_json::Value::String(text) = &value {
        push_unique_query_part(
            parts,
            &truncate_query_part(text, MAX_CONVERSATION_MEMORY_SUMMARY_CHARS),
        );
        return;
    }
    if !value.is_object() {
        return;
    }
    if let Some(summary) = summary_text_field(&value, &["rollingSummary", "rolling_summary"]) {
        push_unique_query_part(
            parts,
            &truncate_query_part(summary, MAX_CONVERSATION_MEMORY_SUMMARY_CHARS),
        );
    }
    for section_keys in [
        &["facts"][..],
        &["decisions"][..],
        &["constraints"][..],
        &["openQuestions", "open_questions"][..],
    ] {
        let Some(items) = section_keys
            .iter()
            .find_map(|key| value.get(*key).and_then(serde_json::Value::as_array))
        else {
            continue;
        };
        for item in items.iter().take(MAX_CONVERSATION_MEMORY_SECTION_ITEMS) {
            if let Some(text) = conversation_memory_item_text(item) {
                push_unique_query_part(
                    parts,
                    &truncate_query_part(text, MAX_CONVERSATION_MEMORY_ITEM_CHARS),
                );
            }
        }
    }
}

fn conversation_memory_item_text(item: &serde_json::Value) -> Option<&str> {
    match item {
        serde_json::Value::String(text) => {
            let text = text.trim();
            (!text.is_empty()).then_some(text)
        }
        serde_json::Value::Object(_) => {
            summary_text_field(item, &["text", "summary", "content", "question"])
        }
        _ => None,
    }
}

fn push_unique_query_part(parts: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if !value.is_empty() && !parts.iter().any(|part| part == value) {
        parts.push(value.to_string());
    }
}

fn truncate_query_part(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, time::Duration};

    use super::{agent_memory_query_text, select_roles_for_task, session_worker_roles};
    use crate::{RoleCatalog, SessionRuntimeConfig};

    #[test]
    fn memory_query_text_includes_explicit_context_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "context.product_ids".to_string(),
            "product:thermos-001".to_string(),
        );
        metadata.insert(
            "context.refs".to_string(),
            "product:thermos-001,campaign:live-001".to_string(),
        );
        metadata.insert(
            "context.selected_summary".to_string(),
            "便携保温杯正在做直播复盘，重点看成交承接。".to_string(),
        );

        let query = agent_memory_query_text("继续复盘", &metadata);

        assert!(query.contains("继续复盘"));
        assert!(query.contains("product:thermos-001"));
        assert!(query.contains("campaign:live-001"));
        assert!(query.contains("便携保温杯"));
    }

    #[test]
    fn memory_query_text_expands_bare_context_ids_to_refs() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.product_ids".to_string(),
            "Thermos-004 product:Thermos-005".to_string(),
        );
        metadata.insert(
            "request.context.campaign_ids".to_string(),
            "Launch-006 campaign:Launch-007".to_string(),
        );

        let query = agent_memory_query_text("继续刚才", &metadata);

        assert!(query.contains("Thermos-004 product:Thermos-005"));
        assert!(query.contains("Launch-006 campaign:Launch-007"));
        assert!(query.contains("product:thermos-004"));
        assert!(query.contains("product:thermos-005"));
        assert!(query.contains("campaign:launch-006"));
        assert!(query.contains("campaign:launch-007"));
    }

    #[test]
    fn memory_query_text_deduplicates_and_caps_metadata_context() {
        let mut metadata = BTreeMap::new();
        metadata.insert("context.product_ids".to_string(), "product:cup".to_string());
        metadata.insert("context.refs".to_string(), "product:cup".to_string());
        metadata.insert(
            "context.selected_summary".to_string(),
            "长摘要".repeat(1_000),
        );

        let query = agent_memory_query_text("product:cup", &metadata);

        assert_eq!(query.matches("product:cup").count(), 1);
        assert!(query.chars().count() <= "product:cup".chars().count() + 1 + 1_200);
    }

    #[test]
    fn memory_query_text_includes_request_prefixed_context_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.refs".to_string(),
            "product:thermos-002,conversation_file:file-a".to_string(),
        );
        metadata.insert(
            "request.workspaceContextPack.summary".to_string(),
            "便携保温杯资料包包含直播间承接反馈。".to_string(),
        );

        let query = agent_memory_query_text("继续刚才", &metadata);

        assert!(query.contains("继续刚才"));
        assert!(query.contains("product:thermos-002"));
        assert!(query.contains("conversation_file:file-a"));
        assert!(query.contains("便携保温杯资料包"));
    }

    #[test]
    fn memory_query_text_includes_combined_and_attachment_refs() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.combined_refs".to_string(),
            "conversation_file:file-combined,workspace_source:source-a".to_string(),
        );
        metadata.insert(
            "attachments.image_refs".to_string(),
            "conversation_file:image-1".to_string(),
        );

        let query = agent_memory_query_text("继续分析资料", &metadata);

        assert!(query.contains("conversation_file:file-combined"));
        assert!(query.contains("workspace_source:source-a"));
        assert!(query.contains("conversation_file:image-1"));
    }

    #[test]
    fn memory_query_text_includes_workspace_pack_content_refs() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "workspaceContextPack.contentRefs".to_string(),
            "conversation_file:content-1,workspace_source:source-b".to_string(),
        );
        metadata.insert(
            "request.workspaceContextPack.contentRefs".to_string(),
            "conversation_file:content-2".to_string(),
        );

        let query = agent_memory_query_text("继续看资料包", &metadata);

        assert!(query.contains("conversation_file:content-1"));
        assert!(query.contains("workspace_source:source-b"));
        assert!(query.contains("conversation_file:content-2"));
    }

    #[test]
    fn memory_query_text_includes_rag_metadata_and_expands_product_ids() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "rag.product_ids".to_string(),
            "Thermos-008 product:Thermos-009".to_string(),
        );
        metadata.insert("rag.query_terms".to_string(), "退款率,成交承接".to_string());
        metadata.insert(
            "request.rag.summary".to_string(),
            "RAG 命中保温杯售后承接资料。".to_string(),
        );

        let query = agent_memory_query_text("继续看检索资料", &metadata);

        assert!(query.contains("Thermos-008 product:Thermos-009"));
        assert!(query.contains("product:thermos-008"));
        assert!(query.contains("product:thermos-009"));
        assert!(query.contains("退款率,成交承接"));
        assert!(query.contains("RAG 命中保温杯售后承接资料。"));
    }

    #[test]
    fn memory_query_text_includes_conversation_history_summaries() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "conversation.history_summary".to_string(),
            "最近对话里确认 QA矩阵智能保温杯 退款率偏高。".to_string(),
        );
        metadata.insert(
            "request.conversation.history_older_summary".to_string(),
            "更早对话记录了直播间成交承接要补客服解释。".to_string(),
        );

        let query = agent_memory_query_text("继续刚才", &metadata);

        assert!(query.contains("继续刚才"));
        assert!(query.contains("QA矩阵智能保温杯"));
        assert!(query.contains("退款率偏高"));
        assert!(query.contains("成交承接"));
    }

    #[test]
    fn memory_query_text_extracts_conversation_memory_summary_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.conversation.memory_summary_json".to_string(),
            serde_json::json!({
                "kind": "conversation_memory_summary",
                "rollingSummary": "上一轮确认夏季防晒衣需要提升点击转化。",
                "facts": [
                    {
                        "source": "conversation_message:old",
                        "text": "用户主推轻薄防晒衣，退款率偏高。",
                        "confidence": "explicit"
                    }
                ],
                "decisions": [
                    {
                        "text": "先按直播间成交承接重写脚本。"
                    }
                ],
                "constraints": [
                    "预算不能超过 3000 元。"
                ],
                "openQuestions": [
                    {
                        "text": "是否需要按渠道拆分预算？"
                    }
                ],
                "lastUserInput": "隐藏最后输入",
                "lastAssistantOutput": "隐藏最后输出",
                "sourceMap": {
                    "lastSessionId": "secret-session",
                    "memoryBudget": {
                        "memoryTextMaxChars": 2400
                    },
                    "debug": "hidden-source-map"
                }
            })
            .to_string(),
        );

        let query = agent_memory_query_text("继续完善方案", &metadata);

        assert!(query.contains("夏季防晒衣"));
        assert!(query.contains("轻薄防晒衣"));
        assert!(query.contains("成交承接"));
        assert!(query.contains("预算不能超过 3000"));
        assert!(query.contains("是否需要按渠道拆分预算"));
        assert!(!query.contains("隐藏最后输入"));
        assert!(!query.contains("隐藏最后输出"));
        assert!(!query.contains("conversation_message:old"));
        assert!(!query.contains("secret-session"));
        assert!(!query.contains("memoryBudget"));
        assert!(!query.contains("sourceMap"));
    }

    #[test]
    fn memory_query_text_extracts_conversation_memory_summary_camel_alias() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "conversation.memorySummaryJson".to_string(),
            serde_json::json!({
                "rollingSummary": "上一轮确认校园保温杯活动要先处理退款率。",
                "facts": [
                    {"text": "客服解释口径需要和直播脚本一起调整。"}
                ],
                "lastUserInput": "隐藏输入",
                "sourceMap": {"lastSessionId": "hidden-session"}
            })
            .to_string(),
        );

        let query = agent_memory_query_text("继续", &metadata);

        assert!(query.contains("校园保温杯活动"));
        assert!(query.contains("客服解释口径"));
        assert!(!query.contains("隐藏输入"));
        assert!(!query.contains("hidden-session"));
    }

    #[test]
    fn memory_query_text_includes_rag_and_workspace_source_titles() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "rag.source_titles".to_string(),
            "product_knowledge:knowledge-1 product=Thermos-010 title=保温杯售后复盘".to_string(),
        );
        metadata.insert(
            "request.workspaceContext.source_titles".to_string(),
            "conversation_file:file-7 title=直播成交承接表".to_string(),
        );

        let query = agent_memory_query_text("继续看资料", &metadata);

        assert!(query.contains("保温杯售后复盘"));
        assert!(query.contains("conversation_file:file-7 title=直播成交承接表"));
    }

    #[test]
    fn memory_query_text_includes_quality_learning_summary() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "qualityLearning.learningSummary".to_string(),
            "[quality_issue 0.82] 复盘回答需要优先说明退款率异常和客服解释口径。".to_string(),
        );

        let query = agent_memory_query_text("继续改回答", &metadata);

        assert!(query.contains("继续改回答"));
        assert!(query.contains("退款率异常"));
        assert!(query.contains("客服解释口径"));
    }

    #[test]
    fn memory_query_text_includes_context_summary_alias() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.summary".to_string(),
            "Selected context notes mention 直播退款率 and 客服解释口径.".to_string(),
        );

        let query = agent_memory_query_text("继续看上下文", &metadata);

        assert!(query.contains("Selected context notes"));
        assert!(query.contains("直播退款率"));
        assert!(query.contains("客服解释口径"));
    }

    #[test]
    fn memory_query_text_extracts_image_summary_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.attachments.image_summaries_json".to_string(),
            serde_json::json!([{
                "sourceRef": "conversation_file:image-9",
                "objectKey": "private/images/image-9.png",
                "summary": "图片显示冷萃咖啡促销海报。",
                "agentContext": "海报主标题是低糖冷萃，促销点是买二送一。",
                "analysisId": "analysis-secret",
                "llmName": "vision-model"
            }])
            .to_string(),
        );

        let query = agent_memory_query_text("继续看图片", &metadata);

        assert!(query.contains("conversation_file:image-9"));
        assert!(query.contains("低糖冷萃"));
        assert!(query.contains("买二送一"));
        assert!(!query.contains("private/images/image-9.png"));
        assert!(!query.contains("analysis-secret"));
        assert!(!query.contains("vision-model"));
        assert!(!query.contains("objectKey"));
    }

    #[test]
    fn memory_query_text_includes_image_text_metadata_aliases() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.attachments.image_titles".to_string(),
            "冷萃咖啡海报".to_string(),
        );
        metadata.insert(
            "request.context.vision_summary".to_string(),
            "画面中心是低糖冷萃咖啡瓶，底部突出19.9元价格。".to_string(),
        );
        metadata.insert(
            "attachments.ocr_text".to_string(),
            "低糖0脂 冷萃咖啡 19.9元".to_string(),
        );
        metadata.insert(
            "request.context.image_notes".to_string(),
            "海报卖点需要补充直播承接话术。".to_string(),
        );
        metadata.insert(
            "context.image_summaries_json".to_string(),
            serde_json::json!({
                "sourceRef": "conversation_file:image-10",
                "agentContext": "图片背景偏暗，促销信息集中在右上角。"
            })
            .to_string(),
        );

        let query = agent_memory_query_text("继续看这张图", &metadata);

        assert!(query.contains("冷萃咖啡海报"));
        assert!(query.contains("低糖冷萃咖啡瓶"));
        assert!(query.contains("低糖0脂 冷萃咖啡 19.9元"));
        assert!(query.contains("直播承接话术"));
        assert!(query.contains("conversation_file:image-10"));
        assert!(query.contains("促销信息集中在右上角"));
    }

    #[test]
    fn memory_query_text_extracts_source_summaries_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.source_summaries".to_string(),
            serde_json::json!([
                {
                    "ref": "conversation_file:profit-1",
                    "title": "profit.xlsx",
                    "summary": "GMV 12000 元，成本 7000 元，广告费 1200 元，订单 180。"
                },
                {
                    "sourceRef": "conversation_file:reviews-1",
                    "fileName": "reviews.docx",
                    "ocr_text": "用户反馈发货慢，客服响应时间偏长。"
                }
            ])
            .to_string(),
        );

        let query = agent_memory_query_text("继续提取证据", &metadata);

        assert!(query.contains("conversation_file:profit-1"));
        assert!(query.contains("profit.xlsx"));
        assert!(query.contains("GMV 12000"));
        assert!(query.contains("conversation_file:reviews-1"));
        assert!(query.contains("reviews.docx"));
        assert!(query.contains("发货慢"));
    }

    #[test]
    fn memory_query_text_extracts_source_summary_namespace_aliases() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "rag.sourceSummaries".to_string(),
            serde_json::json!([{
                "sourceRef": "product_material:mat-88",
                "fileName": "material.md",
                "summary": "材质卖点需要强调亲肤和速干。",
                "objectKey": "private/material/raw.json"
            }])
            .to_string(),
        );
        metadata.insert(
            "request.workspaceContextPack.source_summaries".to_string(),
            serde_json::json!([{
                "ref": "conversation_file:refund-88",
                "title": "refund.csv",
                "text": "退款原因集中在尺码偏小和发货慢。",
                "metadata": {"debug": "hidden"}
            }])
            .to_string(),
        );

        let query = agent_memory_query_text("继续看资料摘要", &metadata);

        assert!(query.contains("product_material:mat-88"));
        assert!(query.contains("material.md"));
        assert!(query.contains("亲肤和速干"));
        assert!(query.contains("conversation_file:refund-88"));
        assert!(query.contains("refund.csv"));
        assert!(query.contains("尺码偏小"));
        assert!(!query.contains("objectKey"));
        assert!(!query.contains("private/material/raw.json"));
        assert!(!query.contains("debug"));
        assert!(!query.contains("metadata"));
    }

    #[test]
    fn memory_query_text_extracts_source_summary_ocr_text_alias() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "workspaceContext.sourceSummaries".to_string(),
            serde_json::json!([{
                "sourceRef": "conversation_file:ocr-88",
                "fileName": "comments.png",
                "ocrText": "评论截图显示尺码偏小和发货慢。",
                "objectKey": "private/ocr/source.png"
            }])
            .to_string(),
        );

        let query = agent_memory_query_text("继续看截图文字", &metadata);

        assert!(query.contains("conversation_file:ocr-88"));
        assert!(query.contains("comments.png"));
        assert!(query.contains("尺码偏小和发货慢"));
        assert!(!query.contains("objectKey"));
        assert!(!query.contains("private/ocr/source.png"));
    }

    #[test]
    fn memory_query_text_extracts_source_summary_index_aliases() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.rag.source_summaries".to_string(),
            serde_json::json!([{
                "sourceId": "knowledge-99",
                "productId": "Thermos-099",
                "name": "售后知识摘要",
                "summary": "退款评论集中在杯盖漏水。",
                "metadata": {"raw": "hidden"}
            }])
            .to_string(),
        );

        let query = agent_memory_query_text("继续看知识摘要", &metadata);

        assert!(query.contains("knowledge-99"));
        assert!(query.contains("Thermos-099"));
        assert!(query.contains("售后知识摘要"));
        assert!(query.contains("杯盖漏水"));
        assert!(!query.contains("metadata"));
        assert!(!query.contains("hidden"));
    }

    #[test]
    fn memory_query_text_extracts_source_index_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.sources_json".to_string(),
            serde_json::json!([
                {
                    "source_type": "product_knowledge",
                    "id": "knowledge-77",
                    "product_id": "Thermos-077",
                    "title": "保温杯售后复盘",
                    "objectKey": "private/source/body.json",
                    "metadata": {"raw": "hidden metadata"},
                    "agentContext": "hidden agent context"
                },
                {
                    "sourceType": "conversation_file",
                    "id": "file-77",
                    "refKey": "conversation_file:file-77",
                    "fileName": "refunds.xlsx"
                }
            ])
            .to_string(),
        );

        let query = agent_memory_query_text("继续看来源", &metadata);

        assert!(query.contains("product_knowledge"));
        assert!(query.contains("knowledge-77"));
        assert!(query.contains("Thermos-077"));
        assert!(query.contains("保温杯售后复盘"));
        assert!(query.contains("conversation_file:file-77"));
        assert!(query.contains("refunds.xlsx"));
        assert!(!query.contains("private/source/body.json"));
        assert!(!query.contains("hidden metadata"));
        assert!(!query.contains("hidden agent context"));
        assert!(!query.contains("objectKey"));
    }

    #[test]
    fn memory_query_text_includes_camel_case_metadata_aliases() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "context.productIds".to_string(),
            "Thermos-010 product:Thermos-011".to_string(),
        );
        metadata.insert(
            "request.context.campaignIds".to_string(),
            "Launch-010 campaign:Launch-011".to_string(),
        );
        metadata.insert(
            "context.combinedRefs".to_string(),
            "conversation_file:file-camel".to_string(),
        );
        metadata.insert(
            "request.rag.queryTerms".to_string(),
            "退款率,客服解释".to_string(),
        );
        metadata.insert(
            "workspaceContext.sourceTitles".to_string(),
            "售后退款表".to_string(),
        );
        metadata.insert(
            "workspaceContextPack.contentRefs".to_string(),
            "workspace_source:camel-source".to_string(),
        );
        metadata.insert(
            "attachments.ocrText".to_string(),
            "海报写着低糖0脂".to_string(),
        );
        metadata.insert(
            "context.sourcesJson".to_string(),
            serde_json::json!({
                "sourceType": "conversation_file",
                "refKey": "conversation_file:camel-json",
                "fileName": "camel.xlsx",
                "objectKey": "hidden/raw.json"
            })
            .to_string(),
        );

        let query = agent_memory_query_text("继续看 camel", &metadata);

        assert!(query.contains("Thermos-010 product:Thermos-011"));
        assert!(query.contains("product:thermos-010"));
        assert!(query.contains("product:thermos-011"));
        assert!(query.contains("Launch-010 campaign:Launch-011"));
        assert!(query.contains("campaign:launch-010"));
        assert!(query.contains("campaign:launch-011"));
        assert!(query.contains("conversation_file:file-camel"));
        assert!(query.contains("退款率,客服解释"));
        assert!(query.contains("售后退款表"));
        assert!(query.contains("workspace_source:camel-source"));
        assert!(query.contains("海报写着低糖0脂"));
        assert!(query.contains("conversation_file:camel-json"));
        assert!(query.contains("camel.xlsx"));
        assert!(!query.contains("hidden/raw.json"));
        assert!(!query.contains("objectKey"));
    }

    #[test]
    fn memory_query_text_includes_context_refs_aliases() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "context.contextRefs".to_string(),
            "product:thermos-020,conversation_file:file-020".to_string(),
        );
        metadata.insert(
            "request.context.context_refs".to_string(),
            "campaign:launch-020".to_string(),
        );

        let query = agent_memory_query_text("继续看显式上下文", &metadata);

        assert!(query.contains("product:thermos-020"));
        assert!(query.contains("conversation_file:file-020"));
        assert!(query.contains("campaign:launch-020"));
    }

    #[test]
    fn memory_query_text_includes_singular_context_id_aliases() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "request.context.productId".to_string(),
            "Thermos-021".to_string(),
        );
        metadata.insert("context.campaignId".to_string(), "Launch-021".to_string());
        metadata.insert("rag.productId".to_string(), "Thermos-022".to_string());

        let query = agent_memory_query_text("继续看单数 id", &metadata);

        assert!(query.contains("Thermos-021"));
        assert!(query.contains("product:thermos-021"));
        assert!(query.contains("Launch-021"));
        assert!(query.contains("campaign:launch-021"));
        assert!(query.contains("Thermos-022"));
        assert!(query.contains("product:thermos-022"));
    }

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
    fn session_worker_roles_register_full_catalog_as_executor_safety_net() {
        let roles = RoleCatalog::builtin();
        let selected = select_roles_for_task(
            &roles,
            &SessionRuntimeConfig::default(),
            "帮我分析漏斗指标和归因口径",
            &BTreeMap::new(),
        );

        let worker_roles = session_worker_roles(&roles, selected)
            .into_iter()
            .map(|(role, routable)| (role.runtime_role.clone(), routable))
            .collect::<Vec<_>>();

        assert!(worker_roles.contains(&("chat".to_string(), true)));
        assert!(worker_roles.iter().any(|(role, _)| role == "data"));
        assert!(worker_roles.iter().any(|(role, _)| role == "ops"));
        assert!(worker_roles.iter().any(|(role, _)| role == "creative"));
        assert!(worker_roles.iter().any(|(role, _)| role == "service"));
        assert!(worker_roles.contains(&("engineering".to_string(), false)));
        assert_eq!(
            worker_roles
                .iter()
                .filter(|(role, _)| role.as_str() == "data")
                .count(),
            1
        );
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
    fn manual_roles_include_configured_task_graph_dependencies() {
        let roles = RoleCatalog::builtin();
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "strategy.collaboration_mode".to_string(),
            "manual".to_string(),
        );
        metadata.insert("strategy.primary_role".to_string(), "ops".to_string());
        metadata.insert(
            "strategy.selected_roles".to_string(),
            "ops,data,creative".to_string(),
        );

        let selected = select_roles_for_task(
            &roles,
            &SessionRuntimeConfig::default(),
            "结合最新公开来源搜索证据，给这个商品一个可执行运营建议",
            &metadata,
        )
        .into_iter()
        .map(|role| role.runtime_role.clone())
        .collect::<Vec<_>>();

        assert_eq!(selected[0], "ops");
        assert!(selected.contains(&"data".to_string()));
        assert!(selected.contains(&"creative".to_string()));
        assert!(selected.contains(&"search".to_string()));
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
