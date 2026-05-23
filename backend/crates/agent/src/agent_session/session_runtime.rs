use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc as tokio_mpsc;

use crate::{
    MemoryBundle, SessionContext, TaskGraphMemorySummary,
    agent::{
        commander::{
            Commander, CommanderChannels, CommanderOptions, CommanderSessionContext,
            TaskGraphRuntime, no_user_facing_content_message, plan_forced_chat_graph,
            plan_quick_chat_graph, sanitize_user_facing_text,
        },
        worker::Worker,
    },
    agent_error::{Result, SettingsError},
    cognition::Cognition,
    core::{
        messaging::MessageRx,
        protocol::{
            AgentId, DispatchContextBudget, DispatchPlannedStep, Message, MessageContext,
            MessageId, Payload, SessionEvent, SessionId, TaskGraph, TaskId, TaskSpec, WorkerId,
        },
    },
    roles::{RoleCatalog, RoleProfile, RoleRouter},
    skills::{SearchAdapterConfig, SkillExecutorSet},
};

pub type SessionEventTx = tokio_mpsc::Sender<SessionEvent>;
pub type SessionEventRx = tokio_mpsc::Receiver<SessionEvent>;

#[derive(Debug, Clone)]
pub struct SessionRuntimeConfig {
    pub response_timeout: Duration,
    pub event_buffer: usize,
    pub commander_fast_route: bool,
    pub commander_fast_route_min_score: f32,
    pub search: SearchAdapterConfig,
}

impl Default for SessionRuntimeConfig {
    fn default() -> Self {
        Self {
            response_timeout: Duration::from_secs(300),
            event_buffer: 1024,
            commander_fast_route: true,
            commander_fast_route_min_score: 2.0,
            search: SearchAdapterConfig::default(),
        }
    }
}

pub struct SessionRuntimeWorkerInput {
    pub role: RoleProfile,
    pub cognition: Box<dyn Cognition + Send>,
}

pub struct SessionRuntimeInput {
    pub session_id: SessionId,
    pub context: MessageContext,
    pub task_description: String,
    pub memory_bundle: MemoryBundle,
    pub commander_cognition: Box<dyn Cognition + Send>,
    pub workers: Vec<SessionRuntimeWorkerInput>,
    pub config: SessionRuntimeConfig,
    pub event_tx: Option<SessionEventTx>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionResult {
    pub session_id: SessionId,
    pub output: String,
    pub task_graphs: Vec<TaskGraphMemorySummary>,
    pub role_summaries: Vec<crate::RoleMemorySummary>,
    pub worker_details: Vec<crate::WorkerDetail>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub evaluation_summary: Option<String>,
}

pub struct SessionRuntime {
    session_id: SessionId,
    context: MessageContext,
    task_description: String,
    session_context: Arc<SessionContext>,
    commander_tx: crate::core::messaging::MessageTx,
    worker_txs: Vec<crate::core::messaging::MessageTx>,
    response_rx: Option<MessageRx>,
    commander_loop: tokio::task::JoinHandle<()>,
    worker_loops: Vec<tokio::task::JoinHandle<()>>,
    response_timeout: Duration,
    commander_fast_route: bool,
    event_tx: Option<SessionEventTx>,
}

impl SessionRuntime {
    pub fn start(input: SessionRuntimeInput) -> Result<Self> {
        let session_context = Arc::new(SessionContext::new());
        session_context.extensions().insert(input.memory_bundle);
        let (commander_tx, commander_rx) = tokio_mpsc::unbounded_channel::<Message>();
        let (response_tx, response_rx) = tokio_mpsc::unbounded_channel::<Message>();

        session_context
            .directory()
            .register_commander_tx(commander_tx.clone());

        if input.workers.is_empty() {
            return Err(
                SettingsError::invalid("session runtime requires at least one worker").into(),
            );
        }

        let mut worker_txs = Vec::new();
        let mut worker_loops = Vec::new();
        let role_catalog = RoleCatalog::new(
            input
                .workers
                .iter()
                .map(|worker| worker.role.clone())
                .collect(),
        );
        for worker_input in input.workers {
            let profile = worker_input.role.to_worker_profile();
            let (worker_tx, worker_rx) = tokio_mpsc::unbounded_channel::<Message>();
            session_context
                .directory()
                .register_worker_tx(profile.worker_id.clone(), worker_tx.clone());
            session_context.worker_catalog().register(profile.clone());

            let skill_executor = SkillExecutorSet::staged_with_search(input.config.search.clone())
                .map_err(|error| {
                    tracing::warn!(error = %error, "worker skill executor initialization failed");
                    error
                })
                .ok();
            let worker = Worker::new(
                worker_input.role,
                worker_input.cognition,
                worker_rx,
                commander_tx.clone(),
                input.event_tx.clone(),
                skill_executor,
            );
            let worker_loop = tokio::spawn(async move {
                let mut worker = worker;
                worker.run().await;
            });

            worker_txs.push(worker_tx);
            worker_loops.push(worker_loop);
        }

        let commander = Commander::new(
            AgentId("commander".to_string()),
            AgentId("external-host".to_string()),
            input.commander_cognition,
            CommanderChannels {
                receiver: commander_rx,
                sender: response_tx,
                event_tx: input.event_tx.clone(),
            },
            session_context.clone(),
            RoleRouter::new(role_catalog),
            CommanderOptions {
                fast_route_enabled: input.config.commander_fast_route
                    && routing_mode_allows_fast_route(&input.context),
                fast_route_min_score: input.config.commander_fast_route_min_score,
            },
        );

        let commander_loop = tokio::spawn(async move {
            let mut commander = commander;
            commander.run().await;
        });

        Ok(Self {
            session_id: input.session_id,
            context: input.context,
            task_description: input.task_description,
            session_context,
            commander_tx,
            worker_txs,
            response_rx: Some(response_rx),
            commander_loop,
            worker_loops,
            response_timeout: input.config.response_timeout,
            commander_fast_route: input.config.commander_fast_route,
            event_tx: input.event_tx,
        })
    }

