use std::{collections::HashMap, sync::Arc};

use tracing::{info, warn};

use super::{
    CommanderPhase, CommanderState, CommanderTask, EvaluationOutcome, RoutedDecision,
    TaskGraphRuntime, TaskMemory, decision_intent_from_json, plan_task_graph,
};
use crate::{
    agent_session::{MemoryRecord, TaskGraphMemorySummary},
    cognition::{Cognition, CognitionInput, CognitionResult, Context, Fact, Intent, IntentKind},
    core::{
        messaging::{MessageRx, MessageTx},
        protocol::{
            AgentId, DecisionIntent, Message, MessageContext, MessageId, Payload, SessionEvent,
            TaskId, TaskSpec, WorkerId, WorkerProfile,
        },
    },
    roles::{
        RoleAssignment, RoleCollaborationPlan, RoleContribution, RoleRoute, RoleRouteInput,
        RoleRouter,
    },
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
        if self.fast_route_enabled && self.try_task_graph_route(role_route.as_ref()).await {
            self.phase = CommanderPhase::Idle;
            self.state = CommanderState::Idle;
            return;
        }
        if self.fast_route_enabled && self.try_fast_route(role_route.as_ref()) {
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
                RoutedDecision {
                    intent: DecisionIntent::Ignore,
                    target_worker_id: None,
                    clarification: None,
                }
            }
        };

        match routed.intent {
            DecisionIntent::ExecuteTask { task } => {
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
                    .map(|question| format!("需要补充信息后再路由：{question}"))
                    .unwrap_or_else(|| "已评估当前输入：本轮无需执行新任务。".to_string());
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
                        content: "已评估该请求：当前策略是暂不接收新任务。".to_string(),
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
                        content: "已评估当前输入：继续保持当前任务。".to_string(),
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

    fn dispatch_assignment_to_worker(
        &self,
        assignment: crate::core::protocol::WorkerAssignment,
    ) -> bool {
        let Some(worker_tx) = self.session_context.get_worker_tx(&assignment.worker_id) else {
            warn!(
                worker_id = %assignment.worker_id.0,
                "no worker tx available for task graph assignment"
            );
            return false;
        };

        let task_message = Message::new_with_context(
            next_message_id(),
            self.current_context.clone(),
            self.id.clone(),
            AgentId(assignment.worker_id.0.clone()),
            Payload::WorkerAssignment { assignment },
        );
        worker_tx.send(task_message).is_ok()
    }

    async fn try_task_graph_route(&mut self, role_route: Option<&RoleRoute>) -> bool {
        let Some(current_task) = self.current_task.clone() else {
            return false;
        };
        if !self.force_collaboration()
            && role_route
                .map(primary_route_score)
                .is_some_and(|score| score < self.fast_route_min_score)
        {
            return false;
        }

        let task_id = TaskId(current_task.id);
        let graph = plan_task_graph(&task_id, &current_task.description, role_route);
        if graph.nodes.iter().any(|node| {
            self.session_context
                .get_worker_tx(&node.worker_id)
                .is_none()
        }) {
            return false;
        }

        self.memory
            .push_progress(format!("task graph planned: {} nodes", graph.nodes.len()));
        self.emit_route_decided(
            "task_graph",
            graph.nodes.first().map(|node| node.role.clone()),
            graph
                .nodes
                .iter()
                .skip(1)
                .map(|node| node.role.clone())
                .collect(),
        );
        self.emit_event(SessionEvent::TaskGraphPlanned {
            session_id: self.session_id_from_context(&self.current_context),
            graph: graph.clone(),
        });
        self.active_collaboration = None;
        self.active_collaboration_task_id = None;
        self.pending_support_workers.clear();
        self.completed_support_workers.clear();
        self.collaboration_outputs.clear();
        let runtime = TaskGraphRuntime::new(graph).with_response_mode(
            self.current_context
                .metadata
                .get("strategy.response_mode")
                .map(String::as_str)
                .or_else(|| {
                    self.current_context
                        .metadata
                        .get("strategy.id")
                        .map(String::as_str)
                }),
        );
        self.schedule_task_graph_runtime(runtime).await;
        true
    }

    fn force_collaboration(&self) -> bool {
        self.current_context
            .metadata
            .get("strategy.collaboration_mode")
            .is_some_and(|value| value == "manual" || value == "single")
            || self
                .current_context
                .metadata
                .get("strategy.id")
                .is_some_and(|value| value == "collab")
    }

    async fn handle_task_graph_report(
        &mut self,
        report: crate::core::protocol::WorkerReport,
        context: &MessageContext,
    ) {
        self.memory.push_progress(format!(
            "task graph node {} reported by {}",
            report.node_id.0, report.worker_id.0
        ));
        self.emit_event(SessionEvent::WorkerFinished {
            session_id: self.session_id_from_context(context),
            worker_id: report.worker_id.clone(),
            task_id: report.task_id.clone(),
        });
        let Some(mut runtime) = self.active_task_graph.take() else {
            if report.role == "chat" {
                let content = direct_chat_report_output(&report);
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text { content },
                ));
                self.phase = CommanderPhase::Idle;
                self.state = CommanderState::Idle;
            }
            return;
        };
        self.emit_event(SessionEvent::TaskNodeReported {
            session_id: self.session_id_from_context(context),
            graph_id: report.graph_id.clone(),
            node_id: report.node_id.clone(),
            worker_id: report.worker_id.clone(),
            report: Box::new(report.clone()),
        });
        let graph_id = runtime.graph.graph_id.clone();
        let worker_id = report.worker_id.clone();
        let node_id = report.node_id.clone();
        let outcome = runtime.apply_report(report);

        match outcome {
            EvaluationOutcome::Passed(evaluation) => {
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodePassed {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                });
            }
            EvaluationOutcome::Rework {
                evaluation,
                instruction,
            } => {
                let attempt = runtime.attempts.get(&node_id).copied().unwrap_or(1) + 1;
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodeReworkRequested {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                    worker_id,
                    attempt,
                    instruction,
                });
            }
            EvaluationOutcome::Failed(evaluation) => {
                let reason = evaluation.reasons.join("; ");
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodeFailed {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                    reason,
                });
            }
        }

        self.schedule_task_graph_runtime(runtime).await;
    }

    async fn schedule_task_graph_runtime(&mut self, mut runtime: TaskGraphRuntime) {
        if runtime.is_finished() {
            self.finish_task_graph(runtime).await;
            return;
        }

        let ready_nodes = runtime.ready_nodes();
        for node in ready_nodes {
            self.emit_event(SessionEvent::TaskNodeReady {
                session_id: self.session_id_from_context(&self.current_context),
                graph_id: runtime.graph.graph_id.clone(),
                node_id: node.id.clone(),
                worker_id: node.worker_id.clone(),
            });
            let assignment = runtime.build_assignment(&node);
            if self.dispatch_assignment_to_worker(assignment.clone()) {
                runtime.mark_running(&assignment);
                self.emit_event(SessionEvent::WorkerStarted {
                    session_id: self.session_id_from_context(&self.current_context),
                    worker_id: assignment.worker_id.clone(),
                    task_id: assignment.task_id.clone(),
                });
                self.emit_event(SessionEvent::TaskNodeStarted {
                    session_id: self.session_id_from_context(&self.current_context),
                    graph_id: assignment.graph_id,
                    node_id: assignment.node_id,
                    worker_id: assignment.worker_id,
                    attempt: assignment.attempt,
                });
            }
        }

        self.active_task_graph = Some(runtime);
    }

    async fn finish_task_graph(&mut self, runtime: TaskGraphRuntime) {
        let content = runtime
            .direct_chat_output()
            .unwrap_or_else(|| runtime.synthesize());
        let process = runtime.process_summary();
        self.session_context
            .record_task_graph_memory(runtime.memory_summary());
        self.emit_event(SessionEvent::TaskGraphFinished {
            session_id: self.session_id_from_context(&self.current_context),
            graph_id: runtime.graph.graph_id.clone(),
            process: Box::new(process),
        });
        let _ = self.sender.send(Message::new_with_context(
            next_message_id(),
            self.current_context.clone(),
            self.id.clone(),
            self.host_id.clone(),
            Payload::Text { content },
        ));
        self.active_task_graph = None;
        self.phase = CommanderPhase::Idle;
        self.state = CommanderState::Idle;
    }

    fn try_fast_route(&mut self, role_route: Option<&RoleRoute>) -> bool {
        let Some(role_route) = role_route else {
            return false;
        };
        if primary_route_score(role_route) < self.fast_route_min_score {
            return false;
        }
        let Some(current_task) = self.current_task.clone() else {
            return false;
        };

        let target_worker = primary_worker_id(role_route);
        if self.session_context.get_worker_tx(&target_worker).is_none() {
            return false;
        }

        let task = TaskSpec {
            id: TaskId(current_task.id),
            description: current_task.description,
        };
        self.prepare_collaboration(Some(role_route), Some(&target_worker));
        self.emit_route_decided(
            "fast_worker",
            Some(role_route.primary_runtime_role.clone()),
            role_route.support_runtime_roles.clone(),
        );
        if !self.dispatch_task_to_worker(task, Some(target_worker)) {
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

        true
    }

    fn prepare_collaboration(
        &mut self,
        role_route: Option<&RoleRoute>,
        target_worker: Option<&WorkerId>,
    ) {
        let Some(role_route) = role_route else {
            return;
        };
        let plan = RoleCollaborationPlan::from_route(role_route);
        if target_worker != Some(&plan.primary.worker_id) {
            return;
        }

        self.memory.set_state(
            "role_route.primary_runtime_role",
            plan.primary.runtime_role.clone(),
        );
        self.memory.set_state(
            "role_route.support_runtime_roles",
            plan.support
                .iter()
                .map(|role| role.runtime_role.clone())
                .collect::<Vec<_>>()
                .join(", "),
        );
        if plan.support.is_empty() {
            return;
        }
        self.emit_event(SessionEvent::CollaborationStarted {
            session_id: self.session_id_from_context(&self.current_context),
            primary_worker_id: plan.primary.worker_id.clone(),
            support_worker_ids: plan
                .support
                .iter()
                .map(|assignment| assignment.worker_id.clone())
                .collect(),
        });
        self.active_collaboration = Some(plan);
        self.active_collaboration_task_id = None;
        self.pending_support_workers.clear();
        self.completed_support_workers.clear();
        self.collaboration_outputs.clear();
    }

    async fn handle_collaboration_worker_finished(
        &mut self,
        worker_id: &WorkerId,
        task_id: &TaskId,
        output: Option<&str>,
        context: &MessageContext,
    ) -> bool {
        let Some(plan) = self.active_collaboration.clone() else {
            return false;
        };

        if worker_id == &plan.primary.worker_id && !plan.support.is_empty() {
            self.active_collaboration_task_id = Some(task_id.clone());
            if let Some(output) = output {
                self.collaboration_outputs.insert(
                    worker_id.clone(),
                    build_contribution(&plan.primary, task_id, output),
                );
            }
            self.emit_event(SessionEvent::CollaborationWorkerFinished {
                session_id: self.session_id_from_context(context),
                worker_id: worker_id.clone(),
                task_id: task_id.clone(),
                content: output.map(ToString::to_string),
            });
            self.dispatch_support_workers(&plan, task_id).await;
            return true;
        }

        if self
            .pending_support_workers
            .iter()
            .any(|item| item == worker_id)
        {
            self.pending_support_workers
                .retain(|item| item != worker_id);
            self.completed_support_workers.push(worker_id.clone());
            if let Some(output) = output
                && let Some(assignment) = plan.assignment_for(worker_id)
            {
                self.collaboration_outputs.insert(
                    worker_id.clone(),
                    build_contribution(assignment, task_id, output),
                );
            }
            self.emit_event(SessionEvent::CollaborationWorkerFinished {
                session_id: self.session_id_from_context(context),
                worker_id: worker_id.clone(),
                task_id: task_id.clone(),
                content: output.map(ToString::to_string),
            });
            if self.pending_support_workers.is_empty() {
                let primary_task_id = self
                    .active_collaboration_task_id
                    .clone()
                    .unwrap_or_else(|| task_id.clone());
                self.send_collaboration_done(context, &primary_task_id)
                    .await;
            }
            return true;
        }

        false
    }

    async fn dispatch_support_workers(
        &mut self,
        plan: &RoleCollaborationPlan,
        primary_task_id: &TaskId,
    ) {
        self.pending_support_workers = plan
            .support
            .iter()
            .map(|assignment| assignment.worker_id.clone())
            .collect();

        let mut dispatched = Vec::new();
        let root_task = self
            .current_task
            .as_ref()
            .map(|task| task.description.as_str())
            .unwrap_or_default();
        let primary_output = self
            .collaboration_outputs
            .get(&plan.primary.worker_id)
            .map(|contribution| contribution.content.as_str())
            .filter(|content| !content.trim().is_empty())
            .unwrap_or("主回答岗位已完成初稿，但没有返回可读摘要。");
        for assignment in &plan.support {
            let task = TaskSpec {
                id: TaskId(format!(
                    "{}:support:{}",
                    primary_task_id.0, assignment.runtime_role
                )),
                description: format!(
                    "原始用户任务：{root_task}\n\n主回答岗位初稿：{primary_output}\n\n作为{}支持角色，请基于同一上下文补充本岗位的独立判断、证据、风险和要交给主岗汇总的行动建议；不要只说“接收上游结果”。",
                    assignment.runtime_role
                ),
            };
            if self.dispatch_task_to_worker(task, Some(assignment.worker_id.clone())) {
                dispatched.push(assignment.worker_id.clone());
            }
        }

        self.pending_support_workers
            .retain(|worker_id| dispatched.iter().any(|item| item == worker_id));
        if self.pending_support_workers.is_empty() {
            let context = self.current_context.clone();
            self.send_collaboration_done(&context, primary_task_id)
                .await;
        }
    }

    async fn send_collaboration_done(&mut self, context: &MessageContext, task_id: &TaskId) {
        let support_workers = self
            .completed_support_workers
            .iter()
            .map(|worker| worker.0.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let content = if support_workers.is_empty() {
            self.active_collaboration
                .as_ref()
                .and_then(|plan| self.collaboration_outputs.get(&plan.primary.worker_id))
                .map(|contribution| contribution.content.clone())
                .unwrap_or_else(|| format!("任务已完成: {}", task_id.0))
        } else {
            self.synthesize_collaboration_output(task_id, &support_workers)
        };
        let _ = self.sender.send(Message::new_with_context(
            next_message_id(),
            context.clone(),
            self.id.clone(),
            self.host_id.clone(),
            Payload::Text { content },
        ));
        self.emit_event(SessionEvent::CollaborationFinished {
            session_id: self.session_id_from_context(context),
        });
        self.active_collaboration = None;
        self.active_collaboration_task_id = None;
        self.pending_support_workers.clear();
        self.completed_support_workers.clear();
        self.collaboration_outputs.clear();
        self.phase = CommanderPhase::Idle;
        self.state = CommanderState::Idle;
    }

    fn emit_route_decided(
        &self,
        route_mode: &str,
        primary_runtime_role: Option<String>,
        support_runtime_roles: Vec<String>,
    ) {
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
            model_name: self.current_context.metadata.get("model.name").cloned(),
            route_mode: route_mode.to_string(),
            primary_runtime_role,
            support_runtime_roles,
        });
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

    fn synthesize_collaboration_output(&self, task_id: &TaskId, support_workers: &str) -> String {
        let task = self
            .current_task
            .as_ref()
            .map(|task| task.description.clone())
            .unwrap_or_default();
        let Some(plan) = &self.active_collaboration else {
            return synthesize_user_facing_collaboration_report(
                task_id,
                &task,
                None,
                support_workers,
                &[],
                self.current_context
                    .metadata
                    .get("context.selected_summary"),
            );
        };

        let mut contributions = Vec::new();
        if let Some(contribution) = self.collaboration_outputs.get(&plan.primary.worker_id) {
            contributions.push(contribution.clone());
        }
        for assignment in &plan.support {
            if let Some(contribution) = self.collaboration_outputs.get(&assignment.worker_id) {
                contributions.push(contribution.clone());
            }
        }

        synthesize_user_facing_collaboration_report(
            task_id,
            &task,
            Some(plan),
            support_workers,
            &contributions,
            self.current_context
                .metadata
                .get("context.selected_summary"),
        )
    }

    fn is_collaboration_worker(&self, worker_id: &WorkerId) -> bool {
        self.active_collaboration
            .as_ref()
            .map(|plan| plan.assignment_for(worker_id).is_some())
            .unwrap_or(false)
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

fn selected_runtime_roles(metadata: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    metadata
        .get("strategy.selected_roles")
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

fn extend_runtime_strategy_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let mode = runtime_response_mode(metadata);
    let answer_style = match mode {
        "quick" => {
            "回答模式：快速简答。优先给直接结论、关键理由和最多 3 个下一步动作；减少铺垫、过程展开和长篇教学，除非用户明确要求详细推导。"
        }
        "execution" => {
            "回答模式：丰富执行。输出完整业务方案，优先使用 Markdown 表格呈现岗位分工、行动清单、负责人、验收指标、风险边界和复盘节奏。"
        }
        _ => {
            "回答模式：教学模式。默认面向电商学生和新手，先给可用结论，再用通俗语言解释概念、判断过程、为什么这么做和下一步怎么练。"
        }
    };
    facts.push(Fact {
        source: "runtime.answer_style".to_string(),
        content: answer_style.to_string(),
        reliability: 1.0,
    });

    let explanation_depth = match runtime_learning_level(metadata) {
        "beginner" => {
            "讲解深度：低。尽量少用术语，遇到 ROI、转化率、漏斗、客单价等专业词要用白话解释，并给简单例子。"
        }
        "professional" => {
            "讲解深度：高。保留关键指标、推导路径、约束假设、风险边界和可复核口径，适合汇报和专业复盘。"
        }
        _ => "讲解深度：中。按电商岗位实操标准解释，术语不过度展开，但关键概念和动作口径要说清楚。",
    };
    facts.push(Fact {
        source: "runtime.explanation_depth".to_string(),
        content: explanation_depth.to_string(),
        reliability: 1.0,
    });

    let role_dispatch = match metadata
        .get("strategy.collaboration_mode")
        .map(String::as_str)
        .unwrap_or("auto")
    {
        "single" => {
            "岗位分配：指定单岗。只让用户指定的一个虚拟员工负责本轮回答，不额外扩展协作岗。"
        }
        "manual" => "岗位分配：指定多岗。主岗负责综合回答，协作岗分别产出独立判断和交接内容。",
        _ if mode == "quick" => {
            "岗位分配：自动分配。快速简答默认倾向单岗短链路，除非用户明确要求多岗位协作。"
        }
        _ => {
            "岗位分配：自动分配。后端根据问题判断一个或多个岗位，复杂任务应体现虚拟员工分工和主岗汇总。"
        }
    };
    facts.push(Fact {
        source: "runtime.role_dispatch".to_string(),
        content: role_dispatch.to_string(),
        reliability: 1.0,
    });
}

fn runtime_response_mode(metadata: &std::collections::BTreeMap<String, String>) -> &str {
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

fn runtime_learning_level(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    metadata
        .get("strategy.learning_level")
        .map(String::as_str)
        .filter(|value| matches!(*value, "beginner" | "higher_vocational" | "professional"))
        .unwrap_or("higher_vocational")
}

fn extend_context_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    if let Some(history) = metadata
        .get("conversation.history_summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        facts.push(Fact {
            source: "conversation.history".to_string(),
            content: format!(
                "同一会话历史上下文如下。回答追问、补充、继续、按前面数据等问题时，必须优先结合这段历史；不要要求用户重复已经在这里提供的信息。\n{history}"
            ),
            reliability: 1.0,
        });
    }
    extend_named_context_fact(
        metadata,
        facts,
        "context.selected_summary",
        "commerce.selected_product_context",
        "用户本轮手动选择的商品/知识上下文如下；只有存在这段事实时，才可按指定商品回答。\n",
    );
    if metadata
        .get("context.selected_summary")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        facts.push(Fact {
            source: "commerce.unselected_product_context".to_string(),
            content:
                "本轮没有用户手动选择的商品/知识上下文。可以结合用户文本、同一会话历史和工作区资料理解问题，但不得把历史商品、测试商品或默认商品当成本轮指定商品；需要单品级分析时应要求用户显式选择或提供商品。"
                    .to_string(),
            reliability: 1.0,
        });
    }
    extend_named_context_fact(
        metadata,
        facts,
        "context.sources_json",
        "commerce.selected_product_context.sources",
        "",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "context.combined_summary",
        "request.combined_context",
        "本轮合并上下文摘要如下。它可能包含会话历史、工作区资料或 RAG 降级信息；只有 source 为 commerce.selected_product_context 时才代表用户手动指定商品。\n",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContext.summary",
        "workspace.context_summary",
        "本轮工作区上下文可用性摘要如下；它不是用户手动选择的商品资料。\n",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.summary",
        "workspace.context_pack_summary",
        "本轮工作区可读资料摘要如下；它不是用户手动选择的商品资料。\n",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.items_json",
        "workspace.context_pack.items",
        "",
    );
}

