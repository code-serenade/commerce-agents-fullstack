use std::time::Duration;

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
            AgentId, Message, Payload, TaskId, WorkerAssignment, WorkerDiagnostic, WorkerId,
            WorkerReportStatus,
        },
    },
    llm_routing::{is_llm_access_denied_message, llm_access_denied_user_message},
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
    runtime: WorkerRuntimeConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkerRuntimeConfig {
    pub cognition_timeout: Duration,
}

impl Default for WorkerRuntimeConfig {
    fn default() -> Self {
        Self {
            cognition_timeout: Duration::from_secs(45),
        }
    }
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
        Self::new_with_config(
            role,
            cognition,
            receiver,
            sender,
            event_tx,
            skill_executor,
            WorkerRuntimeConfig::default(),
        )
    }

    pub(crate) fn new_with_config(
        role: RoleProfile,
        cognition: Box<dyn Cognition + Send>,
        receiver: MessageRx,
        sender: MessageTx,
        event_tx: Option<SessionEventTx>,
        skill_executor: Option<SkillExecutorSet>,
        runtime: WorkerRuntimeConfig,
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
            runtime,
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

        let timeout = self.cognition_timeout_for_task(&task);
        match tokio::time::timeout(timeout, self.cognition.evaluate(input)).await {
            Err(_) => {
                let reason = format!(
                    "worker cognition timed out after {}",
                    format_duration(timeout)
                );
                warn!(
                    task_id = %task.id,
                    worker_id = %self.id.0,
                    role = %self.role.runtime_role,
                    timeout_sec = timeout.as_secs_f64(),
                    "worker cognition timeout"
                );
                self.phase = WorkerPhase::Failed;
                self.memory.set_state("failure_kind", "timeout");
                self.memory.set_error(reason);
            }
            Ok(CognitionResult::Success(output)) => {
                self.handle_cognition_success(&task, output).await;
            }
            Ok(CognitionResult::Failure(failure)) => {
                warn!(error = %failure.description, "worker cognition failure");
                self.phase = WorkerPhase::Failed;
                if is_llm_access_denied_message(&failure.description) {
                    self.memory.set_state("failure_kind", "llm_access_denied");
                    self.memory.set_error(format!(
                        "llm_access_denied: {}",
                        llm_access_denied_user_message()
                    ));
                    return;
                }
                self.memory
                    .set_error(format!("cognition failure: {}", failure.description));
            }
        }
    }

    async fn handle_cognition_success(&mut self, task: &Task, output: serde_json::Value) {
        let output_text = output.to_string();
        self.memory
            .set_state("last_cognition_output", output_text.clone());
        let skill_results = execute_requested_skills(
            &self.id,
            &self.role,
            task,
            &output_text,
            self.skill_executor.clone(),
            self.event_tx.as_ref(),
        )
        .await;
        if let Ok(serialized) = serde_json::to_string(&skill_results) {
            self.memory.set_state("last_skill_results", serialized);
        }
        if !skill_results.is_empty() {
            self.memory
                .push_progress(format!("{} skill request(s) executed", skill_results.len()));
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

    fn cognition_timeout_for_task(&self, task: &Task) -> Duration {
        task.assignment
            .as_ref()
            .and_then(|assignment| assignment.timeout_sec)
            .map(|seconds| Duration::from_secs(seconds.max(1)))
            .unwrap_or(self.runtime.cognition_timeout)
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
                    let diagnostics =
                        with_context_pack_diagnostics(parsed.diagnostics, &task, &self.id);

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
                            diagnostics,
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
        let failure_kind = self.memory.state.get("failure_kind").map(String::as_str);
        let status = if failure_kind.is_some_and(|kind| kind == "timeout") {
            WorkerReportStatus::TimedOut
        } else {
            WorkerReportStatus::Failed
        };
        let evidence = failure_kind
            .map(|kind| vec![format!("failure_kind:{kind}")])
            .unwrap_or_default();
        let diagnostics = failure_kind
            .map(|kind| worker_failure_diagnostic(&task, &self.id, kind, &reason, &status))
            .into_iter()
            .collect::<Vec<_>>();
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
                    evidence: evidence.clone(),
                    risks: Vec::new(),
                    open_questions: vec![reason.clone()],
                    diagnostics: diagnostics.clone(),
                    status,
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
    let root_task = assignment.root_task.trim();
    let objective = assignment.objective.trim();

    if root_task.is_empty() {
        return objective.to_string();
    }
    if objective.is_empty() || objective == root_task {
        return root_task.to_string();
    }

    format!("{root_task}\n\n当前岗位任务：\n{objective}")
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{}s", duration.as_secs())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

fn worker_failure_diagnostic(
    task: &Task,
    worker_id: &AgentId,
    failure_kind: &str,
    reason: &str,
    status: &WorkerReportStatus,
) -> WorkerDiagnostic {
    let mut fields = std::collections::BTreeMap::new();
    push_diagnostic_field(&mut fields, "failure_kind", failure_kind);
    push_diagnostic_field(&mut fields, "status", worker_status_label(status));
    push_diagnostic_field(&mut fields, "worker_id", &worker_id.0);
    push_diagnostic_field(&mut fields, "task_id", &task.id);
    if let Some(assignment) = task.assignment.as_ref() {
        push_diagnostic_field(&mut fields, "role", &assignment.role);
        push_diagnostic_field(&mut fields, "node_id", &assignment.node_id.0);
        push_diagnostic_field(&mut fields, "graph_id", &assignment.graph_id.0);
    }
    push_diagnostic_field(&mut fields, "reason", reason);

    WorkerDiagnostic {
        kind: "failure".to_string(),
        source: failure_kind.to_string(),
        fields,
    }
}

fn with_context_pack_diagnostics(
    mut diagnostics: Vec<WorkerDiagnostic>,
    task: &Task,
    worker_id: &AgentId,
) -> Vec<WorkerDiagnostic> {
    if let Some(diagnostic) = context_pack_failure_diagnostic(task, worker_id) {
        push_unique_diagnostic(&mut diagnostics, diagnostic);
    }
    diagnostics
}

fn push_unique_diagnostic(items: &mut Vec<WorkerDiagnostic>, diagnostic: WorkerDiagnostic) {
    if !items.contains(&diagnostic) {
        items.push(diagnostic);
    }
}

fn context_pack_failure_diagnostic(task: &Task, worker_id: &AgentId) -> Option<WorkerDiagnostic> {
    let metadata = &task.context.metadata;
    let readiness = metadata
        .get("workspaceContextPack.readiness")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())?;
    let item_count = parse_usize_metadata(metadata.get("workspaceContextPack.item_count"))
        .unwrap_or_else(|| csv_count(metadata.get("workspaceContextPack.refs")));
    let omitted_count = json_array_len(metadata.get("workspaceContextPack.omitted_json"));
    let warning_count = json_array_len(metadata.get("workspaceContextPack.warnings_json"));
    let issue_counts =
        context_pack_item_issue_counts(metadata.get("workspaceContextPack.items_json"));

    let has_issue = readiness != "ready"
        || omitted_count > 0
        || warning_count > 0
        || issue_counts.metadata_only > 0
        || issue_counts.extraction_pending > 0
        || issue_counts.extraction_failed > 0
        || issue_counts.extraction_unsupported > 0;
    if !has_issue {
        return None;
    }

    let failure_kind = context_pack_failure_kind(readiness, omitted_count, &issue_counts);
    let mut fields = std::collections::BTreeMap::new();
    push_diagnostic_field(&mut fields, "failure_kind", failure_kind);
    push_diagnostic_field(&mut fields, "status", "partial");
    push_diagnostic_field(&mut fields, "readiness", readiness);
    push_diagnostic_field(&mut fields, "worker_id", &worker_id.0);
    push_diagnostic_field(&mut fields, "task_id", &task.id);
    if let Some(assignment) = task.assignment.as_ref() {
        push_diagnostic_field(&mut fields, "role", &assignment.role);
        push_diagnostic_field(&mut fields, "node_id", &assignment.node_id.0);
        push_diagnostic_field(&mut fields, "graph_id", &assignment.graph_id.0);
    }
    push_diagnostic_field(&mut fields, "item_count", item_count.to_string());
    push_diagnostic_field(&mut fields, "omitted_count", omitted_count.to_string());
    push_diagnostic_field(&mut fields, "warning_count", warning_count.to_string());
    push_diagnostic_field(
        &mut fields,
        "metadata_only_count",
        issue_counts.metadata_only.to_string(),
    );
    push_diagnostic_field(
        &mut fields,
        "extraction_pending_count",
        issue_counts.extraction_pending.to_string(),
    );
    push_diagnostic_field(
        &mut fields,
        "extraction_failed_count",
        issue_counts.extraction_failed.to_string(),
    );
    push_diagnostic_field(
        &mut fields,
        "extraction_unsupported_count",
        issue_counts.extraction_unsupported.to_string(),
    );
    if let Some(preview) =
        context_pack_warning_preview(metadata.get("workspaceContextPack.warnings_json"))
    {
        push_diagnostic_field(&mut fields, "warning_preview", preview);
    }

    Some(WorkerDiagnostic {
        kind: "failure".to_string(),
        source: failure_kind.to_string(),
        fields,
    })
}

fn context_pack_failure_kind(
    readiness: &str,
    omitted_count: usize,
    issue_counts: &ContextPackIssueCounts,
) -> &'static str {
    if issue_counts.extraction_failed > 0 {
        "context_pack_extraction_failed"
    } else if issue_counts.extraction_unsupported > 0 {
        "context_pack_extraction_unsupported"
    } else if issue_counts.extraction_pending > 0 {
        "context_pack_extraction_pending"
    } else if omitted_count > 0 {
        "context_pack_omitted"
    } else if readiness == "empty" {
        "context_pack_empty"
    } else {
        "context_pack_partial"
    }
}

#[derive(Debug, Default)]
struct ContextPackIssueCounts {
    metadata_only: usize,
    extraction_pending: usize,
    extraction_failed: usize,
    extraction_unsupported: usize,
}

fn context_pack_item_issue_counts(value: Option<&String>) -> ContextPackIssueCounts {
    let Some(value) = value else {
        return ContextPackIssueCounts::default();
    };
    let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(value) else {
        return ContextPackIssueCounts::default();
    };
    let mut counts = ContextPackIssueCounts::default();
    for item in items {
        match item
            .get("contentKind")
            .or_else(|| item.get("content_kind"))
            .and_then(serde_json::Value::as_str)
        {
            Some("metadata_only") => counts.metadata_only += 1,
            Some("extraction_pending") => counts.extraction_pending += 1,
            Some("extraction_failed") => counts.extraction_failed += 1,
            Some("extraction_unsupported") => counts.extraction_unsupported += 1,
            _ => {}
        }
    }
    counts
}

fn context_pack_warning_preview(value: Option<&String>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    serde_json::from_str::<Vec<String>>(value)
        .ok()
        .and_then(|items| items.into_iter().find(|item| !item.trim().is_empty()))
        .or_else(|| Some(value.to_string()))
        .map(|item| compact_diagnostic_text(&item, 220))
}

fn parse_usize_metadata(value: Option<&String>) -> Option<usize> {
    value?.trim().parse::<usize>().ok()
}

fn json_array_len(value: Option<&String>) -> usize {
    let Some(value) = value else {
        return 0;
    };
    serde_json::from_str::<Vec<serde_json::Value>>(value)
        .map(|items| items.len())
        .unwrap_or(0)
}

fn csv_count(value: Option<&String>) -> usize {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .count()
        })
        .unwrap_or(0)
}