    pub async fn run_until_complete(&mut self) -> Result<SessionResult> {
        self.emit_event(SessionEvent::Started {
            session_id: self.session_id.clone(),
        })
        .await;

        let memory_kinds = self
            .session_context
            .extensions()
            .with::<MemoryBundle, _>(MemoryBundle::kind_names)
            .unwrap_or_default();
        let memory_count = self
            .session_context
            .extensions()
            .with::<MemoryBundle, _>(|bundle| bundle.records.len())
            .unwrap_or_default();
        self.emit_event(SessionEvent::MemoryLoaded {
            session_id: self.session_id.clone(),
            count: memory_count,
            kinds: memory_kinds,
        })
        .await;
        if let Some(event) = context_loaded_event(&self.session_id, &self.context) {
            self.emit_event(event).await;
        }

        let task_id = TaskId(next_id("task"));
        let mut context = self.context.clone();
        context.session_id = Some(self.session_id.clone());
        context.task_id = Some(task_id.clone());

        if let Some(graph) = self.chat_bypass_graph(&task_id) {
            self.emit_route_decided(
                "chat_bypass",
                Some(graph.nodes.iter().map(|node| node.role.clone()).collect()),
            )
            .await;
            self.emit_dispatch_planned_for_graph("chat_bypass", &graph)
                .await;
            self.dispatch_chat_direct(context.clone(), graph)?;
        } else {
            let message = Message::new_with_context(
                MessageId(next_id("msg")),
                context,
                AgentId("external-host".to_string()),
                AgentId("commander".to_string()),
                Payload::HumanCommand {
                    task: TaskSpec {
                        id: task_id,
                        description: self.task_description.clone(),
                    },
                },
            );

            self.commander_tx
                .send(message)
                .map_err(|_| SettingsError::invalid("failed to send task to session commander"))?;

            self.emit_event(SessionEvent::CommanderThinking {
                session_id: self.session_id.clone(),
            })
            .await;
        }

        let Some(mut response_rx) = self.response_rx.take() else {
            return Err(SettingsError::invalid("session response receiver already used").into());
        };
        let timeout = self.response_timeout;
        let received = match tokio::time::timeout(timeout, response_rx.recv()).await {
            Ok(Some(message)) => message,
            Ok(None) => {
                let reason = "session response channel closed".to_string();
                return Err(SettingsError::invalid(reason).into());
            }
            Err(_) => {
                let reason = "session response timed out".to_string();
                return Err(SettingsError::invalid(reason).into());
            }
        };

        let output = match received.payload {
            Payload::Text { content } => content,
            Payload::WorkerReport { report } => {
                self.emit_event(SessionEvent::WorkerFinished {
                    session_id: self.session_id.clone(),
                    worker_id: report.worker_id.clone(),
                    task_id: report.task_id.clone(),
                })
                .await;

                let mut runtime = TaskGraphRuntime::new(TaskGraph {
                    graph_id: report.graph_id.clone(),
                    root_task: self.task_description_with_conversation_history(),
                    nodes: vec![
                        self.chat_bypass_graph(&report.task_id)
                            .and_then(|graph| graph.nodes.into_iter().next())
                            .ok_or_else(|| {
                                SettingsError::invalid("chat bypass graph is unavailable")
                            })?,
                    ],
                })
                .with_response_mode(
                    self.context
                        .metadata
                        .get("strategy.response_mode")
                        .map(String::as_str)
                        .or_else(|| self.context.metadata.get("strategy.id").map(String::as_str)),
                )
                .with_learning_depth(
                    self.context
                        .metadata
                        .get("strategy.learning_level")
                        .map(String::as_str),
                );
                let assignment = runtime.build_assignment(&runtime.ready_nodes().remove(0));
                runtime.mark_running(&assignment);
                let _ = runtime.apply_report(report);
                let output = runtime
                    .direct_chat_output()
                    .unwrap_or_else(|| runtime.synthesize());
                self.session_context
                    .record_task_graph_memory(runtime.memory_summary());
                output
            }
            _ => "session returned non-text response".to_string(),
        };
        let output =
            sanitize_user_facing_text(&output).unwrap_or_else(no_user_facing_content_message);

        self.emit_event(SessionEvent::Output {
            session_id: self.session_id.clone(),
            content: output.clone(),
        })
        .await;

        let task_graphs = self
            .session_context
            .extensions()
            .with::<Vec<TaskGraphMemorySummary>, _>(Clone::clone)
            .unwrap_or_default();
        let role_summaries = task_graphs
            .iter()
            .flat_map(|summary| summary.role_summaries.clone())
            .collect::<Vec<_>>();
        let worker_details = task_graphs
            .iter()
            .flat_map(|summary| summary.worker_details.clone())
            .collect::<Vec<_>>();
        let risks = unique_strings(
            task_graphs
                .iter()
                .flat_map(|summary| summary.risks.clone())
                .collect(),
        );
        let open_questions = unique_strings(
            task_graphs
                .iter()
                .flat_map(|summary| summary.open_questions.clone())
                .collect(),
        );
        let evaluation_summary = (!task_graphs.is_empty()).then(|| {
            task_graphs
                .iter()
                .filter_map(|summary| summary.evaluation_summary.clone())
                .collect::<Vec<_>>()
                .join("\n")
        });

        Ok(SessionResult {
            session_id: self.session_id.clone(),
            output,
            task_graphs,
            role_summaries,
            worker_details,
            risks,
            open_questions,
            evaluation_summary,
        })
    }