fn extend_named_context_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
    prefix: &str,
) {
    let Some(value) = metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    facts.push(Fact {
        source: source.to_string(),
        content: format!("{prefix}{value}"),
        reliability: 1.0,
    });
}

fn primary_worker_id(role_route: &RoleRoute) -> WorkerId {
    WorkerId(format!("worker.{}", role_route.primary_runtime_role))
}

fn primary_route_score(role_route: &RoleRoute) -> f32 {
    if !role_route.scores.is_empty() && role_route.scores.iter().all(|score| score.score >= 90.0) {
        return role_route
            .scores
            .iter()
            .find(|score| score.role_id == role_route.primary_role)
            .map(|score| score.score)
            .unwrap_or(0.0);
    }
    role_route
        .scores
        .iter()
        .find(|score| score.role_id == role_route.primary_role)
        .map(|score| score.score)
        .unwrap_or(0.0)
}

fn final_worker_message(task_id: &TaskId, output: Option<&str>) -> String {
    output
        .filter(|value| !value.trim().is_empty())
        .map(extract_worker_content)
        .unwrap_or_else(|| format!("任务已完成: {}", task_id.0))
}

fn direct_chat_report_output(report: &crate::core::protocol::WorkerReport) -> String {
    report
        .role_output
        .as_ref()
        .map(|output| output.summary.trim())
        .filter(|summary| !summary.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            let content = report.content.trim();
            (!content.is_empty()).then(|| content.to_string())
        })
        .unwrap_or_else(|| "我在，可以直接告诉我你想聊什么或需要我帮你做什么。".to_string())
}

