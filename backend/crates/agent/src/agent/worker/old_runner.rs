use std::collections::HashMap;

use tracing::warn;

use super::{Task, TaskMemory, WorkerPhase, WorkerState, action_bridge::decision_to_action};
mod output_policy;
use output_policy::{
    apply_category_boundary_output, apply_category_team_output_shape,
    category_skill_boundary_label, default_role_finding, default_role_findings,
    default_role_recommendation, default_role_recommendations, default_role_risks,
    is_category_non_blocking_skill, normalize_role_specific_findings,
    normalize_role_specific_recommendations, normalize_role_specific_risks,
    required_findings_for_role, required_recommendations_for_role, role_display_name,
    role_requires_risk,
};

use crate::{
    action::{Action, ActionResult, ActionState},
    agent_session::SessionEventTx,
    cognition::{Cognition, CognitionInput, CognitionResult, Context, Fact, Intent, IntentKind},
    core::{
        messaging::{MessageRx, MessageTx},
        protocol::{
            AgentId, Message, MessageId, Payload, RoleWorkOutput, SessionEvent, TaskId,
            WorkerAssignment, WorkerId, WorkerReportStatus,
        },
    },
    roles::{RoleProfile, RolePromptBuilder, RolePromptInput, RoleSkillCatalog},
    skills::{
        SkillContext, SkillExecutionResult, SkillExecutionStatus, SkillExecutor, SkillExecutorSet,
        SkillId, SkillRequest,
    },
};

pub type BoxedAction = Box<dyn Action<Result = ActionResult<String, String, ()>> + Send>;

const CATEGORY_LEVEL_BOUNDARY_MARKER: &str = "category_level_boundary";

pub struct Worker {
    id: AgentId,
    role: RoleProfile,
    state: WorkerState,
    phase: WorkerPhase,
    current_task: Option<Task>,
    current_action: Option<BoxedAction>,
    cognition: Box<dyn Cognition + Send>,
    receiver: MessageRx,
    sender: MessageTx,
    memory: TaskMemory,
    skill_executor: Option<SkillExecutorSet>,
    event_tx: Option<SessionEventTx>,
}

impl Worker {
    pub fn new(
        role: RoleProfile,
        cognition: Box<dyn Cognition + Send>,
        receiver: MessageRx,
        sender: MessageTx,
        event_tx: Option<SessionEventTx>,
        skill_executor: Option<SkillExecutorSet>,
    ) -> Self {
        let id = AgentId(format!("worker.{}", role.runtime_role));

        Self {
            id,
            role,
            state: WorkerState::Idle,
            phase: WorkerPhase::Thinking,
            current_task: None,
            current_action: None,
            cognition,
            receiver,
            sender,
            memory: TaskMemory::default(),
            skill_executor,
            event_tx,
        }
    }

    pub async fn run(&mut self) {
        while self.state != WorkerState::Shutdown {
            let Some(message) = self.receiver.recv().await else {
                break;
            };
            self.handle_message(message);

            while self.state == WorkerState::Running {
                self.runtime_step().await;
            }
        }
    }

    async fn runtime_step(&mut self) {
        match self.phase {
            WorkerPhase::Thinking => self.step_thinking().await,
            WorkerPhase::Acting => self.step_acting(),
            WorkerPhase::Finished => self.handle_task_finished(),
            WorkerPhase::Failed => self.handle_task_failed(),
        }
    }