    pub async fn shutdown(self) {
        let _ = self.commander_tx.send(Message::new(
            MessageId(next_id("msg")),
            AgentId("session-runtime".to_string()),
            AgentId("commander".to_string()),
            Payload::Control {
                signal: crate::core::protocol::ControlSignal::Shutdown,
                target: None,
            },
        ));
        for worker_tx in &self.worker_txs {
            let _ = worker_tx.send(Message::new(
                MessageId(next_id("msg")),
                AgentId("session-runtime".to_string()),
                AgentId("worker".to_string()),
                Payload::Text {
                    content: "control:shutdown".to_string(),
                },
            ));
        }

        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            let _ = self.commander_loop.await;
            for worker_loop in self.worker_loops {
                let _ = worker_loop.await;
            }
        })
        .await;
    }

    pub fn session_context(&self) -> Arc<SessionContext> {
        self.session_context.clone()
    }

    fn chat_bypass_graph(&self, task_id: &TaskId) -> Option<TaskGraph> {
        if self.is_explicit_chat_role_selection() {
            let task_for_worker = self.task_description_with_conversation_history();
            return Some(plan_forced_chat_graph(task_id, &task_for_worker));
        }
        if self.commander_fast_route_disabled() || !self.context_allows_fast_route() {
            return None;
        }
        let graph = plan_quick_chat_graph(task_id, &self.task_description)?;
        if self.has_explicit_non_chat_role_selection() {
            return None;
        }
        if self.force_collaboration() && !is_direct_answer_chat_graph(&graph) {
            return None;
        }
        let [node] = graph.nodes.as_slice() else {
            return None;
        };
        if node.role != "chat" || !node.input_refs.is_empty() {
            return None;
        }
        let task_for_worker = self.task_description_with_conversation_history();
        Some(chat_graph_with_task_context(graph, &task_for_worker))
    }

    fn task_description_with_conversation_history(&self) -> String {
        let Some(history) = self
            .context
            .metadata
            .get("conversation.history_summary")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            return self.task_description.clone();
        };

        if self.task_description.contains("同一会话历史上下文") {
            return self.task_description.clone();
        }

        format!(
            "同一会话历史上下文（这是同一用户同一会话内此前消息，用于理解“继续、刚才、按前面、\
             补充后、沿用这些数据”等追问；这不代表用户选择了任何预设商品）：\n{history}\n\\
             n当前用户请求：\n{}",
            self.task_description
        )
    }

    async fn emit_route_decided(&self, route_mode: &str, roles: Option<Vec<String>>) {
        let strategy_id = self.context.metadata.get("strategy.id").cloned();
        let response_mode = self.context.metadata.get("strategy.response_mode").cloned();
        let collaboration_mode = self
            .context
            .metadata
            .get("strategy.collaboration_mode")
            .cloned();
        let roles = roles.unwrap_or_default();
        let primary_runtime_role = roles.first().cloned();
        let support_runtime_roles = unique_support_runtime_roles(
            primary_runtime_role.as_deref(),
            roles.into_iter().skip(1).collect(),
        );
        self.emit_event(SessionEvent::RouteDecided {
            session_id: self.session_id.clone(),
            strategy_id,
            response_mode,
            collaboration_mode,
            learning_level: self
                .context
                .metadata
                .get("strategy.learning_level")
                .cloned(),
            route_mode: route_mode.to_string(),
            primary_runtime_role,
            support_runtime_roles,
        })
        .await;
    }

    async fn emit_dispatch_planned_for_graph(&self, route_mode: &str, graph: &TaskGraph) {
        let (primary_runtime_role, support_runtime_roles) =
            dispatch_runtime_roles_from_graph(graph);
        let context_budget = dispatch_context_budget(&self.context, self.memory_record_count());
        let difficulty = dispatch_difficulty(
            route_mode,
            dispatch_runtime_role_count(primary_runtime_role.as_deref(), &support_runtime_roles),
            &context_budget,
        );
        let planned_steps = dispatch_steps_from_graph(route_mode, graph, &context_budget);
        let student_visible_explanation = dispatch_student_explanation_with_context(
            route_mode,
            graph.nodes.len(),
            &context_budget,
        );
        self.emit_event(SessionEvent::DispatchPlanned {
            session_id: self.session_id.clone(),
            difficulty: difficulty.to_string(),
            route_mode: route_mode.to_string(),
            primary_runtime_role,
            support_runtime_roles,
            selected_skills: dispatch_selected_skills(route_mode, graph),
            planned_steps,
            context_budget,
            reasoning_summary: dispatch_reasoning_summary(route_mode, graph.nodes.len()),
            student_visible_explanation,
        })
        .await;
    }

    fn memory_record_count(&self) -> usize {
        self.session_context
            .extensions()
            .with::<MemoryBundle, _>(|bundle| bundle.records.len())
            .unwrap_or_default()
    }

    fn force_collaboration(&self) -> bool {
        self.context
            .metadata
            .get("strategy.collaboration_mode")
            .is_some_and(|value| value == "manual" || value == "single")
            || self
                .context
                .metadata
                .get("strategy.id")
                .is_some_and(|value| value == "collab")
    }

    fn has_explicit_non_chat_role_selection(&self) -> bool {
        let collaboration_mode = self
            .context
            .metadata
            .get("strategy.collaboration_mode")
            .map(String::as_str)
            .unwrap_or_default();
        if !matches!(collaboration_mode, "manual" | "single") {
            return false;
        }

        let selected_roles =
            split_metadata_list(self.context.metadata.get("strategy.selected_roles"));
        selected_roles.iter().any(|role| role != "chat")
            || self
                .context
                .metadata
                .get("strategy.primary_role")
                .map(|role| role.trim())
                .is_some_and(|role| !role.is_empty() && role != "chat")
    }

    fn is_explicit_chat_role_selection(&self) -> bool {
        let collaboration_mode = self
            .context
            .metadata
            .get("strategy.collaboration_mode")
            .map(String::as_str)
            .unwrap_or_default();
        if !matches!(collaboration_mode, "single" | "manual") {
            return false;
        }
        if self.has_explicit_non_chat_role_selection() {
            return false;
        }

        let selected_roles =
            split_metadata_list(self.context.metadata.get("strategy.selected_roles"));
        let primary_role = self
            .context
            .metadata
            .get("strategy.primary_role")
            .map(|role| role.trim());

        selected_roles.iter().any(|role| role == "chat")
            || primary_role.is_some_and(|role| role == "chat")
    }

    fn commander_fast_route_disabled(&self) -> bool {
        !self.commander_fast_route
    }

    fn context_allows_fast_route(&self) -> bool {
        routing_mode_allows_fast_route(&self.context)
    }

    fn dispatch_chat_direct(&self, context: MessageContext, graph: TaskGraph) -> Result<()> {
        let runtime = TaskGraphRuntime::new(graph)
            .with_response_mode(
                self.context
                    .metadata
                    .get("strategy.response_mode")
                    .map(String::as_str)
                    .or_else(|| self.context.metadata.get("strategy.id").map(String::as_str)),
            )
            .with_learning_depth(
                self.context
                    .metadata
                    .get("strategy.learning_level")
                    .map(String::as_str),
            );
        let Some(node) = runtime.ready_nodes().into_iter().next() else {
            return Err(SettingsError::invalid("chat bypass graph has no ready node").into());
        };
        let assignment = runtime.build_assignment(&node);
        let chat_worker = WorkerId("worker.chat".to_string());
        let Some(worker_tx) = self.session_context.directory().get_worker_tx(&chat_worker) else {
            return Err(SettingsError::invalid("chat worker is not available").into());
        };

        let message = Message::new_with_context(
            MessageId(next_id("msg")),
            context,
            AgentId("external-host".to_string()),
            AgentId(chat_worker.0),
            Payload::WorkerAssignment { assignment },
        );
        worker_tx
            .send(message)
            .map_err(|_| SettingsError::invalid("failed to send task to chat worker").into())
    }

    async fn emit_event(&self, event: SessionEvent) {
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(event).await;
        }
    }
}