fn build_contribution(
    assignment: &RoleAssignment,
    task_id: &TaskId,
    output: &str,
) -> RoleContribution {
    RoleContribution::new(assignment, task_id.clone(), extract_worker_content(output))
}

fn extract_worker_content(output: &str) -> String {
    let trimmed = output.trim();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return trimmed.to_string();
    };

    value
        .pointer("/decision/action/parameters/answer")
        .and_then(|value| value.as_str())
        .or_else(|| {
            value
                .pointer("/role_output/summary")
                .and_then(|value| value.as_str())
        })
        .or_else(|| {
            value
                .pointer("/role_output")
                .and_then(|value| value.as_str())
        })
        .or_else(|| {
            value
                .pointer("/rationale/primary")
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string)
        .unwrap_or_else(|| trimmed.to_string())
}

fn synthesize_user_facing_collaboration_report(
    task_id: &TaskId,
    task: &str,
    plan: Option<&RoleCollaborationPlan>,
    support_workers: &str,
    contributions: &[RoleContribution],
    selected_summary: Option<&String>,
) -> String {
    let roles = collaboration_roles(plan, contributions);
    let primary_role = plan
        .map(|plan| plan.primary.runtime_role.as_str())
        .or_else(|| roles.first().map(String::as_str))
        .unwrap_or("ops");
    let support_roles = roles
        .iter()
        .filter(|role| role.as_str() != primary_role)
        .map(String::as_str)
        .collect::<Vec<_>>();
    let metrics = extract_live_conversion_metrics_for_collaboration(task);

    let mut lines = Vec::new();
    lines.push("# 多智能体协作结论".to_string());
    lines.push(format!("**本轮请求**：{}", task.trim()));
    lines.push(format!("**任务编号**：{}", task_id.0));
    if let Some(selected_summary) = selected_summary
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        lines.push(format!(
            "**商品上下文**：本轮使用用户手动选择的预设商品资料：{}",
            compact_text(selected_summary, 140)
        ));
    } else {
        lines.push("**商品上下文**：本轮未检测到用户手动选择的预设商品；按未指定商品链路回答，不注入历史商品、测试商品或默认商品。".to_string());
    }
    if !support_workers.trim().is_empty() {
        lines.push(format!("**协作岗位**：{}", support_workers.trim()));
    }
    lines.push(String::new());

    lines.push("## 综合回答".to_string());
    if is_live_conversion_collaboration_task(task) {
        lines.push(format!(
            "- **团队判断**：{}先收敛主结论，{}分别补充指标、内容、服务、技术或预算边界；这不是多个回答的简单拼接，而是一组虚拟员工围绕同一上下文协作形成的 final_report。",
            role_display_name_for_collaboration(primary_role),
            if support_roles.is_empty() {
                "协作岗位".to_string()
            } else {
                support_roles
                    .iter()
                    .map(|role| role_display_name_for_collaboration(role))
                    .collect::<Vec<_>>()
                    .join("、")
            }
        ));
        if metrics.is_empty() {
            lines.push("- **健康度边界**：当前问题缺少完整可复核指标，只能先按曝光、点击、加购、下单、付款、退款和咨询承接定义判断框架。".to_string());
        } else {
            lines.push("- **健康度边界**：这组数据能说明本轮漏斗表现，但不能单独判定行业健康度；还需要行业基准、历史同类直播、客单价、毛利、投放成本、退款和客服咨询承接一起复核。".to_string());
            lines.extend(
                live_conversion_metric_lines(&metrics)
                    .into_iter()
                    .map(|line| format!("  - {line}")),
            );
        }
    } else {
        lines.push(format!(
            "- **团队判断**：{}负责把各岗位输入收敛成可执行结论，{}负责补齐证据、动作、风险和待补数据，最终沉淀为 `final_report`。",
            role_display_name_for_collaboration(primary_role),
            if support_roles.is_empty() {
                "协作岗位".to_string()
            } else {
                support_roles
                    .iter()
                    .map(|role| role_display_name_for_collaboration(role))
                    .collect::<Vec<_>>()
                    .join("、")
            }
        ));
    }
    lines.push("- **上下文处理**：同一会话里的历史问题会作为任务背景参与理解，但不会替代本轮的商品选择；只有 `context.selected_summary` 存在时，才进入指定商品链路。".to_string());
    lines.push(String::new());

    lines.push("## 虚拟员工分工（各岗位独立输出）".to_string());
    lines.push("| 岗位 | 负责问题 | 独立判断 | 交给主岗的内容 |".to_string());
    lines.push("| --- | --- | --- | --- |".to_string());
    for role in &roles {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            role_display_name_for_collaboration(role),
            role_responsibility_for_collaboration(role),
            role_independent_judgement(role, task),
            contribution_handoff_for_role(role, contributions)
        ));
    }
    lines.push(String::new());

    let contribution_lines = user_facing_contribution_lines(contributions);
    if !contribution_lines.is_empty() {
        lines.push("## 岗位过程摘要".to_string());
        lines.extend(contribution_lines);
        lines.push(String::new());
    }

    lines.push("## 主岗综合方案".to_string());
    lines.push(format!(
        "- **主回答岗位**：{}负责把各岗位独立判断合成为最终结论，协作岗位只补证据、风险、素材、数据和执行建议，不替代主岗汇总。",
        role_display_name_for_collaboration(primary_role)
    ));
    if is_live_conversion_collaboration_task(task) {
        lines.push("- P0 数据口径：把曝光、点击、加购、下单、付款、退款、咨询量、渠道来源和样本周期补齐，先确认每段转化率是否可复核。".to_string());
        lines.push("- P1 转化承接：针对最低或最关键的漏斗段，分别检查直播开场、卖点讲解、价格利益、信任证明、客服响应和支付阻力。".to_string());
        lines.push("- P2 团队复盘：每天按同一口径沉淀数据、岗位判断、动作负责人和下一轮验证结果，避免只凭单次回答做放量决策。".to_string());
    } else {
        lines.push("- P0 先统一问题口径、业务目标、可用上下文和不能编造的数据边界。".to_string());
        lines.push(
            "- P1 由对应岗位分别产出数据判断、运营动作、内容素材、服务承接和技术复核。".to_string(),
        );
        lines.push("- P2 主岗位把结果收敛成优先级、负责人、验证指标和下一轮追问入口。".to_string());
    }
    lines.push(String::new());

    lines.push("## 交付物".to_string());
    lines.push("- **final_report**：本轮完整回答应沉淀为 Markdown 交付物，保留综合结论、岗位分工、过程依据、风险边界和下一步清单。".to_string());
    lines.push("- **过程依据**：各岗位独立输出会作为报告依据保存，后续用户可在同一历史对话中继续追问、回顾或导出。".to_string());

    lines.join("\n")
}

