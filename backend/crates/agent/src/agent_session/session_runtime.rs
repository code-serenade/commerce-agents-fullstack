use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc as tokio_mpsc;

use crate::{
    MemoryBundle, SessionContext, TaskGraphMemorySummary,
    agent::{
        commander::{
            Commander, CommanderChannels, CommanderOptions, CommanderSessionContext,
            TaskGraphRuntime, plan_quick_chat_graph,
        },
        worker::Worker,
    },
    agent_error::{Result, SettingsError},
    cognition::Cognition,
    core::{
        messaging::MessageRx,
        protocol::{
            AgentId, Message, MessageContext, MessageId, Payload, SessionEvent, SessionId,
            TaskGraph, TaskId, TaskSpec, WorkerId,
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
                fast_route_enabled: input.config.commander_fast_route,
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
                    root_task: self.task_description.clone(),
                    nodes: vec![
                        self.chat_bypass_graph(&report.task_id)
                            .and_then(|graph| graph.nodes.into_iter().next())
                            .ok_or_else(|| {
                                SettingsError::invalid("chat bypass graph is unavailable")
                            })?,
                    ],
                });
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
        if self.commander_fast_route_disabled() {
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
        (node.role == "chat" && node.input_refs.is_empty()).then_some(graph)
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
        let support_runtime_roles = roles.into_iter().skip(1).collect();
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
            model_name: self.context.metadata.get("model.name").cloned(),
            route_mode: route_mode.to_string(),
            primary_runtime_role,
            support_runtime_roles,
        })
        .await;
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

    fn commander_fast_route_disabled(&self) -> bool {
        !self.commander_fast_route
    }

    fn dispatch_chat_direct(&self, context: MessageContext, graph: TaskGraph) -> Result<()> {
        let runtime = TaskGraphRuntime::new(graph);
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

fn is_direct_answer_chat_graph(graph: &TaskGraph) -> bool {
    let [node] = graph.nodes.as_slice() else {
        return false;
    };
    node.role == "chat" && node.title == "直接回答"
}

fn context_loaded_event(session_id: &SessionId, context: &MessageContext) -> Option<SessionEvent> {
    let product_ids = split_metadata_list(context.metadata.get("context.product_ids"));
    let refs = split_metadata_list(context.metadata.get("context.refs"));
    let product_count = parse_usize_metadata(context.metadata.get("context.product_count"));
    let knowledge_count = parse_usize_metadata(context.metadata.get("context.knowledge_count"));
    let source_count = parse_usize_metadata(context.metadata.get("context.source_count"));
    let summary = context
        .metadata
        .get("context.summary")
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