fn routing_mode_allows_fast_route(context: &MessageContext) -> bool {
    context
        .metadata
        .get("strategy.routing_mode")
        .is_some_and(|value| value == "fast")
}

fn next_id(prefix: &str) -> String {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let sequence = NEXT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{millis}-{sequence}")
}

fn unique_strings(items: Vec<String>) -> Vec<String> {
    items.into_iter().fold(Vec::new(), |mut unique, item| {
        if !item.trim().is_empty() && !unique.contains(&item) {
            unique.push(item);
        }
        unique
    })
}

pub(crate) fn dispatch_runtime_roles_from_graph(
    graph: &TaskGraph,
) -> (Option<String>, Vec<String>) {
    let primary_runtime_role = graph.nodes.first().map(|node| node.role.clone());
    let support_runtime_roles = unique_support_runtime_roles(
        primary_runtime_role.as_deref(),
        graph
            .nodes
            .iter()
            .skip(1)
            .map(|node| node.role.clone())
            .collect(),
    );

    (primary_runtime_role, support_runtime_roles)
}

pub(crate) fn unique_support_runtime_roles(
    primary_runtime_role: Option<&str>,
    roles: Vec<String>,
) -> Vec<String> {
    let primary_runtime_role = primary_runtime_role.map(str::trim).unwrap_or_default();
    roles.into_iter().fold(Vec::new(), |mut unique, role| {
        let role = role.trim();
        if !role.is_empty()
            && role != primary_runtime_role
            && !unique.iter().any(|item| item == role)
        {
            unique.push(role.to_string());
        }
        unique
    })
}