    fn handle_message(&mut self, message: Message) {
        let context = message.context.clone();
        let requester = message.from.clone();
        let payload = match message.payload {
            Payload::WorkerAssignment { assignment } => {
                let description = assignment_user_task(&assignment);
                self.start_task(Task {
                    id: assignment.task_id.0.clone(),
                    context,
                    description,
                    requester,
                    assignment: Some(assignment),
                });
                return;
            }
            Payload::Text { content } => content,
            _ => return,
        };
        let payload = payload.trim();

        match payload {
            "control:start" => {
                if self.current_task.is_some() {
                    self.state = WorkerState::Running;
                }
            }
            "control:pause" => {
                self.state = WorkerState::Paused;
                if let Some(action) = self.current_action.as_mut() {
                    action.suspend();
                }
            }
            "control:resume" => {
                self.state = WorkerState::Running;
                if let Some(action) = self.current_action.as_mut() {
                    action.resume();
                }
            }
            "control:shutdown" => {
                self.state = WorkerState::Shutdown;
            }
            _ => {
                if let Some(description) = payload.strip_prefix("task:start:") {
                    self.start_task(Task {
                        id: new_task_id(),
                        context: context.clone(),
                        description: description.trim().to_string(),
                        requester: requester.clone(),
                        assignment: None,
                    });
                }
            }
        }
    }

    fn start_task(&mut self, task: Task) {
        let _ = self.sender.send(Message {
            id: next_message_id(),
            context: task.context.clone(),
            from: self.id.clone(),
            to: task.requester.clone(),
            payload: Payload::WorkerReportStarted {
                worker_id: WorkerId(self.id.0.clone()),
                task_id: TaskId(task.id.clone()),
            },
        });
        self.current_task = Some(task);
        self.current_action = None;
        self.phase = WorkerPhase::Thinking;
        self.state = WorkerState::Running;
        self.memory.clear();
        self.memory.push_progress("task accepted");
    }

    async fn step_thinking(&mut self) {
        let Some(task) = self.current_task.clone() else {
            self.state = WorkerState::Idle;
            return;
        };

        self.memory.push_progress("thinking");

        let input = CognitionInput {
            intent: Intent {
                id: task.id.clone(),
                kind: IntentKind::Planning,
                description: task.description.clone(),
            },
            context: self.build_cognition_context(&task),
        };

        match self.cognition.evaluate(input).await {
            CognitionResult::Success(output) => {
                let output_text = output.to_string();
                self.memory
                    .set_state("last_cognition_output", output_text.clone());
                self.execute_requested_skills(&task, &output_text).await;
                if let Some(action) = decision_to_action(&output) {
                    self.current_action = Some(action);
                    self.phase = WorkerPhase::Acting;
                    self.memory.push_progress("action selected");
                } else {
                    self.phase = WorkerPhase::Finished;
                    self.memory.push_progress("no further action");
                }
            }
            CognitionResult::Failure(failure) => {
                warn!(error = %failure.description, "worker cognition failure");
                self.phase = WorkerPhase::Failed;
                self.memory
                    .set_error(format!("cognition failure: {}", failure.description));
            }
        }
    }

    fn step_acting(&mut self) {
        let Some(action) = self.current_action.as_mut() else {
            self.phase = WorkerPhase::Failed;
            self.memory
                .set_error("acting phase entered without current_action");
            warn!("worker acting phase without action");
            return;
        };

        self.memory.push_progress("acting");
        action.drive();

        match action.state() {
            ActionState::Completed => {
                if let Some(result) = action.take_result() {
                    self.memory
                        .set_state("last_action_status", format!("{:?}", result.status));
                }
                self.current_action = None;
                self.phase = WorkerPhase::Finished;
                self.memory.push_progress("action completed");
            }
            ActionState::Aborted => {
                self.phase = WorkerPhase::Failed;
                self.memory.set_error("action aborted");
                warn!("worker action aborted");
            }
            ActionState::Eligible | ActionState::Active | ActionState::Suspended => {}
        }
    }

