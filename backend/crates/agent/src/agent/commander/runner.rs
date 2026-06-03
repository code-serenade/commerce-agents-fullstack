use std::{collections::HashMap, sync::Arc};

use tracing::{debug, info, warn};

mod collaboration;
mod context;
mod graph;
mod output;
mod route;

use context::{extend_context_facts, extend_runtime_strategy_facts, runtime_response_mode};
use output::{final_worker_message, next_message_id};
use route::{
    normalize_requested_runtime_roles, primary_route_score, primary_worker_id,
    selected_runtime_roles,
};

use super::{
    CommanderPhase, CommanderState, CommanderTask, RoutedDecision, TaskGraphRuntime, TaskMemory,
    decision_intent_from_json, plan_configured_non_chat_task_graph, plan_task_graph,
    role_dispatch_debug::{log_request_received, log_route_decided},
};
use crate::{
    agent_session::{
        MemoryRecord, TaskGraphMemorySummary,
        session_runtime::{
            DispatchAdaptiveProfileInput, dispatch_adaptive_profile, dispatch_context_budget,
            dispatch_difficulty, dispatch_effective_difficulty, dispatch_reasoning_summary,
            dispatch_runtime_role_count, dispatch_runtime_roles_from_graph,
            dispatch_selected_skills, dispatch_steps_from_graph,
            dispatch_student_explanation_with_context, unique_support_runtime_roles,
        },
    },
    cognition::{Cognition, CognitionInput, CognitionResult, Context, Fact, Intent, IntentKind},
    core::{
        messaging::{MessageRx, MessageTx},
        protocol::{
            AgentId, DecisionIntent, Message, MessageContext, Payload, SessionEvent, TaskGraph,
            TaskId, TaskSpec, WorkerId, WorkerProfile,
        },
    },
    roles::{RoleCollaborationPlan, RoleContribution, RoleRoute, RoleRouteInput, RoleRouter},
};

pub trait CommanderSessionContext: Send + Sync {
    fn get_worker_tx(&self, worker_id: &WorkerId) -> Option<MessageTx>;

    fn list_worker_profiles(&self) -> Vec<WorkerProfile>;

    fn memory_records(&self) -> Vec<MemoryRecord> {
        Vec::new()
    }

    fn record_task_graph_memory(&self, _summary: TaskGraphMemorySummary) {}
}

pub struct CommanderChannels {
    pub receiver: MessageRx,
    pub sender: MessageTx,
    pub event_tx: Option<tokio::sync::mpsc::Sender<SessionEvent>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CommanderOptions {
    pub fast_route_enabled: bool,
    pub fast_route_min_score: f32,
}

pub struct Commander {
    id: AgentId,
    host_id: AgentId,
    state: CommanderState,
    phase: CommanderPhase,
    current_task: Option<CommanderTask>,
    current_context: MessageContext,
    cognition: Box<dyn Cognition + Send>,
    receiver: MessageRx,
    sender: MessageTx,
    event_tx: Option<tokio::sync::mpsc::Sender<SessionEvent>>,
    memory: TaskMemory,
    session_context: Arc<dyn CommanderSessionContext>,
    role_router: RoleRouter,
    fast_route_enabled: bool,
    fast_route_min_score: f32,
    active_collaboration: Option<RoleCollaborationPlan>,
    active_collaboration_task_id: Option<TaskId>,
    pending_support_workers: Vec<WorkerId>,
    completed_support_workers: Vec<WorkerId>,
    collaboration_outputs: HashMap<WorkerId, RoleContribution>,
    active_task_graph: Option<TaskGraphRuntime>,
}

impl Commander {
    pub fn new(
        id: AgentId,
        host_id: AgentId,
        cognition: Box<dyn Cognition + Send>,
        channels: CommanderChannels,
        session_context: Arc<dyn CommanderSessionContext>,
        role_router: RoleRouter,
        options: CommanderOptions,
    ) -> Self {
        Self {
            id,
            host_id,
            state: CommanderState::Idle,
            phase: CommanderPhase::Idle,
            current_task: None,
            current_context: MessageContext::default(),
            cognition,
            receiver: channels.receiver,
            sender: channels.sender,
            event_tx: channels.event_tx,
            memory: TaskMemory::default(),
            session_context,
            role_router,
            fast_route_enabled: options.fast_route_enabled,
            fast_route_min_score: options.fast_route_min_score,
            active_collaboration: None,
            active_collaboration_task_id: None,
            pending_support_workers: Vec::new(),
            completed_support_workers: Vec::new(),
            collaboration_outputs: HashMap::new(),
            active_task_graph: None,
        }
    }

