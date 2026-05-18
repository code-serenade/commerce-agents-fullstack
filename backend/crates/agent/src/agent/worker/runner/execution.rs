use tracing::warn;

use super::{
    context::build_cognition_input,
    output::parse_worker_role_output,
    output_normalization::finalize_worker_role_output,
    skill_requests::execute_requested_skills,
    types::BoxedAction,
    util::{new_task_id, next_message_id},
};
use crate::{
    action::ActionState,
    agent::worker::{
        Task, TaskMemory, WorkerPhase, WorkerState, action_bridge::decision_to_action,
    },
    agent_session::SessionEventTx,
    cognition::{Cognition, CognitionResult},
    core::{
        messaging::{MessageRx, MessageTx},
        protocol::{
            AgentId, Message, Payload, TaskId, WorkerAssignment, WorkerId, WorkerReportStatus,
        },
    },
    roles::RoleProfile,
    skills::{SkillExecutionResult, SkillExecutorSet},
};

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

        let input =
            build_cognition_input(self.id.clone(), self.phase, &self.memory, &self.role, &task);

        match self.cognition.evaluate(input).await {
            CognitionResult::Success(output) => {
                let output_text = output.to_string();
                self.memory
                    .set_state("last_cognition_output", output_text.clone());
                let skill_results = execute_requested_skills(
                    &self.id,
                    &self.role,
                    &task,
                    &output_text,
                    self.skill_executor.clone(),
                    self.event_tx.as_ref(),
                )
                .await;
                if let Ok(serialized) = serde_json::to_string(&skill_results) {
                    self.memory.set_state("last_skill_results", serialized);
                }
                if !skill_results.is_empty() {
                    self.memory.push_progress(format!(
                        "{} skill request(s) executed",
                        skill_results.len()
                    ));
                }

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
                    let parsed = finalize_worker_role_output(
                        &assignment.role,
                        &task,
                        parsed,
                        &skill_results,
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
            .map(|assignment| Payload::WorkerReport {
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
}

fn assignment_user_task(assignment: &WorkerAssignment) -> String {
    assignment.objective.clone()
}