    fn handle_task_finished(&mut self) {
        let Some(task) = self.current_task.take() else {
            self.state = WorkerState::Idle;
            return;
        };
        let context = task.context.clone();
        let requester = task.requester.clone();

        let _ = self.sender.send(Message {
            id: next_message_id(),
            context,
            from: self.id.clone(),
            to: requester,
            payload: task
                .assignment
                .as_ref()
                .map(|assignment| {
                    let parsed = self
                        .memory
                        .state
                        .get("last_cognition_output")
                        .map(|output| parse_worker_role_output(output))
                        .unwrap_or_default();
                    let skill_results = self
                        .memory
                        .state
                        .get("last_skill_results")
                        .and_then(|value| {
                            serde_json::from_str::<Vec<SkillExecutionResult>>(value).ok()
                        })
                        .unwrap_or_default();
                    let parsed = normalize_worker_role_output(
                        &assignment.role,
                        &task,
                        merge_skill_results(parsed, &skill_results, &task),
                    );

                    Payload::WorkerReport {
                        report: crate::core::protocol::WorkerReport {
                            graph_id: assignment.graph_id.clone(),
                            node_id: assignment.node_id.clone(),
                            task_id: TaskId(task.id.clone()),
                            worker_id: WorkerId(self.id.0.clone()),
                            role: assignment.role.clone(),
                            content: parsed.content,
                            role_output: parsed.role_output,
                            evidence: parsed.evidence,
                            risks: parsed.risks,
                            open_questions: parsed.open_questions,
                            status: crate::core::protocol::WorkerReportStatus::Completed,
                        },
                    }
                })
                .unwrap_or_else(|| Payload::WorkerReportFinished {
                    worker_id: WorkerId(self.id.0.clone()),
                    task_id: TaskId(task.id),
                    output: self.memory.state.get("last_cognition_output").cloned(),
                }),
        });
        self.phase = WorkerPhase::Thinking;
        self.state = WorkerState::Idle;
    }

    fn handle_task_failed(&mut self) {
        let Some(task) = self.current_task.take() else {
            self.state = WorkerState::Idle;
            return;
        };

        let reason = self
            .memory
            .last_error
            .clone()
            .unwrap_or_else(|| "unknown error".to_string());
        let context = task.context.clone();
        let requester = task.requester.clone();
        let payload = task
            .assignment
            .as_ref()
            .map(|assignment| {
                if should_downgrade_category_worker_failure(&task, &reason) {
                    Payload::WorkerReport {
                        report: category_boundary_failure_report(
                            assignment, &task, &reason, &self.id,
                        ),
                    }
                } else {
                    Payload::WorkerReport {
                        report: crate::core::protocol::WorkerReport {
                            graph_id: assignment.graph_id.clone(),
                            node_id: assignment.node_id.clone(),
                            task_id: TaskId(task.id.clone()),
                            worker_id: WorkerId(self.id.0.clone()),
                            role: assignment.role.clone(),
                            content: reason.clone(),
                            role_output: None,
                            evidence: Vec::new(),
                            risks: Vec::new(),
                            open_questions: vec![reason.clone()],
                            status: WorkerReportStatus::Failed,
                        },
                    }
                }
            })
            .unwrap_or_else(|| Payload::WorkerReportFailed {
                worker_id: WorkerId(self.id.0.clone()),
                task_id: TaskId(task.id.clone()),
                reason: reason.clone(),
            });
        let _ = self.sender.send(Message {
            id: next_message_id(),
            context,
            from: self.id.clone(),
            to: requester,
            payload,
        });
        self.current_action = None;
        self.phase = WorkerPhase::Thinking;
        self.state = WorkerState::Idle;
        warn!(task_id = %task.id, reason = %reason, "worker reporting task failed");
    }
    fn build_cognition_context(&self, task: &Task) -> Context {
        build_context(self.id.clone(), self.phase, &self.memory, &self.role, task)
    }