fn collaboration_roles(
    plan: Option<&RoleCollaborationPlan>,
    contributions: &[RoleContribution],
) -> Vec<String> {
    let mut roles = Vec::new();
    if let Some(plan) = plan {
        push_unique_role(&mut roles, &plan.primary.runtime_role);
        for assignment in &plan.support {
            push_unique_role(&mut roles, &assignment.runtime_role);
        }
    }
    for contribution in contributions {
        push_unique_role(&mut roles, &contribution.runtime_role);
    }
    if roles.is_empty() {
        roles.push("ops".to_string());
    }
    roles
}

fn push_unique_role(roles: &mut Vec<String>, role: &str) {
    let role = role.trim();
    if !role.is_empty() && !roles.iter().any(|item| item == role) {
        roles.push(role.to_string());
    }
}

fn role_display_name_for_collaboration(role: &str) -> &'static str {
    match role {
        "ops" => "运营策略师",
        "data" => "数据分析师",
        "creative" => "内容创意师",
        "service" => "客户成功专员",
        "engineering" => "技术架构师",
        "douyin_ads" => "千川投放专员",
        "accounting" => "财务分析师",
        "design" => "设计师",
        "web" => "Web/SEO 专家",
        "chat" => "通用助手",
        _ => "虚拟员工",
    }
}