fn compact_diagnostic_text(value: &str, max_chars: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut text = value.chars().take(max_chars).collect::<String>();
    text.push_str("...");
    text
}

fn push_diagnostic_field(
    fields: &mut std::collections::BTreeMap<String, String>,
    key: impl Into<String>,
    value: impl AsRef<str>,
) {
    let value = value.as_ref().trim();
    if !value.is_empty() {
        fields.insert(key.into(), value.to_string());
    }
}

fn worker_status_label(status: &WorkerReportStatus) -> &'static str {
    match status {
        WorkerReportStatus::Completed => "completed",
        WorkerReportStatus::Failed => "failed",
        WorkerReportStatus::TimedOut => "timed_out",
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_trait::async_trait;
    use serde_json::json;
    use tokio::sync::mpsc;

    use super::{Worker, WorkerRuntimeConfig};
    use crate::{
        agent::worker::runner::util::next_message_id,
        cognition::{Cognition, CognitionFailure, CognitionInput, CognitionResult, FailureReason},
        core::protocol::{
            AgentId, Message, MessageContext, Payload, TaskGraphId, TaskId, TaskNodeId,
            WorkerAssignment, WorkerId, WorkerReportStatus,
        },
        roles::{RoleId, RoleProfile},
    };

    struct SlowCognition;
    struct AccessDeniedCognition;
    struct SuccessfulCognition;

    #[async_trait]
    impl Cognition for SlowCognition {
        async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
            tokio::time::sleep(Duration::from_millis(50)).await;
            CognitionResult::Success(json!({
                "role_output": {
                    "summary": "late answer"
                }
            }))
        }
    }

    #[async_trait]
    impl Cognition for AccessDeniedCognition {
        async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
            CognitionResult::Failure(CognitionFailure {
                reason: FailureReason::InternalError,
                description: "llm_access_denied: 当前 AI 模型服务配置不可用".to_string(),
            })
        }
    }

    #[async_trait]
    impl Cognition for SuccessfulCognition {
        async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
            CognitionResult::Success(json!({
                "role_output": {
                    "summary": "context pack answer"
                }
            }))
        }
    }

    #[tokio::test]
    async fn worker_reports_timed_out_when_cognition_exceeds_runtime_limit() {
        let (worker_tx, worker_rx) = mpsc::unbounded_channel();
        let (commander_tx, mut commander_rx) = mpsc::unbounded_channel();
        let mut worker = Worker::new_with_config(
            role("ops"),
            Box::new(SlowCognition),
            worker_rx,
            commander_tx,
            None,
            None,
            WorkerRuntimeConfig {
                cognition_timeout: Duration::from_millis(5),
            },
        );

        let worker_loop = tokio::spawn(async move {
            worker.run().await;
        });
        worker_tx
            .send(Message::new_with_context(
                next_message_id(),
                MessageContext::default(),
                AgentId("commander".to_string()),
                AgentId("worker.ops".to_string()),
                Payload::WorkerAssignment {
                    assignment: assignment(None),
                },
            ))
            .expect("worker should accept assignment");

        let mut timed_out = None;
        for _ in 0..3 {
            let message = tokio::time::timeout(Duration::from_secs(1), commander_rx.recv())
                .await
                .expect("worker should report before test timeout")
                .expect("commander channel should remain open");
            if let Payload::WorkerReport { report } = message.payload {
                timed_out = Some(report);
                break;
            }
        }

        let report = timed_out.expect("worker should emit a structured report");
        assert_eq!(report.status, WorkerReportStatus::TimedOut);
        assert!(report.content.contains("timed out"));
        assert!(
            report
                .evidence
                .iter()
                .any(|item| item == "failure_kind:timeout")
        );
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == "failure"
                && diagnostic.source == "timeout"
                && diagnostic
                    .fields
                    .get("failure_kind")
                    .is_some_and(|value| value == "timeout")
                && diagnostic
                    .fields
                    .get("status")
                    .is_some_and(|value| value == "timed_out")
        }));
        assert!(
            report
                .open_questions
                .iter()
                .any(|item| item.contains("timed out"))
        );

        worker_tx
            .send(Message::new(
                next_message_id(),
                AgentId("test".to_string()),
                AgentId("worker.ops".to_string()),
                Payload::Text {
                    content: "control:shutdown".to_string(),
                },
            ))
            .expect("worker should accept shutdown");
        tokio::time::timeout(Duration::from_secs(1), worker_loop)
            .await
            .expect("worker loop should stop")
            .expect("worker task should not panic");
    }

    #[tokio::test]
    async fn assignment_timeout_overrides_worker_default_timeout() {
        let (worker_tx, worker_rx) = mpsc::unbounded_channel();
        let (commander_tx, mut commander_rx) = mpsc::unbounded_channel();
        let mut worker = Worker::new_with_config(
            role("ops"),
            Box::new(SlowCognition),
            worker_rx,
            commander_tx,
            None,
            None,
            WorkerRuntimeConfig {
                cognition_timeout: Duration::from_secs(5),
            },
        );

        let worker_loop = tokio::spawn(async move {
            worker.run().await;
        });
        worker_tx
            .send(Message::new_with_context(
                next_message_id(),
                MessageContext::default(),
                AgentId("commander".to_string()),
                AgentId("worker.ops".to_string()),
                Payload::WorkerAssignment {
                    assignment: assignment(Some(1)),
                },
            ))
            .expect("worker should accept assignment");

        let mut report = None;
        for _ in 0..3 {
            let message = tokio::time::timeout(Duration::from_secs(2), commander_rx.recv())
                .await
                .expect("worker should report before test timeout")
                .expect("commander channel should remain open");
            if let Payload::WorkerReport { report: item } = message.payload {
                report = Some(item);
                break;
            }
        }

        assert_eq!(
            report.expect("worker should emit report").status,
            WorkerReportStatus::Completed
        );

        worker_tx
            .send(Message::new(
                next_message_id(),
                AgentId("test".to_string()),
                AgentId("worker.ops".to_string()),
                Payload::Text {
                    content: "control:shutdown".to_string(),
                },
            ))
            .expect("worker should accept shutdown");
        tokio::time::timeout(Duration::from_secs(1), worker_loop)
            .await
            .expect("worker loop should stop")
            .expect("worker task should not panic");
    }

    #[tokio::test]
    async fn worker_reports_llm_access_denied_as_structured_failure() {
        let (worker_tx, worker_rx) = mpsc::unbounded_channel();
        let (commander_tx, mut commander_rx) = mpsc::unbounded_channel();
        let mut worker = Worker::new_with_config(
            role("data"),
            Box::new(AccessDeniedCognition),
            worker_rx,
            commander_tx,
            None,
            None,
            WorkerRuntimeConfig {
                cognition_timeout: Duration::from_secs(1),
            },
        );

        let worker_loop = tokio::spawn(async move {
            worker.run().await;
        });
        worker_tx
            .send(Message::new_with_context(
                next_message_id(),
                MessageContext::default(),
                AgentId("commander".to_string()),
                AgentId("worker.data".to_string()),
                Payload::WorkerAssignment {
                    assignment: assignment(None),
                },
            ))
            .expect("worker should accept assignment");

        let mut report = None;
        for _ in 0..3 {
            let message = tokio::time::timeout(Duration::from_secs(1), commander_rx.recv())
                .await
                .expect("worker should report before test timeout")
                .expect("commander channel should remain open");
            if let Payload::WorkerReport { report: item } = message.payload {
                report = Some(item);
                break;
            }
        }

        let report = report.expect("worker should emit a structured failure report");
        assert_eq!(report.status, WorkerReportStatus::Failed);
        assert!(report.content.contains("llm_access_denied"));
        assert!(
            report
                .evidence
                .iter()
                .any(|item| item == "failure_kind:llm_access_denied")
        );
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == "failure"
                && diagnostic.source == "llm_access_denied"
                && diagnostic
                    .fields
                    .get("failure_kind")
                    .is_some_and(|value| value == "llm_access_denied")
                && diagnostic
                    .fields
                    .get("status")
                    .is_some_and(|value| value == "failed")
                && diagnostic
                    .fields
                    .get("role")
                    .is_some_and(|value| value == "ops")
        }));
        assert!(
            report
                .open_questions
                .iter()
                .any(|item| item.contains("模型服务配置不可用"))
        );

        worker_tx
            .send(Message::new(
                next_message_id(),
                AgentId("test".to_string()),
                AgentId("worker.data".to_string()),
                Payload::Text {
                    content: "control:shutdown".to_string(),
                },
            ))
            .expect("worker should accept shutdown");
        tokio::time::timeout(Duration::from_secs(1), worker_loop)
            .await
            .expect("worker loop should stop")
            .expect("worker task should not panic");
    }

    #[tokio::test]
    async fn worker_reports_context_pack_issues_as_typed_failure_diagnostic() {
        let (worker_tx, worker_rx) = mpsc::unbounded_channel();
        let (commander_tx, mut commander_rx) = mpsc::unbounded_channel();
        let mut worker = Worker::new_with_config(
            role("ops"),
            Box::new(SuccessfulCognition),
            worker_rx,
            commander_tx,
            None,
            None,
            WorkerRuntimeConfig {
                cognition_timeout: Duration::from_secs(1),
            },
        );

        let worker_loop = tokio::spawn(async move {
            worker.run().await;
        });
        let mut context = MessageContext::default();
        context.metadata.insert(
            "workspaceContextPack.readiness".to_string(),
            "partial".to_string(),
        );
        context.metadata.insert(
            "workspaceContextPack.item_count".to_string(),
            "2".to_string(),
        );
        context.metadata.insert(
            "workspaceContextPack.omitted_json".to_string(),
            serde_json::to_string(&vec!["conversation_file:file-2"]).unwrap(),
        );
        context.metadata.insert(
            "workspaceContextPack.warnings_json".to_string(),
            serde_json::to_string(&vec![
                "1 conversation file(s) failed text extraction; see extraction.failureReason",
            ])
            .unwrap(),
        );
        context.metadata.insert(
            "workspaceContextPack.items_json".to_string(),
            serde_json::to_string(&json!([
                {
                    "refKey": "conversation_file:file-1",
                    "contentKind": "content",
                    "text": "readable text"
                },
                {
                    "refKey": "conversation_file:file-2",
                    "contentKind": "extraction_failed",
                    "text": null
                }
            ]))
            .unwrap(),
        );
        worker_tx
            .send(Message::new_with_context(
                next_message_id(),
                context,
                AgentId("commander".to_string()),
                AgentId("worker.ops".to_string()),
                Payload::WorkerAssignment {
                    assignment: assignment(None),
                },
            ))
            .expect("worker should accept assignment");

        let mut report = None;
        for _ in 0..3 {
            let message = tokio::time::timeout(Duration::from_secs(1), commander_rx.recv())
                .await
                .expect("worker should report before test timeout")
                .expect("commander channel should remain open");
            if let Payload::WorkerReport { report: item } = message.payload {
                report = Some(item);
                break;
            }
        }

        let report = report.expect("worker should emit a structured report");
        assert_eq!(report.status, WorkerReportStatus::Completed);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == "failure"
                && diagnostic.source == "context_pack_extraction_failed"
                && diagnostic
                    .fields
                    .get("failure_kind")
                    .is_some_and(|value| value == "context_pack_extraction_failed")
                && diagnostic
                    .fields
                    .get("readiness")
                    .is_some_and(|value| value == "partial")
                && diagnostic
                    .fields
                    .get("item_count")
                    .is_some_and(|value| value == "2")
                && diagnostic
                    .fields
                    .get("omitted_count")
                    .is_some_and(|value| value == "1")
                && diagnostic
                    .fields
                    .get("extraction_failed_count")
                    .is_some_and(|value| value == "1")
        }));

        worker_tx
            .send(Message::new(
                next_message_id(),
                AgentId("test".to_string()),
                AgentId("worker.ops".to_string()),
                Payload::Text {
                    content: "control:shutdown".to_string(),
                },
            ))
            .expect("worker should accept shutdown");
        tokio::time::timeout(Duration::from_secs(1), worker_loop)
            .await
            .expect("worker loop should stop")
            .expect("worker task should not panic");
    }

    fn role(runtime_role: &str) -> RoleProfile {
        RoleProfile {
            id: RoleId(format!("role.{runtime_role}")),
            name: runtime_role.to_string(),
            runtime_role: runtime_role.to_string(),
            priority: 1,
            domains: Vec::new(),
            keywords: Vec::new(),
            preferred_actions: Vec::new(),
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
        }
    }

    fn assignment(timeout_sec: Option<u64>) -> WorkerAssignment {
        WorkerAssignment {
            graph_id: TaskGraphId("graph-timeout".to_string()),
            node_id: TaskNodeId("ops".to_string()),
            task_id: TaskId("task-timeout".to_string()),
            worker_id: WorkerId("worker.ops".to_string()),
            role: "ops".to_string(),
            attempt: 1,
            root_task: "test timeout".to_string(),
            objective: "test timeout".to_string(),
            llm_usage: Some("fast".to_string()),
            skill_autosuggest: Some(false),
            timeout_sec,
            inputs: Vec::new(),
            rework_instruction: None,
        }
    }
}