pub(crate) fn dispatch_runtime_role_count(
    primary_runtime_role: Option<&str>,
    support_runtime_roles: &[String],
) -> usize {
    usize::from(
        primary_runtime_role
            .map(str::trim)
            .is_some_and(|role| !role.is_empty()),
    ) + support_runtime_roles.len()
}

fn is_direct_answer_chat_graph(graph: &TaskGraph) -> bool {
    let [node] = graph.nodes.as_slice() else {
        return false;
    };
    node.role == "chat" && node.title == "直接回答"
}

fn chat_graph_with_task_context(mut graph: TaskGraph, task: &str) -> TaskGraph {
    if task.trim().is_empty() || graph.root_task == task {
        return graph;
    }
    graph.root_task = task.to_string();
    for node in &mut graph.nodes {
        if node.role != "chat" || node.objective.contains(task) {
            continue;
        }
        node.objective = format!("{}\n\n补充任务上下文：\n{task}", node.objective);
    }
    graph
}

fn context_loaded_event(session_id: &SessionId, context: &MessageContext) -> Option<SessionEvent> {
    let product_ids = split_metadata_list(context.metadata.get("context.product_ids"));
    let refs = split_metadata_list(context.metadata.get("context.refs"));
    let product_count = parse_usize_metadata(context.metadata.get("context.product_count"));
    let knowledge_count = parse_usize_metadata(context.metadata.get("context.knowledge_count"));
    let source_count = parse_usize_metadata(context.metadata.get("context.source_count"));
    let summary = context
        .metadata
        .get("context.selected_summary")
        .map(|value| compact_event_text(value, 600));
    let sources_json = context.metadata.get("context.sources_json").cloned();

    if product_ids.is_empty()
        && refs.is_empty()
        && product_count == 0
        && knowledge_count == 0
        && summary.is_none()
    {
        return None;
    }

    Some(SessionEvent::ContextLoaded {
        session_id: session_id.clone(),
        product_ids,
        refs,
        product_count,
        knowledge_count,
        source_count,
        summary,
        sources_json,
    })
}