fn role_independent_judgement(role: &str, task: &str) -> &'static str {
    if is_live_conversion_collaboration_task(task) {
        return match role {
            "data" => "先计算曝光、点击、加购、下单、付款等漏斗口径和缺失指标。",
            "ops" => "把各段漏斗表现收敛成优先级、动作节奏和复盘方式。",
            "creative" => "把异常段转成直播开场、卖点、标题和短视频脚本测试。",
            "service" => "检查咨询响应、售后承诺、退款和履约边界对转化的影响。",
            "engineering" => "确认埋点、报表刷新、数据接口和失败回滚是否可验证。",
            "douyin_ads" => "拆人群包、素材组、出价、预算节奏和 CPA/ROI 止损线。",
            "accounting" => "把毛利、投放成本、退款和优惠成本纳入健康度判断。",
            "design" => "检查主图、详情页、直播视觉和行动按钮是否承接卖点。",
            "web" => "补充搜索入口、FAQ 和页面承接对自然流量的影响。",
            _ => "围绕本岗位边界补充证据、风险和下一步。",
        };
    }

    match role {
        "data" => "先统一指标口径、样本周期、归因方式和待补数据。",
        "ops" => "把业务目标收敛成优先级、执行节奏、负责人和复盘指标。",
        "creative" => "把策略转成标题、脚本、卖点和素材测试假设。",
        "service" => "补齐客服问答、售后承诺、退款/物流边界和升级 SOP。",
        "engineering" => "确认埋点、接口、报表、数据刷新和回滚边界。",
        "douyin_ads" => "拆解人群、素材、出价、预算节奏和投放止损线。",
        "accounting" => "评估毛利、预算、CPA/ROI 和现金流约束。",
        "design" => "把卖点、信任证明和行动按钮落到视觉承接。",
        "web" => "补关键词、FAQ、页面结构和自然流量承接。",
        _ => "围绕本岗位边界补充判断和交接内容。",
    }
}