    async fn execute_requested_skills(&mut self, task: &Task, output: &str) {
        let requests = resolve_skill_requests(&self.role, task, output);
        if requests.is_empty() {
            return;
        }

        let Some(executor) = self.skill_executor.clone() else {
            let mut results = Vec::new();
            for (index, request) in requests.into_iter().enumerate() {
                let request_id = format!("{}-skill-{}", task.id, index + 1);
                self.emit_skill_requested(task, &request_id, &request);
                let result = skill_executor_unavailable_result(request_id, request.skill_id);
                self.emit_skill_result(task, &result);
                results.push(result);
            }
            if let Ok(serialized) = serde_json::to_string(&results) {
                self.memory.set_state("last_skill_results", serialized);
            }
            self.memory
                .push_progress("skill requests failed: executor unavailable");
            return;
        };

        let mut results = Vec::new();
        for (index, request) in requests.into_iter().enumerate() {
            let request_id = format!("{}-skill-{}", task.id, index + 1);
            self.emit_skill_requested(task, &request_id, &request);
            if !RoleSkillCatalog::staged().can_role_use_skill(&self.role, &request.skill_id) {
                self.memory.push_progress(format!(
                    "skill {} rejected for role {}",
                    request.skill_id, self.role.runtime_role
                ));
                let result =
                    skill_not_allowed_result(request_id, request.skill_id, &self.role.runtime_role);
                self.emit_skill_result(task, &result);
                results.push(result);
                continue;
            }
            let context = build_skill_context(&self.id, &self.role, task, &request, &request_id);
            let runtime_request = SkillRequest::new(request_id, request.skill_id, request.input)
                .with_context(context);
            let result = executor.execute(runtime_request).await;
            self.emit_skill_result(task, &result);
            self.memory.push_progress(format!(
                "skill {} completed with {:?}",
                result.skill_id.as_str(),
                result.status
            ));
            results.push(result);
        }

        if let Ok(serialized) = serde_json::to_string(&results) {
            self.memory.set_state("last_skill_results", serialized);
        }
    }

    fn emit_skill_requested(&self, task: &Task, request_id: &str, request: &ParsedSkillRequest) {
        let Some(session_id) = &task.context.session_id else {
            return;
        };
        self.emit_event(SessionEvent::SkillRequested {
            session_id: session_id.clone(),
            worker_id: WorkerId(self.id.0.clone()),
            task_id: TaskId(task.id.clone()),
            request_id: request_id.to_string(),
            skill_id: request.skill_id.clone(),
            role: self.role.runtime_role.clone(),
            reason: request.reason.clone(),
        });
    }

    fn emit_skill_result(&self, task: &Task, result: &SkillExecutionResult) {
        let Some(session_id) = &task.context.session_id else {
            return;
        };
        let is_non_blocking_category_gap = is_non_blocking_category_skill_gap(result, task);
        let output = if is_non_blocking_category_gap {
            Some(category_level_skill_boundary_output(result))
        } else {
            result.output.clone()
        };
        let status = if is_non_blocking_category_gap {
            skill_status_name(SkillExecutionStatus::Deferred)
        } else {
            skill_status_name(result.status)
        };
        let summary = if is_non_blocking_category_gap {
            Some(
                "品类级边界：本轮未手动选择预设商品或外部执行条件不足，已按非阻塞能力边界记录。"
                    .to_string(),
            )
        } else {
            result.summary.clone().map(compact_event_text)
        };
        let error = if is_non_blocking_category_gap {
            None
        } else {
            result.error.clone().map(compact_event_text)
        };
        self.emit_event(SessionEvent::SkillResult {
            session_id: session_id.clone(),
            worker_id: WorkerId(self.id.0.clone()),
            task_id: TaskId(task.id.clone()),
            request_id: result.request_id.clone(),
            skill_id: result.skill_id.as_str().to_string(),
            role: self.role.runtime_role.clone(),
            status: status.to_string(),
            summary,
            error,
            output,
            execution_mode: metadata_string(&result.metadata, "execution_mode"),
            risk_level: metadata_string(&result.metadata, "risk_level"),
            approval_required: result
                .metadata
                .get("approval_required")
                .and_then(serde_json::Value::as_bool),
        });
    }

    fn emit_event(&self, event: SessionEvent) {
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.try_send(event);
        }
    }
}

include!("runner/skill_resolution.rs");
include!("runner/autosuggest.rs");
include!("runner/inference.rs");
include!("runner/output_normalization.rs");