    pub async fn run(&mut self) {
        while self.state != CommanderState::Shutdown {
            let Some(message) = self.receiver.recv().await else {
                break;
            };
            self.handle_message(message).await;

            while self.state == CommanderState::Running {
                self.runtime_step().await;
            }
        }
    }

    async fn runtime_step(&mut self) {
        match self.phase {
            CommanderPhase::Thinking => self.step_thinking().await,
            CommanderPhase::Idle => {
                self.state = CommanderState::Idle;
            }
        }
    }

    async fn handle_message(&mut self, message: Message) {
        let context = message.context.clone();
        match message.payload {
            Payload::HumanCommand { task } => self.on_human_command(task, context),
            Payload::WorkerReportStarted { worker_id, task_id } => {
                self.memory
                    .push_progress(format!("worker {} started {}", worker_id.0, task_id.0));
                self.emit_event(SessionEvent::WorkerStarted {
                    session_id: self.session_id_from_context(&context),
                    worker_id: worker_id.clone(),
                    task_id: task_id.clone(),
                });
                if self.is_collaboration_worker(&worker_id) {
                    self.emit_event(SessionEvent::CollaborationWorkerStarted {
                        session_id: self.session_id_from_context(&context),
                        worker_id,
                        task_id,
                    });
                }
            }
            Payload::WorkerReportFinished {
                worker_id,
                task_id,
                output,
            } => {
                self.memory
                    .push_progress(format!("worker {} finished {}", worker_id.0, task_id.0));
                self.emit_event(SessionEvent::WorkerFinished {
                    session_id: self.session_id_from_context(&context),
                    worker_id: worker_id.clone(),
                    task_id: task_id.clone(),
                });
                if self
                    .handle_collaboration_worker_finished(
                        &worker_id,
                        &task_id,
                        output.as_deref(),
                        &context,
                    )
                    .await
                {
                    return;
                }
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    context,
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text {
                        content: final_worker_message(&task_id, output.as_deref()),
                    },
                ));
                self.phase = CommanderPhase::Idle;
                self.state = CommanderState::Idle;
            }
            Payload::WorkerReportFailed {
                worker_id,
                task_id,
                reason,
            } => {
                self.memory.push_progress(format!(
                    "worker {} failed {}: {}",
                    worker_id.0, task_id.0, reason
                ));
                self.emit_event(SessionEvent::WorkerFinished {
                    session_id: self.session_id_from_context(&context),
                    worker_id,
                    task_id: task_id.clone(),
                });
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    context,
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text {
                        content: format!("任务失败: {} ({reason})", task_id.0),
                    },
                ));
                self.phase = CommanderPhase::Idle;
                self.state = CommanderState::Idle;
            }
            Payload::WorkerReport { report } => {
                self.handle_task_graph_report(report, &context).await;
            }
            Payload::Control { signal, .. } => {
                if matches!(signal, crate::core::protocol::ControlSignal::Shutdown) {
                    self.state = CommanderState::Shutdown;
                }
            }
            _ => {}
        }
    }

    fn on_human_command(&mut self, task: TaskSpec, context: MessageContext) {
        let incoming_task = CommanderTask {
            id: task.id.0,
            description: task.description,
        };
        let worker_profiles = self.session_context.list_worker_profiles();
        log_request_received(&context, &incoming_task.description, &worker_profiles);

        self.memory.clear();
        self.active_collaboration = None;
        self.active_collaboration_task_id = None;
        self.pending_support_workers.clear();
        self.completed_support_workers.clear();
        self.collaboration_outputs.clear();
        self.active_task_graph = None;
        self.memory.push_progress("human command received");
        self.memory
            .set_state("incoming_task_id", incoming_task.id.clone());
        self.current_task = Some(incoming_task);
        self.current_context = context;
        self.phase = CommanderPhase::Thinking;
        self.state = CommanderState::Running;
    }

    async fn step_thinking(&mut self) {
        let role_route = self.current_role_route();
        let worker_profiles = self.session_context.list_worker_profiles();
        log_route_decided(&self.current_context, role_route.as_ref(), &worker_profiles);
        debug!(
            task_id = self
                .current_task
                .as_ref()
                .map(|task| task.id.as_str())
                .unwrap_or("<none>"),
            routing_mode = self
                .current_context
                .metadata
                .get("strategy.routing_mode")
                .map(String::as_str)
                .unwrap_or("<missing>"),
            response_mode = self
                .current_context
                .metadata
                .get("strategy.response_mode")
                .map(String::as_str)
                .unwrap_or("<missing>"),
            primary_role = role_route
                .as_ref()
                .map(|route| route.primary_runtime_role.as_str())
                .unwrap_or("<none>"),
            support_roles = ?role_route
                .as_ref()
                .map(|route| route.support_runtime_roles.as_slice())
                .unwrap_or(&[]),
            fast_route_enabled = self.fast_route_enabled,
            fast_route_allowed = self.context_allows_fast_route(),
            "commander routing started"
        );
        if self
            .try_configured_task_graph_route(role_route.as_ref())
            .await
        {
            debug!("commander routing completed via configured task graph");
            self.phase = CommanderPhase::Idle;
            self.state = CommanderState::Idle;
            return;
        }
        if self.context_allows_fast_route() && self.try_fast_route(role_route.as_ref()).await {
            debug!("commander routing completed via fast worker route");
            self.phase = CommanderPhase::Idle;
            self.state = CommanderState::Idle;
            return;
        }

        let input = self.build_cognition_input(role_route.as_ref());

        let routed = match self.cognition.evaluate(input).await {
            CognitionResult::Success(output) => {
                info!(output = ?output, "commander cognition output");
                decision_intent_from_json(&output, self.current_task.clone())
            }
            CognitionResult::Failure(failure) => {
                warn!(error = %failure.description, "commander cognition failure");
                self.memory
                    .set_error(format!("cognition failure: {}", failure.description));
                if !crate::llm_routing::is_llm_access_denied_message(&failure.description)
                    && !crate::llm_routing::is_llm_provider_error_message(&failure.description)
                    && !crate::llm_routing::is_llm_context_length_error_message(
                        &failure.description,
                    )
                    && self.try_task_graph_route(role_route.as_ref()).await
                {
                    debug!(
                        error = %failure.description,
                        "commander cognition failed; continuing with deterministic role route"
                    );
                    self.phase = CommanderPhase::Idle;
                    self.state = CommanderState::Idle;
                    return;
                }
                let content =
                    if crate::llm_routing::is_llm_access_denied_message(&failure.description) {
                        crate::llm_routing::llm_access_denied_user_message().to_string()
                    } else if crate::llm_routing::is_llm_provider_error_message(
                        &failure.description,
                    ) {
                        crate::llm_routing::llm_provider_error_user_message().to_string()
                    } else {
                        [
                            "## 当前状态",
                            "",
                            "当前 AI 调度没有成功返回结果，所以这轮没有开始可靠的岗位协作。",
                            "",
                            "## 对本轮回答的影响",
                            "",
                            "- 已收到你的问题和本轮选择的资料。",
                            "- 系统暂时无法可靠判断应该走哪条协作链路。",
                            "- 这不是商品、活动或资料本身的判断结果。",
                            "",
                            "## 下一步",
                            "",
                            "- 可以稍后重试同一个问题。",
                            "- 如果连续出现，请联系管理员检查模型服务日志。",
                        ]
                        .join("\n")
                    };
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    self.current_context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text { content },
                ));
                self.phase = CommanderPhase::Idle;
                self.state = CommanderState::Idle;
                return;
            }
        };

        match routed.intent {
            DecisionIntent::ExecuteTask { task } => {
                let decision_route = self.routed_decision_role_route(
                    role_route.as_ref(),
                    routed.target_worker_id.as_ref(),
                );
                let route = decision_route.or_else(|| {
                    role_route
                        .clone()
                        .filter(|route| Self::fallback_has_non_chat_role(Some(route)))
                });
                let routed_graph = routed.task_graph.filter(|graph| {
                    !Self::fallback_has_non_chat_role(route.as_ref())
                        || !Self::graph_is_chat_only(graph)
                });
                let graph = routed_graph.or_else(|| {
                    plan_configured_non_chat_task_graph(&task.id, &task.description, route.as_ref())
                });
                let graph = graph.unwrap_or_else(|| {
                    if route
                        .as_ref()
                        .is_some_and(|route| route.primary_runtime_role != "chat")
                    {
                        debug!(
                            target_worker = routed
                                .target_worker_id
                                .as_ref()
                                .map(|worker| worker.0.as_str())
                                .unwrap_or("<none>"),
                            primary_role = route
                                .as_ref()
                                .map(|route| route.primary_runtime_role.as_str())
                                .unwrap_or("<none>"),
                            "commander fallback planning task graph from routed role"
                        );
                        plan_task_graph(&task.id, &task.description, route.as_ref())
                    } else {
                        debug!(
                            target_worker = routed
                                .target_worker_id
                                .as_ref()
                                .map(|worker| worker.0.as_str())
                                .unwrap_or("<none>"),
                            "commander fallback planning graph from available worker profiles"
                        );
                        self.route_graph_from_worker_profiles(&task.id, &task.description)
                    }
                });
                if self
                    .schedule_planned_task_graph("task_graph", route.as_ref(), graph)
                    .await
                {
                    self.phase = CommanderPhase::Idle;
                    self.state = CommanderState::Idle;
                    return;
                }
                let target_worker = routed
                    .target_worker_id
                    .or_else(|| role_route.as_ref().map(primary_worker_id));
                self.prepare_collaboration(role_route.as_ref(), target_worker.as_ref());
                if !self.dispatch_task_to_worker(task, target_worker) {
                    let _ = self.sender.send(Message::new_with_context(
                        next_message_id(),
                        self.current_context.clone(),
                        self.id.clone(),
                        self.host_id.clone(),
                        Payload::Text {
                            content: "任务分发失败：未找到可用 worker".to_string(),
                        },
                    ));
                }
            }
            DecisionIntent::Ignore => {
                let content = routed
                    .clarification
                    .map(|question| format!("还需要补充一点信息：{question}"))
                    .unwrap_or_else(|| {
                        "我还不能从这句话里确定要处理的电商任务。请补充要分析的商品、活动、资料或希望输出的结果。"
                            .to_string()
                    });
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    self.current_context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text { content },
                ));
            }
            DecisionIntent::IgnoreNewTask => {
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    self.current_context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text {
                        content: "当前还有任务在处理中，请先等这一轮完成，或说明要调整的下一步。"
                            .to_string(),
                    },
                ));
            }
            DecisionIntent::KeepCurrentTask => {
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    self.current_context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text {
                        content:
                            "我会沿用当前对话里的任务继续处理。请直接补充新的要求或下一步目标。"
                                .to_string(),
                    },
                ));
            }
            DecisionIntent::ReplaceCurrentTask { task } => {
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    self.current_context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text {
                        content: format!("建议替换为新任务：{}", task.description),
                    },
                ));
            }
            DecisionIntent::StrategyHint { hint } => {
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    self.current_context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text {
                        content: format!("策略建议：{hint}"),
                    },
                ));
            }
        }

        self.phase = CommanderPhase::Idle;
        self.state = CommanderState::Idle;
    }

    fn routed_decision_role_route(
        &self,
        fallback: Option<&RoleRoute>,
        target_worker_id: Option<&WorkerId>,
    ) -> Option<RoleRoute> {
        let target_role = target_worker_id
            .and_then(|worker_id| worker_id.0.strip_prefix("worker."))
            .filter(|role| !role.trim().is_empty())?;
        if target_role == "chat" && Self::fallback_has_non_chat_role(fallback) {
            return fallback.cloned();
        }
        if fallback.is_some_and(|route| route.primary_runtime_role == target_role) {
            return fallback.cloned();
        }
        self.role_router
            .route_for_runtime_roles(target_role, &Vec::new())
    }

    fn route_graph_from_worker_profiles(&self, task_id: &TaskId, task: &str) -> TaskGraph {
        let mut nodes = self
            .session_context
            .list_worker_profiles()
            .into_iter()
            .filter_map(|worker| {
                worker
                    .worker_id
                    .0
                    .strip_prefix("worker.")
                    .map(|role| (worker.worker_id.clone(), role.to_string()))
            })
            .filter(|(_, role)| role != "chat")
            .map(|(worker_id, role)| crate::core::protocol::TaskNode {
                id: crate::core::protocol::TaskNodeId(role.clone()),
                role,
                worker_id,
                title: "Commander 生成节点".to_string(),
                objective: task.to_string(),
                llm_usage: None,
                skill_autosuggest: None,
                timeout_sec: None,
                input_refs: Vec::new(),
                acceptance: vec![
                    "输出必须围绕本岗位职责给出可复核的业务判断、依据、风险边界和下一步动作。"
                        .to_string(),
                ],
                max_attempts: 1,
            })
            .collect::<Vec<_>>();
        if nodes.is_empty() {
            return plan_task_graph(task_id, task, None);
        }
        nodes.truncate(3);
        TaskGraph {
            graph_id: crate::core::protocol::TaskGraphId(format!("graph-{}", task_id.0)),
            root_task: task.to_string(),
            nodes,
            adaptive_hint: None,
        }
    }

    fn fallback_has_non_chat_role(fallback: Option<&RoleRoute>) -> bool {
        let Some(route) = fallback else {
            return false;
        };
        route.primary_runtime_role != "chat"
            || route
                .support_runtime_roles
                .iter()
                .any(|role| role != "chat")
    }

    fn graph_is_chat_only(graph: &TaskGraph) -> bool {
        !graph.nodes.is_empty() && graph.nodes.iter().all(|node| node.role == "chat")
    }

    fn context_allows_fast_route(&self) -> bool {
        self.fast_route_enabled
            && self
                .current_context
                .metadata
                .get("strategy.routing_mode")
                .is_some_and(|value| value == "fast")
    }

    fn dispatch_task_to_worker(&self, task: TaskSpec, target_worker: Option<WorkerId>) -> bool {
        let target_worker = target_worker.unwrap_or_else(|| WorkerId("worker.chat".to_string()));
        let Some(worker_tx) = self.session_context.get_worker_tx(&target_worker) else {
            warn!(worker_id = %target_worker.0, "no worker tx available for dispatch");
            return false;
        };

        let task_message = Message::new_with_context(
            next_message_id(),
            self.current_context.clone(),
            self.id.clone(),
            AgentId(target_worker.0.clone()),
            Payload::Text {
                content: format!("task:start:{}", task.description),
            },
        );
        worker_tx.send(task_message).is_ok()
    }

    fn emit_route_decided(
        &self,
        route_mode: &str,
        primary_runtime_role: Option<String>,
        support_runtime_roles: Vec<String>,
    ) {
        let support_runtime_roles =
            unique_support_runtime_roles(primary_runtime_role.as_deref(), support_runtime_roles);
        self.emit_event(SessionEvent::RouteDecided {
            session_id: self.session_id_from_context(&self.current_context),
            strategy_id: self.current_context.metadata.get("strategy.id").cloned(),
            response_mode: self
                .current_context
                .metadata
                .get("strategy.response_mode")
                .cloned(),
            collaboration_mode: self
                .current_context
                .metadata
                .get("strategy.collaboration_mode")
                .cloned(),
            learning_level: self
                .current_context
                .metadata
                .get("strategy.learning_level")
                .cloned(),
            route_mode: route_mode.to_string(),
            primary_runtime_role,
            support_runtime_roles,
        });
    }

    fn emit_dispatch_planned_for_graph(&self, route_mode: &str, graph: &TaskGraph) {
        let (primary_runtime_role, support_runtime_roles) =
            dispatch_runtime_roles_from_graph(graph);
        self.emit_dispatch_planned_for_graph_with_roles(
            route_mode,
            graph,
            (primary_runtime_role, support_runtime_roles),
        );
    }

    fn emit_dispatch_planned_for_graph_with_roles(
        &self,
        route_mode: &str,
        graph: &TaskGraph,
        display_roles: (Option<String>, Vec<String>),
    ) {
        let (primary_runtime_role, support_runtime_roles) = display_roles;
        let context_budget = dispatch_context_budget(
            &self.current_context,
            self.session_context.memory_records().len(),
            self.current_task
                .as_ref()
                .map(|task| task.description.as_str())
                .unwrap_or_default(),
        );
        let difficulty = dispatch_difficulty(
            route_mode,
            dispatch_runtime_role_count(primary_runtime_role.as_deref(), &support_runtime_roles),
            &context_budget,
        )
        .to_string();
        let difficulty = dispatch_effective_difficulty(&difficulty, graph);
        let planned_steps = dispatch_steps_from_graph(route_mode, graph, &context_budget);
        let student_visible_explanation = dispatch_student_explanation_with_context(
            route_mode,
            graph.nodes.len(),
            &context_budget,
        );
        let selected_skills = dispatch_selected_skills(route_mode, graph);
        let adaptive_profile = dispatch_adaptive_profile(
            graph,
            DispatchAdaptiveProfileInput {
                route_mode,
                primary_runtime_role: primary_runtime_role.as_deref(),
                support_runtime_roles: &support_runtime_roles,
                selected_skills: &selected_skills,
                context_budget: &context_budget,
                difficulty: &difficulty,
                student_visible_explanation: &student_visible_explanation,
            },
        );
        self.emit_event(SessionEvent::DispatchPlanned {
            session_id: self.session_id_from_context(&self.current_context),
            difficulty,
            route_mode: route_mode.to_string(),
            primary_runtime_role,
            support_runtime_roles,
            selected_skills,
            planned_steps,
            context_budget,
            adaptive_profile,
            reasoning_summary: dispatch_reasoning_summary(route_mode, graph.nodes.len()),
            student_visible_explanation,
        });
    }

    fn task_graph_display_roles(
        &self,
        role_route: Option<&RoleRoute>,
        graph: &TaskGraph,
    ) -> (Option<String>, Vec<String>) {
        if self
            .current_context
            .metadata
            .get("strategy.collaboration_mode")
            .is_some_and(|value| value == "manual" || value == "single")
            && let Some(route) = role_route
        {
            return (
                Some(route.primary_runtime_role.clone()),
                unique_support_runtime_roles(
                    Some(route.primary_runtime_role.as_str()),
                    route.support_runtime_roles.clone(),
                ),
            );
        }
        dispatch_runtime_roles_from_graph(graph)
    }

    fn format_collaboration_outputs(&self) -> String {
        let Some(plan) = &self.active_collaboration else {
            return "角色贡献：none".to_string();
        };

        let mut lines = Vec::new();
        if let Some(contribution) = self.collaboration_outputs.get(&plan.primary.worker_id) {
            lines.push(format!(
                "primary {}: {}",
                plan.primary.runtime_role, contribution.content
            ));
        }
        for assignment in &plan.support {
            if let Some(contribution) = self.collaboration_outputs.get(&assignment.worker_id) {
                lines.push(format!(
                    "support {}: {}",
                    assignment.runtime_role, contribution.content
                ));
            }
        }

        if lines.is_empty() {
            "角色贡献：none".to_string()
        } else {
            format!("角色贡献：\n{}", lines.join("\n"))
        }
    }

    fn session_id_from_context(
        &self,
        context: &MessageContext,
    ) -> crate::core::protocol::SessionId {
        context
            .session_id
            .clone()
            .unwrap_or_else(|| crate::core::protocol::SessionId("unknown-session".to_string()))
    }

    fn emit_event(&self, event: SessionEvent) {
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.try_send(event);
        }
    }

    fn build_cognition_input(&self, role_route: Option<&RoleRoute>) -> CognitionInput {
        let intent = Intent {
            id: self
                .current_task
                .as_ref()
                .map(|task| task.id.clone())
                .unwrap_or_else(|| "commander-idle".to_string()),
            kind: IntentKind::Decision,
            description: self
                .current_task
                .as_ref()
                .map(|task| task.description.clone())
                .unwrap_or_else(|| "evaluate current situation".to_string()),
        };

        let mut metadata = self
            .current_context
            .metadata
            .iter()
            .map(|(key, value)| (format!("request.{key}"), value.clone()))
            .collect::<HashMap<_, _>>();
        metadata.insert("state".to_string(), format!("{:?}", self.state));
        metadata.insert("phase".to_string(), format!("{:?}", self.phase));
        if let Some(role_route) = role_route {
            metadata.insert(
                "role_route.primary_role".to_string(),
                role_route.primary_role.0.clone(),
            );
            metadata.insert(
                "role_route.primary_runtime_role".to_string(),
                role_route.primary_runtime_role.clone(),
            );
            metadata.insert(
                "role_route.support_roles".to_string(),
                role_route
                    .support_roles
                    .iter()
                    .map(|role| role.0.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        let worker_profiles = self.session_context.list_worker_profiles();
        metadata.insert(
            "available_workers.count".to_string(),
            worker_profiles.len().to_string(),
        );
        for (index, profile) in worker_profiles.iter().enumerate() {
            metadata.insert(
                format!("available_workers.{index}.worker_id"),
                profile.worker_id.0.clone(),
            );
            metadata.insert(
                format!("available_workers.{index}.agent_id"),
                profile.agent_id.clone(),
            );
            metadata.insert(
                format!("available_workers.{index}.capabilities"),
                profile.capabilities.join(", "),
            );
            metadata.insert(
                format!("available_workers.{index}.constraints"),
                profile.constraints.join(", "),
            );
            metadata.insert(
                format!("available_workers.{index}.status"),
                profile.status.clone(),
            );
        }
        for (key, value) in &self.memory.state {
            metadata.insert(format!("memory.{key}"), value.clone());
        }

        let mut facts = self
            .memory
            .progress
            .iter()
            .map(|entry| Fact {
                source: "commander.memory".to_string(),
                content: entry.clone(),
                reliability: 1.0,
            })
            .collect::<Vec<_>>();
        extend_runtime_strategy_facts(&self.current_context.metadata, &mut facts);
        extend_context_facts(&self.current_context.metadata, &mut facts);
        facts.extend(worker_profiles.iter().map(|profile| Fact {
            source: "session_context.worker_catalog".to_string(),
            content: profile.summary_line(),
            reliability: 1.0,
        }));
        for (index, record) in self.session_context.memory_records().iter().enumerate() {
            metadata.insert(
                format!("memory.record.{index}.kind"),
                record.kind.as_str().to_string(),
            );
            if let Some(key) = &record.key {
                metadata.insert(format!("memory.record.{index}.key"), key.clone());
            }
            facts.push(Fact {
                source: format!("memory.{}", record.kind.as_str()),
                content: record.content.clone(),
                reliability: f64::from(record.confidence),
            });
        }

        CognitionInput {
            intent,
            context: Context { facts, metadata },
        }
    }

    fn current_role_route(&self) -> Option<RoleRoute> {
        let task = self.current_task.as_ref()?;
        if let Some(route) = self.manual_role_route() {
            return Some(route);
        }
        Some(self.role_router.route(RoleRouteInput {
            message: task.description.clone(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: self.auto_route_max_roles(),
        }))
    }

    fn auto_route_max_roles(&self) -> usize {
        match runtime_response_mode(&self.current_context.metadata) {
            "quick" => 1,
            "execution" => 4,
            _ => 3,
        }
    }

    fn manual_role_route(&self) -> Option<RoleRoute> {
        let collaboration_mode = self
            .current_context
            .metadata
            .get("strategy.collaboration_mode")
            .map(String::as_str)?;
        if !matches!(collaboration_mode, "manual" | "single") {
            return None;
        }

        let selected_roles = selected_runtime_roles(&self.current_context.metadata);
        if selected_roles.is_empty() {
            return None;
        }
        let requested_primary = self
            .current_context
            .metadata
            .get("strategy.primary_role")
            .filter(|role| selected_roles.iter().any(|item| item == *role))
            .cloned();
        let normalized = normalize_requested_runtime_roles(
            collaboration_mode,
            selected_roles,
            requested_primary,
        );
        let primary = normalized.first()?.clone();
        let support = normalized.into_iter().skip(1).collect::<Vec<_>>();

        self.role_router.route_for_runtime_roles(&primary, &support)
    }
}