fn role_responsibility_for_collaboration(role: &str) -> &'static str {
    match role {
        "data" => "指标口径、样本周期、异常段和可复核证据。",
        "ops" => "业务目标、优先级、执行节奏和最终汇总。",
        "creative" => "卖点、标题、脚本、素材测试和内容承接。",
        "service" => "客服问答、售后承诺、退款物流和服务风险。",
        "engineering" => "埋点、接口、报表刷新、链路验证和回滚。",
        "douyin_ads" => "千川人群、素材组、出价、预算和止损线。",
        "accounting" => "毛利、预算、CPA、ROI 和现金流边界。",
        "design" => "主图、详情页、直播视觉和行动按钮承接。",
        "web" => "搜索入口、FAQ、页面结构和自然流量承接。",
        _ => "本岗位职责内的判断、证据、风险和交接内容。",
    }
}

fn contribution_handoff_for_role(role: &str, contributions: &[RoleContribution]) -> String {
    contributions
        .iter()
        .find(|contribution| contribution.runtime_role == role)
        .map(|contribution| sanitize_contribution_text(&contribution.content))
        .filter(|content| !content.is_empty())
        .unwrap_or_else(|| default_handoff_for_role(role).to_string())
}

fn default_handoff_for_role(role: &str) -> &'static str {
    match role {
        "data" => "把可复核指标、异常段和待补数据交给主岗。",
        "ops" => "把各岗位输入合成主结论、执行节奏和 final_report。",
        "creative" => "把内容钩子、脚本方向和 A/B 测试假设交给团队。",
        "service" => "把咨询承接、售后边界和服务风险交给团队。",
        "engineering" => "把埋点、接口、报表和回滚边界交给团队。",
        "douyin_ads" => "把投放人群、素材、出价和止损线交给团队。",
        "accounting" => "把预算、毛利和 ROI 边界交给团队。",
        "design" => "把视觉承接和页面测试点交给团队。",
        "web" => "把搜索入口和页面承接建议交给团队。",
        _ => "把本岗位结论、证据和风险交给团队。",
    }
}