pub(crate) fn dispatch_steps_from_graph(
    route_mode: &str,
    graph: &TaskGraph,
    context_budget: &DispatchContextBudget,
) -> Vec<DispatchPlannedStep> {
    let mut steps = Vec::new();
    steps.push(DispatchPlannedStep {
        id: "context".to_string(),
        title: "读取上下文".to_string(),
        detail: dispatch_context_step_detail(context_budget),
    });
    steps.push(DispatchPlannedStep {
        id: "route".to_string(),
        title: "判断任务难度和路线".to_string(),
        detail: format!(
            "调度中心选择 {route_mode}，计划 {} 个执行步骤。",
            graph.nodes.len()
        ),
    });
    steps.extend(graph.nodes.iter().take(6).map(|node| DispatchPlannedStep {
        id: node.id.0.clone(),
        title: node.title.clone(),
        detail: node.objective.clone(),
    }));
    steps.push(DispatchPlannedStep {
        id: "answer".to_string(),
        title: "汇总回答".to_string(),
        detail: "把执行结果整理为学生可直接使用的回答和下一步建议。".to_string(),
    });
    steps
}

pub(crate) fn dispatch_context_budget(
    context: &MessageContext,
    memory_record_count: usize,
) -> DispatchContextBudget {
    DispatchContextBudget {
        memory_record_count,
        history_summary_available: context
            .metadata
            .get("conversation.history_summary")
            .is_some_and(|value| !value.trim().is_empty()),
        recent_history_message_count: parse_usize_metadata(
            context.metadata.get("conversation.history_message_count"),
        ),
        history_total_message_count: parse_usize_metadata(
            context
                .metadata
                .get("conversation.history_total_message_count"),
        ),
        history_truncated: context
            .metadata
            .get("conversation.history_truncated")
            .is_some_and(|value| value == "true"),
        history_older_summary_available: context
            .metadata
            .get("conversation.history_older_summary")
            .is_some_and(|value| !value.trim().is_empty()),
        explicit_product_count: split_metadata_list(context.metadata.get("context.product_ids"))
            .len(),
        explicit_context_ref_count: split_metadata_list(context.metadata.get("context.refs")).len(),
        workspace_context_ref_count: split_metadata_list(
            context.metadata.get("workspaceContext.refs"),
        )
        .len(),
        workspace_pack_item_count: parse_usize_metadata(
            context.metadata.get("workspaceContextPack.item_count"),
        )
        .max(split_metadata_list(context.metadata.get("workspaceContextPack.refs")).len()),
        approximate_input_chars: approximate_context_chars(context),
    }
}