fn user_facing_contribution_lines(contributions: &[RoleContribution]) -> Vec<String> {
    contributions
        .iter()
        .filter_map(|contribution| {
            let content = sanitize_contribution_text(&contribution.content);
            (!content.is_empty()).then(|| {
                format!(
                    "- **{}**：{}",
                    role_display_name_for_collaboration(&contribution.runtime_role),
                    content
                )
            })
        })
        .collect()
}

fn sanitize_contribution_text(content: &str) -> String {
    let compact = compact_text(content, 180);
    let lower = compact.to_ascii_lowercase();
    if compact.is_empty()
        || compact.contains("用户任务：")
        || compact.contains("最终汇总：")
        || compact.contains("输出边界")
        || compact.contains("输出结构")
        || lower.contains("support role")
        || lower.contains("data role should provide")
        || lower.contains("service role output lacks")
        || lower.contains("engineering role output lacks")
        || lower.contains("worker reported failed status")
        || lower.starts_with("任务已完成:")
        || (compact.contains("接收") && compact.contains("继续处理"))
    {
        return String::new();
    }
    compact
}

fn compact_text(content: &str, max_chars: usize) -> String {
    let mut text = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if text.chars().count() > max_chars {
        text = text.chars().take(max_chars).collect::<String>();
        text.push_str("...");
    }
    text
}

fn is_live_conversion_collaboration_task(task: &str) -> bool {
    contains_any_for_collaboration(
        task,
        &["直播间", "直播", "转化", "转化率", "点击", "加购", "付款"],
    )
}

fn contains_any_for_collaboration(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

fn extract_live_conversion_metrics_for_collaboration(task: &str) -> HashMap<&'static str, f64> {
    const LABELS: [&str; 5] = ["曝光", "点击", "加购", "下单", "付款"];
    let mut metrics = HashMap::new();
    for label in LABELS {
        if let Some(value) = extract_number_after_label_for_collaboration(task, label) {
            metrics.insert(label, value);
        }
    }
    metrics
}

fn extract_number_after_label_for_collaboration(text: &str, label: &str) -> Option<f64> {
    let start = text.find(label)? + label.len();
    let tail = &text[start..];
    let mut number = String::new();
    let mut started = false;
    for ch in tail.chars() {
        if ch.is_ascii_digit() || (started && ch == '.') {
            started = true;
            number.push(ch);
        } else if started || !ch.is_whitespace() {
            break;
        }
    }
    number.parse::<f64>().ok()
}

fn live_conversion_metric_lines(metrics: &HashMap<&'static str, f64>) -> Vec<String> {
    let mut lines = Vec::new();
    if let (Some(exposure), Some(click)) = (metrics.get("曝光"), metrics.get("点击")) {
        lines.push(format!(
            "曝光到点击约 {:.1}%，先看直播封面、标题、主播开场和人群匹配是否足够吸引。",
            ratio_percent_for_collaboration(*click, *exposure)
        ));
    }
    if let (Some(click), Some(cart)) = (metrics.get("点击"), metrics.get("加购")) {
        lines.push(format!(
            "点击到加购约 {:.1}%，重点检查卖点、价格利益、信任证明和讲解节奏是否承接住兴趣。",
            ratio_percent_for_collaboration(*cart, *click)
        ));
    }
    if let (Some(cart), Some(order)) = (metrics.get("加购"), metrics.get("下单")) {
        lines.push(format!(
            "加购到下单约 {:.1}%，需要排查优惠门槛、库存、运费、活动规则和犹豫点。",
            ratio_percent_for_collaboration(*order, *cart)
        ));
    }
    if let (Some(order), Some(paid)) = (metrics.get("下单"), metrics.get("付款")) {
        lines.push(format!(
            "下单到付款约 {:.1}%，如果接近 100% 说明支付环节阻力不大；后续还要看退款、投诉和履约。",
            ratio_percent_for_collaboration(*paid, *order)
        ));
    }
    if let (Some(exposure), Some(paid)) = (metrics.get("曝光"), metrics.get("付款")) {
        lines.push(format!(
            "曝光到付款总转化约 {:.2}%，单看该链路不是结论，必须结合行业基准、客单价、毛利和投放成本判断健康度。",
            ratio_percent_for_collaboration(*paid, *exposure)
        ));
    }
    lines
}

fn ratio_percent_for_collaboration(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        return 0.0;
    }
    numerator / denominator * 100.0
}

fn next_message_id() -> MessageId {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(1);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let sequence = NEXT_MESSAGE_COUNTER.fetch_add(1, Ordering::Relaxed);

    MessageId(format!("msg-{millis}-{sequence}"))
}

#[cfg(test)]
mod tests {
    use super::synthesize_user_facing_collaboration_report;
    use crate::{
        core::protocol::{TaskId, WorkerId},
        roles::{RoleAssignment, RoleCollaborationPlan, RoleContribution, RoleId},
    };

    #[test]
    fn legacy_collaboration_synthesis_is_user_facing_team_report() {
        let task = "请用Agent集群方式判断：直播间曝光1000、点击80、加购20、下单8、付款7是否健康，不引用任何预设商品，并说明各虚拟员工如何协作。";
        let plan = RoleCollaborationPlan {
            primary: assignment("ops"),
            support: vec![
                assignment("data"),
                assignment("creative"),
                assignment("service"),
            ],
        };
        let contributions = vec![
            contribution(
                "ops",
                "运营主岗先判断直播间健康度，再把数据、内容和服务输入收敛成复盘节奏。",
            ),
            contribution(
                "data",
                "数据岗确认曝光到点击、点击到加购、加购到下单、下单到付款和总转化都需要分段复核。",
            ),
            contribution(
                "creative",
                "内容岗把点击到加购的承接问题转成直播钩子、标题和脚本测试。",
            ),
            contribution(
                "service",
                "服务岗补充咨询响应、售后承诺和退款边界，避免营销承诺超过履约能力。",
            ),
        ];

        let output = synthesize_user_facing_collaboration_report(
            &TaskId("task-live-collab".to_string()),
            task,
            Some(&plan),
            "worker.data, worker.creative, worker.service",
            &contributions,
            None,
        );

        assert!(output.contains("虚拟员工分工"));
        assert!(output.contains("综合回答"));
        assert!(output.contains("各岗位独立输出"));
        assert!(output.contains("主岗综合方案"));
        assert!(output.contains("| 岗位 | 负责问题 | 独立判断 | 交给主岗的内容 |"));
        assert!(output.contains("团队"));
        assert!(output.contains("final_report"));
        assert!(output.contains("曝光到点击约 8.0%"));
        assert!(output.contains("点击到加购约 25.0%"));
        assert!(output.contains("加购到下单约 40.0%"));
        assert!(output.contains("下单到付款约 87.5%"));
        assert!(output.contains("曝光到付款总转化约 0.70%"));
        assert!(!output.contains("用户任务："));
        assert!(!output.contains("最终汇总："));
        assert!(!output.contains("输出边界"));
        assert!(!output.contains("输出结构"));
        assert!(!output.contains("support role"));
        assert!(!output.contains("任务已完成: task-"));
        assert!(!output.contains("千乐咖啡"));
    }

    #[test]
    fn legacy_collaboration_synthesis_hides_internal_evaluator_noise() {
        let task = "请用Agent集群方式判断直播间转化是否健康，不引用任何预设商品。";
        let plan = RoleCollaborationPlan {
            primary: assignment("ops"),
            support: vec![
                assignment("data"),
                assignment("service"),
                assignment("engineering"),
            ],
        };
        let contributions = vec![
            contribution("ops", "任务已完成: task-123"),
            contribution("data", "data role should provide at least two findings"),
            contribution(
                "service",
                "service role output lacks service-specific content",
            ),
            contribution(
                "engineering",
                "engineering role output lacks technical content",
            ),
        ];

        let output = synthesize_user_facing_collaboration_report(
            &TaskId("task-noise".to_string()),
            task,
            Some(&plan),
            "worker.data, worker.service, worker.engineering",
            &contributions,
            None,
        );
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("虚拟员工分工"));
        assert!(output.contains("各岗位独立输出"));
        assert!(output.contains("主岗综合方案"));
        assert!(output.contains("final_report"));
        assert!(!lower.contains("data role should provide"));
        assert!(!lower.contains("service role output lacks"));
        assert!(!lower.contains("engineering role output lacks"));
        assert!(!output.contains("任务已完成: task-123"));
    }

    fn assignment(runtime_role: &str) -> RoleAssignment {
        RoleAssignment {
            role_id: RoleId(format!("role.{runtime_role}")),
            runtime_role: runtime_role.to_string(),
            worker_id: WorkerId(format!("worker.{runtime_role}")),
        }
    }

    fn contribution(runtime_role: &str, content: &str) -> RoleContribution {
        RoleContribution::new(
            &assignment(runtime_role),
            TaskId(format!("task-{runtime_role}")),
            content,
        )
    }
}