pub(crate) fn dispatch_selected_skills(route_mode: &str, graph: &TaskGraph) -> Vec<String> {
    if route_mode == "chat_bypass" {
        return Vec::new();
    }

    let mut skills = Vec::new();
    for node in graph
        .nodes
        .iter()
        .filter(|node| node.skill_autosuggest.unwrap_or(true))
    {
        for skill in planned_skills_for_role(&node.role) {
            if !skills.iter().any(|item| item == skill) {
                skills.push(skill.to_string());
            }
            if skills.len() >= 8 {
                return skills;
            }
        }
    }
    skills
}

pub(crate) fn dispatch_difficulty(
    route_mode: &str,
    step_count: usize,
    context_budget: &DispatchContextBudget,
) -> &'static str {
    if route_mode == "chat_bypass" && step_count <= 1 {
        return "simple";
    }
    if step_count >= 4
        || context_budget.history_summary_available
        || context_budget.history_truncated
        || context_budget.recent_history_message_count >= 8
        || context_budget.approximate_input_chars >= 6000
        || context_budget.explicit_context_ref_count + context_budget.workspace_context_ref_count
            >= 4
    {
        return "complex";
    }
    if step_count >= 2 || context_budget.explicit_product_count > 0 {
        return "normal";
    }
    "simple"
}

pub(crate) fn dispatch_reasoning_summary(route_mode: &str, step_count: usize) -> String {
    match route_mode {
        "chat_bypass" => "问题可以用轻量上下文和直接回答完成，因此走快速调度；如有同会话历史，\
                          会作为追问背景读取。"
            .to_string(),
        "fast_worker" => {
            "问题匹配到一个主要岗位，先由该岗位处理，并按本岗位策略自动补充合适 Skill。".to_string()
        }
        "task_graph" => format!("问题需要拆成 {step_count} 个岗位步骤协作，按任务图顺序执行。"),
        _ => format!("调度中心已选择 {route_mode} 路线。"),
    }
}

pub(crate) fn dispatch_student_explanation(route_mode: &str, step_count: usize) -> String {
    match route_mode {
        "chat_bypass" => {
            "这是一个简单问题，系统会快速读取本轮资料和同会话历史，再直接给出结果。".to_string()
        }
        "fast_worker" => {
            "这个问题主要由一个合适岗位处理，并会按需要调用对应 Skill 完成分析。".to_string()
        }
        "task_graph" => {
            format!("这个问题会拆给 {step_count} 个岗位/步骤协作，过程更长但结果会更完整。")
        }
        _ => "系统会根据本轮问题和上下文选择合适流程。".to_string(),
    }
}

pub(crate) fn dispatch_student_explanation_with_context(
    route_mode: &str,
    step_count: usize,
    context_budget: &DispatchContextBudget,
) -> String {
    let base = dispatch_student_explanation(route_mode, step_count);
    let context_hint = dispatch_context_student_hint(context_budget);
    if context_hint.is_empty() {
        return base;
    }
    format!("{base} {context_hint}")
}

fn dispatch_context_step_detail(context_budget: &DispatchContextBudget) -> String {
    let material_count = dispatch_material_signal_count(context_budget);
    match (
        context_budget.history_summary_available,
        context_budget.recent_history_message_count,
        context_budget.history_total_message_count,
        context_budget.history_truncated,
        material_count,
    ) {
        (true, history_count, total_count, true, materials) if materials > 0 => {
            format!(
                "读取本轮问题、最近 {history_count}/{total_count} 条同会话历史、早期压缩摘要和 \
                 {materials} 个资料信号。"
            )
        }
        (true, history_count, total_count, true, _) => {
            format!("读取本轮问题、最近 {history_count}/{total_count} 条同会话历史和早期压缩摘要。")
        }
        (true, history_count, _, false, materials) if materials > 0 => {
            format!(
                "读取本轮问题、同会话历史摘要（约 {history_count} 条历史消息）和 {materials} \
                 个资料信号。"
            )
        }
        (true, history_count, _, false, _) => {
            format!("读取本轮问题和同会话历史摘要（约 {history_count} 条历史消息）。")
        }
        (false, _, _, _, materials) if materials > 0 => {
            format!("读取本轮问题、会话记忆和 {materials} 个资料信号。")
        }
        _ => "读取本轮问题、会话记忆、商品资料、工作区资料和检索上下文。".to_string(),
    }
}

fn dispatch_context_student_hint(context_budget: &DispatchContextBudget) -> String {
    let material_count = dispatch_material_signal_count(context_budget);
    if context_budget.history_truncated && material_count > 0 {
        return format!(
            "本轮历史较长，已读取最近 {}/{} 条消息并纳入早期压缩摘要，同时读取 {material_count} \
             个资料信号。",
            context_budget.recent_history_message_count, context_budget.history_total_message_count
        );
    }
    if context_budget.history_truncated {
        return format!(
            "本轮历史较长，已读取最近 {}/{} 条消息并纳入早期压缩摘要。",
            context_budget.recent_history_message_count, context_budget.history_total_message_count
        );
    }
    if context_budget.history_summary_available && material_count > 0 {
        return format!(
            "本轮已把同会话历史和 {material_count} 个资料信号纳入调度，所以过程会更重视前后文。"
        );
    }
    if context_budget.history_summary_available {
        return "本轮已把同会话历史纳入调度，适合继续追问或沿用前面数据。".to_string();
    }
    if material_count > 0 {
        return format!("本轮已读取 {material_count} 个资料信号，回答会优先围绕这些资料展开。");
    }
    if context_budget.approximate_input_chars >= 6000 {
        return "本轮输入较长，调度会按更复杂的上下文处理。".to_string();
    }
    String::new()
}

fn dispatch_material_signal_count(context_budget: &DispatchContextBudget) -> usize {
    context_budget.explicit_product_count
        + context_budget.explicit_context_ref_count
        + context_budget.workspace_context_ref_count
        + context_budget.workspace_pack_item_count
        + context_budget.memory_record_count
}

fn planned_skills_for_role(role: &str) -> &'static [&'static str] {
    match role {
        "chat" => &["learning_teaching_explain"],
        "data" => &[
            "query_store_metrics",
            "data_dashboard",
            "data_funnel_analysis",
        ],
        "ops" => &[
            "coordination_skill_chain_planner",
            "coordination_create_task",
            "query_campaign_metrics",
        ],
        "creative" => &["creative_title_ctr_scorer", "creative_video_script"],
        "web" => &[
            "web_keyword_research",
            "web_title_generator",
            "web_seo_optimize",
        ],
        "service" => &["service_ticket_handler", "service_sentiment_analyzer"],
        "accounting" => &["accounting_roi_calc", "accounting_profit_analysis"],
        "douyin_ads" => &["douyin_ads_plan_structure", "douyin_ads_budget_pacing"],
        "design" => &["design_material_spec", "creative_title_ctr_scorer"],
        "engineering" => &[
            "engineering_system_check",
            "engineering_data_readiness_check",
        ],
        _ => &[],
    }
}

fn approximate_context_chars(context: &MessageContext) -> usize {
    [
        "conversation.history_summary",
        "context.selected_summary",
        "context.combined_summary",
        "rag.summary",
        "workspaceContext.summary",
        "workspaceContextPack.summary",
    ]
    .iter()
    .filter_map(|key| context.metadata.get(*key))
    .map(|value| value.chars().count())
    .sum()
}

fn split_metadata_list(value: Option<&String>) -> Vec<String> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_usize_metadata(value: Option<&String>) -> usize {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_default()
}

fn compact_event_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}
