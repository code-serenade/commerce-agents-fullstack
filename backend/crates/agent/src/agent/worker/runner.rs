use std::collections::HashMap;

use tracing::warn;

use super::{Task, TaskMemory, WorkerPhase, WorkerState, action_bridge::decision_to_action};
use crate::{
    action::{Action, ActionResult, ActionState},
    agent_session::SessionEventTx,
    cognition::{Cognition, CognitionInput, CognitionResult, Context, Fact, Intent, IntentKind},
    core::{
        messaging::{MessageRx, MessageTx},
        protocol::{AgentId, Message, MessageId, Payload, SessionEvent, TaskId, WorkerId},
    },
    roles::{RoleProfile, RolePromptBuilder, RolePromptInput, RoleSkillCatalog},
    skills::{
        SkillContext, SkillExecutionResult, SkillExecutionStatus, SkillExecutor, SkillExecutorSet,
        SkillId, SkillRequest,
    },
};

pub type BoxedAction = Box<dyn Action<Result = ActionResult<String, String, ()>> + Send>;

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
                self.start_task(Task {
                    id: assignment.task_id.0.clone(),
                    context,
                    description: assignment.objective.clone(),
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
                description: conversation_aware_task_description(&task),
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

        let _ = self.sender.send(Message {
            id: next_message_id(),
            context: task.context,
            from: self.id.clone(),
            to: task.requester,
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
                    let parsed = merge_skill_results(parsed, &skill_results);

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
        let _ = self.sender.send(Message {
            id: next_message_id(),
            context: task.context,
            from: self.id.clone(),
            to: task.requester,
            payload: task
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
                        status: crate::core::protocol::WorkerReportStatus::Failed,
                    },
                })
                .unwrap_or_else(|| Payload::WorkerReportFailed {
                    worker_id: WorkerId(self.id.0.clone()),
                    task_id: TaskId(task.id.clone()),
                    reason: reason.clone(),
                }),
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
        self.emit_event(SessionEvent::SkillResult {
            session_id: session_id.clone(),
            worker_id: WorkerId(self.id.0.clone()),
            task_id: TaskId(task.id.clone()),
            request_id: result.request_id.clone(),
            skill_id: result.skill_id.as_str().to_string(),
            role: self.role.runtime_role.clone(),
            status: skill_status_name(result.status).to_string(),
            summary: result.summary.clone().map(compact_event_text),
            error: result.error.clone().map(compact_event_text),
            output: result.output.clone(),
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

fn skill_not_allowed_result(
    request_id: String,
    skill_id: String,
    runtime_role: &str,
) -> SkillExecutionResult {
    SkillExecutionResult {
        request_id,
        skill_id: SkillId::new(skill_id.clone()),
        status: SkillExecutionStatus::ValidationError,
        output: None,
        summary: None,
        metadata: serde_json::Map::new(),
        error: Some(format!(
            "skill '{skill_id}' is not allowed for role '{runtime_role}'"
        )),
    }
}

fn resolve_skill_requests(
    role: &RoleProfile,
    task: &Task,
    output: &str,
) -> Vec<ParsedSkillRequest> {
    let requests = parse_skill_requests(output);
    if !requests.is_empty() {
        return requests;
    }
    autosuggest_skill_requests(role, task, output)
}

fn autosuggest_skill_requests(
    role: &RoleProfile,
    task: &Task,
    output: &str,
) -> Vec<ParsedSkillRequest> {
    let text = autosuggest_text(task, output);
    if let Some(request) = autosuggest_learning(&text, task) {
        return vec![request];
    }
    if role.runtime_role == "ops" {
        return autosuggest_ops_requests(&text, task);
    }
    if role.runtime_role == "service" {
        return autosuggest_service_requests(&text, task);
    }
    if role.runtime_role == "data" {
        return autosuggest_data_requests(&text, task);
    }
    if role.runtime_role == "creative" {
        return autosuggest_creative_requests(&text, task);
    }
    if role.runtime_role == "web" {
        return autosuggest_web_requests(&text, task);
    }
    if role.runtime_role == "accounting" {
        return autosuggest_accounting_requests(&text, task);
    }
    if role.runtime_role == "design" {
        return autosuggest_design_requests(&text, task);
    }
    if role.runtime_role == "douyin_ads" {
        return autosuggest_douyin_ads_requests(&text, task);
    }
    if role.runtime_role == "engineering" {
        return autosuggest_engineering_requests(&text, task);
    }
    let Some(request) = autosuggest_for_role(role.runtime_role.as_str(), &text, task) else {
        return Vec::new();
    };
    vec![request]
}

fn autosuggest_for_role(runtime_role: &str, text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    match runtime_role {
        "chat" => autosuggest_chat(text, task),
        "ops" => autosuggest_ops(text, task),
        "data" => autosuggest_data(text, task),
        "accounting" => autosuggest_accounting(text, task),
        "service" => autosuggest_service(text, task),
        "creative" => autosuggest_creative(text, task),
        "web" => autosuggest_web(text, task),
        "design" => autosuggest_design(text, task),
        "douyin_ads" => autosuggest_douyin_ads(text, task),
        "engineering" => autosuggest_engineering(text, task),
        _ => None,
    }
}

fn autosuggest_chat(text: &str, _task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "记住",
            "记下来",
            "保存记忆",
            "保存这个结论",
            "记忆一下",
            "保存这个事实",
            "记下这个决策",
            "记住这个风险",
            "remember this",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_save_memory",
            serde_json::json!({
                "key": infer_memory_key(text),
                "content": compact_task_text(text),
                "memory_type": infer_memory_type(text),
            }),
            "根据记忆保存意图自动补偿协调保存记忆 skill",
        ));
    }
    if contains_any(
        text,
        &[
            "当前状态",
            "任务状态",
            "运行状态",
            "session状态",
            "会话状态",
            "workspace状态",
            "工作区状态",
            "进度怎么样",
            "现在到哪了",
            "agent状态",
            "系统健康",
            "system health",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_status_query",
            serde_json::json!({
                "query_type": infer_status_query_type_from_text(text),
            }),
            "根据状态/进度/会话可见性意图自动补偿协调状态查询 skill",
        ));
    }
    None
}

fn autosuggest_learning(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if !has_learning_teaching_intent(text) {
        return None;
    }

    let mut input = serde_json::Map::new();
    input.insert(
        "topic".to_string(),
        serde_json::json!(infer_learning_topic(text)),
    );
    input.insert(
        "learning_level".to_string(),
        serde_json::json!(infer_learning_level_from_text(text)),
    );
    input.insert(
        "scenario".to_string(),
        serde_json::json!(infer_learning_scenario(text)),
    );
    input.insert(
        "include_self_check".to_string(),
        serde_json::json!(!contains_any(
            text,
            &["不要自检", "不用自检", "不要题", "no quiz"]
        )),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
        if let Some(product) = first_selected_product(task) {
            input.insert("product_name".to_string(), serde_json::json!(product.name));
            if let Some(platform) = product.platform {
                input.insert("platform".to_string(), serde_json::json!(platform));
            }
        }
    }

    Some(autosuggest_request(
        "learning_teaching_explain",
        serde_json::Value::Object(input),
        "根据教学/解释/概念学习意图自动补偿教学解释 skill",
    ))
}

fn has_learning_teaching_intent(text: &str) -> bool {
    let explicit_learning = contains_any(
        text,
        &[
            "教学",
            "讲解",
            "解释",
            "解释一下",
            "解释下",
            "什么是",
            "是什么",
            "区别",
            "原理",
            "为什么",
            "怎么做",
            "怎么理解",
            "常见误区",
            "误区",
            "自检",
            "思考题",
            "教我",
            "学习",
            "科普",
            "tutorial",
            "explain",
            "what is",
            "how to",
        ],
    );
    let explanatory_question = contains_any(text, &["为什么", "怎么做", "why"])
        && contains_any(
            text,
            &["讲", "解释", "教", "学习", "概念", "原理", "误区", "自检"],
        );

    explicit_learning || explanatory_question
}

fn autosuggest_ops_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_ops_native(text, task) {
        if is_ops_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_ops_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_request(
        "ops_execution_plan",
        serde_json::Value::Object(infer_execution_plan_input(text, task)),
        "根据运营岗位任务自动补偿执行计划 skill",
    )]
}

fn autosuggest_ops(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_ops_requests(text, task).into_iter().next()
}

fn autosuggest_douyin_ads_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_douyin_ads_native(text, task) {
        if is_douyin_ads_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_douyin_ads_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_request(
        "douyin_ads_plan_structure",
        serde_json::Value::Object(infer_douyin_ads_plan_structure_input(text, task)),
        "根据千川投放岗位任务自动补偿计划结构 skill",
    )]
}

fn autosuggest_douyin_ads(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_douyin_ads_requests(text, task)
        .into_iter()
        .next()
}

fn autosuggest_douyin_ads_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "直播间",
            "进房",
            "场观",
            "商品点击",
            "下单",
            "付款",
            "漏斗",
            "GPM",
            "gpm",
            "live room",
            "funnel",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_live_funnel_diagnosis",
            serde_json::Value::Object(infer_douyin_ads_live_funnel_input(text, task)),
            "根据直播间投放漏斗/进房/付款转化意图自动补偿千川直播漏斗诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "人群包",
            "定向",
            "人群",
            "相似人群",
            "达人粉丝",
            "再营销",
            "DMP",
            "dmp",
            "audience",
            "targeting",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_audience_package",
            serde_json::Value::Object(infer_douyin_ads_audience_input(text, task)),
            "根据千川人群包/定向意图自动补偿千川人群包设计 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "计划结构",
            "投放计划",
            "冷启动",
            "赛马",
            "短视频引流",
            "campaign",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_plan_structure",
            serde_json::Value::Object(infer_douyin_ads_plan_structure_input(text, task)),
            "根据千川投放/计划结构/冷启动意图自动补偿千川计划结构 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "预算", "消耗", "花费", "节奏", "放量", "稳投", "收缩", "止损", "复盘", "投产", "ROI",
            "roi", "ROAS", "roas", "pacing", "scale",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_budget_pacing",
            serde_json::Value::Object(infer_douyin_ads_budget_pacing_input(text, task)),
            "根据千川预算节奏/ROI/ROAS意图自动补偿千川预算节奏 skill",
        ));
    }

    if contains_any(text, &["千川", "巨量千川", "投流", "投放"]) {
        return Some(autosuggest_request(
            "douyin_ads_plan_structure",
            serde_json::Value::Object(infer_douyin_ads_plan_structure_input(text, task)),
            "根据千川投放任务自动补偿千川计划结构 skill",
        ));
    }

    None
}

fn is_douyin_ads_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "douyin_ads_plan_structure" => {
            has_external_search_intent(text)
                || contains_any(text, &["最新投放规则", "千川规则", "平台规则", "流量规则"])
        }
        "douyin_ads_audience_package" => {
            has_external_search_intent(text)
                || contains_any(text, &["最新人群趋势", "人群趋势", "兴趣趋势"])
        }
        "douyin_ads_budget_pacing" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &["行业ROI", "行业 ROI", "行业ROAS", "行业 ROAS", "基准"],
                )
        }
        "douyin_ads_live_funnel_diagnosis" => {
            has_external_search_intent(text)
                || contains_any(text, &["直播间基准", "直播转化基准", "GPM基准", "gpm基准"])
        }
        _ => has_external_search_intent(text),
    }
}

fn autosuggest_douyin_ads_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> ParsedSkillRequest {
    match native_skill_id {
        "douyin_ads_plan_structure"
            if is_platform_policy_search_intent(text)
                || contains_any(text, &["平台规则", "千川规则", "流量规则", "审核规则"]) =>
        {
            let mut input = search_platform_policy_input(text, task);
            input.insert(
                "platform".to_string(),
                serde_json::json!(selected_product_platform_or_infer(task, text)),
            );
            input.insert("policy_type".to_string(), serde_json::json!("algorithm"));
            autosuggest_request(
                "search_platform_policy",
                serde_json::Value::Object(input),
                "根据千川平台规则/流量规则诉求自动追加平台规则搜索 skill",
            )
        }
        "douyin_ads_budget_pacing" | "douyin_ads_live_funnel_diagnosis" => {
            let mut input = search_industry_benchmarks_input(text, task);
            input.insert("metric".to_string(), serde_json::json!("roi"));
            input.insert(
                "platform".to_string(),
                serde_json::json!(selected_product_platform_or_infer(task, text)),
            );
            autosuggest_request(
                "search_industry_benchmarks",
                serde_json::Value::Object(input),
                "根据千川ROI/直播转化基准诉求自动追加行业基准搜索 skill",
            )
        }
        "douyin_ads_audience_package" => {
            let mut input = search_trends_input(text, task);
            input.insert(
                "query".to_string(),
                serde_json::json!(format!(
                    "{} 抖音 千川 人群 兴趣 趋势",
                    selected_product_category_or_infer(task, text)
                )),
            );
            input.insert("platform".to_string(), serde_json::json!("抖音"));
            autosuggest_request(
                "search_trends",
                serde_json::Value::Object(input),
                "根据千川人群/兴趣趋势诉求自动追加趋势搜索 skill",
            )
        }
        _ => {
            let mut input = search_market_info_input(text, task);
            input.insert(
                "query".to_string(),
                serde_json::json!(format!(
                    "{} 抖音 千川 投放 策略",
                    selected_product_category_or_infer(task, text)
                )),
            );
            autosuggest_request(
                "search_market_info",
                serde_json::Value::Object(input),
                "根据千川投放外部资料诉求自动追加市场资料搜索 skill",
            )
        }
    }
}

fn autosuggest_ops_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "记住",
            "记下来",
            "保存记忆",
            "保存这个结论",
            "保存这个事实",
            "记下这个决策",
            "记住这个风险",
            "remember this",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_save_memory",
            serde_json::json!({
                "key": infer_memory_key(text),
                "content": infer_memory_content(text, task),
                "memory_type": infer_memory_type(text),
            }),
            "根据记忆保存意图自动补偿协调保存记忆 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "创建任务",
            "建个任务",
            "新建任务",
            "安排任务",
            "分派任务",
            "待办",
            "todo",
            "to-do",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_create_task",
            serde_json::json!({
                "title": infer_task_title(text, task),
                "description": compact_task_text(&task.description),
                "owner_role": infer_task_owner_role(text),
                "priority": infer_task_priority(text),
                "acceptance_criteria": infer_task_acceptance(text),
            }),
            "根据建任务/待办/任务安排意图自动补偿协调建任务 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "channel",
            "渠道",
            "投放渠道",
            "流量渠道",
            "渠道分配",
            "渠道策略",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_channel_strategy",
            serde_json::Value::Object(infer_channel_strategy_input(text, task)),
            "根据渠道分配/渠道策略意图自动补偿渠道策略 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "listing",
            "上架",
            "上新文案",
            "上架文案",
            "标题卖点",
            "商品标题",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_listing_copy",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "features": selected_product_features_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据上架文案/标题卖点意图自动补偿上架文案 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "assortment",
            "sku组合",
            "sku 组合",
            "货盘",
            "选品组合",
            "组合规划",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_assortment_planning",
            serde_json::Value::Object(infer_assortment_planning_input(text, task)),
            "根据选品组合/货盘规划意图自动补偿选品规划 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "smart pricing",
            "dynamic pricing",
            "智能定价",
            "动态定价",
            "价格优化",
            "利润定价",
            "最优定价",
            "调价建议",
            "涨价建议",
            "降价建议",
            "清仓定价",
            "库存压力价",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_smart_pricing",
            serde_json::Value::Object(infer_smart_pricing_input(text, task)),
            "根据智能定价/动态调价意图自动补偿智能定价 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "inventory optimizer",
            "inventory optimization",
            "库存优化",
            "库存健康",
            "安全库存",
            "再订货点",
            "经济订货量",
            "周转天数",
            "EOQ",
            "eoq",
            "ROP",
            "rop",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_inventory_optimizer",
            serde_json::Value::Object(infer_inventory_optimizer_input(text, task)),
            "根据库存优化/EOQ/再订货点意图自动补偿库存优化 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "ABC-XYZ",
            "abc-xyz",
            "ABCXYZ",
            "abcxyz",
            "ABC分类",
            "abc分类",
            "XYZ分类",
            "xyz分类",
            "ABC库存",
            "XYZ库存",
            "9格矩阵",
            "九宫格库存",
            "库存分类矩阵",
            "SKU分层",
            "sku分层",
            "高价值SKU",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_abc_xyz_classifier",
            serde_json::Value::Object(infer_abc_xyz_classifier_input(text, task)),
            "根据ABC-XYZ库存分类/九宫格矩阵意图自动补偿ABC-XYZ库存分类 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "ad fatigue",
            "fatigue",
            "ctr decay",
            "click-through decay",
            "click rate decay",
            "广告疲劳",
            "素材疲劳",
            "创意疲劳",
            "CTR衰减",
            "点击率衰减",
            "频次过高",
            "曝光频次",
            "ROAS健康",
            "roas健康",
        ],
    ) {
        return Some(autosuggest_request(
            "ops_ad_fatigue_detector",
            serde_json::Value::Object(infer_ad_fatigue_input(text, task)),
            "根据广告疲劳/CTR衰减/曝光频次意图自动补偿广告疲劳检测 skill",
        ));
    }

    if contains_any(
        text,
        &["price", "pricing", "cost", "定价", "价格", "售价", "成本"],
    ) {
        let numbers = numbers_in_text(text);
        if text.contains("成本") || text.to_ascii_lowercase().contains("cost") {
            let cost = numbers.first().copied().unwrap_or(1.0).max(0.01);
            return Some(autosuggest_request(
                "ops_pricing_strategy",
                serde_json::Value::Object(infer_pricing_strategy_input(text, task, cost)),
                "根据运营定价意图自动补偿定价策略 skill",
            ));
        }
    }

    if contains_any(text, &["inventory", "stock", "库存", "补货", "备货"]) {
        let numbers = numbers_in_text(text);
        if numbers.len() >= 2 {
            return Some(autosuggest_request(
                "ops_inventory_planning",
                serde_json::Value::Object(infer_inventory_planning_input(
                    text,
                    task,
                    numbers[0],
                    numbers[1].round().max(1.0) as i64,
                )),
                "根据库存/补货意图自动补偿库存规划 skill",
            ));
        }
    }

    if let Some(request) = autosuggest_product_or_campaign_metrics(text, task) {
        return Some(request);
    }

    if contains_any(
        text,
        &[
            "campaign",
            "promotion",
            "promo",
            "活动",
            "促销",
            "大促",
            "五一",
            "618",
            "双11",
        ],
    ) {
        let numbers = numbers_in_text(text);
        if let Some(budget) = infer_promo_budget(text).or_else(|| {
            numbers
                .into_iter()
                .filter(|value| !matches!(*value as i64, 618 | 11))
                .find(|value| *value >= 100.0)
        }) {
            return Some(autosuggest_request(
                "ops_promo_planning",
                serde_json::Value::Object(infer_promo_planning_input(text, task, budget)),
                "根据活动/促销意图自动补偿促销策划 skill",
            ));
        }
    }

    if let Some(request) = autosuggest_query_store_metrics(text, false) {
        return Some(request);
    }

    None
}

fn is_ops_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "ops_pricing_strategy" | "ops_smart_pricing" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "市场定价",
                        "竞品价格区间",
                        "实时价格",
                        "最新价格",
                        "最新竞品价",
                        "当前竞品价",
                        "公开竞品价",
                    ],
                )
        }
        "ops_promo_planning" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "大促规则",
                        "报名规则",
                        "平台活动规则",
                        "活动节点",
                        "营销节点",
                        "热门玩法",
                        "促销玩法",
                        "平台玩法",
                    ],
                )
        }
        "ops_channel_strategy" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "渠道趋势",
                        "流量趋势",
                        "获客成本",
                        "投放趋势",
                        "平台流量",
                        "渠道基准",
                        "ROI基准",
                        "roi基准",
                    ],
                )
        }
        "ops_assortment_planning" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "品类趋势",
                        "选品趋势",
                        "热卖趋势",
                        "爆款趋势",
                        "蓝海品类",
                        "市场机会",
                    ],
                )
        }
        "ops_listing_copy" => {
            is_listing_copy_policy_search_intent(text)
                || is_listing_copy_review_search_intent(text)
                || (has_external_search_intent(text)
                    && contains_any(
                        text,
                        &[
                            "标题规则",
                            "标题规范",
                            "上架规则",
                            "发布规则",
                            "违禁词",
                            "审核规则",
                            "热搜词",
                            "标题趋势",
                            "卖点趋势",
                            "搜索词",
                            "搜索量",
                            "买家痛点",
                            "购买顾虑",
                            "用户反馈",
                            "真实反馈",
                            "高点击",
                            "高转化",
                        ],
                    ))
        }
        _ => false,
    }
}

fn autosuggest_ops_search(text: &str, task: &Task, native_skill_id: &str) -> ParsedSkillRequest {
    match native_skill_id {
        "ops_pricing_strategy" | "ops_smart_pricing" => autosuggest_request(
            "search_competitor",
            serde_json::Value::Object(ops_competitor_price_search_input(text, task)),
            "根据运营定价中的实时竞品价/市场定价需求自动追加外部竞品搜索 skill",
        ),
        "ops_promo_planning" if is_ops_platform_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(ops_promotion_policy_search_input(text, task)),
            "根据运营促销中的平台活动规则/报名规则需求自动追加平台政策搜索 skill",
        ),
        "ops_promo_planning" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(ops_promotion_market_search_input(text, task)),
            "根据运营促销中的最新活动节点/热门玩法需求自动追加外部市场搜索 skill",
        ),
        "ops_channel_strategy" if is_industry_benchmark_search_intent(text) => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(ops_channel_benchmark_search_input(text, task)),
            "根据运营渠道策略中的行业投放基准需求自动追加行业基准搜索 skill",
        ),
        "ops_channel_strategy" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(ops_channel_market_search_input(text, task)),
            "根据运营渠道策略中的实时渠道/流量趋势需求自动追加外部市场搜索 skill",
        ),
        "ops_assortment_planning" => autosuggest_request(
            "search_trends",
            serde_json::Value::Object(ops_assortment_trend_search_input(text, task)),
            "根据运营选品/货盘中的实时品类趋势需求自动追加外部趋势搜索 skill",
        ),
        "ops_listing_copy" if is_listing_copy_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(listing_copy_policy_search_input(text, task)),
            "根据上架文案中的标题/发布/内容规则诉求自动追加平台规则搜索 skill",
        ),
        "ops_listing_copy" if is_listing_copy_review_search_intent(text) => autosuggest_request(
            "search_product_reviews",
            serde_json::Value::Object(listing_copy_reviews_search_input(text, task)),
            "根据上架文案中的买家评价/痛点诉求自动追加外部评价搜索 skill",
        ),
        "ops_listing_copy" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(listing_copy_market_search_input(text, task)),
            "根据上架文案中的最新标题/卖点/搜索趋势诉求自动追加外部市场搜索 skill",
        ),
        _ => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(ops_market_search_input(text, task)),
            "根据运营实时资料需求自动追加外部市场搜索 skill",
        ),
    }
}

fn is_ops_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "平台规则",
            "平台政策",
            "平台活动规则",
            "大促规则",
            "报名规则",
            "活动规则",
            "活动政策",
            "平台公告",
            "佣金",
            "费率",
        ],
    )
}

fn is_listing_copy_policy_search_intent(text: &str) -> bool {
    (has_external_search_intent(text)
        || contains_any(text, &["最新规则", "平台规则", "平台政策", "规则变化"]))
        && contains_any(
            text,
            &[
                "平台规则",
                "平台政策",
                "标题规则",
                "标题规范",
                "上架规则",
                "发布规则",
                "内容规则",
                "内容规范",
                "违禁词",
                "审核规则",
                "发布审核",
                "违规",
            ],
        )
}

fn is_listing_copy_review_search_intent(text: &str) -> bool {
    has_external_search_intent(text)
        && contains_any(
            text,
            &[
                "用户评价",
                "真实评价",
                "商品评价",
                "买家评价",
                "口碑",
                "差评",
                "好评",
                "吐槽",
                "真实反馈",
                "用户反馈",
                "买家痛点",
                "购买痛点",
                "购买顾虑",
                "消费顾虑",
                "痛点",
                "顾虑",
                "review",
                "reviews",
            ],
        )
}

fn ops_competitor_price_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_competitor_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(ops_search_subject(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("focus".to_string(), serde_json::json!("price"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 14)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_promotion_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("promotion"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_promotion_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 促销活动 热门玩法 活动节点 2026",
            selected_product_platform_or_infer(task, text),
            ops_search_subject(text, task)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("news"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_channel_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 渠道趋势 流量趋势 获客成本 投放ROI 2026",
            selected_product_platform_or_infer(task, text),
            selected_product_category_or_infer(task, text)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_channel_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert(
        "industry".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    input.insert("metric".to_string(), serde_json::json!("roi"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_assortment_trend_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 热卖趋势 爆款趋势 选品机会 2026",
            selected_product_platform_or_infer(task, text),
            selected_product_category_or_infer(task, text)
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn listing_copy_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("content"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn listing_copy_reviews_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_product_reviews_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(ops_search_subject(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_listing_copy_review_search_focus(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn listing_copy_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 上架标题 标题关键词 卖点表达 搜索趋势 高点击 2026",
            selected_product_platform_or_infer(task, text),
            ops_search_subject(text, task)
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn infer_listing_copy_review_search_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "差评",
            "负面",
            "投诉",
            "吐槽",
            "不满意",
            "痛点",
            "顾虑",
            "踩雷",
            "negative",
        ],
    ) {
        "negative"
    } else if contains_any(text, &["好评", "正面", "满意", "positive"]) {
        "positive"
    } else {
        "overall"
    }
}

fn ops_market_search_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} 电商运营 市场情报 2026",
            selected_product_platform_or_infer(task, text),
            ops_search_subject(text, task)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_ops_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn ops_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| infer_search_query_for_task(text, task))
}

fn infer_ops_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn autosuggest_data(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "comprehensive diagnosis",
            "comprehensive_diagnosis",
            "综合诊断",
            "全面诊断",
            "业务诊断",
            "经营诊断",
            "经营体检",
            "业务体检",
            "一体化诊断",
            "整体诊断",
            "全链路诊断",
        ],
    ) {
        return Some(autosuggest_request(
            "data_comprehensive_diagnosis",
            serde_json::Value::Object(infer_comprehensive_diagnosis_input(text, task)),
            "根据综合诊断/经营体检意图自动补偿综合业务诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "dashboard",
            "看板",
            "数据概览",
            "经营概览",
            "经营指标",
            "核心指标",
            "店铺大盘",
            "大盘",
        ],
    ) {
        return Some(autosuggest_request(
            "data_dashboard",
            serde_json::Value::Object(infer_dashboard_input(text, task)),
            "根据数据看板/经营概览意图自动补偿数据看板 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "competitor",
            "竞品",
            "对标",
            "竞对",
            "价格带对比",
            "竞品分析",
        ],
    ) {
        let numbers = numbers_in_text(text);
        let request = autosuggest_request(
            "data_competitor_analysis",
            serde_json::json!({
                "product_ref": selected_product_ref(task),
                "our_product": first_selected_product(task)
                    .map(|product| product.name)
                    .unwrap_or_else(|| infer_search_query_for_task(text, task)),
                "our_price": numbers.first().copied().unwrap_or(99.0),
            }),
            "根据竞品/对标意图自动补偿竞品分析 skill",
        );
        return Some(request);
    }

    if contains_any(
        text,
        &["channel roi", "渠道roi", "渠道 roi", "投放渠道", "渠道投产"],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_channel_roi",
            serde_json::json!({
                "days": infer_period_days(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道投产/归因意图自动补偿渠道ROI skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道ROI skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "attribution",
            "归因模型",
            "多触点归因",
            "渠道归因",
            "投放归因",
            "归因分析",
        ],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_attribution_analysis",
            serde_json::json!({
                "days": infer_period_days(text),
                "attribution_model": infer_attribution_model(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道归因/多触点归因意图自动补偿渠道归因 skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道归因 skill"
            },
        ));
    }

    if let Some(request) = autosuggest_search(text, task) {
        return Some(request);
    }

    if contains_any(
        text,
        &[
            "ltv",
            "LTV",
            "客户价值",
            "生命周期价值",
            "获客成本",
            "回收周期",
            "复购价值",
        ],
    ) {
        return Some(autosuggest_request(
            "data_ltv_calculator",
            serde_json::Value::Object(infer_ltv_calculator_input(text, task)),
            "根据LTV/客户价值/获客成本回收意图自动补偿 LTV 客户价值 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "cohort",
            "cohort analysis",
            "retention",
            "留存",
            "留存率",
            "复购率",
            "队列留存",
            "同期群",
            "留存队列",
            "留存分析",
            "复购留存",
            "新客留存",
            "用户留存",
            "客户留存",
            "流失节点",
        ],
    ) {
        return Some(autosuggest_request(
            "data_cohort_analysis",
            serde_json::Value::Object(infer_cohort_analysis_input(text, task)),
            "根据队列留存/同期群/复购留存意图自动补偿队列留存 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "price elasticity",
            "elasticity",
            "价格弹性",
            "价格敏感",
            "需求弹性",
            "调价测试",
            "涨价影响",
            "降价影响",
        ],
    ) {
        return Some(autosuggest_request(
            "data_price_elasticity",
            serde_json::Value::Object(infer_price_elasticity_input(text, task)),
            "根据价格弹性/价格敏感/调价影响意图自动补偿价格弹性 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "a/b",
            "ab test",
            "ab_test",
            "a b test",
            "AB测试",
            "A/B测试",
            "A B测试",
            "对照组",
            "控制组",
            "实验组",
            "测试组",
            "转化对比",
            "实验显著",
            "推全",
        ],
    ) {
        return Some(autosuggest_request(
            "data_ab_test_analyzer",
            serde_json::Value::Object(infer_ab_test_analyzer_input(text, task)),
            "根据A/B测试/对照组/实验组意图自动补偿 A/B 测试分析 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "multi period",
            "multi-period",
            "multi_period",
            "multi period trend",
            "multi-period trend",
            "多周期",
            "多周期趋势",
            "多周期环比",
            "多周期对比",
            "多周期分析",
            "环比趋势",
            "同比环比",
            "移动平均",
            "趋势动能",
        ],
    ) {
        return Some(autosuggest_request(
            "data_multi_period_trend",
            serde_json::Value::Object(infer_multi_period_trend_input(text, task)),
            "根据多周期趋势/环比/移动平均意图自动补偿多周期趋势 skill",
        ));
    }

    if let Some(request) = autosuggest_product_or_campaign_metrics(text, task) {
        return Some(request);
    }

    if let Some(request) = autosuggest_query_store_metrics(text, false) {
        return Some(request);
    }

    if contains_any(
        text,
        &["refund", "退款", "售后占比", "退货率", "帕累托", "退款分解"],
    ) {
        let items = infer_refund_items(text);
        return Some(autosuggest_request(
            "data_refund_decomposition",
            serde_json::json!({
                "days": infer_period_days(text),
                "top_n": 5,
                "product_ref": selected_product_ref(task),
                "items": items,
            }),
            if items.is_empty() {
                "根据退款分解/帕累托意图自动补偿退款分析 skill"
            } else {
                "根据用户显式提供的退款项自动补偿退款分析 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "seasonal",
            "seasonality",
            "seasonal decompose",
            "季节性",
            "周期性",
            "周期分解",
            "季节分解",
            "周内波动",
            "周期波动",
            "周期趋势",
        ],
    ) {
        return Some(autosuggest_request(
            "data_seasonal_decompose",
            serde_json::Value::Object(infer_seasonal_decompose_input(text, task)),
            "根据季节性/周期性/周内波动意图自动补偿季节性分解 skill",
        ));
    }

    if contains_any(text, &["anomaly", "异常", "波动", "下滑", "暴涨", "暴跌"]) {
        return Some(autosuggest_request(
            "data_anomaly_diagnosis",
            serde_json::Value::Object(infer_anomaly_diagnosis_input(text, task)),
            "根据异常诊断意图自动补偿数据异常 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "demand forecast",
            "demand_forecast",
            "需求预测",
            "销量预测",
            "订单预测",
            "备货预测",
            "补货预测",
            "库存预测",
            "补货量",
            "备货量",
        ],
    ) {
        return Some(autosuggest_request(
            "data_demand_forecast",
            serde_json::Value::Object(infer_demand_forecast_input(text, task)),
            "根据需求预测/备货预测/补货预测意图自动补偿需求预测 skill",
        ));
    }

    if contains_any(text, &["trend", "forecast", "预测", "趋势"]) {
        return Some(autosuggest_request(
            "data_trend_forecast",
            serde_json::Value::Object(infer_trend_forecast_input(text, task)),
            "根据趋势/预测意图自动补偿趋势预测 skill",
        ));
    }

    if contains_any(
        text,
        &["customer", "segment", "rfm", "客户", "人群", "分层"],
    ) {
        return Some(autosuggest_request(
            "data_customer_segmentation",
            serde_json::Value::Object(infer_customer_segmentation_input(text, task)),
            "根据客户分层意图自动补偿 RFM skill",
        ));
    }

    let numbers = numbers_in_text(text);
    let mut input = serde_json::Map::new();
    if numbers.len() >= 5 {
        input.insert(
            "impressions".to_string(),
            serde_json::json!(numbers[0].round() as i64),
        );
        input.insert(
            "clicks".to_string(),
            serde_json::json!(numbers[1].round() as i64),
        );
        input.insert(
            "add_to_cart".to_string(),
            serde_json::json!(numbers[2].round() as i64),
        );
        input.insert(
            "orders".to_string(),
            serde_json::json!(numbers[3].round() as i64),
        );
        input.insert(
            "payments".to_string(),
            serde_json::json!(numbers[4].round() as i64),
        );
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_store_metrics_platform(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    Some(autosuggest_request(
        "data_funnel_analysis",
        serde_json::Value::Object(input),
        "根据数据/转化意图自动补偿漏斗分析 skill",
    ))
}

fn autosuggest_data_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if is_data_price_search_chain_intent(text) {
        let primary = autosuggest_request(
            "data_price_elasticity",
            serde_json::Value::Object(infer_price_elasticity_input(text, task)),
            "根据价格弹性/价格敏感/调价影响意图自动补偿价格弹性 skill",
        );
        return vec![primary, autosuggest_data_competitor_search(text, task)];
    }

    if is_data_competitor_search_chain_intent(text) {
        let numbers = numbers_in_text(text);
        let primary = autosuggest_request(
            "data_competitor_analysis",
            serde_json::json!({
                "product_ref": selected_product_ref(task),
                "our_product": first_selected_product(task)
                    .map(|product| product.name)
                    .unwrap_or_else(|| infer_search_query_for_task(text, task)),
                "our_price": numbers.first().copied().unwrap_or(99.0),
            }),
            "根据竞品/对标意图自动补偿竞品分析 skill",
        );
        return vec![primary, autosuggest_data_competitor_search(text, task)];
    }

    if let Some(primary) = autosuggest_data_native_for_external_search(text, task) {
        if let Some(search) = autosuggest_data_external_search(text, task, &primary.skill_id) {
            return vec![primary, search];
        }
        return vec![primary];
    }

    let Some(primary) = autosuggest_data(text, task) else {
        return Vec::new();
    };

    if let Some(search) = autosuggest_data_external_search(text, task, &primary.skill_id) {
        return vec![primary, search];
    }

    vec![primary]
}

fn autosuggest_data_native_for_external_search(
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    if !has_external_search_intent(text) && !is_industry_benchmark_search_intent(text) {
        return None;
    }

    if contains_any(
        text,
        &[
            "comprehensive diagnosis",
            "comprehensive_diagnosis",
            "综合诊断",
            "全面诊断",
            "业务诊断",
            "经营诊断",
            "经营体检",
            "业务体检",
            "一体化诊断",
            "整体诊断",
            "全链路诊断",
        ],
    ) {
        return Some(autosuggest_request(
            "data_comprehensive_diagnosis",
            serde_json::Value::Object(infer_comprehensive_diagnosis_input(text, task)),
            "根据综合诊断/经营体检意图自动补偿综合业务诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "dashboard",
            "看板",
            "数据概览",
            "经营概览",
            "经营指标",
            "核心指标",
            "店铺大盘",
            "大盘",
        ],
    ) {
        return Some(autosuggest_request(
            "data_dashboard",
            serde_json::Value::Object(infer_dashboard_input(text, task)),
            "根据数据看板/经营概览意图自动补偿数据看板 skill",
        ));
    }

    if contains_any(
        text,
        &["channel roi", "渠道roi", "渠道 roi", "投放渠道", "渠道投产"],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_channel_roi",
            serde_json::json!({
                "days": infer_period_days(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道投产/归因意图自动补偿渠道ROI skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道ROI skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "attribution",
            "归因模型",
            "多触点归因",
            "渠道归因",
            "投放归因",
            "归因分析",
        ],
    ) {
        let channels = infer_channel_roi_rows(text);
        return Some(autosuggest_request(
            "data_attribution_analysis",
            serde_json::json!({
                "days": infer_period_days(text),
                "attribution_model": infer_attribution_model(text),
                "channels": channels,
                "product_ref": selected_product_ref(task),
            }),
            if channels.is_empty() {
                "根据渠道归因/多触点归因意图自动补偿渠道归因 skill"
            } else {
                "根据用户显式提供的渠道指标自动补偿渠道归因 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "multi period",
            "multi-period",
            "multi_period",
            "multi period trend",
            "multi-period trend",
            "多周期",
            "多周期趋势",
            "多周期环比",
            "多周期对比",
            "多周期分析",
            "环比趋势",
            "同比环比",
            "移动平均",
            "趋势动能",
        ],
    ) {
        return Some(autosuggest_request(
            "data_multi_period_trend",
            serde_json::Value::Object(infer_multi_period_trend_input(text, task)),
            "根据多周期趋势/环比/移动平均意图自动补偿多周期趋势 skill",
        ));
    }

    if contains_any(
        text,
        &["refund", "退款", "售后占比", "退货率", "帕累托", "退款分解"],
    ) {
        let items = infer_refund_items(text);
        return Some(autosuggest_request(
            "data_refund_decomposition",
            serde_json::json!({
                "days": infer_period_days(text),
                "top_n": 5,
                "product_ref": selected_product_ref(task),
                "items": items,
            }),
            if items.is_empty() {
                "根据退款分解/帕累托意图自动补偿退款分析 skill"
            } else {
                "根据用户显式提供的退款项自动补偿退款分析 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "seasonal",
            "seasonality",
            "seasonal decompose",
            "季节性",
            "周期性",
            "周期分解",
            "季节分解",
            "周内波动",
            "周期波动",
            "周期趋势",
        ],
    ) {
        return Some(autosuggest_request(
            "data_seasonal_decompose",
            serde_json::Value::Object(infer_seasonal_decompose_input(text, task)),
            "根据季节性/周期性/周内波动意图自动补偿季节性分解 skill",
        ));
    }

    if contains_any(text, &["anomaly", "异常", "波动", "下滑", "暴涨", "暴跌"]) {
        return Some(autosuggest_request(
            "data_anomaly_diagnosis",
            serde_json::Value::Object(infer_anomaly_diagnosis_input(text, task)),
            "根据异常诊断意图自动补偿数据异常 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "demand forecast",
            "demand_forecast",
            "需求预测",
            "销量预测",
            "订单预测",
            "备货预测",
            "补货预测",
            "库存预测",
            "补货量",
            "备货量",
        ],
    ) {
        return Some(autosuggest_request(
            "data_demand_forecast",
            serde_json::Value::Object(infer_demand_forecast_input(text, task)),
            "根据需求预测/备货预测/补货预测意图自动补偿需求预测 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "trend forecast",
            "forecast",
            "趋势预测",
            "趋势分析",
            "预测",
            "预估",
        ],
    ) {
        return Some(autosuggest_request(
            "data_trend_forecast",
            serde_json::Value::Object(infer_trend_forecast_input(text, task)),
            "根据趋势/预测意图自动补偿趋势预测 skill",
        ));
    }

    None
}

fn is_data_competitor_search_chain_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "competitor",
            "竞品",
            "竞对",
            "竞争对手",
            "对标",
            "同类商品",
            "价格带对比",
            "竞品分析",
        ],
    ) && has_external_search_intent(text)
}

fn is_data_price_search_chain_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "price elasticity",
            "elasticity",
            "价格弹性",
            "价格敏感",
            "需求弹性",
            "调价测试",
            "涨价影响",
            "降价影响",
            "市场定价",
            "竞品价格区间",
            "价格区间",
        ],
    ) && (has_external_search_intent(text)
        || contains_any(text, &["市场定价", "竞品价格区间", "实时价格", "最新价格"]))
}

fn autosuggest_data_competitor_search(text: &str, task: &Task) -> ParsedSkillRequest {
    let input = search_competitor_input(text, task);
    autosuggest_request(
        "search_competitor",
        serde_json::Value::Object(input),
        "根据数据分析中的实时竞品/市场定价需求自动追加外部搜索 skill",
    )
}

fn autosuggest_data_external_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> Option<ParsedSkillRequest> {
    if !is_data_external_search_chain_intent(text, native_skill_id) {
        return None;
    }

    Some(match native_skill_id {
        "data_channel_roi" | "data_attribution_analysis" => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(data_channel_benchmark_search_input(text, task)),
            "根据数据分析中的行业 ROI / 渠道效率基准需求自动追加行业基准搜索 skill",
        ),
        "data_refund_decomposition" => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(data_refund_benchmark_search_input(text, task)),
            "根据退款分析中的行业退款率基准需求自动追加行业基准搜索 skill",
        ),
        "data_trend_forecast"
        | "data_demand_forecast"
        | "data_multi_period_trend"
        | "data_seasonal_decompose" => autosuggest_request(
            "search_trends",
            serde_json::Value::Object(data_trend_search_input(text, task, native_skill_id)),
            "根据数据趋势/需求预测中的实时市场趋势需求自动追加趋势搜索 skill",
        ),
        "data_anomaly_diagnosis" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(data_anomaly_market_search_input(text, task)),
            "根据异常诊断中的外部市场原因/公开资料需求自动追加市场搜索 skill",
        ),
        "data_comprehensive_diagnosis" | "data_dashboard" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(data_market_context_search_input(
                text,
                task,
                native_skill_id,
            )),
            "根据经营诊断/看板中的外部市场资料需求自动追加市场搜索 skill",
        ),
        _ => return None,
    })
}

fn is_data_external_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "data_channel_roi" | "data_attribution_analysis" => {
            is_industry_benchmark_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "行业roi",
                        "行业 ROI",
                        "行业ROAS",
                        "行业 roas",
                        "渠道基准",
                        "投放基准",
                        "投产基准",
                        "渠道效率基准",
                        "最新ROI",
                        "最新 ROI",
                        "实时ROI",
                        "实时 ROI",
                    ],
                )
        }
        "data_refund_decomposition" => {
            is_industry_benchmark_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "行业退款率",
                        "平均退款率",
                        "退款率基准",
                        "售后率基准",
                        "退货率基准",
                        "最新退款率",
                        "实时退款率",
                    ],
                )
        }
        "data_trend_forecast"
        | "data_demand_forecast"
        | "data_multi_period_trend"
        | "data_seasonal_decompose" => {
            has_external_search_intent(text)
                && contains_any(
                    text,
                    &[
                        "市场趋势",
                        "消费趋势",
                        "需求趋势",
                        "热卖趋势",
                        "品类趋势",
                        "行业趋势",
                        "季节趋势",
                        "爆款",
                        "热卖",
                        "外部趋势",
                        "趋势参考",
                    ],
                )
        }
        "data_anomaly_diagnosis" => {
            has_external_search_intent(text)
                && contains_any(
                    text,
                    &[
                        "外部原因",
                        "市场原因",
                        "行业原因",
                        "平台原因",
                        "公开资料",
                        "公开来源",
                        "市场动态",
                        "行业动态",
                        "新闻",
                    ],
                )
        }
        "data_comprehensive_diagnosis" | "data_dashboard" => {
            (has_external_search_intent(text) || is_industry_benchmark_search_intent(text))
                && contains_any(
                    text,
                    &[
                        "市场报告",
                        "市场行情",
                        "行业动态",
                        "行业报告",
                        "公开资料",
                        "公开来源",
                        "行业基准",
                        "行业均值",
                        "行业平均",
                        "经营基准",
                    ],
                )
        }
        _ => false,
    }
}

fn data_channel_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert("metric".to_string(), serde_json::json!("roi"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_refund_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert("metric".to_string(), serde_json::json!("refund"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_trend_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(data_trend_search_query(text, task, native_skill_id)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_data_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_anomaly_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "{} {} {} 数据异常 外部原因 行业动态 2026",
            infer_search_platform_for_task(text, task),
            data_search_subject(text, task),
            infer_metric(text)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("news"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_data_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_market_context_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(data_market_context_query(text, task, native_skill_id)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_data_search_days(text, 60)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn data_trend_search_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = infer_search_platform_for_task(text, task);
    let subject = data_search_subject(text, task);
    match native_skill_id {
        "data_demand_forecast" => {
            format!("{platform} {subject} 需求趋势 热卖趋势 备货预测 2026")
        }
        "data_seasonal_decompose" => {
            format!("{platform} {subject} 季节趋势 周期波动 消费趋势 2026")
        }
        "data_multi_period_trend" => {
            format!("{platform} {subject} 多周期趋势 增长趋势 市场趋势 2026")
        }
        _ => format!("{platform} {subject} 市场趋势 消费趋势 热卖 2026"),
    }
}

fn data_market_context_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = infer_search_platform_for_task(text, task);
    let subject = data_search_subject(text, task);
    match native_skill_id {
        "data_dashboard" => format!("{platform} {subject} 电商经营指标 行业动态 市场报告 2026"),
        _ => format!("{platform} {subject} 电商经营诊断 市场行情 行业报告 2026"),
    }
}

fn data_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| infer_search_query_for_task(text, task))
}

fn infer_data_search_days(text: &str, default: i64) -> i64 {
    infer_explicit_period_days(text).unwrap_or(default)
}

fn autosuggest_accounting_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_accounting_native(text) {
        if is_accounting_finance_search_chain_intent(text, &primary.skill_id) {
            return vec![primary, autosuggest_accounting_finance_search(text, task)];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_request(
        "accounting_budget_plan",
        serde_json::json!({}),
        "根据财务岗位任务自动补偿预算编制 skill",
    )]
}

fn autosuggest_accounting(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_accounting_requests(text, task)
        .into_iter()
        .next()
}

fn autosuggest_accounting_native(text: &str) -> Option<ParsedSkillRequest> {
    let numbers = numbers_in_text(text);

    if is_accounting_finance_market_search_text(text) && !has_accounting_native_work_intent(text) {
        return None;
    }

    if contains_any(
        text,
        &[
            "财务健康",
            "财务诊断",
            "财务体检",
            "健康评级",
            "财务健康诊断",
            "financial narrative",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_financial_narrative",
            serde_json::Value::Object(infer_accounting_financial_input(text)),
            "根据财务健康/诊断意图自动补偿财务健康诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "p&l",
            "P&L",
            "利润表",
            "损益表",
            "完整利润表",
            "pl statement",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_pl_statement",
            serde_json::Value::Object(infer_accounting_pl_input(text)),
            "根据 P&L/利润表意图自动补偿完整利润表 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "GMV瀑布",
            "gmv瀑布",
            "利润瀑布",
            "利润漏损",
            "漏损项",
            "waterfall",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_gmv_waterfall",
            serde_json::Value::Object(infer_accounting_gmv_waterfall_input(text)),
            "根据 GMV/利润瀑布意图自动补偿利润瀑布 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "预算vs实际",
            "预算VS实际",
            "预算与实际",
            "预算执行",
            "预算偏差",
            "超支",
            "budget vs actual",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_budget_vs_actual",
            serde_json::Value::Object(infer_accounting_budget_vs_actual_input(text)),
            "根据预算执行/预算偏差意图自动补偿预算vs实际 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "compliance",
            "合规",
            "税务",
            "纳税",
            "开票",
            "发票",
            "现金收款",
        ],
    ) {
        return Some(autosuggest_request(
            "accounting_compliance_check",
            serde_json::json!({
                "monthly_revenue": numbers.first().copied().unwrap_or(500_000.0),
                "tax_type": if contains_any(text, &["一般纳税人"]) { "一般纳税人" } else { "小规模" },
                "invoice_rate": infer_percent_number(text, 80.0),
                "cash_ratio": infer_second_percent_number(text, 10.0),
            }),
            "根据财税合规意图自动补偿合规检查 skill",
        ));
    }

    if contains_any(text, &["roi", "roas", "投产", "投入产出", "回报"]) {
        return Some(autosuggest_request(
            "accounting_roi_calc",
            serde_json::json!({
                "investment": numbers.first().copied(),
                "revenue_generated": numbers.get(1).copied(),
            }),
            "根据 ROI/投产意图自动补偿 ROI 计算 skill",
        ));
    }

    if contains_any(text, &["cost", "margin", "成本", "毛利"]) && numbers.len() >= 2 {
        return Some(autosuggest_request(
            "accounting_cost_calc",
            serde_json::json!({
                "purchase_cost": numbers[0],
                "selling_price": numbers[1],
            }),
            "根据成本/毛利意图自动补偿成本核算 skill",
        ));
    }

    if contains_any(text, &["profit", "利润", "盈利", "亏损"]) {
        return Some(autosuggest_request(
            "accounting_profit_analysis",
            serde_json::json!({
                "revenue": numbers.first().copied().unwrap_or(0.0),
            }),
            "根据利润分析意图自动补偿利润分析 skill",
        ));
    }

    if let Some(request) = autosuggest_query_store_metrics(text, false) {
        return Some(request);
    }

    None
}

fn infer_accounting_financial_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    insert_optional_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "销售额", "成交额", "营收", "收入"],
    );
    insert_optional_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "广告投放",
            "投放费",
            "ad_spend",
            "ad spend",
        ],
    );
    insert_optional_metric_value(
        &mut input,
        "orders",
        text,
        &["订单量", "订单数", "订单", "orders"],
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_accounting_analysis_days(text, 30)),
    );
    input.insert(
        "context".to_string(),
        serde_json::json!(compact_task_text(text)),
    );
    input
}

fn infer_accounting_pl_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    insert_optional_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "销售额", "成交额", "营收", "收入"],
    );
    insert_optional_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "广告投放",
            "投放费",
            "ad_spend",
            "ad spend",
        ],
    );
    insert_optional_metric_value(
        &mut input,
        "orders",
        text,
        &["订单量", "订单数", "订单", "orders"],
    );
    insert_optional_metric_value(
        &mut input,
        "cogs_rate",
        text,
        &["商品成本率", "成本率", "cogs_rate"],
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_accounting_analysis_days(text, 30)),
    );
    input
}

fn infer_accounting_gmv_waterfall_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_accounting_pl_input(text);
    insert_optional_metric_value(
        &mut input,
        "refund_rate",
        text,
        &["退款率", "退货率", "refund_rate"],
    );
    insert_optional_metric_value(
        &mut input,
        "logistics_rate",
        text,
        &["物流费率", "物流成本率", "logistics_rate"],
    );
    insert_optional_metric_value(
        &mut input,
        "platform_fee_rate",
        text,
        &["平台佣金率", "平台费率", "佣金率", "platform_fee_rate"],
    );
    input
}

fn infer_accounting_budget_vs_actual_input(
    text: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    insert_optional_metric_value(
        &mut input,
        "budget_gmv",
        text,
        &["预算GMV", "预算gmv", "目标GMV", "目标gmv", "预算销售额"],
    );
    insert_optional_metric_value(
        &mut input,
        "actual_gmv",
        text,
        &["实际GMV", "实际gmv", "实际销售额", "实际营收"],
    );
    insert_optional_metric_value(
        &mut input,
        "budget_ad_spend",
        text,
        &["预算广告费", "预算投放费", "预算广告投入"],
    );
    insert_optional_metric_value(
        &mut input,
        "actual_ad_spend",
        text,
        &["实际广告费", "实际投放费", "实际广告投入"],
    );
    input
}

fn insert_optional_metric_value(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text: &str,
    aliases: &[&str],
) {
    if let Some(value) = extract_metric_value(text, aliases) {
        input.insert(key.to_string(), serde_json::json!(value));
    }
}

fn is_accounting_finance_market_search_text(text: &str) -> bool {
    contains_any(
        text,
        &[
            "行业财务基准",
            "财务基准",
            "财务对标",
            "财务benchmark",
            "财务 benchmark",
            "利润率基准",
            "行业利润率",
            "毛利率基准",
            "净利率基准",
            "广告ROI基准",
            "广告 ROI 基准",
        ],
    ) || (has_external_search_intent(text)
        && contains_any(
            text,
            &[
                "财务",
                "利润率",
                "毛利率",
                "净利率",
                "财报",
                "现金流",
                "广告ROI",
            ],
        ))
}

fn has_accounting_native_work_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "做",
            "分析",
            "诊断",
            "体检",
            "计算",
            "核算",
            "生成",
            "编制",
            "预测",
            "检查",
            "评估",
            "帮我看",
            "出一份",
            "利润表",
            "损益表",
            "GMV瀑布",
            "gmv瀑布",
            "利润瀑布",
            "预算执行",
            "预算偏差",
            "财务健康",
            "财务诊断",
        ],
    )
}

fn is_accounting_finance_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "accounting_financial_narrative"
            | "accounting_pl_statement"
            | "accounting_gmv_waterfall"
            | "accounting_budget_vs_actual"
    ) && (has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "行业财务基准",
                "财务基准",
                "财务对标",
                "财务benchmark",
                "财务 benchmark",
                "利润率基准",
                "行业利润率",
                "毛利率基准",
                "净利率基准",
                "广告ROI基准",
                "广告 ROI 基准",
                "财务公开来源",
            ],
        ))
}

fn autosuggest_accounting_finance_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_market_info",
        serde_json::Value::Object(accounting_finance_search_input(text, task)),
        "根据财务诊断中的行业财务基准/财务对标需求自动追加外部财务搜索 skill",
    )
}

fn accounting_finance_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(accounting_finance_search_query(text, task)),
    );
    input.insert("topic".to_string(), serde_json::json!("finance"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_accounting_search_days(text, 60)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn accounting_finance_search_query(text: &str, task: &Task) -> String {
    if let Some(product) = first_selected_product(task) {
        let category = product
            .category
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "电商消费品".to_string());
        return format!(
            "{} {} 电商行业 毛利率 净利率 广告ROI 财务基准 2026",
            product.name, category
        );
    }

    let category = infer_category(text);
    if category != "电商消费品" {
        format!("{category} 电商行业 毛利率 净利率 广告ROI 财务基准 2026")
    } else {
        "电商行业 毛利率 净利率 广告ROI 财务基准 2026".to_string()
    }
}

fn infer_accounting_analysis_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn infer_accounting_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn autosuggest_service_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if is_service_benchmark_chain_intent(text)
        && let Some(primary) = autosuggest_service_native(text, task)
    {
        return vec![primary, autosuggest_service_benchmark_search(text, task)];
    }

    if let Some(primary) = autosuggest_service_native(text, task)
        && is_service_platform_policy_search_chain_intent(text, &primary.skill_id)
    {
        return vec![
            primary,
            autosuggest_service_platform_policy_search(text, task),
        ];
    }

    if let Some(primary) = autosuggest_service_native(text, task)
        && is_service_review_search_chain_intent(text, &primary.skill_id)
    {
        return vec![primary, autosuggest_service_review_search(text, task)];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    autosuggest_service_native(text, task).into_iter().collect()
}

fn autosuggest_service(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_service_requests(text, task).into_iter().next()
}

fn autosuggest_service_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "driver",
            "驱动因素",
            "满意度原因",
            "投诉分类",
            "nps驱动",
            "nps driver",
        ],
    ) {
        return Some(autosuggest_request(
            "service_nps_driver_analysis",
            serde_json::json!({
                "platform": infer_platform(text),
            }),
            "根据满意度驱动因素意图自动补偿NPS驱动分析 skill",
        ));
    }

    if contains_any(
        text,
        &["faq", "FAQ", "常见问题", "问答库", "标准话术", "话术库"],
    ) {
        return Some(autosuggest_request(
            "service_faq_playbook",
            serde_json::json!({
                "questions": infer_faq_questions(text),
                "platform": infer_platform(text),
                "product_ref": selected_product_ref(task),
            }),
            "根据FAQ/标准话术意图自动补偿FAQ话术库 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "查询商品",
            "查商品",
            "商品信息",
            "商品资料",
            "商品上下文",
            "商品详情",
            "商品知识",
            "引用商品",
            "选中商品",
        ],
    ) {
        return Some(autosuggest_request(
            "service_query_product",
            serde_json::json!({
                "product_ref": selected_product_ref(task),
                "product_name": first_selected_product(task).map(|product| product.name),
            }),
            "根据客服商品查询意图自动补偿商品上下文查询 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "ticket",
            "工单",
            "话术",
            "客服话术",
            "安抚",
            "客户投诉",
            "生气",
            "很气",
            "物流延迟",
            "发错",
            "质量问题",
        ],
    ) {
        return Some(autosuggest_request(
            "service_ticket_handler",
            serde_json::json!({
                "issue_type": infer_service_issue_type(text),
                "customer_message": compact_task_text(&task.description),
                "customer_emotion": infer_service_emotion(text),
                "platform": infer_platform(text),
                "product_ref": selected_product_ref(task),
            }),
            "根据客服工单/话术意图自动补偿工单处理 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "upgrade",
            "escalation",
            "升级",
            "主管",
            "投诉平台",
            "平台介入",
            "曝光",
            "举报",
            "维权",
            "威胁投诉",
        ],
    ) {
        return Some(autosuggest_request(
            "service_escalation_flow",
            serde_json::json!({
                "issue_description": compact_task_text(&task.description),
                "customer_emotion": infer_service_emotion(text),
                "previous_contacts": numbers_in_text(text).first().copied().map(|value| value.round() as i64),
                "product_ref": selected_product_ref(task),
            }),
            "根据客服升级/投诉意图自动补偿升级处理 skill",
        ));
    }

    if contains_any(text, &["return", "refund", "退款", "退货", "换货", "售后"]) {
        return Some(autosuggest_request(
            "service_return_handler",
            serde_json::json!({
                "reason": infer_return_reason(text),
                "days_since_receipt": numbers_in_text(text).first().copied().map(|value| value.round() as i64),
                "product_ref": selected_product_ref(task),
            }),
            "根据退换货/售后意图自动补偿退换货处理 skill",
        ));
    }

    if contains_any(text, &["nps", "NPS", "满意度", "推荐值", "客户忠诚"]) {
        return Some(autosuggest_request(
            "service_nps_analyzer",
            serde_json::json!({
                "platform": infer_platform(text),
            }),
            "根据NPS/满意度意图自动补偿NPS分析 skill",
        ));
    }

    if contains_any(text, &["情绪", "情感", "差评", "不满意", "愤怒", "投诉"]) {
        return Some(autosuggest_request(
            "service_sentiment_analyzer",
            serde_json::json!({
                "customer_text": compact_task_text(&task.description),
                "platform": infer_platform(text),
            }),
            "根据客服情绪识别意图自动补偿情感分析 skill",
        ));
    }

    Some(autosuggest_request(
        "service_dsr_improvement",
        serde_json::json!({
            "platform": infer_platform(text),
        }),
        "根据客服岗位任务自动补偿 DSR 提升 skill",
    ))
}

fn is_service_benchmark_chain_intent(text: &str) -> bool {
    let has_service_metric = contains_any(
        text,
        &[
            "dsr",
            "DSR",
            "服务评分",
            "店铺评分",
            "评分低",
            "满意度",
            "nps",
            "NPS",
            "推荐值",
            "客户忠诚",
            "驱动因素",
            "投诉分类",
            "客服响应",
        ],
    );
    has_service_metric
        && (is_industry_benchmark_search_intent(text)
            || contains_any(
                text,
                &["行业基准", "行业均值", "行业平均", "实时基准", "最新基准"],
            ))
}

fn autosuggest_service_benchmark_search(text: &str, task: &Task) -> ParsedSkillRequest {
    let input = search_industry_benchmarks_input(text, task);
    autosuggest_request(
        "search_industry_benchmarks",
        serde_json::Value::Object(input),
        "根据客服DSR/NPS分析中的行业基准需求自动追加外部搜索 skill",
    )
}

fn is_service_platform_policy_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "service_ticket_handler"
            | "service_faq_playbook"
            | "service_escalation_flow"
            | "service_return_handler"
    ) && is_service_platform_policy_search_intent(text)
        && has_service_policy_work_intent(text, native_skill_id)
}

fn is_service_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "平台规则",
            "平台政策",
            "平台公告",
            "最新规则",
            "最新政策",
            "售后规则",
            "售后政策",
            "退货规则",
            "退款规则",
            "退换货规则",
            "换货规则",
            "运费规则",
            "运费险",
            "赔付标准",
            "赔偿标准",
            "补偿标准",
            "投诉规则",
            "平台介入规则",
            "纠纷规则",
            "纠纷处理",
            "七天无理由",
            "7天无理由",
            "假一赔",
            "仅退款",
            "售后时效",
            "policy",
        ],
    ) && (has_external_search_intent(text)
        || contains_any(text, &["最新", "规则", "政策", "标准", "公告"]))
}

fn has_service_policy_work_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "service_ticket_handler" => contains_any(
            text,
            &[
                "工单", "处理", "话术", "回复", "安抚", "客户", "用户", "客服", "投诉", "ticket",
            ],
        ),
        "service_faq_playbook" => contains_any(
            text,
            &[
                "faq",
                "FAQ",
                "常见问题",
                "问答库",
                "标准话术",
                "话术库",
                "客服话术",
            ],
        ),
        "service_escalation_flow" => contains_any(
            text,
            &[
                "升级",
                "主管",
                "投诉平台",
                "平台介入",
                "维权",
                "举报",
                "纠纷",
                "escalation",
            ],
        ),
        "service_return_handler" => contains_any(
            text,
            &[
                "退款",
                "退货",
                "换货",
                "售后",
                "退换货",
                "返修",
                "赔付",
                "运费",
                "return",
                "refund",
            ],
        ),
        _ => false,
    }
}

fn autosuggest_service_platform_policy_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_platform_policy",
        serde_json::Value::Object(service_platform_policy_search_input(text, task)),
        "根据客服处理中的平台售后/赔付/投诉规则诉求自动追加平台规则搜索 skill",
    )
}

fn service_platform_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert("policy_type".to_string(), serde_json::json!("general"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_service_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn is_service_review_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "service_sentiment_analyzer"
            | "service_ticket_handler"
            | "service_return_handler"
            | "service_nps_driver_analysis"
    ) && is_product_review_search_intent(text)
        && has_service_review_work_intent(text, native_skill_id)
}

fn has_service_review_work_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "service_sentiment_analyzer" => contains_any(
            text,
            &[
                "分析",
                "识别",
                "判断",
                "研判",
                "情绪",
                "情感",
                "sentiment",
                "analyze",
            ],
        ),
        "service_ticket_handler" => contains_any(
            text,
            &[
                "工单",
                "处理",
                "话术",
                "回复",
                "安抚",
                "客户说",
                "用户说",
                "ticket",
            ],
        ),
        "service_return_handler" => contains_any(
            text,
            &[
                "退款",
                "退货",
                "换货",
                "售后",
                "退换货",
                "返修",
                "return",
                "refund",
            ],
        ),
        "service_nps_driver_analysis" => contains_any(
            text,
            &[
                "驱动因素",
                "满意度原因",
                "投诉分类",
                "nps驱动",
                "NPS",
                "nps",
            ],
        ),
        _ => false,
    }
}

fn autosuggest_service_review_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_product_reviews",
        serde_json::Value::Object(service_product_reviews_search_input(text, task)),
        "根据客服处理中的真实评价/口碑/差评诉求自动追加外部评价搜索 skill",
    )
}

fn service_product_reviews_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_product_reviews_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(service_review_search_subject(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_service_review_search_focus(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_service_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn service_review_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| infer_search_query_for_task(text, task))
}

fn infer_service_review_search_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &["差评", "负面", "投诉", "吐槽", "不满意", "negative"],
    ) {
        "negative"
    } else if contains_any(text, &["好评", "正面", "满意", "positive"]) {
        "positive"
    } else {
        "overall"
    }
}

fn infer_service_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn autosuggest_creative_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_creative_native(text, task) {
        if is_creative_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_creative_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_creative_title_ctr(text, task)]
}

fn autosuggest_creative(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_creative_requests(text, task).into_iter().next()
}

fn autosuggest_creative_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["live", "直播"]) {
        let product_count = selected_product_count_or_default(task, 6);
        return Some(autosuggest_request(
            "creative_live_script",
            serde_json::json!({
                "duration_hours": infer_live_duration_hours(text),
                "product_count": product_count,
                "product_refs": selected_product_refs(task),
                "live_type": if contains_any(text, &["new", "新品"]) { "新品首发" } else { "日常带货" },
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据直播/内容任务自动补偿直播脚本 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "video",
            "short video",
            "短视频",
            "视频脚本",
            "拍摄脚本",
            "开箱",
            "测评",
        ],
    ) {
        return Some(autosuggest_request(
            "creative_video_script",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "video_type": infer_video_type(text),
                "duration": infer_video_duration_seconds(text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据短视频/脚本意图自动补偿短视频脚本 skill",
        ));
    }

    if contains_any(text, &["seeding", "种草", "笔记", "小红书", "分享文案"]) {
        return Some(autosuggest_request(
            "creative_seeding_copy",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "selling_points": selected_product_features_or_infer(task, text),
                "target_audience": infer_audience(text),
                "tone": if contains_any(text, &["测评", "专业"]) { "专业测评" } else { "闺蜜分享" },
            }),
            "根据种草/笔记意图自动补偿种草文案 skill",
        ));
    }

    if contains_any(text, &["ip", "IP", "人设", "定位", "账号定位", "品牌定位"]) {
        return Some(autosuggest_request(
            "creative_ip_branding",
            serde_json::json!({
                "brand_name": selected_product_brand_name_or_task(task),
                "category": selected_product_category_or_infer(task, text),
                "target_audience": infer_audience(text),
                "differentiator": "专业+有温度",
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据 IP/品牌定位意图自动补偿 IP 定位 skill",
        ));
    }

    if contains_any(
        text,
        &["calendar", "schedule", "排期", "日历", "内容计划", "选题"],
    ) {
        return Some(autosuggest_request(
            "creative_content_calendar",
            serde_json::json!({
                "period": if contains_any(text, &["月", "monthly", "month"]) { "月" } else { "周" },
                "platforms": [infer_platform(text), "小红书"],
                "weekly_posts": numbers_in_text(text).first().copied().unwrap_or(5.0).round().clamp(1.0, 14.0) as i64,
            }),
            "根据内容排期/选题意图自动补偿内容排期 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "article",
            "soft article",
            "软文",
            "长文",
            "推广文章",
            "公众号",
        ],
    ) {
        return Some(autosuggest_request(
            "creative_product_article",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "article_type": if contains_any(text, &["对比"]) { "对比评测" } else if contains_any(text, &["攻略"]) { "使用攻略" } else { "测评软文" },
                "platform": selected_product_article_platform_or_default(task, text),
            }),
            "根据软文/推广文章意图自动补偿产品软文 skill",
        ));
    }

    if contains_any(text, &["trend", "hot", "趋势", "热点", "爆款", "流行"]) {
        return Some(autosuggest_request(
            "creative_trend_catch",
            serde_json::json!({
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据趋势/热点意图自动补偿趋势捕捉 skill",
        ));
    }

    None
}

fn autosuggest_creative_title_ctr(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "creative_title_ctr_scorer",
        serde_json::json!({
            "title": extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description)),
            "platform": infer_platform(text),
        }),
        "根据创意岗位任务自动补偿标题 CTR 评分 skill",
    )
}

fn is_creative_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    let has_fresh_or_external = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "当前",
                "近期",
                "今年",
                "今日",
                "今天",
                "热搜",
                "榜单",
                "站外",
                "公开来源",
                "公开资料",
            ],
        );

    match native_skill_id {
        "creative_video_script" | "creative_seeding_copy" => {
            (has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "爆款", "热门", "热点", "趋势", "风格", "标签", "话题", "内容",
                    ],
                ))
                || contains_any(
                    text,
                    &[
                        "最新爆款",
                        "实时爆款",
                        "热门风格",
                        "爆款话题",
                        "热搜",
                        "榜单",
                    ],
                )
        }
        "creative_content_calendar" => {
            has_fresh_or_external
                || contains_any(
                    text,
                    &[
                        "营销节点",
                        "内容节点",
                        "热门话题",
                        "热点话题",
                        "节日热点",
                        "节日营销",
                    ],
                )
        }
        "creative_trend_catch" => {
            has_fresh_or_external
                || contains_any(text, &["最新趋势", "实时趋势", "热搜", "榜单", "热点话题"])
        }
        "creative_live_script" => {
            (has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "直播",
                        "直播间",
                        "带货",
                        "场控",
                        "话术",
                        "玩法",
                        "互动",
                        "转化",
                        "爆款",
                        "热门",
                        "节奏",
                        "脚本",
                    ],
                ))
                || contains_any(
                    text,
                    &[
                        "热门直播间",
                        "爆款直播",
                        "直播间玩法",
                        "直播间脚本",
                        "实时直播",
                        "最新直播",
                    ],
                )
        }
        "creative_product_article" => {
            (has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "软文",
                        "长文",
                        "公众号",
                        "推广文章",
                        "文章",
                        "内容",
                        "风格",
                        "话题",
                        "爆款",
                        "热门",
                        "种草",
                        "测评",
                        "攻略",
                        "对比",
                        "公开资料",
                    ],
                ))
                || contains_any(
                    text,
                    &[
                        "热门软文",
                        "爆款软文",
                        "爆款文章",
                        "热门文章",
                        "最新软文",
                        "最新文章",
                        "公众号爆款",
                    ],
                )
        }
        _ => false,
    }
}

fn autosuggest_creative_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> ParsedSkillRequest {
    match native_skill_id {
        "creative_content_calendar" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(creative_calendar_market_search_input(text, task)),
            "根据内容排期中的营销节点/热门话题需求自动追加外部市场搜索 skill",
        ),
        _ => autosuggest_request(
            "search_trends",
            serde_json::Value::Object(creative_trend_search_input(text, task, native_skill_id)),
            "根据创意内容中的实时爆款/趋势需求自动追加外部趋势搜索 skill",
        ),
    }
}

fn creative_trend_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(creative_trend_search_query(text, task, native_skill_id)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_creative_search_days(text, 14)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn creative_calendar_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "电商营销节日 热门话题 内容节点 {} 2026",
            selected_product_platform_or_infer(task, text)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("news"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_creative_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn creative_trend_search_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = infer_creative_search_subject(text, task);

    match native_skill_id {
        "creative_video_script" => {
            format!(
                "{platform} {subject} 爆款视频 {} 热门内容 2026",
                infer_video_type(text)
            )
        }
        "creative_seeding_copy" => {
            format!("{platform} {subject} 种草文案 热门风格 爆款话题 2026")
        }
        "creative_trend_catch" => {
            let category = selected_product_category_or_infer(task, text);
            format!("{category} {platform} 热门内容趋势 爆款 2026")
        }
        "creative_live_script" => {
            format!("{platform} {subject} 直播带货 直播间玩法 场控话术 转化节奏 2026")
        }
        "creative_product_article" => {
            format!("{platform} {subject} 产品软文 爆款文章 推广长文 内容风格 2026")
        }
        _ => format!("{subject} {platform} 内容趋势 热点 爆款 2026"),
    }
}

fn infer_creative_search_subject(text: &str, task: &Task) -> String {
    if let Some(product) = first_selected_product(task) {
        return product.name;
    }

    for prefix in ["请帮", "帮", "给", "为"] {
        let Some((_, rest)) = text.split_once(prefix) else {
            continue;
        };
        let rest = rest.trim_start();
        let stop = [
            "写", "做", "生成", "排", "分析", "设计", "出", "，", ",", "。", "；", ";", " ",
        ]
        .iter()
        .filter_map(|token| rest.find(token))
        .min();
        let Some(stop) = stop else {
            continue;
        };
        let candidate = rest[..stop].trim();
        let char_count = candidate.chars().count();
        if (2..=20).contains(&char_count) {
            return candidate.to_string();
        }
    }

    if let Some(subject) = extract_creative_object_subject(text) {
        return subject;
    }

    infer_search_query_for_task(text, task)
}

fn extract_creative_object_subject(text: &str) -> Option<String> {
    let action_index = ["写", "生成", "制作", "做", "出"]
        .iter()
        .filter_map(|token| text.find(token).map(|index| index + token.len()))
        .min()?;
    let rest = text[action_index..].trim_start_matches(['一', '套', '篇', '个', '条', '份']);
    let stop = [
        "直播",
        "短视频",
        "视频",
        "脚本",
        "种草",
        "笔记",
        "软文",
        "长文",
        "推广文章",
        "公众号",
        "话术",
        "文案",
        "，",
        ",",
        "。",
        "；",
        ";",
    ]
    .iter()
    .filter_map(|token| rest.find(token))
    .min()?;
    let candidate = rest[..stop].trim();
    let char_count = candidate.chars().count();
    if (2..=20).contains(&char_count) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn infer_creative_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn infer_live_duration_hours(text: &str) -> f64 {
    let Some(value) = numbers_in_text(text).first().copied() else {
        return 3.0;
    };

    if contains_any(text, &["分钟", "min", "minute", "minutes"]) {
        return (value / 60.0).clamp(0.5, 12.0);
    }

    if contains_any(text, &["小时", "个小时", "直播时长", "hour", "hours"]) {
        return value.clamp(0.5, 12.0);
    }

    3.0
}

fn autosuggest_web_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_web_native(text, task) {
        if is_web_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_web_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_web_title_seo(text, task)]
}

fn autosuggest_web(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_web_requests(text, task).into_iter().next()
}

fn autosuggest_web_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["seo", "SEO", "搜索优化", "排名", "提排"]) {
        return Some(autosuggest_request(
            "web_seo_optimize",
            serde_json::json!({
                "target_keywords": extract_keyword_candidates(text),
                "current_rank": infer_web_current_rank(text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
            }),
            "根据 SEO/提排意图自动补偿 SEO 优化 skill",
        ));
    }

    if contains_any(text, &["keyword", "关键词", "搜索词", "长尾词", "蓝海词"]) {
        let product_name = first_selected_product(task).map(|product| product.name);
        let seed_keyword = product_name.unwrap_or_else(|| web_search_subject(text, task));
        return Some(autosuggest_request(
            "web_keyword_research",
            serde_json::json!({
                "seed_keyword": seed_keyword,
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
            }),
            "根据关键词/SEO研究意图自动补偿关键词矩阵 skill",
        ));
    }

    if contains_any(text, &["标题", "title", "标题生成", "起标题"]) {
        return Some(autosuggest_request(
            "web_title_generator",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "keywords": extract_keyword_candidates(text),
                "platform": selected_product_platform_or_infer(task, text),
                "style": if contains_any(text, &["品牌"]) { "品牌型" } else if contains_any(text, &["信息"]) { "信息型" } else { "营销型" },
            }),
            "根据标题生成意图自动补偿标题生成 skill",
        ));
    }

    if contains_any(text, &["店铺", "装修", "store"]) {
        return Some(autosuggest_request(
            "web_store_design",
            serde_json::json!({
                "store_type": if contains_any(text, &["旗舰"]) { "旗舰店" } else if contains_any(text, &["专营"]) { "专营店" } else { "电商店铺" },
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
                "brand_tone": if contains_any(text, &["高端"]) { "高端奢华" } else if contains_any(text, &["年轻"]) { "年轻活力" } else { "专业可信" },
            }),
            "根据店铺装修意图自动补偿店铺设计 skill",
        ));
    }

    if contains_any(text, &["详情文案", "产品文案", "卖点文案", "description"]) {
        return Some(autosuggest_request(
            "web_product_description",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "platform": selected_product_platform_or_infer(task, text),
                "style": if contains_any(text, &["故事"]) { "故事型" } else if contains_any(text, &["信息"]) { "信息型" } else { "营销型" },
            }),
            "根据详情页文案意图自动补偿产品描述 skill",
        ));
    }

    if contains_any(
        text,
        &["page", "conversion", "页面", "转化", "详情页", "落地页"],
    ) {
        return Some(autosuggest_request(
            "web_page_conversion",
            serde_json::json!({
                "page_type": infer_page_type(text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据页面转化意图自动补偿页面转化 skill",
        ));
    }

    None
}

fn autosuggest_web_title_seo(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "web_title_seo_scorer",
        serde_json::json!({
            "title": extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description)),
            "target_keywords": extract_keyword_candidates(text),
            "platform": infer_platform(text),
        }),
        "根据 Web/SEO 岗位任务自动补偿标题 SEO 评分 skill",
    )
}

fn is_web_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    let has_fresh_or_external = has_web_external_search_intent(text);
    match native_skill_id {
        "web_seo_optimize" => {
            has_fresh_or_external
                || contains_any(
                    text,
                    &[
                        "算法",
                        "搜索算法",
                        "平台规则",
                        "平台政策",
                        "算法调整",
                        "流量规则",
                        "排名机制",
                        "关键词竞争",
                        "搜索量",
                        "流量趋势",
                        "权重",
                        "CPC",
                        "cpc",
                    ],
                )
        }
        "web_keyword_research" => {
            has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "关键词竞争",
                        "搜索量",
                        "CPC",
                        "cpc",
                        "热度",
                        "流量",
                        "竞争",
                        "出价",
                    ],
                )
        }
        "web_page_conversion" => {
            is_industry_benchmark_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "转化率基准",
                        "行业转化率",
                        "行业均值",
                        "行业平均",
                        "平均转化率",
                        "实时基准",
                        "最新基准",
                        "benchmark",
                        "CVR",
                        "cvr",
                    ],
                )
        }
        "web_title_generator" => {
            is_listing_copy_policy_search_intent(text)
                || (has_fresh_or_external
                    && contains_any(
                        text,
                        &[
                            "标题规则",
                            "标题规范",
                            "平台规则",
                            "流量规则",
                            "违禁词",
                            "审核规则",
                            "关键词竞争",
                            "搜索量",
                            "CPC",
                            "cpc",
                            "热搜词",
                            "标题趋势",
                            "高点击",
                            "点击率",
                            "CTR",
                            "ctr",
                        ],
                    ))
        }
        "web_product_description" => {
            is_listing_copy_policy_search_intent(text)
                || is_listing_copy_review_search_intent(text)
                || (has_fresh_or_external
                    && contains_any(
                        text,
                        &[
                            "详情页趋势",
                            "详情文案趋势",
                            "卖点趋势",
                            "高转化",
                            "转化卖点",
                            "买家痛点",
                            "购买顾虑",
                            "用户反馈",
                            "真实反馈",
                            "用户评价",
                            "真实评价",
                            "商品评价",
                            "买家评价",
                            "口碑",
                        ],
                    ))
        }
        _ => false,
    }
}

fn has_web_external_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "google",
            "搜一下",
            "搜搜",
            "查一下",
            "查查",
            "联网",
            "实时",
            "最新",
            "公开来源",
            "公开资料",
            "外部资料",
            "网上",
            "资料来源",
            "来源",
            "不要缓存",
            "当前",
            "近期",
            "今年",
            "今日",
            "今天",
            "站外",
            "fresh",
            "live",
            "source",
        ],
    )
}

fn autosuggest_web_search(text: &str, task: &Task, native_skill_id: &str) -> ParsedSkillRequest {
    match native_skill_id {
        "web_page_conversion" => autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(web_conversion_benchmark_search_input(text, task)),
            "根据页面转化分析中的行业转化率基准需求自动追加外部搜索 skill",
        ),
        "web_seo_optimize" if is_web_platform_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(web_platform_policy_search_input(text, task)),
            "根据 SEO 优化中的平台搜索算法/流量规则需求自动追加平台政策搜索 skill",
        ),
        "web_seo_optimize" | "web_keyword_research" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_seo_market_search_input(text, task, native_skill_id)),
            "根据 Web 增长中的关键词竞争/搜索量/SEO实时资料需求自动追加外部市场搜索 skill",
        ),
        "web_title_generator" if is_listing_copy_policy_search_intent(text) => autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(listing_copy_policy_search_input(text, task)),
            "根据标题生成中的平台标题/内容规则诉求自动追加平台规则搜索 skill",
        ),
        "web_title_generator" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_listing_copy_market_search_input(
                text,
                task,
                native_skill_id,
            )),
            "根据标题生成中的最新关键词/搜索趋势/高点击诉求自动追加外部市场搜索 skill",
        ),
        "web_product_description" if is_listing_copy_review_search_intent(text) => {
            autosuggest_request(
                "search_product_reviews",
                serde_json::Value::Object(listing_copy_reviews_search_input(text, task)),
                "根据详情文案中的买家评价/痛点诉求自动追加外部评价搜索 skill",
            )
        }
        "web_product_description" if is_listing_copy_policy_search_intent(text) => {
            autosuggest_request(
                "search_platform_policy",
                serde_json::Value::Object(listing_copy_policy_search_input(text, task)),
                "根据详情文案中的平台内容/发布规则诉求自动追加平台规则搜索 skill",
            )
        }
        "web_product_description" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_listing_copy_market_search_input(
                text,
                task,
                native_skill_id,
            )),
            "根据详情文案中的最新详情页/卖点/转化趋势诉求自动追加外部市场搜索 skill",
        ),
        _ => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(web_seo_market_search_input(text, task, native_skill_id)),
            "根据 Web 增长实时资料需求自动追加外部市场搜索 skill",
        ),
    }
}

fn is_web_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "算法",
            "搜索算法",
            "平台规则",
            "平台政策",
            "算法调整",
            "流量规则",
            "排名机制",
            "权重",
            "规则",
            "policy",
            "algorithm",
        ],
    )
}

fn web_seo_market_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(web_seo_market_query(text, task, native_skill_id)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_web_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_platform_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("algorithm"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_web_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_conversion_benchmark_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_industry_benchmarks_input(text, task);
    input.insert(
        "industry".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    input.insert("metric".to_string(), serde_json::json!("conversion"));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_listing_copy_market_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(web_listing_copy_market_query(text, task, native_skill_id)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_web_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn web_listing_copy_market_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = web_search_subject(text, task);
    match native_skill_id {
        "web_title_generator" => {
            format!("{platform} {subject} 商品标题 关键词 搜索趋势 高点击 CTR 2026")
        }
        "web_product_description" => {
            format!("{platform} {subject} 详情页文案 卖点表达 转化趋势 买家痛点 2026")
        }
        _ => format!("{platform} {subject} Web增长 文案趋势 2026"),
    }
}

fn web_seo_market_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = web_search_subject(text, task);
    match native_skill_id {
        "web_keyword_research" => {
            format!("{platform} {subject} 关键词竞争 搜索量 CPC 流量热度 2026")
        }
        _ => format!("{platform} SEO 搜索优化 {subject} 关键词竞争 流量 2026"),
    }
}

fn web_search_subject(text: &str, task: &Task) -> String {
    if let Some(product) = first_selected_product(task) {
        return product.name;
    }
    extract_keyword_candidates(text)
        .into_iter()
        .find(|keyword| !matches!(keyword.as_str(), "SEO" | "关键词" | "标题" | "转化"))
        .unwrap_or_else(|| infer_category(text).to_owned())
}

fn infer_web_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn infer_web_current_rank(text: &str) -> Option<i64> {
    if !contains_any(
        text,
        &[
            "当前排名",
            "现在排名",
            "排名第",
            "第",
            "名",
            "排第",
            "提排到",
            "rank",
            "ranking",
        ],
    ) {
        return None;
    }
    numbers_in_text(text)
        .first()
        .copied()
        .map(|value| value.round().max(1.0) as i64)
}

fn autosuggest_design_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if is_design_pure_material_policy_search_intent(text) {
        return vec![autosuggest_design_material_policy_search(text, task)];
    }

    if is_design_pure_visual_search_intent(text) {
        return vec![autosuggest_design_visual_search(
            text,
            task,
            infer_design_visual_search_skill_id(text),
        )];
    }

    let Some(primary) = autosuggest_design_native(text, task) else {
        return Vec::new();
    };

    if is_design_visual_trend_search_chain_intent(text, &primary.skill_id) {
        let search = autosuggest_design_visual_search(text, task, &primary.skill_id);
        return vec![primary, search];
    }

    if is_design_color_trend_search_chain_intent(text, &primary.skill_id) {
        return vec![primary, autosuggest_design_color_trend_search(text, task)];
    }

    if is_design_material_policy_search_chain_intent(text, &primary.skill_id) {
        return vec![
            primary,
            autosuggest_design_material_policy_search(text, task),
        ];
    }

    vec![primary]
}

fn autosuggest_design(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_design_requests(text, task).into_iter().next()
}

fn autosuggest_design_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["material", "asset", "素材", "规格", "尺寸"]) {
        return Some(autosuggest_request(
            "design_material_spec",
            serde_json::json!({
                "usage": infer_design_usage(text),
                "platforms": [selected_product_platform_or_infer(task, text)],
            }),
            "根据素材/规格意图自动补偿素材规范 skill",
        ));
    }

    if contains_any(text, &["详情页", "详情", "长图"]) {
        return Some(autosuggest_request(
            "design_detail_page",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "platform": selected_product_platform_or_infer(task, text),
                "style": infer_design_style(text),
            }),
            "根据详情页设计意图自动补偿详情页设计 skill",
        ));
    }

    if contains_any(text, &["配色", "色彩", "品牌色", "颜色"]) {
        return Some(autosuggest_request(
            "design_color_scheme",
            serde_json::json!({
                "brand_tone": infer_design_style(text),
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
            }),
            "根据配色/品牌色意图自动补偿配色方案 skill",
        ));
    }

    if contains_any(text, &["海报", "banner", "Banner", "活动图", "活动海报"]) {
        return Some(autosuggest_request(
            "design_campaign_poster",
            serde_json::json!({
                "campaign_name": if contains_any(text, &["618"]) { "618大促" } else if contains_any(text, &["双11", "双十一"]) { "双11大促" } else { "营销活动" },
                "discount_info": extract_discount_hint(text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
                "style": infer_design_style(text),
            }),
            "根据活动海报意图自动补偿海报设计 skill",
        ));
    }

    Some(autosuggest_request(
        "design_main_image",
        serde_json::json!({
            "product_name": selected_product_name_or_task(task),
            "product_ref": selected_product_ref(task),
            "platform": selected_product_platform_or_infer(task, text),
        }),
        "根据设计岗位任务自动补偿主图设计 skill",
    ))
}

fn is_design_visual_trend_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "design_main_image" | "design_campaign_poster" | "design_detail_page"
    ) && has_design_visual_trend_search_intent(text)
}

fn is_design_color_trend_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    native_skill_id == "design_color_scheme" && has_design_color_trend_search_intent(text)
}

fn is_design_material_policy_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    native_skill_id == "design_material_spec" && has_design_material_policy_search_intent(text)
}

fn has_design_visual_trend_search_intent(text: &str) -> bool {
    let has_design_signal = contains_any(
        text,
        &[
            "视觉趋势",
            "设计趋势",
            "主图趋势",
            "详情页趋势",
            "海报趋势",
            "爆款主图",
            "爆款海报",
            "高点击率",
            "高转化详情页",
            "竞品视觉",
            "热门视觉",
            "流行视觉",
            "风格趋势",
            "素材趋势",
            "设计参考",
            "视觉参考",
        ],
    );
    let has_fresh_or_evidence = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "最新",
                "近期",
                "当前",
                "今年",
                "2026",
                "公开来源",
                "公开资料",
                "参考",
                "对标",
                "竞品",
                "爆款",
            ],
        );

    has_design_signal && has_fresh_or_evidence
}

fn has_design_color_trend_search_intent(text: &str) -> bool {
    let has_color_signal = contains_any(
        text,
        &[
            "配色趋势",
            "色彩趋势",
            "流行色",
            "年度色",
            "品牌色趋势",
            "竞品配色",
            "热门配色",
            "流行配色",
            "视觉色彩",
            "color trend",
            "palette trend",
        ],
    ) || (contains_any(text, &["配色", "色彩", "品牌色", "颜色"])
        && contains_any(text, &["趋势", "流行", "热门", "竞品", "参考", "对标"]));

    let has_fresh_or_evidence = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "最新",
                "近期",
                "当前",
                "今年",
                "2026",
                "公开来源",
                "公开资料",
                "参考",
                "对标",
                "竞品",
            ],
        );

    has_color_signal && has_fresh_or_evidence
}

fn has_design_material_policy_search_intent(text: &str) -> bool {
    let has_material_signal =
        contains_any(
            text,
            &[
                "素材规范",
                "素材规格",
                "广告素材规格",
                "广告位规格",
                "尺寸规范",
                "尺寸规则",
                "平台尺寸",
                "信息流尺寸",
                "上传规范",
                "文件大小",
                "安全区",
                "审核规范",
                "格式要求",
                "ad spec",
                "asset spec",
            ],
        ) || (contains_any(text, &["素材", "规格", "尺寸", "广告位", "格式"])
            && contains_any(
                text,
                &["规则", "规范", "要求", "审核", "平台", "官方", "最新"],
            ));

    let has_policy_or_fresh = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "最新",
                "近期",
                "当前",
                "今年",
                "2026",
                "规则",
                "规范",
                "要求",
                "平台",
                "官方",
                "公开来源",
                "公开资料",
            ],
        );

    has_material_signal && has_policy_or_fresh
}

fn is_design_pure_visual_search_intent(text: &str) -> bool {
    has_design_visual_trend_search_intent(text)
        && contains_any(
            text,
            &["查", "查一下", "搜索", "搜", "联网", "公开来源", "公开资料"],
        )
        && !contains_any(
            text,
            &[
                "做",
                "生成",
                "规划",
                "输出",
                "设计方案",
                "设计一版",
                "制作",
                "创建",
                "给方案",
                "出方案",
                "优化",
                "改版",
            ],
        )
}

fn is_design_pure_material_policy_search_intent(text: &str) -> bool {
    has_design_material_policy_search_intent(text)
        && contains_any(
            text,
            &["查", "查一下", "搜索", "搜", "联网", "公开来源", "公开资料"],
        )
        && !contains_any(
            text,
            &[
                "做",
                "生成",
                "规划",
                "输出",
                "整理",
                "设计方案",
                "制作",
                "创建",
                "给方案",
                "出方案",
                "交付",
            ],
        )
}

fn infer_design_visual_search_skill_id(text: &str) -> &'static str {
    if contains_any(text, &["海报", "banner", "Banner", "活动图", "活动海报"]) {
        "design_campaign_poster"
    } else if contains_any(text, &["详情页", "详情", "长图"]) {
        "design_detail_page"
    } else {
        "design_main_image"
    }
}

fn autosuggest_design_color_trend_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_trends",
        serde_json::Value::Object(design_color_trend_search_input(text, task)),
        "根据设计配色任务中的最新色彩趋势/竞品配色参考需求自动追加外部趋势搜索 skill",
    )
}

fn design_color_trend_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(design_color_trend_search_query(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_design_search_days(text, 90)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn design_color_trend_search_query(text: &str, task: &Task) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = design_visual_search_subject(text, task);
    let style = infer_design_style(text);
    format!("{platform} {subject} {style} 品牌配色 色彩趋势 流行色 竞品配色 2026")
}

fn autosuggest_design_material_policy_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_platform_policy",
        serde_json::Value::Object(design_material_policy_search_input(text, task)),
        "根据设计素材规范任务中的平台尺寸/审核/格式规则需求自动追加平台规则搜索 skill",
    )
}

fn design_material_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("material"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_design_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn autosuggest_design_visual_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> ParsedSkillRequest {
    autosuggest_request(
        "search_trends",
        serde_json::Value::Object(design_visual_trend_search_input(
            text,
            task,
            native_skill_id,
        )),
        "根据设计任务中的最新视觉趋势/爆款参考需求自动追加外部趋势搜索 skill",
    )
}

fn design_visual_trend_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(design_visual_trend_search_query(
            text,
            task,
            native_skill_id
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_design_search_days(
            text,
            design_visual_trend_default_days(native_skill_id),
        )),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn design_visual_trend_search_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = design_visual_search_subject(text, task);

    match native_skill_id {
        "design_campaign_poster" => {
            format!(
                "{} {platform} {subject} 活动海报 设计趋势 爆款 2026",
                infer_campaign_name(text)
            )
        }
        "design_detail_page" => {
            format!("{platform} {subject} 详情页设计 高转化 视觉趋势 2026")
        }
        _ => format!("{platform} {subject} 主图设计 爆款 高点击率 视觉趋势 2026"),
    }
}

fn design_visual_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| selected_product_category_or_infer(task, text))
}

fn design_visual_trend_default_days(native_skill_id: &str) -> i64 {
    match native_skill_id {
        "design_campaign_poster" => 30,
        "design_detail_page" => 60,
        _ => 60,
    }
}

fn infer_design_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}

fn autosuggest_engineering_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_engineering_native(text) {
        if is_engineering_external_knowledge_search_chain_intent(text, &primary.skill_id) {
            return vec![
                primary,
                autosuggest_engineering_knowledge_search(text, task),
            ];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    Vec::new()
}

fn autosuggest_engineering(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_engineering_requests(text, task)
        .into_iter()
        .next()
}

fn autosuggest_engineering_native(text: &str) -> Option<ParsedSkillRequest> {
    if has_platform_health_check_intent(text) {
        let test_live = contains_any(text, &["实际", "真实", "连通", "连通性", "测试", "live"]);
        return Some(autosuggest_request(
            "engineering_system_check",
            serde_json::json!({
                "platform": infer_platform_health_check_platform(text),
                "test_live": test_live,
            }),
            if test_live {
                "根据平台接口连通性意图自动补偿平台健康检查 skill，实际检测需审批"
            } else {
                "根据平台连接配置检查意图自动补偿平台健康检查 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "状态",
            "status",
            "进度",
            "session",
            "会话",
            "workspace",
            "工作区",
            "run",
            "任务节点",
            "系统健康",
            "health",
            "agent运行",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_status_query",
            serde_json::json!({
                "query_type": infer_status_query_type_from_text(text),
            }),
            "根据运行状态/系统健康意图自动补偿协调状态查询 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "architecture",
            "架构",
            "重构",
            "扩展性",
            "系统设计",
            "系统架构",
        ],
    ) {
        return Some(autosuggest_request(
            "engineering_arch_review",
            serde_json::json!({
                "system_type": infer_system_type(text),
                "daily_orders": infer_integer_or_default(text, 1000),
                "pain_points": infer_pain_points(text),
            }),
            "根据架构评估意图自动补偿架构评估 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "bug",
            "故障",
            "报错",
            "异常报错",
            "排查",
            "白屏",
            "错误日志",
        ],
    ) {
        return Some(autosuggest_request(
            "engineering_bug_analysis",
            serde_json::json!({
                "error_type": infer_bug_type(text),
                "context": compact_task_text(text),
                "frequency": infer_bug_frequency(text),
            }),
            "根据故障排查意图自动补偿Bug排查 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "performance",
            "性能",
            "慢",
            "卡",
            "延迟",
            "qps",
            "吞吐",
            "优化",
        ],
    ) && !has_tech_selection_intent(text)
    {
        let numbers = numbers_in_text(text);
        return Some(autosuggest_request(
            "engineering_perf_optimize",
            serde_json::json!({
                "bottleneck": infer_perf_bottleneck(text),
                "current_qps": numbers.first().copied().unwrap_or(500.0),
                "target_qps": numbers.get(1).copied().unwrap_or(2_000.0),
                "tech_stack": infer_stack_hint(text),
            }),
            "根据性能优化意图自动补偿性能优化 skill",
        ));
    }

    if has_tech_selection_intent(text) {
        return Some(autosuggest_request(
            "engineering_tech_selection",
            serde_json::json!({
                "domain": infer_tech_domain(text),
                "current_stack": infer_stack_hint(text),
                "constraints": infer_pain_points(text),
            }),
            "根据技术选型意图自动补偿技术选型 skill",
        ));
    }

    Some(autosuggest_request(
        "engineering_sla_monitor",
        serde_json::json!({}),
        "根据工程岗位任务自动补偿 SLA 监控 skill",
    ))
}

fn has_tech_selection_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "tech selection",
            "技术选型",
            "框架选择",
            "数据库选型",
            "选型",
        ],
    )
}

fn is_engineering_external_knowledge_search_chain_intent(
    text: &str,
    native_skill_id: &str,
) -> bool {
    match native_skill_id {
        "engineering_bug_analysis" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "已知解决方案",
                        "解决方案",
                        "最佳实践",
                        "公开资料",
                        "公开来源",
                        "社区案例",
                        "案例",
                    ],
                )
        }
        "engineering_tech_selection" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "benchmark",
                        "Benchmark",
                        "基准测试",
                        "性能对比",
                        "最新对比",
                        "技术对比",
                        "公开资料",
                        "公开来源",
                    ],
                )
        }
        "engineering_perf_optimize" | "engineering_arch_review" => {
            has_external_search_intent(text)
                && contains_any(
                    text,
                    &[
                        "benchmark",
                        "Benchmark",
                        "基准",
                        "最佳实践",
                        "公开资料",
                        "公开来源",
                        "性能资料",
                        "架构案例",
                        "案例",
                    ],
                )
        }
        _ => false,
    }
}

fn autosuggest_engineering_knowledge_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_market_info",
        serde_json::Value::Object(engineering_knowledge_search_input(text, task)),
        "根据工程任务中的外部资料/benchmark/最佳实践需求自动追加外部市场资料搜索 skill",
    )
}

fn engineering_knowledge_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(engineering_knowledge_search_query(text)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_engineering_search_days(text, 90)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn engineering_knowledge_search_query(text: &str) -> String {
    if contains_any(
        text,
        &[
            "bug",
            "故障",
            "报错",
            "异常报错",
            "排查",
            "白屏",
            "错误日志",
        ],
    ) {
        return format!(
            "{} {} 解决方案 排查 最佳实践 2026",
            infer_bug_type(text),
            infer_stack_hint(text)
        );
    }

    if has_tech_selection_intent(text) {
        return format!(
            "{} 技术选型 对比 性能 benchmark 2026",
            infer_tech_domain(text)
        );
    }

    if contains_any(
        text,
        &[
            "performance",
            "性能",
            "慢",
            "卡",
            "延迟",
            "qps",
            "吞吐",
            "优化",
        ],
    ) {
        return format!(
            "{} {} 性能优化 benchmark 最佳实践 2026",
            infer_stack_hint(text),
            infer_perf_bottleneck(text)
        );
    }

    format!(
        "{} {} 架构 benchmark 最佳实践 2026",
        infer_system_type(text),
        infer_stack_hint(text)
    )
}

fn infer_engineering_search_days(text: &str, default: i64) -> i64 {
    infer_explicit_period_days(text).unwrap_or(default)
}

fn autosuggest_search(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if is_platform_policy_search_intent(text) {
        let input = search_platform_policy_input(text, task);
        return Some(autosuggest_request(
            "search_platform_policy",
            serde_json::Value::Object(input),
            "根据平台规则/政策实时查询意图自动补偿外部搜索 skill",
        ));
    }

    if is_product_review_search_intent(text) {
        let input = search_product_reviews_input(text, task);
        return Some(autosuggest_request(
            "search_product_reviews",
            serde_json::Value::Object(input),
            "根据用户评价/口碑实时查询意图自动补偿外部搜索 skill",
        ));
    }

    if is_industry_benchmark_search_intent(text) {
        let input = search_industry_benchmarks_input(text, task);
        return Some(autosuggest_request(
            "search_industry_benchmarks",
            serde_json::Value::Object(input),
            "根据行业基准/均值查询意图自动补偿外部搜索 skill",
        ));
    }

    if !has_external_search_intent(text) {
        return None;
    }

    if contains_any(
        text,
        &["competitor", "竞品", "竞对", "竞争对手", "对标", "同类商品"],
    ) {
        let input = search_competitor_input(text, task);
        return Some(autosuggest_request(
            "search_competitor",
            serde_json::Value::Object(input),
            "根据竞品外部资料查询意图自动补偿外部搜索 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "trend",
            "hot",
            "趋势",
            "热点",
            "热卖",
            "爆款",
            "流行",
            "消费趋势",
        ],
    ) {
        let input = search_trends_input(text, task);
        return Some(autosuggest_request(
            "search_trends",
            serde_json::Value::Object(input),
            "根据趋势/热点实时查询意图自动补偿外部搜索 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "market",
            "市场",
            "行业",
            "新闻",
            "资讯",
            "报告",
            "行情",
            "资料",
            "公开来源",
            "财务",
            "财务基准",
            "财务对标",
            "毛利率",
            "净利率",
        ],
    ) {
        let input = search_market_info_input(text, task);
        return Some(autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(input),
            "根据市场行情/公开资料查询意图自动补偿外部搜索 skill",
        ));
    }

    None
}

fn autosuggest_query_store_metrics(
    text: &str,
    explicit_metrics_only: bool,
) -> Option<ParsedSkillRequest> {
    let input = infer_store_metrics_input(text);
    let has_explicit_metrics = store_metrics_input_has_metrics(&input);
    let has_query_intent = has_store_metrics_query_intent(text);
    if explicit_metrics_only && !has_explicit_metrics {
        return None;
    }
    if !has_query_intent {
        return None;
    }

    Some(autosuggest_request(
        "query_store_metrics",
        serde_json::Value::Object(input),
        if has_explicit_metrics {
            "根据用户显式提供的店铺指标自动补偿店铺指标查询 skill"
        } else {
            "根据查询/加载店铺指标意图自动补偿店铺指标查询 skill"
        },
    ))
}

fn autosuggest_product_or_campaign_metrics(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if has_campaign_metrics_query_intent(text) {
        let input = infer_campaign_metrics_input(text);
        let has_explicit_metrics = metric_query_input_has_metrics(&input);
        return Some(autosuggest_request(
            "query_campaign_metrics",
            serde_json::Value::Object(input),
            if has_explicit_metrics {
                "根据用户显式提供的活动指标自动补偿活动指标查询 skill"
            } else {
                "根据查询/加载活动指标意图自动补偿活动指标查询 skill"
            },
        ));
    }

    if has_product_metrics_query_intent(text) {
        let input = infer_product_metrics_input(text, task);
        let has_explicit_metrics = metric_query_input_has_metrics(&input);
        return Some(autosuggest_request(
            "query_product_metrics",
            serde_json::Value::Object(input),
            if has_explicit_metrics {
                "根据用户显式提供的商品指标自动补偿商品指标查询 skill"
            } else {
                "根据查询/加载商品指标意图自动补偿商品指标查询 skill"
            },
        ));
    }

    None
}

fn autosuggest_request(
    skill_id: impl Into<String>,
    input: serde_json::Value,
    reason: impl Into<String>,
) -> ParsedSkillRequest {
    ParsedSkillRequest {
        skill_id: skill_id.into(),
        input: compact_skill_input(input),
        reason: Some(reason.into()),
    }
}

fn compact_skill_input(input: serde_json::Value) -> serde_json::Value {
    match input {
        serde_json::Value::Object(values) => serde_json::Value::Object(
            values
                .into_iter()
                .filter_map(|(key, value)| {
                    compact_skill_input_field(value).map(|value| (key, value))
                })
                .collect(),
        ),
        value => value,
    }
}

fn compact_skill_input_field(value: serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Array(values) => {
            let values = values
                .into_iter()
                .filter_map(compact_skill_input_field)
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(serde_json::Value::Array(values))
            }
        }
        serde_json::Value::Object(values) => {
            let values = values
                .into_iter()
                .filter_map(|(key, value)| {
                    compact_skill_input_field(value).map(|value| (key, value))
                })
                .collect();
            Some(serde_json::Value::Object(values))
        }
        value => Some(value),
    }
}

fn autosuggest_text(task: &Task, output: &str) -> String {
    let mut parts = vec![task.description.clone(), output.to_string()];
    for key in [
        "context.summary",
        "context.source_titles",
        "rag.summary",
        "rag.source_titles",
    ] {
        if let Some(value) = task.context.metadata.get(key) {
            parts.push(value.clone());
        }
    }
    if let Some(assignment) = &task.assignment {
        parts.push(assignment.objective.clone());
        parts.extend(assignment.inputs.iter().map(|input| input.content.clone()));
    }
    parts.join("\n")
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}

fn numbers_in_text(text: &str) -> Vec<f64> {
    let mut values = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
            continue;
        }
        push_parsed_number(&mut values, &mut current);
    }
    push_parsed_number(&mut values, &mut current);
    values
}

fn infer_store_metrics_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_store_metrics_platform(text)),
    );
    input.insert("include_anomalies".to_string(), serde_json::json!(true));

    insert_metric_value(
        &mut input,
        "previous_gmv",
        text,
        &["previous_gmv", "last_gmv", "上期GMV", "上周GMV", "前期GMV"],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &[
            "本期GMV",
            "当前GMV",
            "GMV",
            "gmv",
            "销售额",
            "成交额",
            "营收",
        ],
    );
    insert_metric_value(
        &mut input,
        "previous_orders",
        text,
        &[
            "previous_orders",
            "last_orders",
            "上期订单",
            "上周订单",
            "前期订单",
        ],
    );
    insert_metric_value(
        &mut input,
        "orders",
        text,
        &[
            "本期订单",
            "当前订单",
            "订单数",
            "订单",
            "成交订单",
            "支付订单",
        ],
    );
    insert_metric_value(
        &mut input,
        "previous_visitors",
        text,
        &[
            "previous_visitors",
            "last_visitors",
            "上期UV",
            "上周UV",
            "前期UV",
        ],
    );
    insert_metric_value(
        &mut input,
        "visitors",
        text,
        &["本期UV", "当前UV", "UV", "uv", "访客数", "访客"],
    );
    insert_metric_value(
        &mut input,
        "clicks",
        text,
        &["点击数", "点击", "clicks", "click_count"],
    );
    insert_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "投放费用",
            "ad_spend",
            "ad spend",
            "ad_cost",
        ],
    );
    insert_metric_value(
        &mut input,
        "refund_amount",
        text,
        &["退款金额", "退款额", "退款", "refund_amount", "refunds"],
    );
    insert_metric_value(
        &mut input,
        "gross_profit",
        text,
        &["毛利额", "毛利", "gross_profit", "profit"],
    );

    input
}

fn infer_dashboard_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_store_metrics_input(text);
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    insert_metric_value(
        &mut input,
        "target_gmv",
        text,
        &[
            "目标GMV",
            "目标销售额",
            "目标成交额",
            "target_gmv",
            "goal_gmv",
        ],
    );
    input
}

fn infer_comprehensive_diagnosis_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_dashboard_input(text, task);
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_diagnosis_focus(text)),
    );

    let channels = infer_channel_roi_rows(text);
    if !channels.is_empty() {
        input.insert("channels".to_string(), serde_json::json!(channels));
    }

    let items = infer_refund_items(text);
    if !items.is_empty() {
        input.insert("items".to_string(), serde_json::json!(items));
    }

    let values = numbers_in_text(text);
    if values.len() >= 3 {
        input.insert("values".to_string(), serde_json::json!(values.clone()));
    }
    if values.len() >= 5 {
        input
            .entry("impressions".to_string())
            .or_insert_with(|| serde_json::json!(values[0].round().max(0.0) as i64));
        input
            .entry("clicks".to_string())
            .or_insert_with(|| serde_json::json!(values[1].round().max(0.0) as i64));
        input
            .entry("add_to_cart".to_string())
            .or_insert_with(|| serde_json::json!(values[2].round().max(0.0) as i64));
        input
            .entry("orders".to_string())
            .or_insert_with(|| serde_json::json!(values[3].round().max(0.0) as i64));
        input
            .entry("payments".to_string())
            .or_insert_with(|| serde_json::json!(values[4].round().max(0.0) as i64));
    }

    input
}

fn infer_diagnosis_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "重点看增长",
            "重点分析增长",
            "聚焦增长",
            "增长为主",
            "增长优先",
            "focus growth",
        ],
    ) {
        "growth"
    } else if contains_any(text, &["profit", "利润", "毛利", "盈利", "ROI", "roi"]) {
        "profit"
    } else if contains_any(text, &["retention", "留存", "复购", "客户", "会员"]) {
        "retention"
    } else if contains_any(text, &["risk", "风险", "异常", "退款", "下滑"]) {
        "risk"
    } else if contains_any(text, &["growth", "增长", "放量", "规模", "GMV", "gmv"]) {
        "growth"
    } else {
        "all"
    }
}

fn infer_product_metrics_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_common_metrics_input(text);
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if let Some(product) = first_selected_product(task) {
        input.insert("product_name".to_string(), serde_json::json!(product.name));
    } else {
        let product_name = infer_product_name(text);
        if !product_name.trim().is_empty() {
            input.insert("product_name".to_string(), serde_json::json!(product_name));
        }
    }
    if let Some(product_id) = extract_labeled_token(text, &["商品ID", "product_id", "product id"])
    {
        input.insert("product_id".to_string(), serde_json::json!(product_id));
    }
    if let Some(sku_id) = extract_labeled_token(text, &["SKU", "sku_id", "sku id", "规格ID"]) {
        input.insert("sku_id".to_string(), serde_json::json!(sku_id));
    }
    insert_metric_value(
        &mut input,
        "inventory",
        text,
        &["库存", "剩余库存", "inventory", "stock"],
    );
    insert_metric_value(
        &mut input,
        "stockout_count",
        text,
        &["缺货次数", "断货次数", "stockout_count", "stockout"],
    );
    input
}

fn infer_campaign_metrics_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = infer_common_metrics_input(text);
    let campaign_name = infer_campaign_name(text);
    if !campaign_name.trim().is_empty() {
        input.insert(
            "campaign_name".to_string(),
            serde_json::json!(campaign_name),
        );
    }
    if let Some(campaign_id) =
        extract_labeled_token(text, &["活动ID", "campaign_id", "campaign id"])
    {
        input.insert("campaign_id".to_string(), serde_json::json!(campaign_id));
    }
    input.insert(
        "campaign_type".to_string(),
        serde_json::json!(infer_campaign_type(text)),
    );
    insert_metric_value(&mut input, "budget", text, &["预算", "budget", "活动预算"]);
    insert_metric_value(
        &mut input,
        "spend",
        text,
        &["活动花费", "实际花费", "已花费", "消耗", "spend", "cost"],
    );
    insert_metric_value(
        &mut input,
        "impressions",
        text,
        &[
            "曝光",
            "曝光量",
            "展现",
            "展现量",
            "impressions",
            "exposure",
        ],
    );
    insert_metric_value(
        &mut input,
        "coupon_cost",
        text,
        &["优惠券成本", "券成本", "让利成本", "coupon_cost"],
    );
    input
}

fn infer_common_metrics_input(text: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_store_metrics_platform(text)),
    );
    insert_metric_value(
        &mut input,
        "previous_gmv",
        text,
        &["previous_gmv", "last_gmv", "上期GMV", "上周GMV", "前期GMV"],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &[
            "本期GMV",
            "当前GMV",
            "活动GMV",
            "商品GMV",
            "GMV",
            "gmv",
            "销售额",
            "成交额",
            "营收",
        ],
    );
    insert_metric_value(
        &mut input,
        "orders",
        text,
        &["订单数", "订单", "销量", "成交单", "orders", "order_count"],
    );
    insert_metric_value(
        &mut input,
        "visitors",
        text,
        &["访客数", "访客", "UV", "uv", "visitors"],
    );
    insert_metric_value(
        &mut input,
        "clicks",
        text,
        &["点击数", "点击", "clicks", "click_count"],
    );
    insert_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告费",
            "广告花费",
            "投放费用",
            "投放花费",
            "ad_spend",
            "ad spend",
            "ad_cost",
        ],
    );
    insert_metric_value(
        &mut input,
        "refund_amount",
        text,
        &["退款金额", "退款额", "退款", "refund_amount", "refunds"],
    );
    insert_metric_value(
        &mut input,
        "gross_profit",
        text,
        &["毛利额", "毛利", "gross_profit", "profit"],
    );
    input
}

fn insert_metric_value(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text: &str,
    aliases: &[&str],
) {
    if let Some(value) = extract_metric_value(text, aliases) {
        input.insert(key.to_string(), serde_json::json!(value));
    }
}

fn infer_trend_forecast_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "metric_name".to_string(),
        serde_json::json!(infer_metric(text)),
    );
    input.insert(
        "forecast_periods".to_string(),
        serde_json::json!(infer_forecast_periods(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_anomaly_diagnosis_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "metric_name".to_string(),
        serde_json::json!(infer_metric(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_customer_segmentation_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if let Some(total_customers) = extract_metric_value(
        text,
        &[
            "客户总数",
            "总客户数",
            "客户数",
            "用户数",
            "total_customers",
        ],
    ) {
        input.insert(
            "total_customers".to_string(),
            serde_json::json!(total_customers.round().max(1.0) as i64),
        );
    }
    insert_metric_value(
        &mut input,
        "avg_order_value",
        text,
        &["客单价", "平均客单价", "AOV", "aov", "avg_order_value"],
    );
    insert_metric_value(
        &mut input,
        "avg_orders_per_customer",
        text,
        &[
            "人均订单数",
            "人均购买次数",
            "平均订单数",
            "avg_orders_per_customer",
        ],
    );
    insert_metric_value(
        &mut input,
        "recency_days",
        text,
        &["平均购买间隔", "购买间隔", "最近购买间隔", "recency_days"],
    );
    input
}

fn infer_ab_test_analyzer_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "test_name".to_string(),
        serde_json::json!(infer_ab_test_name(text)),
    );
    input.insert(
        "metric_name".to_string(),
        serde_json::json!(infer_ab_metric(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }

    if let Some(segment) = ab_group_segment(text, AB_CONTROL_LABELS, AB_TEST_LABELS) {
        insert_ab_group_metrics(&mut input, "control", &segment);
    }
    if let Some(segment) = ab_group_segment(text, AB_TEST_LABELS, AB_CONTROL_LABELS) {
        insert_ab_group_metrics(&mut input, "test", &segment);
    }
    insert_ab_direct_metrics(&mut input, text);

    let required_keys = [
        "control_visitors",
        "control_conversions",
        "test_visitors",
        "test_conversions",
    ];
    if required_keys.iter().any(|key| !input.contains_key(*key)) {
        let numbers = numbers_in_text(text);
        if numbers.len() >= 4 {
            for (key, value) in required_keys.iter().zip(numbers.into_iter().take(4)) {
                input
                    .entry((*key).to_string())
                    .or_insert_with(|| serde_json::json!(value.round().max(0.0) as i64));
            }
        }
    }

    input
}

const AB_CONTROL_LABELS: &[&str] = &[
    "对照组",
    "控制组",
    "control group",
    "control",
    "A组",
    "A 组",
    "A版",
    "A 版",
];
const AB_TEST_LABELS: &[&str] = &[
    "实验组",
    "测试组",
    "test group",
    "variant",
    "B组",
    "B 组",
    "B版",
    "B 版",
];

fn ab_group_segment(text: &str, labels: &[&str], other_labels: &[&str]) -> Option<String> {
    let combined = labels
        .iter()
        .chain(other_labels.iter())
        .copied()
        .collect::<Vec<_>>();
    labels
        .iter()
        .find_map(|label| segment_after_label(text, label, &combined))
        .filter(|segment| !segment.trim().is_empty())
}

fn insert_ab_group_metrics(
    input: &mut serde_json::Map<String, serde_json::Value>,
    prefix: &str,
    segment: &str,
) {
    if let Some(value) = extract_metric_value(
        segment,
        &[
            "访客数",
            "访客",
            "UV",
            "uv",
            "样本量",
            "样本",
            "人数",
            "visitors",
            "visitor",
            "users",
            "traffic",
        ],
    ) {
        input.insert(
            format!("{prefix}_visitors"),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
    let conversions = extract_metric_value(
        segment,
        &[
            "转化数",
            "转化人数",
            "支付人数",
            "成交人数",
            "成交数",
            "订单数",
            "订单",
            "conversions",
            "conversion_count",
            "orders",
        ],
    )
    .or_else(|| {
        if contains_any(segment, &["转化率", "conversion rate", "cvr"]) {
            None
        } else {
            extract_metric_value(segment, &["转化", "成交", "支付"])
        }
    });
    if let Some(value) = conversions {
        input.insert(
            format!("{prefix}_conversions"),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
}

fn insert_ab_direct_metrics(input: &mut serde_json::Map<String, serde_json::Value>, text: &str) {
    insert_ab_integer(
        input,
        "control_visitors",
        text,
        &[
            "control_visitors",
            "control visitors",
            "对照组访客数",
            "对照组访客",
            "控制组访客数",
            "控制组访客",
            "A组访客数",
            "A组访客",
            "A版访客",
        ],
    );
    insert_ab_integer(
        input,
        "control_conversions",
        text,
        &[
            "control_conversions",
            "control conversions",
            "对照组转化数",
            "对照组转化",
            "控制组转化数",
            "控制组转化",
            "A组转化数",
            "A组转化",
            "A版转化",
        ],
    );
    insert_ab_integer(
        input,
        "test_visitors",
        text,
        &[
            "test_visitors",
            "test visitors",
            "实验组访客数",
            "实验组访客",
            "测试组访客数",
            "测试组访客",
            "B组访客数",
            "B组访客",
            "B版访客",
        ],
    );
    insert_ab_integer(
        input,
        "test_conversions",
        text,
        &[
            "test_conversions",
            "test conversions",
            "实验组转化数",
            "实验组转化",
            "测试组转化数",
            "测试组转化",
            "B组转化数",
            "B组转化",
            "B版转化",
        ],
    );
}

fn insert_ab_integer(
    input: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    text: &str,
    aliases: &[&str],
) {
    if let Some(value) = extract_metric_value(text, aliases) {
        input.insert(
            key.to_string(),
            serde_json::json!(value.round().max(0.0) as i64),
        );
    }
}

fn infer_ab_test_name(text: &str) -> &'static str {
    if contains_any(text, &["首屏", "详情首屏", "详情页首屏", "above the fold"]) {
        "详情页首屏A/B测试"
    } else if contains_any(text, &["主图", "封面", "image"]) {
        "主图A/B测试"
    } else if contains_any(text, &["标题", "title"]) {
        "标题A/B测试"
    } else if contains_any(text, &["详情页", "详情", "detail"]) {
        "详情页A/B测试"
    } else if contains_any(text, &["直播", "live"]) {
        "直播间A/B测试"
    } else {
        "A/B测试"
    }
}

fn infer_ab_metric(text: &str) -> &'static str {
    if contains_any(text, &["点击", "ctr", "CTR", "click"]) {
        "点击率"
    } else if contains_any(text, &["支付", "成交", "订单", "purchase", "order"]) {
        "支付转化率"
    } else {
        "转化率"
    }
}

fn infer_forecast_periods(text: &str) -> i64 {
    if let Some(days) = extract_metric_value(text, &["未来", "预测", "forecast_periods"]) {
        days.round().clamp(1.0, 90.0) as i64
    } else {
        7
    }
}

fn infer_ltv_calculator_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    insert_metric_value(
        &mut input,
        "cac",
        text,
        &["CAC", "cac", "获客成本", "获客成本CAC"],
    );
    insert_metric_value(
        &mut input,
        "avg_order_value",
        text,
        &["客单价", "AOV", "aov", "avg_order_value"],
    );
    insert_metric_value(
        &mut input,
        "purchase_frequency",
        text,
        &["购买频次", "月购买频次", "复购频次", "purchase_frequency"],
    );
    insert_metric_value(
        &mut input,
        "gross_margin",
        text,
        &["毛利率", "gross_margin"],
    );
    insert_metric_value(
        &mut input,
        "monthly_retention",
        text,
        &["月留存率", "留存率", "monthly_retention"],
    );
    input
}

fn infer_cohort_analysis_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "cohort_unit".to_string(),
        serde_json::json!(infer_cohort_unit(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if let Some(new_customers) = extract_metric_value(
        text,
        &["首期新客", "新客数", "新增客户", "新客", "new_customers"],
    ) {
        input.insert(
            "new_customers".to_string(),
            serde_json::json!(new_customers.round().max(1.0) as i64),
        );
    }
    input
}

fn infer_seasonal_decompose_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    input.insert(
        "period".to_string(),
        serde_json::json!(infer_seasonal_period(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_multi_period_trend_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    input.insert(
        "periods".to_string(),
        serde_json::json!(infer_multi_periods(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_multi_periods(text: &str) -> Vec<i64> {
    if contains_any(text, &["周内", "周度", "每周", "weekly", "week"]) {
        vec![7, 14, 28]
    } else {
        vec![7, 14, 30]
    }
}

fn infer_price_elasticity_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let price_points = infer_price_quantity_points(text);
    if !price_points.is_empty() {
        input.insert("price_points".to_string(), serde_json::json!(price_points));
    }
    insert_metric_value(&mut input, "cost", text, &["成本", "单品成本", "cost"]);
    insert_metric_value(
        &mut input,
        "current_price",
        text,
        &["当前售价", "当前价格", "现价", "current_price"],
    );
    input
}

fn infer_smart_pricing_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    insert_metric_value(
        &mut input,
        "cost",
        text,
        &["单位成本", "单品成本", "成本", "cost"],
    );
    insert_metric_value(
        &mut input,
        "current_price",
        text,
        &[
            "当前售价",
            "当前价格",
            "现价",
            "售价",
            "current_price",
            "price",
        ],
    );
    insert_metric_value(
        &mut input,
        "current_monthly_sales",
        text,
        &[
            "当前月销量",
            "月销量",
            "monthly_sales",
            "current_monthly_sales",
        ],
    );
    insert_metric_value(
        &mut input,
        "stock_days",
        text,
        &["库存天数", "可售天数", "stock_days"],
    );
    insert_metric_value(
        &mut input,
        "elasticity",
        text,
        &["弹性系数", "价格弹性", "elasticity"],
    );
    if let Some(value) = input.get("elasticity").and_then(serde_json::Value::as_f64)
        && value > 0.0
    {
        input.insert("elasticity".to_string(), serde_json::json!(-value));
    }
    let competitor_prices = infer_competitor_prices(text);
    if !competitor_prices.is_empty() {
        input.insert(
            "competitor_prices".to_string(),
            serde_json::json!(competitor_prices),
        );
    }
    input
}

fn infer_pricing_strategy_input(
    text: &str,
    task: &Task,
    fallback_cost: f64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "cost".to_string(),
        serde_json::json!(
            extract_metric_value(text, &["单位成本", "单品成本", "成本", "cost"])
                .unwrap_or(fallback_cost)
                .max(0.01)
        ),
    );
    input.insert(
        "competitor_prices".to_string(),
        serde_json::json!(infer_pricing_strategy_competitor_prices(
            text,
            fallback_cost
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input
}

fn infer_pricing_strategy_competitor_prices(text: &str, fallback_cost: f64) -> Vec<f64> {
    let prices = infer_competitor_prices(text);
    if prices.is_empty() {
        numbers_in_text(text)
            .into_iter()
            .filter(|value| (*value - fallback_cost).abs() > 0.01)
            .collect()
    } else {
        prices
    }
}

fn infer_promo_planning_input(
    text: &str,
    task: &Task,
    budget: f64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "product".to_string(),
        serde_json::json!(selected_product_name_or_task(task)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert("budget".to_string(), serde_json::json!(budget));
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input
}

fn infer_execution_plan_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "goal".to_string(),
        serde_json::json!(compact_task_text(&task.description)),
    );
    input.insert("stage".to_string(), serde_json::json!(infer_stage(text)));
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input
}

fn infer_channel_strategy_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let numbers = numbers_in_text(text);
    let mut input = serde_json::Map::new();
    input.insert(
        "product_type".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "budget".to_string(),
        serde_json::json!(
            extract_metric_value(text, &["渠道预算", "月预算", "预算", "budget"])
                .or_else(|| {
                    numbers
                        .into_iter()
                        .filter(|value| !matches!(*value as i64, 618 | 11))
                        .find(|value| *value >= 100.0)
                })
                .unwrap_or(10_000.0)
                .max(0.0)
        ),
    );
    input.insert(
        "target_audience".to_string(),
        serde_json::json!(infer_audience(text)),
    );
    input
}

fn infer_assortment_planning_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let numbers = numbers_in_text(text);
    let mut input = serde_json::Map::new();
    input.insert(
        "category".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "budget".to_string(),
        serde_json::json!(
            extract_metric_value(
                text,
                &["选品预算", "货盘预算", "采购预算", "预算", "budget"]
            )
            .or_else(|| {
                numbers
                    .into_iter()
                    .filter(|value| !matches!(*value as i64, 618 | 11))
                    .find(|value| *value >= 100.0)
            })
            .unwrap_or(50_000.0)
            .max(0.0)
        ),
    );
    input
}

fn infer_inventory_planning_input(
    text: &str,
    task: &Task,
    avg_daily_demand: f64,
    lead_time_days: i64,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "avg_daily_demand".to_string(),
        serde_json::json!(
            extract_metric_value(
                text,
                &["日均销量", "日均需求", "日销量", "avg_daily_demand"]
            )
            .unwrap_or(avg_daily_demand)
            .max(0.0)
        ),
    );
    input.insert(
        "lead_time_days".to_string(),
        serde_json::json!(
            extract_metric_value(
                text,
                &["供货周期", "补货周期", "交货周期", "lead_time_days"]
            )
            .map(|value| value.round().max(1.0) as i64)
            .unwrap_or(lead_time_days.max(1))
        ),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input
}

fn infer_douyin_ads_plan_structure_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if first_selected_product(task).is_some()
        || contains_any(text, &["商品", "产品", "咖啡", "SKU", "sku"])
    {
        input.insert(
            "product_name".to_string(),
            serde_json::json!(selected_product_name_or_task(task)),
        );
    }
    input.insert(
        "goal".to_string(),
        serde_json::json!(infer_douyin_ads_goal(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("stage".to_string(), serde_json::json!(infer_stage(text)));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_explicit_period_days(text).unwrap_or(7)),
    );
    insert_metric_value(
        &mut input,
        "budget",
        text,
        &[
            "投放预算",
            "广告预算",
            "千川预算",
            "总预算",
            "预算",
            "budget",
        ],
    );
    insert_metric_value(
        &mut input,
        "target_roas",
        text,
        &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
    );
    input
}

fn infer_douyin_ads_audience_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    if first_selected_product(task).is_some()
        || contains_any(text, &["商品", "产品", "咖啡", "SKU", "sku"])
    {
        input.insert(
            "product_name".to_string(),
            serde_json::json!(selected_product_name_or_task(task)),
        );
    }
    input.insert(
        "target_audience".to_string(),
        serde_json::json!(infer_audience(text)),
    );
    input.insert(
        "category".to_string(),
        serde_json::json!(selected_product_category_or_infer(task, text)),
    );
    input.insert("stage".to_string(), serde_json::json!(infer_stage(text)));
    insert_metric_value(
        &mut input,
        "budget",
        text,
        &[
            "人群预算",
            "投放预算",
            "广告预算",
            "千川预算",
            "预算",
            "budget",
        ],
    );
    input
}

fn infer_douyin_ads_budget_pacing_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_explicit_period_days(text).unwrap_or(7)),
    );
    insert_metric_value(
        &mut input,
        "budget",
        text,
        &[
            "投放预算",
            "广告预算",
            "千川预算",
            "总预算",
            "预算",
            "budget",
        ],
    );
    insert_metric_value(
        &mut input,
        "spend",
        text,
        &[
            "已消耗",
            "消耗",
            "花费",
            "广告费",
            "广告花费",
            "spend",
            "ad_spend",
        ],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "成交额", "销售额", "营收"],
    );
    if let Some(current_roas) = infer_douyin_ads_current_roas(text) {
        input.insert("current_roas".to_string(), serde_json::json!(current_roas));
    }
    insert_metric_value(
        &mut input,
        "target_roas",
        text,
        &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
    );
    input
}

fn infer_douyin_ads_live_funnel_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    insert_metric_value(
        &mut input,
        "impressions",
        text,
        &["曝光量", "曝光", "展现", "展示", "impressions", "exposure"],
    );
    insert_metric_value(
        &mut input,
        "clicks",
        text,
        &["点击数", "点击", "广告点击", "clicks", "click"],
    );
    insert_metric_value(
        &mut input,
        "live_room_entries",
        text,
        &["进房数", "进房", "进入直播间", "场观", "live_room_entries"],
    );
    insert_metric_value(
        &mut input,
        "product_clicks",
        text,
        &["商品点击数", "商品点击", "点击商品", "product_clicks"],
    );
    insert_metric_value(
        &mut input,
        "orders",
        text,
        &["下单数", "下单", "订单数", "订单", "orders"],
    );
    insert_metric_value(
        &mut input,
        "payments",
        text,
        &["付款数", "付款", "支付数", "支付", "payments"],
    );
    insert_metric_value(
        &mut input,
        "ad_spend",
        text,
        &[
            "广告消耗",
            "广告花费",
            "广告费",
            "消耗",
            "spend",
            "ad_spend",
        ],
    );
    insert_metric_value(
        &mut input,
        "gmv",
        text,
        &["GMV", "gmv", "成交额", "销售额", "营收"],
    );
    input
}

fn infer_douyin_ads_goal(text: &str, task: &Task) -> String {
    if contains_any(text, &["直播", "直播间", "进房", "场观"]) {
        "直播间成交与进房转化".to_string()
    } else if contains_any(text, &["短视频", "视频", "素材"]) {
        "短视频引流与商品转化".to_string()
    } else if contains_any(text, &["新品", "冷启动", "起量"]) {
        "新品冷启动起量".to_string()
    } else {
        compact_task_text(&task.description)
    }
}

fn infer_douyin_ads_current_roas(text: &str) -> Option<f64> {
    extract_metric_value(
        text,
        &[
            "当前ROAS",
            "当前roas",
            "实际ROAS",
            "实际roas",
            "当前投产",
            "实际投产",
            "当前ROI",
            "当前roi",
            "实际ROI",
            "实际roi",
        ],
    )
    .or_else(|| {
        if contains_any(
            text,
            &["目标ROAS", "目标roas", "目标投产", "目标ROI", "target_roas"],
        ) {
            None
        } else {
            extract_metric_value(text, &["ROAS", "roas", "投产", "ROI", "roi"])
        }
    })
}

fn infer_promo_budget(text: &str) -> Option<f64> {
    extract_metric_value(text, &["活动预算", "总预算", "预算", "budget"])
        .filter(|value| *value >= 100.0)
}

fn infer_competitor_prices(text: &str) -> Vec<f64> {
    let mut prices = text
        .split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(|segment| {
            let trimmed = segment.trim();
            if !contains_any(
                trimmed,
                &["竞品", "对手", "竞对", "同行", "competitor", "competing"],
            ) {
                return None;
            }
            extract_last_metric_value(
                trimmed,
                &[
                    "竞品价格",
                    "竞品价",
                    "对手价格",
                    "竞对价格",
                    "同行价格",
                    "价格",
                    "售价",
                    "price",
                ],
            )
        })
        .collect::<Vec<_>>();
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    prices.dedup_by(|a, b| (*a - *b).abs() < 0.01);
    prices
}

fn infer_inventory_optimizer_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let skus = infer_inventory_skus(text);
    if !skus.is_empty() {
        input.insert("skus".to_string(), serde_json::json!(skus));
    }
    insert_metric_value(
        &mut input,
        "ordering_cost",
        text,
        &["订货成本", "下单成本", "ordering_cost"],
    );
    insert_metric_value(
        &mut input,
        "holding_rate",
        text,
        &["持有成本率", "库存持有成本率", "holding_rate"],
    );
    insert_metric_value(
        &mut input,
        "lead_time_days",
        text,
        &["交货周期", "供货周期", "补货周期", "lead_time_days"],
    );
    if let Some(service_level) = infer_service_level(text) {
        input.insert(
            "service_level".to_string(),
            serde_json::json!(service_level),
        );
    }
    input
}

fn infer_inventory_skus(text: &str) -> Vec<serde_json::Value> {
    text.split(['；', ';', '\n'])
        .filter_map(parse_inventory_sku_segment)
        .collect()
}

fn infer_ad_fatigue_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    let channels = infer_ad_fatigue_channels(text);
    if !channels.is_empty() {
        input.insert("channels".to_string(), serde_json::json!(channels));
    }
    insert_metric_value(
        &mut input,
        "total_budget",
        text,
        &["总预算", "广告总预算", "total_budget"],
    );
    input
}

fn infer_ad_fatigue_channels(text: &str) -> Vec<serde_json::Value> {
    const PLATFORM_LABELS: &[&str] = &[
        "抖音",
        "douyin",
        "tiktok",
        "淘宝",
        "taobao",
        "天猫",
        "tmall",
        "京东",
        "jd",
        "拼多多",
        "pdd",
        "小红书",
        "xiaohongshu",
        "快手",
        "kuaishou",
        "直通车",
        "千川",
    ];

    let mut rows = Vec::new();
    let mut seen = Vec::<String>::new();
    for label in PLATFORM_LABELS {
        let Some(segment) = segment_after_label(text, label, PLATFORM_LABELS) else {
            continue;
        };
        let name = canonical_ad_channel_name(label);
        if seen.iter().any(|value| value == name) {
            continue;
        }
        let ctr_baseline = extract_metric_value(
            &segment,
            &[
                "CTR基准",
                "基准CTR",
                "历史CTR",
                "baseline_ctr",
                "ctr_baseline",
            ],
        );
        let ctr_recent = extract_metric_value(
            &segment,
            &["近期CTR", "当前CTR", "最近CTR", "recent_ctr", "ctr_recent"],
        );
        let impressions_per_user = extract_metric_value(
            &segment,
            &[
                "曝光频次",
                "频次",
                "人均曝光",
                "impressions_per_user",
                "frequency",
            ],
        );
        let budget = extract_metric_value(
            &segment,
            &["预算", "广告预算", "投放预算", "budget", "ad_budget"],
        );
        let roas = extract_metric_value(&segment, &["ROAS", "roas", "投产", "roi", "ROI"]);

        if [ctr_baseline, ctr_recent, impressions_per_user, budget, roas]
            .iter()
            .all(Option::is_none)
        {
            continue;
        }

        let mut row = serde_json::Map::new();
        row.insert("name".to_string(), serde_json::json!(name));
        if let Some(value) = ctr_baseline {
            row.insert("ctr_baseline".to_string(), serde_json::json!(value));
        }
        if let Some(value) = ctr_recent {
            row.insert("ctr_recent".to_string(), serde_json::json!(value));
        }
        if let Some(value) = impressions_per_user {
            row.insert("impressions_per_user".to_string(), serde_json::json!(value));
        }
        if let Some(value) = budget {
            row.insert("budget".to_string(), serde_json::json!(value));
        }
        if let Some(value) = roas {
            row.insert("roas".to_string(), serde_json::json!(value));
        }
        seen.push(name.to_string());
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

fn canonical_ad_channel_name(label: &str) -> &'static str {
    let lower = label.to_ascii_lowercase();
    if matches!(label, "直通车") {
        "直通车"
    } else if matches!(label, "千川") {
        "千川"
    } else {
        canonical_channel_name(lower.as_str())
    }
}

fn infer_abc_xyz_classifier_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    let skus = infer_abc_xyz_skus(text);
    if !skus.is_empty() {
        input.insert("skus".to_string(), serde_json::json!(skus));
    }
    if let Some(top_n) = extract_metric_value(text, &["top_n", "前N", "前n", "展示前"]) {
        input.insert(
            "top_n".to_string(),
            serde_json::json!(top_n.round().clamp(1.0, 100.0) as i64),
        );
    }
    input
}

fn infer_abc_xyz_skus(text: &str) -> Vec<serde_json::Value> {
    text.split(['；', ';', '\n'])
        .filter_map(parse_abc_xyz_sku_segment)
        .collect()
}

fn parse_abc_xyz_sku_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if !contains_any(trimmed, &["sku", "SKU", "款", "货品", "商品"]) {
        return None;
    }
    let gmv = extract_last_metric_value(trimmed, &["GMV", "gmv", "销售额", "成交额", "营收"])?;
    let history = infer_abc_xyz_sales_history(trimmed);
    let mut row = serde_json::Map::new();
    row.insert(
        "sku_id".to_string(),
        serde_json::json!(infer_abc_xyz_sku_id(trimmed)),
    );
    row.insert("gmv".to_string(), serde_json::json!(gmv));
    if !history.is_empty() {
        row.insert("sales_history".to_string(), serde_json::json!(history));
    }
    Some(serde_json::Value::Object(row))
}

fn infer_abc_xyz_sales_history(segment: &str) -> Vec<f64> {
    let Some(history_segment) =
        segment_after_abc_history_label(segment, &["历史销量", "销量历史", "sales_history"])
    else {
        return Vec::new();
    };
    numbers_in_text(&history_segment)
}

fn segment_after_abc_history_label(text: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let mut best: Option<(usize, &str)> = None;
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        if let Some(index) = lower.find(&label_lower)
            && best
                .map(|(best_index, _)| index < best_index)
                .unwrap_or(true)
        {
            best = Some((index, *label));
        }
    }
    let (index, label) = best?;
    let start = index + label.len();
    let tail = text[start..].trim();
    let end = tail.find(['；', ';', '\n']).unwrap_or(tail.len());
    Some(tail[..end].trim().to_string())
}

fn infer_abc_xyz_sku_id(segment: &str) -> String {
    let trimmed = segment
        .trim()
        .trim_start_matches(['-', '•', '*', ' '])
        .trim();
    let lower = trimmed.to_ascii_lowercase();
    let trimmed = lower
        .rfind("sku")
        .map(|index| trimmed[index..].trim())
        .unwrap_or(trimmed);
    let metric_markers = [
        "GMV",
        "gmv",
        "销售额",
        "成交额",
        "营收",
        "历史销量",
        "销量历史",
        "sales_history",
    ];
    let lower = trimmed.to_ascii_lowercase();
    let end = metric_markers
        .iter()
        .filter_map(|marker| lower.find(&marker.to_ascii_lowercase()))
        .min()
        .unwrap_or(trimmed.len());
    let raw = trimmed[..end]
        .trim()
        .trim_matches(['：', ':', ',', '，', ';', '；', ' '])
        .trim();
    if raw.is_empty() {
        "SKU".to_string()
    } else {
        raw.to_string()
    }
}

fn parse_inventory_sku_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if !contains_any(trimmed, &["sku", "SKU", "款", "货品", "商品"]) {
        return None;
    }
    let monthly_sales = extract_last_metric_value(
        trimmed,
        &["月销量", "月销", "monthly_sales", "monthly sales", "销量"],
    )?;
    let cost = extract_last_metric_value(
        trimmed,
        &["单位成本", "单品成本", "成本", "cost", "unit_cost"],
    )?;
    let stock_qty = extract_last_metric_value(
        trimmed,
        &[
            "当前库存",
            "库存数量",
            "库存",
            "stock_qty",
            "stock",
            "inventory",
        ],
    )?;
    Some(serde_json::json!({
        "name": infer_inventory_sku_name(trimmed),
        "monthly_sales": monthly_sales,
        "cost": cost,
        "stock_qty": stock_qty,
    }))
}

fn infer_inventory_sku_name(segment: &str) -> String {
    let trimmed = segment
        .trim()
        .trim_start_matches(['-', '•', '*', ' '])
        .trim();
    let lower = trimmed.to_ascii_lowercase();
    let trimmed = lower
        .rfind("sku")
        .map(|index| trimmed[index..].trim())
        .unwrap_or(trimmed);
    let metric_markers = [
        "月销量",
        "月销",
        "monthly_sales",
        "monthly sales",
        "销量",
        "单位成本",
        "单品成本",
        "成本",
        "cost",
        "unit_cost",
        "当前库存",
        "库存数量",
        "库存",
        "stock_qty",
        "stock",
        "inventory",
    ];
    let lower = trimmed.to_ascii_lowercase();
    let end = metric_markers
        .iter()
        .filter_map(|marker| {
            let marker_lower = marker.to_ascii_lowercase();
            lower.find(&marker_lower)
        })
        .min()
        .unwrap_or(trimmed.len());
    let raw = trimmed[..end]
        .trim()
        .trim_matches(['：', ':', ',', '，', ';', '；', ' '])
        .trim();
    let raw = raw
        .rfind(['：', ':'])
        .map(|index| raw[index + 1..].trim())
        .unwrap_or(raw);
    let lower_raw = raw.to_ascii_lowercase();
    let raw = lower_raw
        .rfind("sku")
        .map(|index| raw[index..].trim())
        .unwrap_or(raw);
    if raw.is_empty() {
        "SKU".to_string()
    } else {
        raw.to_string()
    }
}

fn infer_service_level(text: &str) -> Option<String> {
    for level in ["99%", "98%", "95%", "90%"] {
        if text.contains(level) {
            return Some(level.to_string());
        }
    }
    let lower = text.to_ascii_lowercase();
    for (needle, level) in [
        ("service_level99", "99%"),
        ("service_level98", "98%"),
        ("service_level95", "95%"),
        ("service_level90", "90%"),
        ("service level 99", "99%"),
        ("service level 98", "98%"),
        ("service level 95", "95%"),
        ("service level 90", "90%"),
    ] {
        if lower.contains(needle) {
            return Some(level.to_string());
        }
    }
    None
}

fn infer_demand_forecast_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert("metric".to_string(), serde_json::json!(infer_metric(text)));
    if let Some(product_ref) = selected_product_ref(task) {
        input.insert("product_ref".to_string(), serde_json::json!(product_ref));
    }
    let values = numbers_in_text(text);
    if !values.is_empty() {
        input.insert("values".to_string(), serde_json::json!(values));
    }
    input
}

fn infer_price_quantity_points(text: &str) -> Vec<serde_json::Value> {
    text.split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(parse_price_quantity_segment)
        .collect()
}

fn parse_price_quantity_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return None;
    }
    let price = extract_last_metric_value(trimmed, &["价格点", "售价", "单价", "price", "价格"])?;
    let quantity = extract_metric_value(
        trimmed,
        &["销量", "销售量", "件数", "销量件数", "quantity", "qty"],
    )?;
    Some(serde_json::json!({
        "price": price,
        "quantity": quantity,
    }))
}

fn extract_last_metric_value(text: &str, aliases: &[&str]) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    for alias in aliases {
        let alias_lower = alias.to_ascii_lowercase();
        let index = lower
            .rmatch_indices(&alias_lower)
            .next()
            .map(|(index, _)| index);
        let Some(index) = index else {
            continue;
        };
        let start = index + alias_lower.len();
        if let Some(value) = parse_metric_number_after_alias(&text[start..]) {
            return Some(value);
        }
    }
    None
}

fn store_metrics_input_has_metrics(input: &serde_json::Map<String, serde_json::Value>) -> bool {
    [
        "gmv",
        "previous_gmv",
        "orders",
        "previous_orders",
        "visitors",
        "previous_visitors",
        "clicks",
        "ad_spend",
        "refund_amount",
        "gross_profit",
    ]
    .iter()
    .any(|key| input.contains_key(*key))
}

fn metric_query_input_has_metrics(input: &serde_json::Map<String, serde_json::Value>) -> bool {
    [
        "gmv",
        "previous_gmv",
        "orders",
        "visitors",
        "clicks",
        "ad_spend",
        "refund_amount",
        "gross_profit",
        "inventory",
        "stockout_count",
        "budget",
        "spend",
        "impressions",
        "coupon_cost",
    ]
    .iter()
    .any(|key| input.contains_key(*key))
}

fn infer_channel_roi_rows(text: &str) -> Vec<serde_json::Value> {
    const PLATFORM_LABELS: &[&str] = &[
        "抖音",
        "douyin",
        "tiktok",
        "淘宝",
        "taobao",
        "天猫",
        "tmall",
        "京东",
        "jd",
        "拼多多",
        "pdd",
        "小红书",
        "xiaohongshu",
        "快手",
        "kuaishou",
    ];

    let mut rows = Vec::new();
    let mut seen = Vec::<String>::new();
    for label in PLATFORM_LABELS {
        let Some(segment) = segment_after_label(text, label, PLATFORM_LABELS) else {
            continue;
        };
        let name = canonical_channel_name(label);
        if seen.iter().any(|value| value == name) {
            continue;
        }

        let gmv = extract_metric_value(&segment, &["GMV", "gmv", "销售额", "成交额", "营收"]);
        let ad_spend = extract_metric_value(
            &segment,
            &[
                "广告花费",
                "广告费",
                "投放花费",
                "投放消耗",
                "消耗",
                "花费",
                "ad_spend",
                "ad cost",
            ],
        );
        let orders = extract_metric_value(&segment, &["订单数", "订单", "支付订单", "orders"]);
        let uv = extract_metric_value(&segment, &["访客数", "访客", "UV", "uv"]);
        let conversion_rate = extract_metric_value(
            &segment,
            &["转化率", "成交转化率", "conversion_rate", "cvr"],
        );

        if [gmv, ad_spend, orders, uv, conversion_rate]
            .iter()
            .all(Option::is_none)
        {
            continue;
        }

        let mut row = serde_json::Map::new();
        row.insert("name".to_string(), serde_json::json!(name));
        if let Some(value) = gmv {
            row.insert("gmv".to_string(), serde_json::json!(value));
        }
        if let Some(value) = ad_spend {
            row.insert("ad_spend".to_string(), serde_json::json!(value));
        }
        if let Some(value) = orders {
            row.insert(
                "orders".to_string(),
                serde_json::json!(value.round() as i64),
            );
        }
        if let Some(value) = uv {
            row.insert("uv".to_string(), serde_json::json!(value.round() as i64));
        }
        if let Some(value) = conversion_rate {
            row.insert("conversion_rate".to_string(), serde_json::json!(value));
        }
        seen.push(name.to_string());
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

fn segment_after_label(text: &str, label: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();
    let index = lower.find(&label_lower)?;
    let start = index + label.len();
    let mut end = text.len();

    for other in labels {
        if *other == label {
            continue;
        }
        let other_lower = other.to_ascii_lowercase();
        if let Some(offset) = lower[start..].find(&other_lower) {
            let candidate = start + offset;
            if candidate > start && candidate < end {
                end = candidate;
            }
        }
    }

    Some(text[start..end].trim().to_string())
}

fn canonical_channel_name(label: &str) -> &'static str {
    let lower = label.to_ascii_lowercase();
    if matches!(label, "抖音") || matches!(lower.as_str(), "douyin" | "tiktok") {
        "抖音"
    } else if matches!(label, "天猫") || lower == "tmall" {
        "天猫"
    } else if matches!(label, "淘宝") || lower == "taobao" {
        "淘宝"
    } else if matches!(label, "京东") || lower == "jd" {
        "京东"
    } else if matches!(label, "拼多多") || lower == "pdd" {
        "拼多多"
    } else if matches!(label, "小红书") || lower == "xiaohongshu" {
        "小红书"
    } else if matches!(label, "快手") || lower == "kuaishou" {
        "快手"
    } else {
        "未知渠道"
    }
}

fn infer_attribution_model(text: &str) -> &'static str {
    if contains_any(text, &["last_touch", "末次", "最后触点", "末次归因"]) {
        "last_touch"
    } else if contains_any(text, &["linear", "线性", "线性归因"]) {
        "linear"
    } else if contains_any(text, &["time_decay", "时间衰减", "衰减归因"]) {
        "time_decay"
    } else if contains_any(text, &["position", "位置归因", "u形", "U形", "首末"]) {
        "position"
    } else {
        "all"
    }
}

fn infer_refund_items(text: &str) -> Vec<serde_json::Value> {
    text.split(['，', ',', '；', ';', '、', '\n'])
        .filter_map(parse_refund_item_segment)
        .collect()
}

fn parse_refund_item_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.rsplit(['：', ':']).next().unwrap_or(segment).trim();
    if trimmed.is_empty() {
        return None;
    }
    let amount =
        extract_metric_value(trimmed, &["退款金额", "退款额", "退款", "退货金额", "退货"])?;
    let name = refund_item_name(trimmed).unwrap_or("综合退款");
    Some(serde_json::json!({
        "name": name,
        "refund_amount": amount,
    }))
}

fn refund_item_name(segment: &str) -> Option<&str> {
    for alias in ["退款金额", "退款额", "退款", "退货金额", "退货"] {
        if let Some(index) = segment.find(alias) {
            let name = segment[..index]
                .trim()
                .trim_matches(['：', ':', '-', '，', ',', '；', ';', ' ']);
            return if name.is_empty() { None } else { Some(name) };
        }
    }
    None
}

fn has_store_metrics_query_intent(text: &str) -> bool {
    if contains_any(
        text,
        &[
            "query_store_metrics",
            "store metrics",
            "store_metrics",
            "查询店铺指标",
            "加载店铺指标",
            "读取店铺指标",
            "查询店铺数据",
            "加载店铺数据",
            "读取店铺数据",
            "经营数据基线",
            "数据基线",
        ],
    ) {
        return true;
    }

    contains_any(text, &["店铺指标", "店铺数据", "经营指标", "经营数据"])
        && contains_any(
            text,
            &[
                "查询",
                "加载",
                "读取",
                "拉取",
                "拿一下",
                "看一下",
                "先",
                "作为",
                "基线",
                "load",
                "query",
                "read",
            ],
        )
}

fn has_product_metrics_query_intent(text: &str) -> bool {
    if contains_any(
        text,
        &[
            "query_product_metrics",
            "product metrics",
            "product_metrics",
            "查询商品指标",
            "加载商品指标",
            "读取商品指标",
            "查询商品数据",
            "加载商品数据",
            "读取商品数据",
            "商品表现",
            "单品表现",
            "SKU表现",
            "sku表现",
            "商品数据基线",
        ],
    ) {
        return true;
    }

    contains_any(
        text,
        &[
            "商品指标",
            "商品数据",
            "单品指标",
            "单品数据",
            "SKU指标",
            "sku指标",
        ],
    ) && contains_any(
        text,
        &[
            "查询",
            "加载",
            "读取",
            "拉取",
            "看一下",
            "复盘",
            "分析",
            "基线",
            "load",
            "query",
            "read",
        ],
    )
}

fn has_campaign_metrics_query_intent(text: &str) -> bool {
    if contains_any(
        text,
        &[
            "query_campaign_metrics",
            "campaign metrics",
            "campaign_metrics",
            "查询活动指标",
            "加载活动指标",
            "读取活动指标",
            "查询活动数据",
            "加载活动数据",
            "读取活动数据",
            "活动表现",
            "活动数据基线",
            "投放活动数据",
        ],
    ) {
        return true;
    }

    contains_any(
        text,
        &[
            "活动指标",
            "活动数据",
            "活动复盘",
            "campaign指标",
            "campaign数据",
        ],
    ) && contains_any(
        text,
        &[
            "查询",
            "加载",
            "读取",
            "拉取",
            "看一下",
            "复盘",
            "分析",
            "基线",
            "load",
            "query",
            "read",
        ],
    )
}

fn has_external_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "search",
            "google",
            "搜",
            "搜索",
            "搜一下",
            "搜搜",
            "查一下",
            "查查",
            "联网",
            "实时",
            "最新",
            "公开来源",
            "公开资料",
            "外部资料",
            "网上",
            "资料来源",
            "来源",
            "不要缓存",
            "fresh",
            "live",
            "source",
        ],
    )
}

fn is_platform_policy_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "平台规则",
            "平台政策",
            "政策变化",
            "最新规则",
            "平台公告",
            "算法调整",
            "流量规则",
            "佣金",
            "费率",
            "违规",
            "违禁词",
            "大促规则",
            "报名规则",
            "policy",
        ],
    ) && (has_external_search_intent(text) || contains_any(text, &["最新", "规则", "政策"]))
}

fn is_product_review_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "用户评价",
            "真实评价",
            "商品评价",
            "买家评价",
            "口碑",
            "差评",
            "好评",
            "吐槽",
            "真实反馈",
            "review",
            "reviews",
        ],
    ) && has_external_search_intent(text)
}

fn is_industry_benchmark_search_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "行业基准",
            "行业均值",
            "行业平均",
            "基准数据",
            "benchmark",
            "benchmarks",
            "平均转化率",
            "平均客单价",
            "行业roi",
            "行业 ROI",
            "行业退款率",
        ],
    ) && (has_external_search_intent(text) || contains_any(text, &["行业", "基准", "均值"]))
}

fn infer_search_platform(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if text.contains("抖音") || lower.contains("douyin") || lower.contains("tiktok") {
        "抖音"
    } else if text.contains("淘宝") || lower.contains("taobao") {
        "淘宝"
    } else if text.contains("天猫") || lower.contains("tmall") {
        "天猫"
    } else if text.contains("京东") || lower.contains("jd") {
        "京东"
    } else if text.contains("拼多多") || lower.contains("pdd") {
        "拼多多"
    } else if text.contains("小红书") || lower.contains("xiaohongshu") || lower.contains("red") {
        "小红书"
    } else if text.contains("快手") {
        "快手"
    } else {
        "电商"
    }
}

fn infer_search_query(text: &str) -> String {
    let category = infer_category(text);
    if category != "电商消费品" {
        return category.to_string();
    }
    let keywords = extract_keyword_candidates(text);
    if let Some(keyword) = keywords.first() {
        return keyword.clone();
    }
    compact_task_text(text)
}

fn infer_search_query_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = first_selected_product(task)
    {
        if let Some(category) = product.category.filter(|value| !value.is_empty()) {
            return format!("{} {}", product.name, category);
        }
        return product.name;
    }
    infer_search_query(text)
}

fn infer_search_industry_for_task(text: &str, task: &Task) -> String {
    if refers_to_selected_product(text)
        && let Some(product) = first_selected_product(task)
        && let Some(category) = product.category.filter(|value| !value.is_empty())
    {
        return category;
    }
    infer_search_industry(text)
}

fn infer_search_industry(text: &str) -> String {
    infer_category(text).to_string()
}

fn infer_search_platform_for_task(text: &str, task: &Task) -> String {
    let explicit_platform = infer_search_platform(text);
    if explicit_platform != "电商" {
        return explicit_platform.to_string();
    }
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| explicit_platform.to_string())
}

fn refers_to_selected_product(text: &str) -> bool {
    contains_any(
        text,
        &[
            "这个商品",
            "这个产品",
            "该商品",
            "该产品",
            "本品",
            "当前商品",
            "所选商品",
            "选中商品",
            "这个SKU",
            "该SKU",
            "this product",
            "selected product",
        ],
    )
}

fn insert_selected_product_search_context(
    input: &mut serde_json::Map<String, serde_json::Value>,
    task: &Task,
) {
    if let Some(product) = first_selected_product(task) {
        input.insert(
            "product_ref".to_string(),
            serde_json::json!(product.ref_key()),
        );
        input.insert("product_name".to_string(), serde_json::json!(product.name));
        if let Some(category) = product.category.filter(|value| !value.is_empty()) {
            input.insert("product_category".to_string(), serde_json::json!(category));
        }
    }
}

fn insert_search_max_results(input: &mut serde_json::Map<String, serde_json::Value>, text: &str) {
    if let Some(max_results) = infer_search_max_results(text) {
        input.insert(
            "max_results".to_string(),
            serde_json::json!(max_results.clamp(1, 12)),
        );
    }
}

fn search_product_reviews_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_search_review_focus(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_industry_benchmarks_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "industry".to_string(),
        serde_json::json!(infer_search_industry_for_task(text, task)),
    );
    input.insert(
        "metric".to_string(),
        serde_json::json!(infer_search_benchmark_metric(text)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_competitor_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "focus".to_string(),
        serde_json::json!(infer_search_competitor_focus(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_trends_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_market_info_input(text: &str, task: &Task) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "query".to_string(),
        serde_json::json!(infer_search_query_for_task(text, task)),
    );
    input.insert(
        "topic".to_string(),
        serde_json::json!(infer_search_topic(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn search_platform_policy_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = serde_json::Map::new();
    input.insert(
        "platform".to_string(),
        serde_json::json!(infer_search_platform_for_task(text, task)),
    );
    input.insert(
        "policy_type".to_string(),
        serde_json::json!(infer_search_policy_type(text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_period_days(text)),
    );
    input.insert(
        "_needs_fresh_data".to_string(),
        serde_json::json!(infer_search_bypass_cache(text)),
    );
    insert_search_max_results(&mut input, text);
    insert_selected_product_search_context(&mut input, task);
    input
}

fn infer_search_bypass_cache(text: &str) -> bool {
    contains_any(
        text,
        &[
            "实时",
            "最新",
            "刚刚",
            "今天",
            "联网",
            "公开来源",
            "公开资料",
            "外部资料",
            "真实",
            "真实评价",
            "真实口碑",
            "真实反馈",
            "不要缓存",
            "fresh",
            "live",
            "_needs_fresh_data",
            "bypass_cache",
        ],
    )
}

fn infer_search_max_results(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[..start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start..end].parse::<i64>() else {
            continue;
        };
        if value <= 0 {
            continue;
        }
        let before = text[..start]
            .chars()
            .rev()
            .take(16)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let after = text[end..].chars().take(16).collect::<String>();

        if is_search_result_count_context(&before, &after) {
            return Some(value.clamp(1, 12));
        }
    }
    None
}

fn is_search_result_count_context(before: &str, after: &str) -> bool {
    let trimmed_after = after.trim_start();
    let has_after_result_unit = [
        "条来源",
        "个来源",
        "条结果",
        "个结果",
        "条公开资料",
        "个公开资料",
        "条证据",
        "个证据",
        "篇资料",
        "篇来源",
        "sources",
        "source",
        "results",
        "result",
        "links",
        "items",
    ]
    .iter()
    .any(|unit| trimmed_after.starts_with(unit));
    let has_before_limit_word = contains_any(
        before,
        &[
            "最多", "至多", "只要", "只看", "返回", "给我", "列出", "展示", "查", "搜", "top",
            "limit", "max", "maximum",
        ],
    );
    let has_after_short_unit = trimmed_after.starts_with('条')
        || trimmed_after.starts_with('个')
        || trimmed_after.starts_with("篇");

    has_after_result_unit || (has_before_limit_word && has_after_short_unit)
}

fn infer_search_policy_type(text: &str) -> &'static str {
    if contains_any(text, &["算法", "搜索", "流量", "排名", "权重", "algorithm"]) {
        "algorithm"
    } else if contains_any(text, &["佣金", "费率", "扣点", "服务费", "fee"]) {
        "fee"
    } else if contains_any(text, &["大促", "活动", "618", "双11", "报名", "promotion"]) {
        "promotion"
    } else if contains_any(
        text,
        &[
            "素材",
            "尺寸",
            "规格",
            "广告位",
            "格式",
            "安全区",
            "文件大小",
            "asset",
            "material",
            "spec",
        ],
    ) {
        "material"
    } else if contains_any(text, &["内容", "违禁词", "审核", "违规", "发布", "content"])
    {
        "content"
    } else {
        "general"
    }
}

fn infer_search_review_focus(text: &str) -> &'static str {
    if contains_any(text, &["差评", "负面", "吐槽", "投诉", "踩雷", "negative"]) {
        "negative"
    } else if contains_any(text, &["好评", "正面", "优点", "满意", "positive"]) {
        "positive"
    } else {
        "overall"
    }
}

fn infer_search_competitor_focus(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "价格",
            "定价",
            "售价",
            "price",
            "价格弹性",
            "价格敏感",
            "调价",
            "市场定价",
            "价格区间",
        ],
    ) {
        "price"
    } else if contains_any(text, &["促销", "活动", "优惠", "promotion"]) {
        "promotion"
    } else if contains_any(text, &["评价", "口碑", "差评", "reviews"]) {
        "reviews"
    } else if contains_any(text, &["卖点", "功能", "特点", "features"]) {
        "features"
    } else {
        "general"
    }
}

fn infer_search_benchmark_metric(text: &str) -> &'static str {
    if contains_any(text, &["转化", "转化率", "conversion", "cvr"]) {
        "conversion"
    } else if contains_any(text, &["客单价", "aov"]) {
        "aov"
    } else if contains_any(
        text,
        &["dsr", "DSR", "服务评分", "店铺评分", "描述相符", "物流服务"],
    ) {
        "dsr"
    } else if contains_any(text, &["nps", "NPS", "满意度"]) {
        "nps"
    } else if contains_any(text, &["退款", "退货", "refund"]) {
        "refund"
    } else if contains_any(text, &["roi", "ROI", "roas", "ROAS", "投产"]) {
        "roi"
    } else {
        "general"
    }
}

fn infer_search_topic(text: &str) -> &'static str {
    if contains_any(
        text,
        &[
            "融资",
            "财报",
            "金融",
            "finance",
            "财务",
            "财务基准",
            "财务对标",
            "利润率",
            "毛利率",
            "净利率",
            "现金流",
            "广告ROI",
            "广告 ROI",
        ],
    ) {
        "finance"
    } else if contains_any(text, &["新闻", "资讯", "公告", "news"]) {
        "news"
    } else {
        "general"
    }
}

fn infer_store_metrics_platform(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if text.contains("抖音") || lower.contains("douyin") || lower.contains("tiktok") {
        "抖音"
    } else if text.contains("淘宝") || lower.contains("taobao") {
        "淘宝"
    } else if text.contains("天猫") || lower.contains("tmall") {
        "天猫"
    } else if text.contains("京东") || lower.contains("jd") {
        "京东"
    } else if text.contains("拼多多") || lower.contains("pdd") {
        "拼多多"
    } else if text.contains("快手") {
        "快手"
    } else {
        "all"
    }
}

fn extract_metric_value(text: &str, aliases: &[&str]) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    for alias in aliases {
        let alias_lower = alias.to_ascii_lowercase();
        let Some(index) = lower.find(&alias_lower) else {
            continue;
        };
        let start = index + alias_lower.len();
        if let Some(value) = parse_metric_number_after_alias(&text[start..]) {
            return Some(value);
        }
    }
    None
}

fn parse_metric_number_after_alias(segment: &str) -> Option<f64> {
    let mut chars = segment.char_indices().peekable();
    let mut number_start = None;
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_ascii_digit() || ch == '.' {
            number_start = Some(index);
            break;
        }
        if matches!(
            ch,
            ':' | '：' | '=' | '＝' | ' ' | '\t' | '约' | '为' | '是' | '￥' | '¥'
        ) {
            chars.next();
            continue;
        }
        if index > 12 {
            return None;
        }
        chars.next();
    }

    let start = number_start?;
    let mut end = start;
    let mut dot_seen = false;
    for (index, ch) in segment[start..].char_indices() {
        if ch.is_ascii_digit() || ch == ',' {
            end = start + index + ch.len_utf8();
            continue;
        }
        if ch == '.' && !dot_seen {
            dot_seen = true;
            end = start + index + ch.len_utf8();
            continue;
        }
        break;
    }

    let raw_number = segment[start..end].replace(',', "");
    let mut value = raw_number.parse::<f64>().ok()?;
    if !value.is_finite() {
        return None;
    }

    let unit_tail = segment[end..].trim_start();
    if unit_tail.starts_with('亿') {
        value *= 100_000_000.0;
    } else if unit_tail.starts_with('万')
        || unit_tail.starts_with('w')
        || unit_tail.starts_with('W')
    {
        value *= 10_000.0;
    } else if unit_tail.starts_with('千')
        || unit_tail.starts_with('k')
        || unit_tail.starts_with('K')
    {
        value *= 1_000.0;
    }
    Some(value.max(0.0))
}

fn push_parsed_number(values: &mut Vec<f64>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    if let Ok(value) = current.parse::<f64>()
        && value.is_finite()
    {
        values.push(value);
    }
    current.clear();
}

fn infer_platform(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if text.contains("抖音") || lower.contains("douyin") || lower.contains("tiktok") {
        "抖音"
    } else if text.contains("京东") || lower.contains("jd") {
        "京东"
    } else if text.contains("拼多多") || lower.contains("pdd") {
        "拼多多"
    } else if text.contains("快手") {
        "快手"
    } else {
        "淘宝"
    }
}

fn infer_cohort_unit(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if text.contains("月") || lower.contains("month") || lower.contains("monthly") {
        "month"
    } else {
        "week"
    }
}

fn infer_seasonal_period(text: &str) -> i64 {
    let lower = text.to_ascii_lowercase();
    if text.contains("月") || lower.contains("monthly") {
        30
    } else {
        7
    }
}

fn infer_stage(text: &str) -> &'static str {
    if contains_any(text, &["cold start", "new product", "冷启动", "新品"]) {
        "冷启动"
    } else if contains_any(text, &["mature", "成熟"]) {
        "成熟期"
    } else if contains_any(text, &["decline", "衰退"]) {
        "衰退期"
    } else {
        "成长期"
    }
}

fn infer_metric(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    if text.contains("转化") || lower.contains("conversion") || lower.contains("cvr") {
        "conversion_rate"
    } else if lower.contains("order") || text.contains("订单") {
        "orders"
    } else if lower.contains("uv") || text.contains("访客") {
        "uv"
    } else {
        "gmv"
    }
}

fn infer_return_reason(text: &str) -> &'static str {
    if text.contains("质量") {
        "质量问题"
    } else if text.contains("发错") {
        "发错货"
    } else if text.contains("破损") {
        "破损"
    } else if text.contains("尺码") || text.contains("大小") {
        "尺码不合"
    } else {
        "不喜欢"
    }
}

fn infer_service_issue_type(text: &str) -> &'static str {
    if contains_any(text, &["物流", "快递", "未到", "延迟", "没收到"]) {
        "物流延迟"
    } else if contains_any(text, &["质量", "破损", "坏了", "假货", "描述不符"]) {
        "质量问题"
    } else if contains_any(text, &["退款", "退货", "换货", "售后"]) {
        "退款退货"
    } else if contains_any(text, &["发错", "少发", "漏发"]) {
        "发错货"
    } else if contains_any(text, &["投诉", "差评", "曝光", "维权", "举报"]) {
        "投诉"
    } else {
        "售后咨询"
    }
}

fn infer_service_emotion(text: &str) -> &'static str {
    if contains_any(
        text,
        &["威胁投诉", "投诉平台", "曝光", "举报", "维权", "骗子"],
    ) {
        "威胁投诉"
    } else if contains_any(text, &["愤怒", "生气", "气死", "垃圾", "太差"]) {
        "愤怒"
    } else if contains_any(text, &["着急", "急", "催", "怎么还", "等了"]) {
        "焦虑"
    } else if contains_any(text, &["满意", "好评", "谢谢"]) {
        "满意"
    } else {
        "普通"
    }
}

fn infer_faq_questions(text: &str) -> Vec<String> {
    let mut questions = Vec::new();
    if contains_any(text, &["发货", "物流", "时效"]) {
        questions.push("什么时候发货？".to_owned());
    }
    if contains_any(text, &["退", "换", "退款", "售后"]) {
        questions.push("支持退货退款吗？怎么操作？".to_owned());
    }
    if contains_any(text, &["质量", "正品"]) {
        questions.push("质量有保障吗？".to_owned());
    }
    if contains_any(text, &["优惠", "价格", "活动"]) {
        questions.push("有优惠活动吗？".to_owned());
    }
    if contains_any(text, &["尺码", "规格"]) {
        questions.push("怎么选尺码/规格？".to_owned());
    }
    if questions.is_empty() {
        questions.extend(
            [
                "什么时候发货？",
                "支持退货退款吗？怎么操作？",
                "质量有保障吗？",
                "有优惠活动吗？",
                "怎么选尺码/规格？",
            ]
            .iter()
            .map(|value| (*value).to_owned()),
        );
    }
    questions
}

fn infer_page_type(text: &str) -> &'static str {
    if text.contains("详情") || text.to_ascii_lowercase().contains("detail") {
        "商品详情"
    } else if text.contains("购物车") || text.to_ascii_lowercase().contains("cart") {
        "购物车"
    } else if text.contains("结算") || text.to_ascii_lowercase().contains("checkout") {
        "结算"
    } else if text.contains("列表") || text.to_ascii_lowercase().contains("listing") {
        "列表页"
    } else {
        "首页"
    }
}

fn infer_design_usage(text: &str) -> &'static str {
    if text.contains("详情") {
        "详情页"
    } else if text.contains("信息流") || text.contains("广告") {
        "信息流广告"
    } else if text.contains("直播") {
        "直播封面"
    } else {
        "主图套组"
    }
}

fn infer_design_style(text: &str) -> &'static str {
    if contains_any(text, &["高端", "奢华", "质感"]) {
        "高端奢华"
    } else if contains_any(text, &["年轻", "活力", "可爱"]) {
        "年轻活力"
    } else if contains_any(text, &["国潮", "复古"]) {
        "国潮复古"
    } else if contains_any(text, &["科技", "数码"]) {
        "科技感"
    } else if contains_any(text, &["清新", "自然"]) {
        "清新自然"
    } else {
        "简约现代"
    }
}

fn extract_discount_hint(text: &str) -> &'static str {
    if contains_any(text, &["满减", "满"]) {
        "满减优惠"
    } else if contains_any(text, &["折", "折扣"]) {
        "限时折扣"
    } else if contains_any(text, &["赠", "赠品"]) {
        "下单赠礼"
    } else {
        "限时优惠"
    }
}

fn infer_video_type(text: &str) -> &'static str {
    if contains_any(text, &["review", "测评"]) {
        "测评"
    } else if contains_any(text, &["tutorial", "教程", "教学", "怎么用"]) {
        "教程"
    } else if contains_any(text, &["plot", "story", "剧情"]) {
        "剧情"
    } else if contains_any(text, &["unbox", "开箱"]) {
        "开箱"
    } else {
        "种草"
    }
}

fn infer_video_duration_seconds(text: &str) -> i64 {
    let numbers = numbers_in_text(text);
    if numbers.is_empty() {
        return 30;
    }
    if contains_any(
        text,
        &["秒", "sec", "second", "seconds", "时长", "duration"],
    ) {
        return numbers[0].round().clamp(10.0, 180.0) as i64;
    }
    if contains_any(text, &["分钟", "minute", "minutes"]) {
        return (numbers[0] * 60.0).round().clamp(10.0, 180.0) as i64;
    }
    30
}

fn infer_audience(text: &str) -> &'static str {
    if contains_any(text, &["student", "学生"]) {
        "学生用户"
    } else if contains_any(text, &["office", "commute", "通勤", "白领", "上班"]) {
        "通勤白领"
    } else if contains_any(text, &["mom", "妈妈", "宝妈", "家庭"]) {
        "家庭消费人群"
    } else if contains_any(text, &["young", "年轻", "女生", "女性"]) {
        "年轻女性"
    } else {
        "目标消费者"
    }
}

fn infer_category(text: &str) -> &'static str {
    if contains_any(text, &["coffee", "咖啡", "饮品"]) {
        "咖啡饮品"
    } else if contains_any(text, &["beauty", "美妆", "护肤"]) {
        "美妆护肤"
    } else if contains_any(text, &["food", "食品", "零食"]) {
        "食品零食"
    } else if contains_any(text, &["clothes", "服饰", "穿搭"]) {
        "服饰穿搭"
    } else if contains_any(text, &["home", "家居", "家电"]) {
        "家居生活"
    } else {
        "电商消费品"
    }
}

fn infer_product_name(text: &str) -> String {
    compact_task_text(text)
}

fn infer_campaign_name(text: &str) -> String {
    if contains_any(text, &["618"]) {
        "618大促".to_string()
    } else if contains_any(text, &["双11", "双十一"]) {
        "双11大促".to_string()
    } else if contains_any(text, &["五一"]) {
        "五一活动".to_string()
    } else {
        compact_task_text(text)
    }
}

fn infer_campaign_type(text: &str) -> &'static str {
    if contains_any(text, &["投放", "广告", "ad", "paid"]) {
        "ad"
    } else if contains_any(text, &["直播", "live"]) {
        "live"
    } else if contains_any(text, &["新品", "上新", "冷启动", "launch"]) {
        "launch"
    } else if contains_any(text, &["促销", "大促", "满减", "优惠", "promotion"]) {
        "promotion"
    } else {
        "general"
    }
}

fn extract_labeled_token(text: &str, labels: &[&str]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        let Some(index) = lower.find(&label_lower) else {
            continue;
        };
        let start = index + label_lower.len();
        if let Some(token) = parse_token_after_label(&text[start..]) {
            return Some(token);
        }
    }
    None
}

fn parse_token_after_label(segment: &str) -> Option<String> {
    let mut chars = segment.char_indices().peekable();
    let mut token_start = None;
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            token_start = Some(index);
            break;
        }
        if matches!(ch, ':' | '：' | '=' | '＝' | ' ' | '\t' | '#' | '号') {
            chars.next();
            continue;
        }
        if index > 12 {
            return None;
        }
        chars.next();
    }

    let start = token_start?;
    let mut end = start;
    for (index, ch) in segment[start..].char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
            end = start + index + ch.len_utf8();
            continue;
        }
        break;
    }
    let token = segment[start..end].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn infer_period_days(text: &str) -> i64 {
    let numbers = numbers_in_text(text);
    numbers
        .iter()
        .find(|value| **value >= 1.0 && **value <= 365.0)
        .map(|value| value.round() as i64)
        .unwrap_or(30)
}

fn infer_explicit_period_days(text: &str) -> Option<i64> {
    for (start, ch) in text.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if start > 0
            && text[..start]
                .chars()
                .last()
                .is_some_and(|prev| prev.is_ascii_digit() || prev == '.')
        {
            continue;
        }

        let mut end = start + ch.len_utf8();
        let suffix_start = end;
        for (offset, next) in text[suffix_start..].char_indices() {
            if next.is_ascii_digit() {
                end = suffix_start + offset + next.len_utf8();
                continue;
            }
            break;
        }

        let Ok(value) = text[start..end].parse::<i64>() else {
            continue;
        };
        if !(1..=365).contains(&value) {
            continue;
        }

        let unit_tail = text[end..].trim_start();
        let Some(multiplier) = explicit_period_multiplier(unit_tail) else {
            continue;
        };

        let before = text[..start]
            .chars()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();
        if contains_any(
            &before,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        ) || unit_tail.starts_with("天内")
            || unit_tail.starts_with("日内")
            || unit_tail.starts_with("周内")
            || unit_tail.starts_with("月内")
        {
            return Some((value * multiplier).clamp(1, 365));
        }
    }
    None
}

fn explicit_period_multiplier(unit_tail: &str) -> Option<i64> {
    let lower = unit_tail.to_ascii_lowercase();
    if unit_tail.starts_with('天') || unit_tail.starts_with('日') || lower.starts_with("day") {
        Some(1)
    } else if unit_tail.starts_with('周') || lower.starts_with("week") {
        Some(7)
    } else if unit_tail.starts_with('月') || lower.starts_with("month") {
        Some(30)
    } else if unit_tail.starts_with('年') || lower.starts_with("year") {
        Some(365)
    } else {
        None
    }
}

fn infer_integer_or_default(text: &str, default: i64) -> i64 {
    numbers_in_text(text)
        .first()
        .copied()
        .map(|value| value.round() as i64)
        .unwrap_or(default)
}

fn infer_percent_number(text: &str, default: f64) -> f64 {
    numbers_in_text(text).first().copied().unwrap_or(default)
}

fn infer_second_percent_number(text: &str, default: f64) -> f64 {
    numbers_in_text(text).get(1).copied().unwrap_or(default)
}

fn infer_feature_list(text: &str) -> Vec<String> {
    let mut features = Vec::new();
    for feature in [
        "高颜值",
        "高性价比",
        "便携",
        "低糖",
        "耐用",
        "送礼",
        "通勤",
        "防漏",
    ] {
        if text.contains(feature) {
            features.push(feature.to_string());
        }
    }
    if features.is_empty() {
        features.push("高性价比".to_string());
        features.push("核心卖点明确".to_string());
    }
    features
}

fn infer_system_type(text: &str) -> &'static str {
    if contains_any(text, &["erp", "ERP"]) {
        "ERP"
    } else if contains_any(text, &["crm", "CRM"]) {
        "CRM"
    } else if contains_any(text, &["数据平台", "报表平台", "bi", "BI"]) {
        "数据平台"
    } else if contains_any(text, &["小程序", "mini program"]) {
        "小程序"
    } else {
        "电商平台"
    }
}

fn infer_bug_type(text: &str) -> &'static str {
    if contains_any(text, &["timeout", "超时"]) {
        "接口超时"
    } else if contains_any(text, &["白屏"]) {
        "页面白屏"
    } else if contains_any(text, &["支付失败", "支付"]) {
        "支付失败"
    } else if contains_any(text, &["库存不一致", "库存"]) {
        "库存不一致"
    } else if contains_any(text, &["oom", "内存"]) {
        "OOM"
    } else if contains_any(text, &["死锁"]) {
        "死锁"
    } else {
        "数据异常"
    }
}

fn infer_bug_frequency(text: &str) -> &'static str {
    if contains_any(text, &["必现", "每次"]) {
        "必现"
    } else if contains_any(text, &["频繁", "经常"]) {
        "频繁"
    } else {
        "偶发"
    }
}

fn infer_perf_bottleneck(text: &str) -> &'static str {
    if contains_any(text, &["database", "db", "数据库", "sql"]) {
        "数据库"
    } else if contains_any(text, &["front", "前端", "页面"]) {
        "前端"
    } else if contains_any(text, &["cache", "缓存", "redis"]) {
        "缓存"
    } else if contains_any(text, &["network", "网络"]) {
        "网络"
    } else if contains_any(text, &["concurrency", "并发"]) {
        "并发"
    } else {
        "接口"
    }
}

fn infer_stack_hint(text: &str) -> String {
    let mut parts = Vec::new();
    for stack in [
        "Rust", "Python", "FastAPI", "React", "Postgres", "MySQL", "Redis", "Docker", "Nginx",
    ] {
        if text.contains(stack)
            || text
                .to_ascii_lowercase()
                .contains(&stack.to_ascii_lowercase())
        {
            parts.push(stack.to_string());
        }
    }
    if parts.is_empty() {
        "未明确技术栈".to_string()
    } else {
        parts.join("/")
    }
}

fn infer_learning_topic(text: &str) -> String {
    if contains_any(text, &["ROI", "roi", "ROAS", "roas", "投产"]) {
        return "ROI和ROAS的区别".to_string();
    }
    if contains_any(text, &["CTR", "ctr", "点击率"]) {
        return "点击率CTR".to_string();
    }
    if contains_any(text, &["CVR", "cvr", "转化率"]) {
        return "转化率CVR".to_string();
    }
    if contains_any(text, &["DSR", "dsr", "店铺评分", "服务评分"]) {
        return "DSR店铺评分".to_string();
    }
    if contains_any(text, &["NPS", "nps", "净推荐值"]) {
        return "NPS净推荐值".to_string();
    }
    if contains_any(text, &["复购", "复购率"]) {
        return "复购率".to_string();
    }
    if contains_any(text, &["客单价", "AOV", "aov"]) {
        return "客单价AOV".to_string();
    }

    compact_task_text(text)
        .trim_start_matches("请")
        .trim_start_matches("帮我")
        .trim_start_matches("给我")
        .trim()
        .to_string()
}

fn infer_learning_level_from_text(text: &str) -> &'static str {
    if contains_any(text, &["本科", "大学", "undergraduate"]) {
        "undergraduate"
    } else if contains_any(text, &["小白", "新手", "入门", "零基础", "beginner"]) {
        "beginner"
    } else if contains_any(text, &["专家", "深入", "高级", "专业", "expert"]) {
        "expert"
    } else {
        "practitioner"
    }
}

fn infer_learning_scenario(text: &str) -> &'static str {
    if contains_any(
        text,
        &["投放", "广告", "千川", "ROI", "roi", "ROAS", "roas"],
    ) {
        "电商投放复盘"
    } else if contains_any(text, &["客服", "售后", "退款", "DSR", "dsr", "NPS", "nps"]) {
        "客服售后分析"
    } else if contains_any(
        text,
        &["转化", "详情页", "主图", "标题", "CTR", "ctr", "点击率"],
    ) {
        "商品转化优化"
    } else {
        "电商经营学习"
    }
}

fn infer_tech_domain(text: &str) -> &'static str {
    if contains_any(text, &["queue", "mq", "消息队列"]) {
        "消息队列"
    } else if contains_any(text, &["database", "数据库", "mysql", "postgres"]) {
        "数据库"
    } else if contains_any(text, &["cache", "缓存", "redis"]) {
        "缓存"
    } else if contains_any(text, &["search", "搜索", "elasticsearch"]) {
        "搜索引擎"
    } else if contains_any(text, &["monitor", "监控"]) {
        "监控"
    } else {
        "Web框架"
    }
}

fn infer_pain_points(text: &str) -> Vec<String> {
    let mut pains = Vec::new();
    for point in [
        "扩展性不足",
        "性能瓶颈",
        "接口超时",
        "并发压力",
        "运维复杂",
        "数据一致性风险",
    ] {
        if text.contains(point) {
            pains.push(point.to_string());
        }
    }
    if pains.is_empty() {
        pains.push("需要结合业务上下文进一步确认痛点".to_string());
    }
    pains
}

fn infer_status_query_type_from_text(text: &str) -> &'static str {
    if contains_any(text, &["workspace", "工作区"]) {
        "workspace_status"
    } else if contains_any(text, &["session", "会话"]) {
        "session_status"
    } else if contains_any(text, &["task", "任务", "节点", "进度"]) {
        "task_status"
    } else if contains_any(text, &["agent", "worker"]) {
        "agent_status"
    } else {
        "system_health"
    }
}

fn has_platform_health_check_intent(text: &str) -> bool {
    let mentions_platform = contains_any(
        text,
        &[
            "淘宝",
            "天猫",
            "抖音",
            "千川",
            "京东",
            "拼多多",
            "pdd",
            "jd",
            "taobao",
            "tmall",
            "douyin",
            "tiktok",
            "api",
            "API",
            "平台",
        ],
    );
    let mentions_health = contains_any(
        text,
        &[
            "接口健康",
            "健康检查",
            "连接状态",
            "连接配置",
            "连通性",
            "连通",
            "测试连接",
            "API健康",
            "api健康",
            "同步状态",
            "凭证",
            "密钥",
            "授权",
        ],
    );
    mentions_platform && mentions_health
}

fn infer_platform_health_check_platform(text: &str) -> String {
    let platform = infer_search_platform(text);
    if platform == "电商" {
        String::new()
    } else {
        platform.to_string()
    }
}

fn extract_title_candidate(text: &str) -> Option<String> {
    for quote in ['"', '\'', '“', '”', '「', '」'] {
        let mut parts = text.split(quote);
        let _ = parts.next();
        if let Some(candidate) = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(compact_task_text(candidate));
        }
    }
    None
}

fn extract_keyword_candidates(text: &str) -> Vec<String> {
    ["SEO", "关键词", "标题", "转化", "新品", "直播", "抖音"]
        .iter()
        .filter(|keyword| text.contains(**keyword))
        .map(|keyword| (*keyword).to_string())
        .take(4)
        .collect()
}

fn compact_task_text(value: &str) -> String {
    let trimmed = value.trim();
    const MAX_CHARS: usize = 80;
    if trimmed.chars().count() <= MAX_CHARS {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(MAX_CHARS).collect::<String>();
    output.push_str("...");
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedProductContext {
    id: String,
    name: String,
    platform: Option<String>,
    category: Option<String>,
    brand: Option<String>,
    description: Option<String>,
    selling_points: Vec<String>,
}

fn selected_products_from_task(task: &Task) -> Vec<SelectedProductContext> {
    let Some(summary) = task
        .context
        .metadata
        .get("context.selected_summary")
        .map(String::as_str)
    else {
        return Vec::new();
    };

    let mut products = Vec::new();
    let mut pending_selling_points: Vec<(String, String)> = Vec::new();
    for line in summary.lines() {
        if let Some(product) = parse_product_summary_line(line) {
            products.push(product);
            continue;
        }
        if let Some(point) = parse_knowledge_summary_line(line) {
            pending_selling_points.push(point);
        }
    }

    for (product_id, point) in pending_selling_points {
        if let Some(product) = products.iter_mut().find(|product| product.id == product_id)
            && !product.selling_points.contains(&point)
        {
            product.selling_points.push(point);
        }
    }

    products
}

fn first_selected_product(task: &Task) -> Option<SelectedProductContext> {
    selected_products_from_task(task).into_iter().next()
}

fn selected_product_ref(task: &Task) -> Option<String> {
    first_selected_product(task).map(|product| product.ref_key())
}

fn selected_product_refs(task: &Task) -> Vec<String> {
    selected_products_from_task(task)
        .into_iter()
        .map(|product| product.ref_key())
        .collect()
}

fn selected_product_name_or_task(task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| compact_task_text(&task.description))
}

fn selected_product_brand_name_or_task(task: &Task) -> String {
    first_selected_product(task)
        .and_then(|product| {
            product
                .brand
                .filter(|brand| !brand.is_empty())
                .or(Some(product.name))
        })
        .unwrap_or_else(|| compact_task_text(&task.description))
}

fn selected_product_platform_or_infer(task: &Task, text: &str) -> String {
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| infer_platform(text).to_string())
}

fn selected_product_category_or_infer(task: &Task, text: &str) -> String {
    first_selected_product(task)
        .and_then(|product| product.category)
        .filter(|category| !category.is_empty())
        .unwrap_or_else(|| infer_category(text).to_string())
}

fn selected_product_features_or_infer(task: &Task, text: &str) -> Vec<String> {
    let mut features = first_selected_product(task)
        .map(|product| {
            let mut values = product.selling_points;
            if let Some(description) = product.description.filter(|value| !value.is_empty()) {
                values.push(description);
            }
            values
        })
        .unwrap_or_default();
    if features.is_empty() {
        features = infer_feature_list(text);
    }
    features
}

fn selected_product_count_or_default(task: &Task, default: i64) -> i64 {
    let count = selected_products_from_task(task).len();
    if count == 0 { default } else { count as i64 }
}

fn selected_product_article_platform_or_default(task: &Task, text: &str) -> String {
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| {
            if text.contains("小红书") {
                "小红书".to_string()
            } else {
                "微信公众号".to_string()
            }
        })
}

impl SelectedProductContext {
    fn ref_key(&self) -> String {
        if self.id.starts_with("product:") {
            self.id.clone()
        } else {
            format!("product:{}", self.id)
        }
    }
}

fn parse_product_summary_line(line: &str) -> Option<SelectedProductContext> {
    let line = line.trim();
    let rest = line.strip_prefix("Product source product:")?;
    let mut parts = rest.splitn(2, '|');
    let id = parts.next()?.trim();
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let name = fields.get("name")?.trim();
    if id.is_empty() || name.is_empty() {
        return None;
    }
    Some(SelectedProductContext {
        id: id.to_string(),
        name: name.to_string(),
        platform: non_empty_field(&fields, "platform"),
        category: non_empty_field(&fields, "category"),
        brand: non_empty_field(&fields, "brand"),
        description: non_empty_field(&fields, "description"),
        selling_points: Vec::new(),
    })
}

fn parse_knowledge_summary_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let rest = line.strip_prefix("Knowledge source product_knowledge:")?;
    let mut parts = rest.splitn(2, '|');
    let _knowledge_id = parts.next()?;
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let product_id = fields.get("product")?.trim();
    let content = fields.get("content")?.trim();
    if product_id.is_empty() || content.is_empty() {
        return None;
    }
    Some((product_id.to_string(), content.to_string()))
}

fn parse_pipe_fields(value: &str) -> std::collections::BTreeMap<String, String> {
    value
        .split('|')
        .filter_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn non_empty_field(
    fields: &std::collections::BTreeMap<String, String>,
    key: &str,
) -> Option<String> {
    fields
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn infer_memory_key(text: &str) -> String {
    let compact = compact_task_text(text);
    let mut key = compact
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    while key.contains("__") {
        key = key.replace("__", "_");
    }
    key = key.trim_matches('_').to_string();
    if key.is_empty() {
        "workspace_memory".to_string()
    } else {
        key.chars().take(80).collect()
    }
}

fn infer_memory_type(text: &str) -> &'static str {
    if contains_any(text, &["风险", "warning", "隐患"]) {
        "warning"
    } else if contains_any(text, &["决策", "决定", "decision"]) {
        "decision"
    } else if contains_any(text, &["洞察", "发现", "insight"]) {
        "insight"
    } else {
        "fact"
    }
}

fn infer_memory_content(text: &str, task: &Task) -> String {
    let compact = compact_task_text(text);
    if compact.chars().count() >= 24 {
        return compact.chars().take(500).collect();
    }
    compact_task_text(&task.description)
        .chars()
        .take(500)
        .collect()
}

fn infer_task_title(text: &str, task: &Task) -> String {
    extract_title_candidate(text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| compact_task_text(&task.description))
}

fn infer_task_owner_role(text: &str) -> &'static str {
    if contains_any(text, &["数据", "data", "分析"]) {
        "data"
    } else if contains_any(text, &["客服", "售后", "service"]) {
        "service"
    } else if contains_any(text, &["设计", "主图", "design"]) {
        "design"
    } else if contains_any(text, &["财务", "roi", "accounting"]) {
        "accounting"
    } else if contains_any(text, &["技术", "工程", "engineering", "开发"]) {
        "engineering"
    } else if contains_any(text, &["seo", "店铺", "web"]) {
        "web"
    } else if contains_any(text, &["创意", "视频", "creative", "文案"]) {
        "creative"
    } else {
        "ops"
    }
}

fn infer_task_priority(text: &str) -> i64 {
    if contains_any(text, &["紧急", "立刻", "马上", "高优", "urgent", "asap"]) {
        8
    } else if contains_any(text, &["重要", "优先", "priority"]) {
        6
    } else {
        3
    }
}

fn infer_task_acceptance(text: &str) -> String {
    if let Some(index) = text.find("验收") {
        return compact_task_text(&text[index..]);
    }
    if let Some(index) = text.find("目标") {
        return compact_task_text(&text[index..]);
    }
    "产出明确结果并可在工作区复用".to_string()
}

fn build_context(
    worker_id: AgentId,
    phase: WorkerPhase,
    memory: &TaskMemory,
    role: &RoleProfile,
    task: &Task,
) -> Context {
    let mut metadata = HashMap::new();
    metadata.insert("worker_id".to_string(), worker_id.0);
    metadata.insert("role.id".to_string(), role.id.0.clone());
    metadata.insert("role.name".to_string(), role.name.clone());
    metadata.insert("role.runtime_role".to_string(), role.runtime_role.clone());
    metadata.insert("phase".to_string(), format!("{phase:?}"));

    for (key, value) in &task.context.metadata {
        metadata.insert(format!("request.{key}"), value.clone());
    }

    for (key, value) in &memory.state {
        metadata.insert(format!("memory.{key}"), value.clone());
    }

    if let Some(assignment) = &task.assignment {
        metadata.insert(
            "assignment.graph_id".to_string(),
            assignment.graph_id.0.clone(),
        );
        metadata.insert(
            "assignment.node_id".to_string(),
            assignment.node_id.0.clone(),
        );
        metadata.insert("assignment.role".to_string(), assignment.role.clone());
        metadata.insert(
            "assignment.attempt".to_string(),
            assignment.attempt.to_string(),
        );
        metadata.insert(
            "assignment.input_count".to_string(),
            assignment.inputs.len().to_string(),
        );
        if let Some(instruction) = &assignment.rework_instruction {
            metadata.insert(
                "assignment.rework_instruction".to_string(),
                instruction.clone(),
            );
        }
    }

    let mut facts = Vec::new();
    extend_context_facts(&task.context.metadata, &mut facts);
    if let Some(assignment) = &task.assignment {
        facts.push(Fact {
            source: "worker.assignment".to_string(),
            content: format!(
                "node={} role={} attempt={} objective={}",
                assignment.node_id.0, assignment.role, assignment.attempt, assignment.objective
            ),
            reliability: 1.0,
        });
        for input in &assignment.inputs {
            facts.push(Fact {
                source: format!("worker.assignment.input.{}", input.node_id.0),
                content: input.content.clone(),
                reliability: 1.0,
            });
        }
    }

    let role_prompt = RolePromptBuilder::build_worker_prompt(&RolePromptInput {
        role: role.clone(),
        task: task.description.clone(),
        facts: memory.progress.clone(),
    });

    facts.push(Fact {
        source: "roles.prompt".to_string(),
        content: role_prompt,
        reliability: 1.0,
    });
    facts.extend(
        memory
            .progress
            .iter()
            .map(|entry| Fact {
                source: "worker.runtime".to_string(),
                content: entry.clone(),
                reliability: 1.0,
            })
            .collect::<Vec<_>>(),
    );

    Context { facts, metadata }
}

fn conversation_aware_task_description(task: &Task) -> String {
    if task.description.contains("同一会话历史上下文") {
        return task.description.clone();
    }

    let Some(history) = task
        .context
        .metadata
        .get("conversation.history_summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return task.description.clone();
    };

    format!(
        "同一会话历史上下文（用于理解追问、继续、补充、按前面数据等表达；这不代表用户选择了任何预设商品）：\n{history}\n\n当前用户请求：\n{}",
        task.description
    )
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

#[derive(Debug, Clone, Default)]
struct ParsedWorkerRoleOutput {
    content: String,
    role_output: Option<crate::core::protocol::RoleWorkOutput>,
    evidence: Vec<String>,
    risks: Vec<String>,
    open_questions: Vec<String>,
}

fn parse_worker_role_output(output: &str) -> ParsedWorkerRoleOutput {
    let trimmed = output.trim();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return ParsedWorkerRoleOutput {
            content: trimmed.to_string(),
            ..ParsedWorkerRoleOutput::default()
        };
    };

    let role_output = parse_role_work_output(&value);
    let evidence = role_output
        .as_ref()
        .map(|output| output.evidence.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| string_array_field(&value, "evidence"));
    let risks = role_output
        .as_ref()
        .map(|output| output.risks.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| string_array_field(&value, "risks"));
    let open_questions = role_output
        .as_ref()
        .map(|output| output.open_questions.clone())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| string_array_field(&value, "open_questions"));
    let content = role_output
        .as_ref()
        .map(format_role_work_output_content)
        .filter(|content| !content.trim().is_empty())
        .unwrap_or_else(|| extract_worker_content(trimmed));

    ParsedWorkerRoleOutput {
        content,
        role_output,
        evidence,
        risks,
        open_questions,
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedSkillRequest {
    skill_id: String,
    input: serde_json::Value,
    reason: Option<String>,
}

fn parse_skill_requests(output: &str) -> Vec<ParsedSkillRequest> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(output.trim()) else {
        return Vec::new();
    };
    value
        .get("skill_requests")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(parse_skill_request)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_skill_request(value: &serde_json::Value) -> Option<ParsedSkillRequest> {
    let skill_id = value
        .get("skill_id")
        .or_else(|| value.get("id"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_string();
    let input = value
        .get("input")
        .or_else(|| value.get("arguments"))
        .or_else(|| value.get("args"))
        .or_else(|| value.get("parameters"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let reason = string_field(value, "reason");

    Some(ParsedSkillRequest {
        skill_id,
        input,
        reason,
    })
}

fn build_skill_context(
    worker_id: &AgentId,
    role: &RoleProfile,
    task: &Task,
    request: &ParsedSkillRequest,
    request_id: &str,
) -> SkillContext {
    let mut context = SkillContext::new()
        .with_invocation_id(request_id.to_string())
        .with_agent_id(worker_id.0.clone())
        .with_metadata("task_id", serde_json::json!(task.id))
        .with_metadata("task_description", serde_json::json!(task.description))
        .with_metadata("role_id", serde_json::json!(role.id.0))
        .with_metadata("role_name", serde_json::json!(role.name))
        .with_metadata("role_runtime", serde_json::json!(role.runtime_role));
    if let Some(user_id) = &task.context.user_id {
        context = context.with_metadata("message.user_id", serde_json::json!(user_id.0.clone()));
    }
    if let Some(workspace_id) = &task.context.workspace_id {
        context = context.with_metadata(
            "message.workspace_id",
            serde_json::json!(workspace_id.0.clone()),
        );
    }
    if let Some(agent_id) = &task.context.agent_id {
        context = context.with_metadata("message.agent_id", serde_json::json!(agent_id.0.clone()));
    }
    if let Some(session_id) = &task.context.session_id {
        context = context.with_session_id(session_id.0.clone());
        context = context.with_metadata(
            "message.session_id",
            serde_json::json!(session_id.0.clone()),
        );
    }
    if let Some(task_id) = &task.context.task_id {
        context = context.with_metadata("message.task_id", serde_json::json!(task_id.0.clone()));
    }
    for (key, value) in &task.context.metadata {
        context = context.with_metadata(format!("request.{key}"), serde_json::json!(value));
    }
    if let Some(reason) = &request.reason {
        context = context.with_metadata("request_reason", serde_json::json!(reason));
    }
    if let Some(assignment) = &task.assignment {
        let upstream_roles = assignment
            .inputs
            .iter()
            .map(|input| input.role.as_str())
            .collect::<Vec<_>>()
            .join(",");
        context = context
            .with_metadata("graph_id", serde_json::json!(assignment.graph_id.0))
            .with_metadata("node_id", serde_json::json!(assignment.node_id.0))
            .with_metadata(
                "assignment_role",
                serde_json::json!(assignment.role.clone()),
            )
            .with_metadata("attempt", serde_json::json!(assignment.attempt))
            .with_metadata(
                "assignment_input_count",
                serde_json::json!(assignment.inputs.len()),
            )
            .with_metadata("assignment_input_roles", serde_json::json!(upstream_roles));
        if let Some(instruction) = &assignment.rework_instruction {
            context = context.with_metadata(
                "assignment_rework_instruction",
                serde_json::json!(instruction),
            );
        }
    }
    context
}

fn merge_skill_results(
    mut parsed: ParsedWorkerRoleOutput,
    results: &[SkillExecutionResult],
) -> ParsedWorkerRoleOutput {
    if results.is_empty() {
        return parsed;
    }

    let role_output = parsed
        .role_output
        .get_or_insert_with(crate::core::protocol::RoleWorkOutput::default);

    for result in results {
        match result.status {
            SkillExecutionStatus::Success => {
                let evidence = format_success_skill_evidence(result);
                push_unique(&mut parsed.evidence, evidence.clone());
                push_unique(&mut role_output.evidence, evidence);
            }
            SkillExecutionStatus::Deferred | SkillExecutionStatus::PendingApproval => {
                let risk = format_incomplete_skill_risk(result);
                let question = format_incomplete_skill_question(result);
                push_unique(&mut parsed.risks, risk.clone());
                push_unique(&mut parsed.open_questions, question.clone());
                push_unique(&mut role_output.risks, risk);
                push_unique(&mut role_output.open_questions, question);
            }
            SkillExecutionStatus::NotFound
            | SkillExecutionStatus::ValidationError
            | SkillExecutionStatus::Failed => {
                let risk = format_failed_skill_risk(result);
                push_unique(&mut parsed.risks, risk.clone());
                push_unique(&mut parsed.open_questions, risk.clone());
                push_unique(&mut role_output.risks, risk.clone());
                push_unique(&mut role_output.open_questions, risk);
            }
        }
    }

    parsed.content = parsed
        .role_output
        .as_ref()
        .map(format_role_work_output_content)
        .filter(|content| !content.trim().is_empty())
        .unwrap_or(parsed.content);
    parsed
}

fn format_success_skill_evidence(result: &SkillExecutionResult) -> String {
    let summary = result
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|summary| !summary.is_empty())
        .map(ToString::to_string)
        .or_else(|| result.output.as_ref().map(compact_json_preview))
        .unwrap_or_else(|| "completed".to_string());
    format!(
        "skill:{} status=success summary={}",
        result.skill_id.as_str(),
        summary
    )
}

fn format_incomplete_skill_risk(result: &SkillExecutionResult) -> String {
    format!(
        "skill:{} status={:?} reason={}",
        result.skill_id.as_str(),
        result.status,
        result
            .error
            .as_deref()
            .or(result.summary.as_deref())
            .unwrap_or("runtime adapter required")
    )
}

fn format_incomplete_skill_question(result: &SkillExecutionResult) -> String {
    format!(
        "skill:{} 尚未产出可用结果，需要补齐适配器、审批或外部执行后再判断。",
        result.skill_id.as_str()
    )
}

fn format_failed_skill_risk(result: &SkillExecutionResult) -> String {
    format!(
        "skill:{} status={:?} error={}",
        result.skill_id.as_str(),
        result.status,
        result.error.as_deref().unwrap_or("unknown error")
    )
}

fn compact_json_preview(value: &serde_json::Value) -> String {
    let mut text = value.to_string();
    const MAX_LEN: usize = 240;
    if text.chars().count() > MAX_LEN {
        text = text.chars().take(MAX_LEN).collect::<String>();
        text.push_str("...");
    }
    text
}

fn skill_executor_unavailable_result(request_id: String, skill_id: String) -> SkillExecutionResult {
    SkillExecutionResult {
        request_id,
        skill_id: SkillId::new(skill_id),
        status: SkillExecutionStatus::Failed,
        output: None,
        summary: None,
        metadata: serde_json::Map::new(),
        error: Some("skill executor unavailable".to_string()),
    }
}

fn compact_event_text(value: String) -> String {
    let trimmed = value.trim();
    const MAX_LEN: usize = 240;
    if trimmed.chars().count() <= MAX_LEN {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(MAX_LEN).collect::<String>();
    output.push_str("...");
    output
}

fn metadata_string(
    metadata: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    metadata.get(key).and_then(|value| {
        value
            .as_str()
            .map(ToString::to_string)
            .or_else(|| value.as_object().map(|_| value.to_string()))
    })
}

fn skill_status_name(status: SkillExecutionStatus) -> &'static str {
    match status {
        SkillExecutionStatus::Success => "success",
        SkillExecutionStatus::Deferred => "deferred",
        SkillExecutionStatus::PendingApproval => "pending_approval",
        SkillExecutionStatus::NotFound => "not_found",
        SkillExecutionStatus::ValidationError => "validation_error",
        SkillExecutionStatus::Failed => "failed",
    }
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if value.trim().is_empty() || items.iter().any(|item| item == &value) {
        return;
    }
    items.push(value);
}

fn parse_role_work_output(
    value: &serde_json::Value,
) -> Option<crate::core::protocol::RoleWorkOutput> {
    let source = value.get("role_output").unwrap_or(value);
    if let Some(text) = source.as_str() {
        let summary = text.trim().to_string();
        return (!summary.is_empty()).then_some(crate::core::protocol::RoleWorkOutput {
            summary,
            evidence: string_array_field(value, "evidence"),
            risks: string_array_field(value, "risks"),
            open_questions: string_array_field(value, "open_questions"),
            ..crate::core::protocol::RoleWorkOutput::default()
        });
    }

    if !source.is_object() {
        return None;
    }

    let summary = string_field(source, "summary")
        .or_else(|| string_field(source, "结论"))
        .or_else(|| string_field(value, "summary"))
        .unwrap_or_default();
    let findings = string_array_field(source, "findings");
    let recommendations = string_array_field(source, "recommendations")
        .into_iter()
        .chain(string_array_field(source, "actions"))
        .collect::<Vec<_>>();
    let evidence = string_array_field(source, "evidence")
        .into_iter()
        .chain(string_array_field(value, "evidence"))
        .collect::<Vec<_>>();
    let risks = string_array_field(source, "risks")
        .into_iter()
        .chain(string_array_field(value, "risks"))
        .collect::<Vec<_>>();
    let open_questions = string_array_field(source, "open_questions")
        .into_iter()
        .chain(string_array_field(value, "open_questions"))
        .collect::<Vec<_>>();

    let output = crate::core::protocol::RoleWorkOutput {
        summary,
        findings,
        recommendations,
        evidence,
        risks,
        open_questions,
    };
    has_role_work_output_content(&output).then_some(output)
}

fn has_role_work_output_content(output: &crate::core::protocol::RoleWorkOutput) -> bool {
    !output.summary.trim().is_empty()
        || !output.findings.is_empty()
        || !output.recommendations.is_empty()
        || !output.evidence.is_empty()
        || !output.risks.is_empty()
        || !output.open_questions.is_empty()
}

fn format_role_work_output_content(output: &crate::core::protocol::RoleWorkOutput) -> String {
    let mut sections = Vec::new();
    if !output.summary.trim().is_empty() {
        sections.push(output.summary.trim().to_string());
    }
    extend_section(&mut sections, "发现", &output.findings);
    extend_section(&mut sections, "建议", &output.recommendations);
    extend_section(&mut sections, "风险", &output.risks);
    extend_section(&mut sections, "缺口", &output.open_questions);
    sections.join("\n")
}

fn extend_section(lines: &mut Vec<String>, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    lines.push(format!(
        "{}：{}",
        title,
        items
            .iter()
            .filter(|item| !item.trim().is_empty())
            .map(|item| item.trim().to_string())
            .collect::<Vec<_>>()
            .join("；")
    ));
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn string_array_field(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn new_task_id() -> String {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_TASK_COUNTER: AtomicU64 = AtomicU64::new(1);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let sequence = NEXT_TASK_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("task-{millis}-{sequence}")
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
    use super::{
        autosuggest_skill_requests, merge_skill_results, parse_skill_requests,
        parse_worker_role_output, resolve_skill_requests,
    };
    use crate::{
        core::protocol::MessageContext,
        roles::{RoleCatalog, RoleProfile},
        skills::{SkillExecutionResult, SkillExecutionStatus, SkillId},
    };

    fn role(runtime_role: &str) -> RoleProfile {
        RoleCatalog::builtin()
            .roles()
            .iter()
            .find(|role| role.runtime_role == runtime_role)
            .expect("role should exist")
            .clone()
    }

    fn task(description: &str) -> super::Task {
        super::Task {
            id: "task-autosuggest".to_string(),
            context: MessageContext::default(),
            description: description.to_string(),
            requester: crate::core::protocol::AgentId("test".to_string()),
            assignment: None,
        }
    }

    fn task_with_metadata(description: &str, metadata: &[(&str, &str)]) -> super::Task {
        let mut task = task(description);
        for (key, value) in metadata {
            task.context
                .metadata
                .insert((*key).to_string(), (*value).to_string());
        }
        task
    }

    fn selected_product_summary() -> String {
        [
            "Commerce product context for selected user products.",
            "Product source product:11111111-1111-1111-1111-111111111111 | name=五一咖啡组合 | platform=抖音 | category=咖啡饮品 | brand=Atlas | price=99 CNY | status=active | description=节日主推组合",
            "Knowledge source product_knowledge:22222222-2222-2222-2222-222222222222 | product=11111111-1111-1111-1111-111111111111 | title=核心卖点 | type=selling_point | source=manual | confidence=0.90 | content=低糖、便携、适合五一出游",
        ]
        .join("\n")
    }

    #[test]
    fn explicit_skill_requests_are_not_overridden_by_autosuggest() {
        let requests = resolve_skill_requests(
            &role("accounting"),
            &task("calculate ROI"),
            r#"{
              "skill_requests": [
                {
                  "skill_id": "accounting_cost_calc",
                  "input": {"purchase_cost": 20, "selling_price": 49}
                }
              ]
            }"#,
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "accounting_cost_calc");
    }

    #[test]
    fn autosuggests_ops_execution_plan_without_llm_tool_json() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("help plan a new product launch and conversion improvement"),
            r#"{"role_output":{"summary":"Need an operations plan."}}"#,
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_execution_plan");
        assert_eq!(requests[0].input["stage"], "冷启动");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_learning_teaching_explain_for_chat() {
        let requests = autosuggest_skill_requests(
            &role("chat"),
            &task("请教学解释ROI和ROAS的区别，给我一个电商场景例子和常见误区"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "learning_teaching_explain");
        assert_eq!(requests[0].input["topic"], "ROI和ROAS的区别");
        assert_eq!(requests[0].input["scenario"], "电商投放复盘");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_learning_uses_only_explicit_selected_product() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "请结合这个商品讲解转化率CVR为什么重要",
                &[("context.selected_summary", &selected_product_summary())],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "learning_teaching_explain");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
    }

    #[test]
    fn autosuggests_more_specific_ops_skills_by_intent() {
        let cases = [
            (
                "记住这个结论：五一咖啡活动应该优先保留高 ROI 搜索计划",
                "coordination_save_memory",
            ),
            (
                "给这个投放复盘建个任务，安排给数据同学，目标是输出问题拆解",
                "coordination_create_task",
            ),
            (
                "给咖啡新品做渠道分配和渠道策略，预算20000",
                "ops_channel_strategy",
            ),
            ("给咖啡新品做618促销活动，预算20000", "ops_promo_planning"),
            (
                "帮这个保温杯写上架文案和标题卖点，突出便携防漏",
                "ops_listing_copy",
            ),
            (
                "做一份零食货盘和SKU组合规划，选品预算80000",
                "ops_assortment_planning",
            ),
            (
                "给咖啡新品做智能定价：成本38，当前售价99，月销量820，竞品价89，竞品价109，库存天数45",
                "ops_smart_pricing",
            ),
            (
                "做SKU库存优化和EOQ：SKU冷萃A 月销量300 成本40 库存500；SKU拿铁B 月销量180 成本35 库存160",
                "ops_inventory_optimizer",
            ),
            (
                "做ABC-XYZ库存分类：SKU冷萃A GMV12万 历史销量100 102 98 101；SKU拿铁B GMV6万 历史销量60 88 44 96",
                "ops_abc_xyz_classifier",
            ),
            (
                "检测广告疲劳和CTR衰减：抖音CTR基准3.2，近期CTR2.4，频次6，预算5000，ROAS3.2",
                "ops_ad_fatigue_detector",
            ),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("ops"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn selected_product_context_feeds_pricing_strategy_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做定价策略：成本42，竞品价89，竞品价99，竞品价109",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_pricing_strategy");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[0].input["cost"], 42.0);
        let prices = requests[0].input["competitor_prices"]
            .as_array()
            .expect("competitor prices should be parsed");
        assert_eq!(prices, &[89.0, 99.0, 109.0]);
    }

    #[test]
    fn pricing_strategy_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("给咖啡新品做定价策略：成本42，竞品价89，竞品价99，竞品价109"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_pricing_strategy");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["cost"], 42.0);
    }

    #[test]
    fn autosuggests_smart_pricing_with_explicit_price_inputs() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task(
                "给咖啡新品做动态定价和调价建议：成本38，当前售价99，月销量820，竞品价89，竞品价109，竞品价119，库存天数45，价格弹性1.6",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_smart_pricing");
        assert_eq!(requests[0].input["cost"], 38.0);
        assert_eq!(requests[0].input["current_price"], 99.0);
        assert_eq!(requests[0].input["current_monthly_sales"], 820.0);
        assert_eq!(requests[0].input["stock_days"], 45.0);
        assert_eq!(requests[0].input["elasticity"], -1.6);
        let prices = requests[0].input["competitor_prices"]
            .as_array()
            .expect("competitor prices should be parsed");
        assert_eq!(prices, &[89.0, 109.0, 119.0]);
    }

    #[test]
    fn autosuggests_promo_planning_with_labeled_budget_not_campaign_number() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("给冷萃咖啡做618大促活动方案，活动预算2万"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_promo_planning");
        assert_eq!(requests[0].input["budget"], 20000.0);
        assert!(
            requests[0].input["product"]
                .as_str()
                .unwrap_or_default()
                .contains("冷萃咖啡")
        );
    }

    #[test]
    fn selected_product_context_feeds_promo_planning_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做618促销活动，预算20000",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_promo_planning");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product"], "五一咖啡组合");
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[0].input["budget"], 20000.0);
    }

    #[test]
    fn promo_planning_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("给冷萃咖啡做618促销活动，预算20000"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_promo_planning");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_douyin_ads_plan_structure_with_selected_product() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("douyin_ads"),
            &task_with_metadata(
                "给这个商品做千川冷启动计划结构，千川预算2万，目标ROAS2.8，最近7天",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "douyin_ads_plan_structure");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[0].input["budget"], 20000.0);
        assert_eq!(requests[0].input["target_roas"], 2.8);
    }

    #[test]
    fn douyin_ads_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("douyin_ads"),
            &task("给咖啡新品做千川投放计划，预算20000，目标ROAS2.5"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "douyin_ads_plan_structure");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_douyin_ads_budget_pacing_from_metrics() {
        let requests = autosuggest_skill_requests(
            &role("douyin_ads"),
            &task("复盘千川预算节奏：总预算5万，已消耗1.2万，GMV4.2万，目标ROAS3"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "douyin_ads_budget_pacing");
        assert_eq!(requests[0].input["budget"], 50000.0);
        assert_eq!(requests[0].input["spend"], 12000.0);
        assert_eq!(requests[0].input["gmv"], 42000.0);
        assert_eq!(requests[0].input["target_roas"], 3.0);
    }

    #[test]
    fn autosuggests_douyin_ads_live_funnel_from_metrics() {
        let requests = autosuggest_skill_requests(
            &role("douyin_ads"),
            &task(
                "诊断直播间漏斗：曝光10万，点击3200，进房1800，商品点击360，下单72，付款60，广告消耗1.2万，GMV4.2万",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "douyin_ads_live_funnel_diagnosis");
        assert_eq!(requests[0].input["impressions"], 100000.0);
        assert_eq!(requests[0].input["clicks"], 3200.0);
        assert_eq!(requests[0].input["live_room_entries"], 1800.0);
        assert_eq!(requests[0].input["product_clicks"], 360.0);
        assert_eq!(requests[0].input["orders"], 72.0);
        assert_eq!(requests[0].input["payments"], 60.0);
        assert_eq!(requests[0].input["ad_spend"], 12000.0);
        assert_eq!(requests[0].input["gmv"], 42000.0);
    }

    #[test]
    fn douyin_ads_external_context_adds_visible_search_step() {
        let requests = autosuggest_skill_requests(
            &role("douyin_ads"),
            &task("复盘千川预算节奏：预算5万，消耗1万，GMV3万，并查行业ROI基准，最多3条来源"),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "douyin_ads_budget_pacing");
        assert_eq!(requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(requests[1].input["metric"], "roi");
        assert_eq!(requests[1].input["max_results"], 3);
    }

    #[test]
    fn selected_product_context_feeds_channel_strategy_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做渠道分配和渠道策略，渠道预算20000",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_channel_strategy");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product_type"], "咖啡饮品");
        assert_eq!(requests[0].input["budget"], 20000.0);
    }

    #[test]
    fn channel_strategy_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("给冷萃咖啡做618渠道分配和渠道策略，预算20000"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_channel_strategy");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["budget"], 20000.0);
    }

    #[test]
    fn selected_product_context_feeds_assortment_planning_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做货盘和SKU组合规划，选品预算80000",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_assortment_planning");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["category"], "咖啡饮品");
        assert_eq!(requests[0].input["budget"], 80000.0);
    }

    #[test]
    fn assortment_planning_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("做一份零食货盘和SKU组合规划，选品预算80000"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_assortment_planning");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["budget"], 80000.0);
    }

    #[test]
    fn selected_product_context_feeds_inventory_planning_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做库存和补货规划：日均销量120，供货周期7天",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_inventory_planning");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["avg_daily_demand"], 120.0);
        assert_eq!(requests[0].input["lead_time_days"], 7);
        assert_eq!(requests[0].input["platform"], "抖音");
    }

    #[test]
    fn inventory_planning_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("日均销量120，供货周期7天，帮冷萃咖啡算库存和补货点"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_inventory_planning");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["avg_daily_demand"], 120.0);
        assert_eq!(requests[0].input["lead_time_days"], 7);
    }

    #[test]
    fn selected_product_context_feeds_execution_plan_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做新品冷启动运营执行计划",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_execution_plan");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["stage"], "冷启动");
    }

    #[test]
    fn execution_plan_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("帮冷萃咖啡做新品冷启动运营执行计划"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_execution_plan");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["stage"], "冷启动");
    }

    #[test]
    fn selected_product_context_feeds_smart_pricing_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做智能定价：成本38，当前售价99，月销量820，竞品价89，竞品价109，库存天数45",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_smart_pricing");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[0].input["cost"], 38.0);
        assert_eq!(requests[0].input["current_price"], 99.0);
    }

    #[test]
    fn smart_pricing_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("给咖啡新品做智能定价：成本38，当前售价99，竞品价89，竞品价109"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_smart_pricing");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_ops_pricing_search_chain_when_realtime_requested() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task(
                "给咖啡新品做智能定价：成本38，当前售价99，并联网查最新竞品价格区间，最多5条来源",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "ops_smart_pricing");
        assert_eq!(requests[1].skill_id, "search_competitor");
        assert_eq!(requests[1].input["focus"], "price");
        assert_eq!(requests[1].input["platform"], "淘宝");
        assert_eq!(requests[1].input["days"], 14);
        assert_eq!(requests[1].input["max_results"], 5);
        assert_eq!(requests[1].input["_needs_fresh_data"], true);
        assert!(requests[1].input.get("product_ref").is_none());
        assert!(requests[1].input.get("product_name").is_none());
    }

    #[test]
    fn autosuggests_ops_promo_policy_search_chain_when_rules_requested() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task(
                "给冷萃咖啡做618促销活动，活动预算2万，查抖音最新大促规则和报名规则，最多4条来源",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "ops_promo_planning");
        assert_eq!(requests[0].input["budget"], 20000.0);
        assert_eq!(requests[1].skill_id, "search_platform_policy");
        assert_eq!(requests[1].input["platform"], "抖音");
        assert_eq!(requests[1].input["policy_type"], "promotion");
        assert_eq!(requests[1].input["max_results"], 4);
        assert_eq!(requests[1].input["_needs_fresh_data"], true);
    }

    #[test]
    fn autosuggests_ops_channel_and_assortment_search_chains() {
        let channel_requests = autosuggest_skill_requests(
            &role("ops"),
            &task("给咖啡新品做渠道分配和渠道策略，预算20000，查最新行业ROI基准"),
            "{}",
        );

        assert_eq!(channel_requests.len(), 2);
        assert_eq!(channel_requests[0].skill_id, "ops_channel_strategy");
        assert_eq!(channel_requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(channel_requests[1].input["metric"], "roi");
        assert_eq!(channel_requests[1].input["industry"], "咖啡饮品");
        assert_eq!(channel_requests[1].input["_needs_fresh_data"], true);

        let assortment_requests = autosuggest_skill_requests(
            &role("ops"),
            &task("做一份零食货盘和SKU组合规划，选品预算80000，联网查最近30天热卖趋势"),
            "{}",
        );

        assert_eq!(assortment_requests.len(), 2);
        assert_eq!(assortment_requests[0].skill_id, "ops_assortment_planning");
        assert_eq!(assortment_requests[1].skill_id, "search_trends");
        assert_eq!(assortment_requests[1].input["platform"], "淘宝");
        assert_eq!(assortment_requests[1].input["days"], 30);
        assert_eq!(assortment_requests[1].input["_needs_fresh_data"], true);
    }

    #[test]
    fn autosuggests_ops_listing_copy_search_chains_when_requested() {
        let policy_requests = autosuggest_skill_requests(
            &role("ops"),
            &task("帮冷萃咖啡写上架文案和商品标题，查抖音最新标题规则和违禁词，最多4条来源"),
            "{}",
        );

        assert_eq!(policy_requests.len(), 2);
        assert_eq!(policy_requests[0].skill_id, "ops_listing_copy");
        assert_eq!(policy_requests[1].skill_id, "search_platform_policy");
        assert_eq!(policy_requests[1].input["platform"], "抖音");
        assert_eq!(policy_requests[1].input["policy_type"], "content");
        assert_eq!(policy_requests[1].input["max_results"], 4);
        assert_eq!(policy_requests[1].input["_needs_fresh_data"], true);
        assert!(policy_requests[1].input.get("product_ref").is_none());

        let review_requests = autosuggest_skill_requests(
            &role("ops"),
            &task("帮冷萃咖啡写上架文案和标题卖点，联网查买家痛点、真实评价和购买顾虑"),
            "{}",
        );

        assert_eq!(review_requests.len(), 2);
        assert_eq!(review_requests[0].skill_id, "ops_listing_copy");
        assert_eq!(review_requests[1].skill_id, "search_product_reviews");
        assert_eq!(review_requests[1].input["focus"], "negative");
        assert_eq!(review_requests[1].input["platform"], "淘宝");
        assert_eq!(review_requests[1].input["days"], 30);
        assert_eq!(review_requests[1].input["_needs_fresh_data"], true);

        let market_requests = autosuggest_skill_requests(
            &role("ops"),
            &task("帮冷萃咖啡写上架文案和商品标题，联网查最新标题趋势和搜索量，最多3条来源"),
            "{}",
        );

        assert_eq!(market_requests.len(), 2);
        assert_eq!(market_requests[0].skill_id, "ops_listing_copy");
        assert_eq!(market_requests[1].skill_id, "search_market_info");
        assert_eq!(market_requests[1].input["topic"], "general");
        assert_eq!(market_requests[1].input["max_results"], 3);
        assert!(
            market_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("上架标题")
                    && query.contains("搜索趋势")
                    && query.contains("高点击"))
        );
    }

    #[test]
    fn ops_listing_copy_remains_native_without_search_intent() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("帮这个保温杯写上架文案和标题卖点，突出便携防漏"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_listing_copy");
    }

    #[test]
    fn ops_listing_copy_search_preserves_selected_product_context() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "帮这个商品写上架文案和标题卖点，联网查买家痛点和真实评价",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "ops_listing_copy");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].skill_id, "search_product_reviews");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(requests[1].input["platform"], "抖音");
    }

    #[test]
    fn ops_search_chain_keeps_selected_product_explicit() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做定价策略：成本42，并联网查最新竞品价格区间",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "ops_pricing_strategy");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].skill_id, "search_competitor");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(requests[1].input["platform"], "抖音");
    }

    #[test]
    fn autosuggests_inventory_optimizer_with_explicit_sku_rows() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task(
                "做库存优化和安全库存：SKU冷萃A 月销量300 成本40 库存500；SKU拿铁B 月销量180 成本35 库存160，供货周期14，服务水平98%",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_inventory_optimizer");
        assert_eq!(requests[0].input["lead_time_days"], 14.0);
        assert_eq!(requests[0].input["service_level"], "98%");
        let skus = requests[0].input["skus"]
            .as_array()
            .expect("skus should be parsed");
        assert_eq!(skus.len(), 2);
        assert_eq!(skus[0]["name"], "SKU冷萃A");
        assert_eq!(skus[0]["monthly_sales"], 300.0);
        assert_eq!(skus[0]["cost"], 40.0);
        assert_eq!(skus[0]["stock_qty"], 500.0);
        assert_eq!(skus[1]["name"], "SKU拿铁B");
        assert_eq!(skus[1]["monthly_sales"], 180.0);
        assert_eq!(skus[1]["cost"], 35.0);
        assert_eq!(skus[1]["stock_qty"], 160.0);
    }

    #[test]
    fn selected_product_context_feeds_inventory_optimizer_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做库存健康和再订货点优化：SKU冷萃A 月销量300 成本40 库存500",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_inventory_optimizer");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        let skus = requests[0].input["skus"]
            .as_array()
            .expect("skus should be parsed");
        assert_eq!(skus[0]["name"], "SKU冷萃A");
    }

    #[test]
    fn inventory_optimizer_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("做库存优化：SKU冷萃A 月销量300 成本40 库存500"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_inventory_optimizer");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_abc_xyz_classifier_with_explicit_sku_rows() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task(
                "做ABC-XYZ库存分类和九宫格库存矩阵：SKU冷萃A GMV12万 历史销量100 102 98 101；SKU拿铁B GMV6万 历史销量60 88 44 96，展示前2",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_abc_xyz_classifier");
        assert_eq!(requests[0].input["top_n"], 2);
        let skus = requests[0].input["skus"]
            .as_array()
            .expect("abc xyz skus should be parsed");
        assert_eq!(skus.len(), 2);
        assert_eq!(skus[0]["sku_id"], "SKU冷萃A");
        assert_eq!(skus[0]["gmv"], 120000.0);
        assert_eq!(
            skus[0]["sales_history"]
                .as_array()
                .expect("sales history should be parsed"),
            &[100.0, 102.0, 98.0, 101.0]
        );
        assert_eq!(skus[1]["sku_id"], "SKU拿铁B");
        assert_eq!(skus[1]["gmv"], 60000.0);
    }

    #[test]
    fn selected_product_context_feeds_abc_xyz_classifier_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品做ABC-XYZ库存分类：SKU冷萃A GMV12万 历史销量100 102 98 101",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_abc_xyz_classifier");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        let skus = requests[0].input["skus"]
            .as_array()
            .expect("abc xyz skus should be parsed");
        assert_eq!(skus[0]["sku_id"], "SKU冷萃A");
    }

    #[test]
    fn abc_xyz_classifier_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("做ABC-XYZ库存分类：SKU冷萃A GMV12万 历史销量100 102 98 101"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_abc_xyz_classifier");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn basic_replenishment_still_routes_to_inventory_planning() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("日均销量120，供货周期7天，帮我算补货点"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_inventory_planning");
        assert_eq!(requests[0].input["avg_daily_demand"], 120.0);
        assert_eq!(requests[0].input["lead_time_days"], 7);
    }

    #[test]
    fn autosuggests_ad_fatigue_detector_with_explicit_channel_rows() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task(
                "检测广告疲劳和素材疲劳：抖音CTR基准3.2，近期CTR2.4，频次6，预算5000，ROAS3.2；淘宝CTR基准2.8，近期CTR1.7，曝光频次9，预算3000，ROAS2.1，总预算9000",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_ad_fatigue_detector");
        assert_eq!(requests[0].input["total_budget"], 9000.0);
        let channels = requests[0].input["channels"]
            .as_array()
            .expect("ad fatigue channels should be parsed");
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0]["name"], "抖音");
        assert_eq!(channels[0]["ctr_baseline"], 3.2);
        assert_eq!(channels[0]["ctr_recent"], 2.4);
        assert_eq!(channels[0]["impressions_per_user"], 6.0);
        assert_eq!(channels[0]["budget"], 5000.0);
        assert_eq!(channels[0]["roas"], 3.2);
        assert_eq!(channels[1]["name"], "淘宝");
        assert_eq!(channels[1]["ctr_baseline"], 2.8);
        assert_eq!(channels[1]["ctr_recent"], 1.7);
        assert_eq!(channels[1]["impressions_per_user"], 9.0);
        assert_eq!(channels[1]["budget"], 3000.0);
        assert_eq!(channels[1]["roas"], 2.1);
    }

    #[test]
    fn selected_product_context_feeds_ad_fatigue_detector_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "给这个商品检测广告疲劳：抖音CTR基准3.2，近期CTR2.4，频次6，预算5000，ROAS3.2",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_ad_fatigue_detector");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        let channels = requests[0].input["channels"]
            .as_array()
            .expect("ad fatigue channels should be parsed");
        assert_eq!(channels[0]["name"], "抖音");
        assert_eq!(channels[0]["ctr_recent"], 2.4);
    }

    #[test]
    fn ad_fatigue_detector_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("检测广告疲劳：抖音CTR基准3.2，近期CTR2.4，频次6，预算5000，ROAS3.2"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_ad_fatigue_detector");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_data_funnel_analysis_from_metric_task() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "analyze conversion funnel: impressions 10000 clicks 500 cart 120 orders 60 payments 54",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_funnel_analysis");
        assert_eq!(requests[0].input["impressions"], 10000);
        assert_eq!(requests[0].input["clicks"], 500);
    }

    #[test]
    fn selected_product_context_feeds_funnel_analysis_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品转化漏斗：曝光10000 点击500 加购120 下单60 付款54",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_funnel_analysis");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["impressions"], 10000);
        assert_eq!(requests[0].input["payments"], 54);
    }

    #[test]
    fn funnel_analysis_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("分析商品转化漏斗：曝光10000 点击500 加购120 下单60 付款54"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_funnel_analysis");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_comprehensive_diagnosis_with_explicit_metrics() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "做一次综合诊断，重点看增长：GMV26万，上期GMV23万，订单2100，UV6.2万，广告费7.2万，退款1.56万，毛利6.8万，目标GMV30万",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_comprehensive_diagnosis");
        assert_eq!(requests[0].input["focus"], "growth");
        assert_eq!(requests[0].input["gmv"], 260000.0);
        assert_eq!(requests[0].input["previous_gmv"], 230000.0);
        assert_eq!(requests[0].input["orders"], 2100.0);
        assert_eq!(requests[0].input["visitors"], 62000.0);
        assert_eq!(requests[0].input["ad_spend"], 72000.0);
        assert_eq!(requests[0].input["refund_amount"], 15600.0);
        assert_eq!(requests[0].input["gross_profit"], 68000.0);
        assert_eq!(requests[0].input["target_gmv"], 300000.0);
    }

    #[test]
    fn selected_product_context_feeds_comprehensive_diagnosis_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "对这个商品做综合业务诊断：GMV26万，上期GMV23万，订单2100，UV6.2万，广告费7.2万，退款1.56万",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_comprehensive_diagnosis");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["gmv"], 260000.0);
        assert_eq!(requests[0].input["orders"], 2100.0);
    }

    #[test]
    fn comprehensive_diagnosis_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做综合业务诊断：GMV26万，上期GMV23万，订单2100，UV6.2万"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_comprehensive_diagnosis");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["gmv"], 260000.0);
    }

    #[test]
    fn autosuggests_more_specific_data_skills_by_intent() {
        let cases = [
            (
                "请做竞品分析，对标市场上同价位咖啡品牌",
                "data_competitor_analysis",
            ),
            (
                "复盘投放渠道ROI和归因表现，最近14天看下",
                "data_channel_roi",
            ),
            (
                "把退款和售后问题做帕累托分析，看看前5大问题",
                "data_refund_decomposition",
            ),
            (
                "打开店铺大盘，看一下最近30天核心经营指标和预警",
                "data_dashboard",
            ),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("data"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn selected_product_context_feeds_competitor_analysis_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "请对这个商品做竞品分析，对标同价位咖啡品牌",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_competitor_analysis");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["our_product"], "五一咖啡组合");
    }

    #[test]
    fn autosuggests_data_competitor_search_chain_when_realtime_requested() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("请做冷萃咖啡竞品分析，联网搜最新竞品价格和公开资料，最多5条来源"),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "data_competitor_analysis");
        assert_eq!(requests[0].input["our_product"], "咖啡饮品");
        assert_eq!(requests[1].skill_id, "search_competitor");
        assert_eq!(requests[1].input["query"], "咖啡饮品");
        assert_eq!(requests[1].input["focus"], "price");
        assert_eq!(requests[1].input["_needs_fresh_data"], true);
        assert_eq!(requests[1].input["max_results"], 5);
        assert!(requests[1].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_data_price_elasticity_search_chain_when_market_price_requested() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品价格弹性：价格90销量560，价格110销量440，成本42，当前售价99，并联网查实时市场定价和竞品价格区间",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "data_price_elasticity");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].skill_id, "search_competitor");
        assert_eq!(requests[1].input["query"], "五一咖啡组合 咖啡饮品");
        assert_eq!(requests[1].input["focus"], "price");
        assert_eq!(requests[1].input["platform"], "抖音");
        assert_eq!(requests[1].input["_needs_fresh_data"], true);
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
    }

    #[test]
    fn autosuggests_data_trend_search_chains_when_external_context_requested() {
        let demand_requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "做咖啡饮品需求预测和备货建议：最近订单120 128 134 145 151 162，并联网查最新市场趋势，最多4条来源",
            ),
            "{}",
        );

        assert_eq!(demand_requests.len(), 2);
        assert_eq!(demand_requests[0].skill_id, "data_demand_forecast");
        assert_eq!(demand_requests[0].input["metric"], "orders");
        assert_eq!(demand_requests[1].skill_id, "search_trends");
        assert_eq!(demand_requests[1].input["days"], 30);
        assert_eq!(demand_requests[1].input["max_results"], 4);
        assert_eq!(demand_requests[1].input["_needs_fresh_data"], true);
        assert!(
            demand_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("需求趋势")
                    && query.contains("备货预测")
                    && query.contains("咖啡"))
        );
        assert!(demand_requests[1].input.get("product_ref").is_none());
        assert!(demand_requests[1].input.get("product_name").is_none());

        let multi_period_requests = autosuggest_skill_requests(
            &role("data"),
            &task("做多周期趋势分析：100 108 115 126 140 152 161 180，并联网查最近14天热卖趋势"),
            "{}",
        );

        assert_eq!(multi_period_requests.len(), 2);
        assert_eq!(multi_period_requests[0].skill_id, "data_multi_period_trend");
        assert_eq!(multi_period_requests[1].skill_id, "search_trends");
        assert_eq!(multi_period_requests[1].input["days"], 14);
    }

    #[test]
    fn data_trend_search_chain_preserves_selected_product_context_only_when_selected() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "预测这个商品需求和补货量：最近订单120 128 134 145 151 162，并联网查最新需求趋势",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "data_demand_forecast");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].skill_id, "search_trends");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(requests[1].input["platform"], "抖音");
        assert!(
            requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合"))
        );
    }

    #[test]
    fn autosuggests_data_benchmark_search_chains_when_requested() {
        let channel_requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "复盘投放渠道ROI：抖音GMV12万，广告费3万，订单600，UV2万，并查行业ROI基准，最多3条来源",
            ),
            "{}",
        );

        assert_eq!(channel_requests.len(), 2);
        assert_eq!(channel_requests[0].skill_id, "data_channel_roi");
        assert_eq!(channel_requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(channel_requests[1].input["metric"], "roi");
        assert_eq!(channel_requests[1].input["platform"], "抖音");
        assert_eq!(channel_requests[1].input["max_results"], 3);
        assert!(channel_requests[1].input.get("product_ref").is_none());

        let refund_requests = autosuggest_skill_requests(
            &role("data"),
            &task("拆解退款：质量问题退款2.4万，物流破损退款9000，并查咖啡饮品行业退款率基准"),
            "{}",
        );

        assert_eq!(refund_requests.len(), 2);
        assert_eq!(refund_requests[0].skill_id, "data_refund_decomposition");
        assert_eq!(refund_requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(refund_requests[1].input["metric"], "refund");
        assert_eq!(refund_requests[1].input["_needs_fresh_data"], true);
    }

    #[test]
    fn autosuggests_data_market_context_search_chain_for_diagnosis() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做综合业务诊断：GMV26万，上期GMV23万，订单2100，并联网查最新市场报告和行业动态"),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "data_comprehensive_diagnosis");
        assert_eq!(requests[0].input["gmv"], 260000.0);
        assert_eq!(requests[1].skill_id, "search_market_info");
        assert_eq!(requests[1].input["topic"], "general");
        assert_eq!(requests[1].input["days"], 60);
        assert_eq!(requests[1].input["_needs_fresh_data"], true);
        assert!(requests[1].input["query"].as_str().is_some_and(
            |query| query.contains("经营诊断")
                && query.contains("市场行情")
                && query.contains("行业报告")
        ));
        assert!(requests[1].input.get("product_ref").is_none());
    }

    #[test]
    fn selected_product_context_feeds_dashboard_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "打开这个商品经营看板：GMV18万，订单1800，UV3.6万，广告费3.6万，目标GMV20万",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_dashboard");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["gmv"], 180000.0);
        assert_eq!(requests[0].input["orders"], 1800.0);
        assert_eq!(requests[0].input["visitors"], 36000.0);
        assert_eq!(requests[0].input["ad_spend"], 36000.0);
        assert_eq!(requests[0].input["target_gmv"], 200000.0);
    }

    #[test]
    fn dashboard_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("打开经营看板：GMV18万，订单1800，UV3.6万，广告费3.6万"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_dashboard");
        assert!(requests[0].input.get("product_ref").is_none());
        assert_eq!(requests[0].input["gmv"], 180000.0);
        assert_eq!(requests[0].input["orders"], 1800.0);
    }

    #[test]
    fn autosuggests_channel_roi_with_explicit_channel_metrics() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "复盘投放渠道ROI：抖音GMV12万，广告费3万，订单600，UV2万；淘宝GMV8万，广告费2.6万，订单420，UV1.6万",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_channel_roi");
        let channels = requests[0].input["channels"]
            .as_array()
            .expect("channels should be parsed");
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0]["name"], "抖音");
        assert_eq!(channels[0]["gmv"], 120000.0);
        assert_eq!(channels[0]["ad_spend"], 30000.0);
        assert_eq!(channels[0]["orders"], 600);
        assert_eq!(channels[0]["uv"], 20000);
        assert_eq!(channels[1]["name"], "淘宝");
        assert_eq!(channels[1]["gmv"], 80000.0);
        assert_eq!(channels[1]["ad_spend"], 26000.0);
    }

    #[test]
    fn autosuggests_attribution_analysis_with_explicit_channel_metrics() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "做渠道归因分析，用时间衰减模型：抖音GMV18万，广告费4.8万，UV5万，转化率3.2；淘宝GMV12万，广告费2.2万，UV4.2万，转化率2.4",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_attribution_analysis");
        assert_eq!(requests[0].input["attribution_model"], "time_decay");
        let channels = requests[0].input["channels"]
            .as_array()
            .expect("channels should be parsed");
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0]["name"], "抖音");
        assert_eq!(channels[0]["gmv"], 180000.0);
        assert_eq!(channels[0]["ad_spend"], 48000.0);
        assert_eq!(channels[0]["uv"], 50000);
        assert_eq!(channels[0]["conversion_rate"], 3.2);
    }

    #[test]
    fn selected_product_context_feeds_channel_roi_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "复盘这个商品渠道ROI：抖音GMV12万，广告费3万，订单600，UV2万",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_channel_roi");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        let channels = requests[0].input["channels"]
            .as_array()
            .expect("channels should be parsed");
        assert_eq!(channels[0]["name"], "抖音");
        assert_eq!(channels[0]["gmv"], 120000.0);
    }

    #[test]
    fn selected_product_context_feeds_attribution_analysis_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "对这个商品做渠道归因分析：抖音GMV18万，广告费4.8万，UV5万；淘宝GMV12万，广告费2.2万，UV4.2万",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_attribution_analysis");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        let channels = requests[0].input["channels"]
            .as_array()
            .expect("channels should be parsed");
        assert_eq!(channels[0]["name"], "抖音");
        assert_eq!(channels[0]["gmv"], 180000.0);
    }

    #[test]
    fn attribution_analysis_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "渠道归因分析：抖音GMV18万，广告费4.8万，UV5万；淘宝GMV12万，广告费2.2万，UV4.2万",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_attribution_analysis");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn channel_roi_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("复盘渠道ROI：抖音GMV12万，广告费3万，订单600，UV2万"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_channel_roi");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_refund_decomposition_with_explicit_refund_items() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("把退款做帕累托分析：质量问题退款2.4万，物流破损退款9000，色差退款4000"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_refund_decomposition");
        let items = requests[0].input["items"]
            .as_array()
            .expect("refund items should be parsed");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["name"], "质量问题");
        assert_eq!(items[0]["refund_amount"], 24000.0);
        assert_eq!(items[1]["name"], "物流破损");
        assert_eq!(items[1]["refund_amount"], 9000.0);
        assert_eq!(items[2]["name"], "色差");
        assert_eq!(items[2]["refund_amount"], 4000.0);
    }

    #[test]
    fn selected_product_context_feeds_refund_decomposition_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "拆解这个商品退款：质量问题退款2.4万，物流破损退款9000",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_refund_decomposition");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        let items = requests[0].input["items"]
            .as_array()
            .expect("refund items should be parsed");
        assert_eq!(items[0]["name"], "质量问题");
        assert_eq!(items[0]["refund_amount"], 24000.0);
    }

    #[test]
    fn refund_decomposition_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("拆解退款：质量问题退款2.4万，物流破损退款9000"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_refund_decomposition");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_ltv_calculator_with_explicit_value_metrics() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("分析客户LTV：CAC35，客单价128，月购买频次1.8，毛利率42，月留存率38"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_ltv_calculator");
        assert_eq!(requests[0].input["cac"], 35.0);
        assert_eq!(requests[0].input["avg_order_value"], 128.0);
        assert_eq!(requests[0].input["purchase_frequency"], 1.8);
        assert_eq!(requests[0].input["gross_margin"], 42.0);
        assert_eq!(requests[0].input["monthly_retention"], 38.0);
    }

    #[test]
    fn selected_product_context_feeds_ltv_calculator_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品客户价值：CAC35，客单价128，月购买频次1.8，毛利率42，月留存率38",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_ltv_calculator");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["cac"], 35.0);
        assert_eq!(requests[0].input["avg_order_value"], 128.0);
    }

    #[test]
    fn ltv_calculator_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("分析客户LTV：CAC35，客单价128，月购买频次1.8，毛利率42，月留存率38"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_ltv_calculator");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_customer_segmentation_with_explicit_rfm_metrics() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做客户分层RFM：客户总数1200，客单价168，人均订单数2.3，平均购买间隔21"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_customer_segmentation");
        assert_eq!(requests[0].input["total_customers"], 1200);
        assert_eq!(requests[0].input["avg_order_value"], 168.0);
        assert_eq!(requests[0].input["avg_orders_per_customer"], 2.3);
        assert_eq!(requests[0].input["recency_days"], 21.0);
    }

    #[test]
    fn selected_product_context_feeds_customer_segmentation_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品的人群分层：客户总数1200，客单价168，人均订单数2.3，平均购买间隔21",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_customer_segmentation");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["total_customers"], 1200);
    }

    #[test]
    fn customer_segmentation_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做客户分层RFM：客户总数1200，客单价168，人均订单数2.3，平均购买间隔21"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_customer_segmentation");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_ab_test_analyzer_with_explicit_experiment_terms() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做A/B测试：对照组访客10000转化320，实验组访客9800转化390，看看能不能推全"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_ab_test_analyzer");
        assert_eq!(requests[0].input["test_name"], "A/B测试");
        assert_eq!(requests[0].input["metric_name"], "转化率");
        assert_eq!(requests[0].input["control_visitors"], 10000);
        assert_eq!(requests[0].input["control_conversions"], 320);
        assert_eq!(requests[0].input["test_visitors"], 9800);
        assert_eq!(requests[0].input["test_conversions"], 390);
    }

    #[test]
    fn selected_product_context_feeds_ab_test_analyzer_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品详情页首屏A/B测试：对照组访客10000转化320，实验组访客9800转化390",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_ab_test_analyzer");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["test_name"], "详情页首屏A/B测试");
        assert_eq!(requests[0].input["control_visitors"], 10000);
        assert_eq!(requests[0].input["test_conversions"], 390);
    }

    #[test]
    fn ab_test_analyzer_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做A/B测试：对照组访客10000转化320，实验组访客9800转化390"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_ab_test_analyzer");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_cohort_analysis_with_explicit_retention_terms() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做月度队列留存分析：首期新客180，看看第1月后的流失节点"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_cohort_analysis");
        assert_eq!(requests[0].input["cohort_unit"], "month");
        assert_eq!(requests[0].input["new_customers"], 180);
    }

    #[test]
    fn selected_product_context_feeds_cohort_analysis_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品的新客留存和复购留存：首期新客180",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_cohort_analysis");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["new_customers"], 180);
    }

    #[test]
    fn cohort_analysis_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做周度队列留存分析：首期新客180"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_cohort_analysis");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_anomaly_diagnosis_with_explicit_series() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("诊断GMV异常波动：12000 12100 11950 12200 18800"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_anomaly_diagnosis");
        assert_eq!(requests[0].input["metric_name"], "gmv");
        let values = requests[0].input["values"]
            .as_array()
            .expect("values should be parsed");
        assert_eq!(values.len(), 5);
        assert_eq!(values[0], 12000.0);
        assert_eq!(values[4], 18800.0);
    }

    #[test]
    fn selected_product_context_feeds_anomaly_diagnosis_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "诊断这个商品订单异常下滑：120 121 119 122 88",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_anomaly_diagnosis");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["metric_name"], "orders");
    }

    #[test]
    fn anomaly_diagnosis_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("诊断GMV异常波动：12000 12100 11950 12200 18800"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_anomaly_diagnosis");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_price_elasticity_with_explicit_price_points() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做价格弹性分析：价格90销量560，价格110销量440，成本42，当前售价99"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_price_elasticity");
        let points = requests[0].input["price_points"]
            .as_array()
            .expect("price points should be parsed");
        assert_eq!(points.len(), 2);
        assert_eq!(points[0]["price"], 90.0);
        assert_eq!(points[0]["quantity"], 560.0);
        assert_eq!(points[1]["price"], 110.0);
        assert_eq!(points[1]["quantity"], 440.0);
        assert_eq!(requests[0].input["cost"], 42.0);
        assert_eq!(requests[0].input["current_price"], 99.0);
    }

    #[test]
    fn selected_product_context_feeds_price_elasticity_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品价格弹性：价格90销量560，价格110销量440，成本42，当前售价99",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_price_elasticity");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        let points = requests[0].input["price_points"]
            .as_array()
            .expect("price points should be parsed");
        assert_eq!(points[0]["price"], 90.0);
        assert_eq!(points[0]["quantity"], 560.0);
    }

    #[test]
    fn price_elasticity_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做价格弹性分析：价格90销量560，价格110销量440"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_price_elasticity");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_multi_period_trend_with_explicit_series() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做GMV多周期趋势和移动平均分析：100 108 115 126 140 152 161 180"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_multi_period_trend");
        assert_eq!(requests[0].input["metric"], "gmv");
        let periods = requests[0].input["periods"]
            .as_array()
            .expect("periods should be parsed");
        assert_eq!(periods, &[7, 14, 30]);
        let values = requests[0].input["values"]
            .as_array()
            .expect("values should be parsed");
        assert_eq!(values.len(), 8);
        assert_eq!(values[0], 100.0);
        assert_eq!(values[7], 180.0);
    }

    #[test]
    fn selected_product_context_feeds_multi_period_trend_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品的订单多周期环比和趋势动能：100 108 115 126 140 152 161 180",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_multi_period_trend");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["metric"], "orders");
    }

    #[test]
    fn multi_period_trend_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做多周期趋势分析：100 108 115 126 140 152 161 180"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_multi_period_trend");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_trend_forecast_with_explicit_series() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("预测GMV未来3天趋势：100 112 124 141"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_trend_forecast");
        assert_eq!(requests[0].input["metric_name"], "gmv");
        assert_eq!(requests[0].input["forecast_periods"], 3);
        let values = requests[0].input["values"]
            .as_array()
            .expect("values should be parsed");
        assert_eq!(values.len(), 5);
        assert_eq!(values[0], 3.0);
        assert_eq!(values[1], 100.0);
        assert_eq!(values[4], 141.0);
    }

    #[test]
    fn selected_product_context_feeds_trend_forecast_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "预测这个商品订单未来3天趋势：100 112 124 141",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_trend_forecast");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["metric_name"], "orders");
        assert_eq!(requests[0].input["forecast_periods"], 3);
    }

    #[test]
    fn trend_forecast_without_selection_has_no_empty_product_ref() {
        let requests =
            autosuggest_skill_requests(&role("data"), &task("预测GMV趋势：100 112 124 141"), "{}");

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_trend_forecast");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_demand_forecast_with_explicit_series() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做需求预测和备货建议：最近订单120 128 134 145 151 162"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_demand_forecast");
        assert_eq!(requests[0].input["metric"], "orders");
        let values = requests[0].input["values"]
            .as_array()
            .expect("values should be parsed");
        assert_eq!(values.len(), 6);
        assert_eq!(values[0], 120.0);
        assert_eq!(values[5], 162.0);
    }

    #[test]
    fn selected_product_context_feeds_demand_forecast_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "预测这个商品需求和补货量：最近订单120 128 134 145 151 162",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_demand_forecast");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["metric"], "orders");
        let values = requests[0].input["values"]
            .as_array()
            .expect("values should be parsed");
        assert_eq!(values[0], 120.0);
    }

    #[test]
    fn demand_forecast_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做需求预测：最近订单120 128 134 145 151 162"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_demand_forecast");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_seasonal_decompose_with_explicit_series() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "做订单季节性分解，分析周内波动：120 132 141 150 168 181 190 128 139 149 158 176 188 201",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_seasonal_decompose");
        assert_eq!(requests[0].input["metric"], "orders");
        assert_eq!(requests[0].input["period"], 7);
        let values = requests[0].input["values"]
            .as_array()
            .expect("values should be parsed");
        assert_eq!(values.len(), 14);
        assert_eq!(values[0], 120.0);
        assert_eq!(values[13], 201.0);
    }

    #[test]
    fn selected_product_context_feeds_seasonal_decompose_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "分析这个商品的订单季节性和周期波动：120 132 141 150 168 181 190 128 139 149 158 176 188 201",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_seasonal_decompose");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["metric"], "orders");
    }

    #[test]
    fn seasonal_decompose_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("做订单季节性分解：120 132 141 150 168 181 190 128 139 149 158 176 188 201"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_seasonal_decompose");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_query_store_metrics_from_explicit_metric_query() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "查询店铺指标：GMV18万，订单1800，UV3.6万，点击7200，广告费3.6万，退款5400，毛利5.4万",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "query_store_metrics");
        assert_eq!(requests[0].input["gmv"], 180000.0);
        assert_eq!(requests[0].input["orders"], 1800.0);
        assert_eq!(requests[0].input["visitors"], 36000.0);
        assert_eq!(requests[0].input["clicks"], 7200.0);
        assert_eq!(requests[0].input["ad_spend"], 36000.0);
        assert_eq!(requests[0].input["refund_amount"], 5400.0);
        assert_eq!(requests[0].input["gross_profit"], 54000.0);
        assert_eq!(requests[0].input["platform"], "all");
    }

    #[test]
    fn autosuggests_query_store_metrics_for_metric_loading_without_values() {
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task("先加载店铺指标作为这次运营复盘的数据基线"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "query_store_metrics");
        assert_eq!(requests[0].input["platform"], "all");
        assert!(requests[0].input.get("gmv").is_none());
    }

    #[test]
    fn autosuggests_query_store_metrics_keeps_explicit_platform() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("查询抖音店铺指标：GMV12万，订单1000，UV2万"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "query_store_metrics");
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[0].input["gmv"], 120000.0);
        assert_eq!(requests[0].input["orders"], 1000.0);
        assert_eq!(requests[0].input["visitors"], 20000.0);
    }

    #[test]
    fn autosuggest_query_store_metrics_does_not_override_dashboard_intent() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("打开店铺大盘，看一下最近30天核心经营指标和预警，GMV18万"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "data_dashboard");
        assert_eq!(requests[0].input["gmv"], 180000.0);
    }

    #[test]
    fn autosuggests_product_and_campaign_metrics_by_intent() {
        let product_requests = autosuggest_skill_requests(
            &role("data"),
            &task(
                "查询商品指标：冷萃咖啡 GMV9.6万，订单1200，UV2.4万，广告费2.4万，库存8，缺货次数1",
            ),
            "{}",
        );

        assert_eq!(product_requests.len(), 1);
        assert_eq!(product_requests[0].skill_id, "query_product_metrics");
        assert_eq!(product_requests[0].input["gmv"], 96000.0);
        assert_eq!(product_requests[0].input["orders"], 1200.0);
        assert_eq!(product_requests[0].input["visitors"], 24000.0);
        assert_eq!(product_requests[0].input["ad_spend"], 24000.0);
        assert_eq!(product_requests[0].input["inventory"], 8.0);
        assert_eq!(product_requests[0].input["stockout_count"], 1.0);

        let campaign_requests = autosuggest_skill_requests(
            &role("ops"),
            &task("复盘618活动数据：预算5万，活动花费4.2万，GMV21万，订单1800，曝光18万，点击7200"),
            "{}",
        );

        assert_eq!(campaign_requests.len(), 1);
        assert_eq!(campaign_requests[0].skill_id, "query_campaign_metrics");
        assert_eq!(campaign_requests[0].input["budget"], 50000.0);
        assert_eq!(campaign_requests[0].input["spend"], 42000.0);
        assert_eq!(campaign_requests[0].input["gmv"], 210000.0);
        assert_eq!(campaign_requests[0].input["orders"], 1800.0);
        assert_eq!(campaign_requests[0].input["impressions"], 180000.0);
        assert_eq!(campaign_requests[0].input["clicks"], 7200.0);
    }

    #[test]
    fn autosuggests_search_skills_from_external_search_intent() {
        let cases = [
            (
                "联网搜一下咖啡饮品最近30天抖音消费趋势和爆款",
                "data",
                "search_trends",
                true,
            ),
            (
                "搜冷萃咖啡竞品公开资料，重点看价格",
                "web",
                "search_competitor",
                true,
            ),
            (
                "查抖音最新平台规则和内容违规政策",
                "service",
                "search_platform_policy",
                true,
            ),
            (
                "搜冷萃咖啡用户差评和真实口碑",
                "service",
                "search_product_reviews",
                true,
            ),
            (
                "查咖啡饮品行业基准数据，重点看ROI",
                "data",
                "search_industry_benchmarks",
                false,
            ),
            (
                "搜索咖啡行业最新市场报告和公开来源",
                "ops",
                "search_market_info",
                true,
            ),
        ];

        for (prompt, runtime_role, expected_skill, expected_fresh) in cases {
            let requests = autosuggest_skill_requests(&role(runtime_role), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
            assert_eq!(
                requests[0].input["_needs_fresh_data"], expected_fresh,
                "{prompt}"
            );
        }
    }

    #[test]
    fn autosuggest_search_extracts_natural_language_max_results() {
        let cases = [
            (
                "联网搜一下咖啡饮品最近30天抖音消费趋势和爆款，最多3条来源",
                "data",
                "search_trends",
                3,
            ),
            (
                "搜冷萃咖啡竞品公开资料，top 4 sources，重点看价格",
                "web",
                "search_competitor",
                4,
            ),
            (
                "查抖音最新平台规则和内容违规政策，返回20条结果",
                "service",
                "search_platform_policy",
                12,
            ),
        ];

        for (prompt, runtime_role, expected_skill, expected_max_results) in cases {
            let requests = autosuggest_skill_requests(&role(runtime_role), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
            assert_eq!(
                requests[0].input["max_results"], expected_max_results,
                "{prompt}"
            );
        }
    }

    #[test]
    fn autosuggest_search_does_not_treat_period_as_max_results() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("联网搜一下咖啡饮品最近30天抖音消费趋势和爆款"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "search_trends");
        assert_eq!(requests[0].input["days"], 30);
        assert!(requests[0].input.get("max_results").is_none());
    }

    #[test]
    fn selected_product_context_feeds_search_queries() {
        let summary = selected_product_summary();
        let cases = [
            (
                "搜这个商品竞品公开资料，重点看价格",
                "web",
                "search_competitor",
                "query",
                "五一咖啡组合 咖啡饮品",
            ),
            (
                "搜这个商品用户差评和真实口碑",
                "service",
                "search_product_reviews",
                "query",
                "五一咖啡组合 咖啡饮品",
            ),
            (
                "联网查这个商品最近消费趋势和爆款",
                "data",
                "search_trends",
                "query",
                "五一咖啡组合 咖啡饮品",
            ),
            (
                "查这个商品所在行业基准数据，重点看ROI",
                "data",
                "search_industry_benchmarks",
                "industry",
                "咖啡饮品",
            ),
        ];

        for (prompt, runtime_role, expected_skill, key, expected_value) in cases {
            let requests = autosuggest_skill_requests(
                &role(runtime_role),
                &task_with_metadata(prompt, &[("context.selected_summary", &summary)]),
                "{}",
            );
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
            assert_eq!(requests[0].input[key], expected_value, "{prompt}");
            assert_eq!(
                requests[0].input["product_ref"], "product:11111111-1111-1111-1111-111111111111",
                "{prompt}"
            );
            assert_eq!(
                requests[0].input["product_name"], "五一咖啡组合",
                "{prompt}"
            );
            assert_eq!(
                requests[0].input["product_category"], "咖啡饮品",
                "{prompt}"
            );
            if requests[0].input.get("platform").is_some() {
                assert_eq!(requests[0].input["platform"], "抖音", "{prompt}");
            }
        }
    }

    #[test]
    fn search_without_selection_has_no_empty_product_context() {
        let cases = [
            (
                "搜冷萃咖啡竞品公开资料，重点看价格",
                "web",
                "search_competitor",
            ),
            (
                "搜冷萃咖啡用户差评和真实口碑",
                "service",
                "search_product_reviews",
            ),
            ("联网搜咖啡饮品最近消费趋势和爆款", "data", "search_trends"),
        ];

        for (prompt, runtime_role, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role(runtime_role), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
            assert!(requests[0].input.get("product_ref").is_none(), "{prompt}");
            assert!(requests[0].input.get("product_name").is_none(), "{prompt}");
            assert!(
                requests[0].input.get("product_category").is_none(),
                "{prompt}"
            );
        }
    }

    #[test]
    fn autosuggest_search_preserves_native_analysis_without_external_intent() {
        let cases = [
            (
                "请做竞品分析，对标市场上同价位咖啡品牌",
                "data",
                "data_competitor_analysis",
            ),
            ("预测咖啡GMV未来7天趋势", "data", "data_trend_forecast"),
            (
                "给冷萃咖啡做关键词研究和长尾词矩阵",
                "web",
                "web_keyword_research",
            ),
            (
                "分析咖啡饮品在抖音的内容趋势",
                "creative",
                "creative_trend_catch",
            ),
        ];

        for (prompt, runtime_role, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role(runtime_role), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn autosuggests_accounting_compliance_by_intent() {
        let requests = autosuggest_skill_requests(
            &role("accounting"),
            &task("帮我看下这个店铺的税务合规、开票率和现金收款占比有没有风险"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "accounting_compliance_check");
    }

    #[test]
    fn autosuggests_accounting_finance_search_chain_when_requested() {
        let requests = autosuggest_skill_requests(
            &role("accounting"),
            &task(
                "做一份财务健康诊断：GMV 500000，广告费50000，订单3000，并联网查最新电商行业毛利率、净利率和广告ROI财务基准，最多4条来源",
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "accounting_financial_narrative");
        assert_eq!(requests[0].input["gmv"], 500000.0);
        assert_eq!(requests[0].input["ad_spend"], 50000.0);
        assert_eq!(requests[0].input["orders"], 3000.0);
        assert_eq!(requests[1].skill_id, "search_market_info");
        assert_eq!(requests[1].input["topic"], "finance");
        assert_eq!(requests[1].input["days"], 60);
        assert_eq!(requests[1].input["max_results"], 4);
        assert_eq!(requests[1].input["_needs_fresh_data"], true);
        assert!(
            requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("毛利率")
                    && query.contains("净利率")
                    && query.contains("广告ROI"))
        );
        assert!(requests[1].input.get("product_ref").is_none());
        assert!(requests[1].input.get("product_name").is_none());
        assert!(requests[1].input.get("product_category").is_none());
    }

    #[test]
    fn accounting_finance_search_chain_keeps_selected_product_explicit() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("accounting"),
            &task_with_metadata(
                "给这个商品做财务健康诊断，并查最新行业财务基准",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "accounting_financial_narrative");
        assert_eq!(requests[1].skill_id, "search_market_info");
        assert_eq!(requests[1].input["topic"], "finance");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert!(requests[1].input["query"].as_str().is_some_and(|query| {
            query.contains("五一咖啡组合")
                && query.contains("咖啡饮品")
                && query.contains("财务基准")
        }));
    }

    #[test]
    fn accounting_pure_finance_search_does_not_trigger_native_skill() {
        let requests = autosuggest_skill_requests(
            &role("accounting"),
            &task("查一下最新电商行业财务基准、毛利率和净利率公开来源，最多3条来源"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "search_market_info");
        assert_eq!(requests[0].input["topic"], "finance");
        assert_eq!(requests[0].input["max_results"], 3);
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_creative_content_skill_by_intent() {
        let cases = [
            ("帮冷萃咖啡写一条抖音短视频脚本", "creative_video_script"),
            ("生成小红书种草笔记文案", "creative_seeding_copy"),
            ("给一个咖啡品牌做品牌IP人设定位", "creative_ip_branding"),
            ("排一周内容日历和选题", "creative_content_calendar"),
            ("分析咖啡饮品在抖音的内容趋势", "creative_trend_catch"),
            ("写一篇冷萃咖啡产品软文", "creative_product_article"),
            ("写直播带货开场话术", "creative_live_script"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("creative"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn autosuggests_creative_trend_search_chain_when_requested() {
        let video_requests = autosuggest_skill_requests(
            &role("creative"),
            &task("帮冷萃咖啡写抖音短视频脚本，联网查最新爆款内容趋势，最多4条来源"),
            "{}",
        );

        assert_eq!(video_requests.len(), 2);
        assert_eq!(video_requests[0].skill_id, "creative_video_script");
        assert_eq!(
            video_requests[0].input["product_name"],
            "帮冷萃咖啡写抖音短视频脚本，联网查最新爆款内容趋势，最多4条来源"
        );
        assert_eq!(video_requests[0].input["duration"], 30);
        assert_eq!(video_requests[1].skill_id, "search_trends");
        assert_eq!(video_requests[1].input["platform"], "抖音");
        assert_eq!(video_requests[1].input["days"], 14);
        assert_eq!(video_requests[1].input["max_results"], 4);
        assert_eq!(video_requests[1].input["_needs_fresh_data"], true);
        assert!(
            video_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("爆款视频")
                    && query.contains("热门内容")
                    && query.contains("冷萃咖啡"))
        );
        assert!(video_requests[1].input.get("product_ref").is_none());
        assert!(video_requests[1].input.get("product_name").is_none());
        assert!(video_requests[1].input.get("product_category").is_none());

        let calendar_requests = autosuggest_skill_requests(
            &role("creative"),
            &task("排一周内容日历和选题，查最新营销节点和热门话题"),
            "{}",
        );

        assert_eq!(calendar_requests.len(), 2);
        assert_eq!(calendar_requests[0].skill_id, "creative_content_calendar");
        assert_eq!(calendar_requests[1].skill_id, "search_market_info");
        assert_eq!(calendar_requests[1].input["topic"], "news");
        assert_eq!(calendar_requests[1].input["days"], 30);
        assert_eq!(calendar_requests[1].input["_needs_fresh_data"], true);
        assert!(
            calendar_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("营销节日")
                    && query.contains("热门话题")
                    && query.contains("内容节点"))
        );
    }

    #[test]
    fn autosuggests_creative_live_and_article_search_chains_when_requested() {
        let live_requests = autosuggest_skill_requests(
            &role("creative"),
            &task("写一套抖音直播带货话术，联网查最新直播间玩法和转化节奏，最多4条来源"),
            "{}",
        );

        assert_eq!(live_requests.len(), 2);
        assert_eq!(live_requests[0].skill_id, "creative_live_script");
        assert_eq!(live_requests[0].input["duration_hours"], 3.0);
        assert_eq!(live_requests[1].skill_id, "search_trends");
        assert_eq!(live_requests[1].input["platform"], "抖音");
        assert_eq!(live_requests[1].input["days"], 14);
        assert_eq!(live_requests[1].input["max_results"], 4);
        assert_eq!(live_requests[1].input["_needs_fresh_data"], true);
        assert!(
            live_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("直播带货")
                    && query.contains("直播间玩法")
                    && query.contains("场控话术"))
        );
        assert!(live_requests[1].input.get("product_ref").is_none());
        assert!(live_requests[1].input.get("product_name").is_none());
        assert!(live_requests[1].input.get("product_category").is_none());

        let article_requests = autosuggest_skill_requests(
            &role("creative"),
            &task("写一篇冷萃咖啡产品软文，查最新公众号爆款文章和热门内容风格"),
            "{}",
        );

        assert_eq!(article_requests.len(), 2);
        assert_eq!(article_requests[0].skill_id, "creative_product_article");
        assert_eq!(article_requests[1].skill_id, "search_trends");
        assert_eq!(article_requests[1].input["_needs_fresh_data"], true);
        assert!(
            article_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("产品软文")
                    && query.contains("爆款文章")
                    && query.contains("冷萃咖啡"))
        );
        assert!(article_requests[1].input.get("product_ref").is_none());
    }

    #[test]
    fn creative_live_and_article_search_preserve_selected_product_context() {
        let summary = selected_product_summary();

        let live_requests = autosuggest_skill_requests(
            &role("creative"),
            &task_with_metadata(
                "给这个商品写抖音直播带货脚本，联网查热门直播间玩法",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(live_requests.len(), 2);
        assert_eq!(live_requests[0].skill_id, "creative_live_script");
        assert_eq!(live_requests[0].input["product_count"], 1);
        assert_eq!(live_requests[1].skill_id, "search_trends");
        assert_eq!(
            live_requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(live_requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(live_requests[1].input["product_category"], "咖啡饮品");
        assert!(
            live_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合") && query.contains("直播带货"))
        );

        let article_requests = autosuggest_skill_requests(
            &role("creative"),
            &task_with_metadata(
                "给这个商品写公众号产品软文，联网查最新爆款文章风格",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(article_requests.len(), 2);
        assert_eq!(article_requests[0].skill_id, "creative_product_article");
        assert_eq!(
            article_requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(article_requests[1].skill_id, "search_trends");
        assert_eq!(
            article_requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(article_requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(article_requests[1].input["product_category"], "咖啡饮品");
    }

    #[test]
    fn creative_live_and_article_remain_native_without_search_intent() {
        let live_requests =
            autosuggest_skill_requests(&role("creative"), &task("写直播带货开场话术"), "{}");

        assert_eq!(live_requests.len(), 1);
        assert_eq!(live_requests[0].skill_id, "creative_live_script");

        let article_requests =
            autosuggest_skill_requests(&role("creative"), &task("写一篇冷萃咖啡产品软文"), "{}");

        assert_eq!(article_requests.len(), 1);
        assert_eq!(article_requests[0].skill_id, "creative_product_article");
    }

    #[test]
    fn creative_search_chain_preserves_selected_product_context_only_when_selected() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("creative"),
            &task_with_metadata(
                "帮这个商品写抖音短视频脚本，联网查最新爆款内容趋势",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "creative_video_script");
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].skill_id, "search_trends");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert!(
            requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合"))
        );
    }

    #[test]
    fn selected_product_context_feeds_creative_skill_input() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("creative"),
            &task_with_metadata(
                "帮我写一条短视频脚本",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "creative_video_script");
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
    }

    #[test]
    fn selected_product_context_feeds_ops_listing_copy_features() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("ops"),
            &task_with_metadata(
                "写上架文案和标题卖点",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_listing_copy");
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        let features = requests[0].input["features"]
            .as_array()
            .expect("features should be an array");
        assert!(
            features
                .iter()
                .any(|value| value == "低糖、便携、适合五一出游")
        );
    }

    #[test]
    fn selected_product_context_feeds_web_description_input() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("web"),
            &task_with_metadata(
                "写商品详情文案和卖点文案",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "web_product_description");
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
    }

    #[test]
    fn selected_product_context_feeds_design_detail_page_input() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("design"),
            &task_with_metadata(
                "规划商品详情页长图设计",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "design_detail_page");
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
    }

    #[test]
    fn selected_product_context_feeds_live_script_product_refs() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("creative"),
            &task_with_metadata(
                "写直播带货开场话术",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "creative_live_script");
        let refs = requests[0].input["product_refs"]
            .as_array()
            .expect("product_refs should be an array");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], "product:11111111-1111-1111-1111-111111111111");
        assert_eq!(requests[0].input["product_count"], 1);
    }

    #[test]
    fn selected_product_context_feeds_service_query_product_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("service"),
            &task_with_metadata(
                "查询商品资料，客服回复客户前确认这个商品信息",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "service_query_product");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
    }

    #[test]
    fn selected_product_context_feeds_product_metrics_ref() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task_with_metadata(
                "查询这个商品指标：GMV9.6万，订单1200，UV2.4万，广告费2.4万，库存8",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "query_product_metrics");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[0].input["gmv"], 96000.0);
        assert_eq!(requests[0].input["orders"], 1200.0);
        assert_eq!(requests[0].input["visitors"], 24000.0);
    }

    #[test]
    fn product_metrics_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("data"),
            &task("查询商品指标：冷萃咖啡 GMV9.6万，订单1200，UV2.4万，广告费2.4万，库存8"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "query_product_metrics");
        assert!(requests[0].input.get("product_ref").is_none());
        assert!(
            requests[0].input["product_name"]
                .as_str()
                .unwrap_or_default()
                .contains("冷萃咖啡")
        );
    }

    #[test]
    fn service_query_product_without_selection_has_no_empty_product_ref() {
        let requests = autosuggest_skill_requests(
            &role("service"),
            &task("查询商品资料，客服回复客户前确认这个商品信息"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "service_query_product");
        assert!(requests[0].input.get("product_ref").is_none());
        assert!(requests[0].input.get("product_name").is_none());
    }

    #[test]
    fn selected_product_context_feeds_service_support_product_refs() {
        let summary = selected_product_summary();
        let cases = [
            (
                "客户投诉物流延迟很生气，生成工单处理话术",
                "service_ticket_handler",
            ),
            ("生成客服常见FAQ话术库", "service_faq_playbook"),
            (
                "客户威胁投诉平台，需要主管升级处理流程",
                "service_escalation_flow",
            ),
            ("客户要求退款退货，帮我处理售后", "service_return_handler"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(
                &role("service"),
                &task_with_metadata(prompt, &[("context.selected_summary", &summary)]),
                "{}",
            );
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
            assert_eq!(
                requests[0].input["product_ref"], "product:11111111-1111-1111-1111-111111111111",
                "{prompt}"
            );
        }
    }

    #[test]
    fn service_support_without_selection_has_no_empty_product_ref() {
        let cases = [
            (
                "客户投诉物流延迟很生气，生成工单处理话术",
                "service_ticket_handler",
            ),
            ("生成客服常见FAQ话术库", "service_faq_playbook"),
            (
                "客户威胁投诉平台，需要主管升级处理流程",
                "service_escalation_flow",
            ),
            ("客户要求退款退货，帮我处理售后", "service_return_handler"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("service"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
            assert!(requests[0].input.get("product_ref").is_none(), "{prompt}");
        }
    }

    #[test]
    fn selected_product_context_is_not_inferred_from_history_summary() {
        let requests = autosuggest_skill_requests(
            &role("creative"),
            &task_with_metadata(
                "帮我写一条短视频脚本",
                &[("conversation.history_summary", "上一次用户分析过千乐咖啡")],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "creative_video_script");
        assert_ne!(requests[0].input["product_name"], "千乐咖啡");
        assert_eq!(requests[0].input["product_name"], "帮我写一条短视频脚本");
        assert!(requests[0].input.get("product_ref").is_none());
    }

    #[test]
    fn autosuggests_web_growth_skill_by_intent() {
        let cases = [
            ("给冷萃咖啡做关键词研究和长尾词矩阵", "web_keyword_research"),
            ("帮这个商品做SEO搜索优化和提排方案", "web_seo_optimize"),
            ("给保温杯生成淘宝标题", "web_title_generator"),
            ("规划一个淘宝旗舰店店铺装修方案", "web_store_design"),
            ("写商品详情文案和卖点文案", "web_product_description"),
            ("分析商品详情页转化率低的问题", "web_page_conversion"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("web"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn autosuggests_web_seo_search_chain_when_requested() {
        let policy_requests = autosuggest_skill_requests(
            &role("web"),
            &task("给冷萃咖啡做SEO优化，联网查最新搜索算法和流量规则，最多4条来源"),
            "{}",
        );

        assert_eq!(policy_requests.len(), 2);
        assert_eq!(policy_requests[0].skill_id, "web_seo_optimize");
        assert_eq!(policy_requests[0].input["platform"], "淘宝");
        assert!(policy_requests[0].input.get("current_rank").is_none());
        assert_eq!(policy_requests[1].skill_id, "search_platform_policy");
        assert_eq!(policy_requests[1].input["platform"], "淘宝");
        assert_eq!(policy_requests[1].input["policy_type"], "algorithm");
        assert_eq!(policy_requests[1].input["days"], 30);
        assert_eq!(policy_requests[1].input["max_results"], 4);
        assert_eq!(policy_requests[1].input["_needs_fresh_data"], true);
        assert!(policy_requests[1].input.get("product_ref").is_none());
        assert!(policy_requests[1].input.get("product_name").is_none());
        assert!(
            policy_requests[1]
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("平台搜索算法"))
        );

        let market_requests = autosuggest_skill_requests(
            &role("web"),
            &task("给冷萃咖啡做SEO优化，联网查最新关键词竞争和搜索量，最多3条来源"),
            "{}",
        );

        assert_eq!(market_requests.len(), 2);
        assert_eq!(market_requests[0].skill_id, "web_seo_optimize");
        assert_eq!(market_requests[1].skill_id, "search_market_info");
        assert_eq!(market_requests[1].input["topic"], "general");
        assert_eq!(market_requests[1].input["days"], 30);
        assert_eq!(market_requests[1].input["max_results"], 3);
        assert_eq!(market_requests[1].input["_needs_fresh_data"], true);
        assert!(market_requests[1].input["query"].as_str().is_some_and(
            |query| query.contains("SEO")
                && query.contains("关键词竞争")
                && query.contains("咖啡饮品")
        ));
    }

    #[test]
    fn autosuggests_web_keyword_and_conversion_search_chains_when_requested() {
        let keyword_requests = autosuggest_skill_requests(
            &role("web"),
            &task("给冷萃咖啡做关键词研究，联网查最新搜索量、CPC和竞争热度，最多5条来源"),
            "{}",
        );

        assert_eq!(keyword_requests.len(), 2);
        assert_eq!(keyword_requests[0].skill_id, "web_keyword_research");
        assert_eq!(keyword_requests[0].input["seed_keyword"], "咖啡饮品");
        assert_eq!(keyword_requests[1].skill_id, "search_market_info");
        assert_eq!(keyword_requests[1].input["max_results"], 5);
        assert_eq!(keyword_requests[1].input["_needs_fresh_data"], true);
        assert!(
            keyword_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("关键词竞争")
                    && query.contains("搜索量")
                    && query.contains("CPC"))
        );

        let conversion_requests = autosuggest_skill_requests(
            &role("web"),
            &task("分析商品详情页转化率低的问题，查最新行业转化率基准"),
            "{}",
        );

        assert_eq!(conversion_requests.len(), 2);
        assert_eq!(conversion_requests[0].skill_id, "web_page_conversion");
        assert_eq!(conversion_requests[0].input["page_type"], "商品详情");
        assert_eq!(
            conversion_requests[1].skill_id,
            "search_industry_benchmarks"
        );
        assert_eq!(conversion_requests[1].input["industry"], "电商消费品");
        assert_eq!(conversion_requests[1].input["metric"], "conversion");
        assert_eq!(conversion_requests[1].input["platform"], "淘宝");
        assert_eq!(conversion_requests[1].input["_needs_fresh_data"], true);
    }

    #[test]
    fn web_search_chain_preserves_selected_product_context_only_when_selected() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("web"),
            &task_with_metadata(
                "帮这个商品做SEO优化，联网查最新关键词竞争和搜索量",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "web_seo_optimize");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[1].skill_id, "search_market_info");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert!(
            requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合"))
        );

        let plain_requests = autosuggest_skill_requests(
            &role("web"),
            &task("给冷萃咖啡做SEO优化，联网查最新关键词竞争和搜索量"),
            "{}",
        );

        assert_eq!(plain_requests.len(), 2);
        assert_eq!(plain_requests[1].skill_id, "search_market_info");
        assert!(plain_requests[1].input.get("product_ref").is_none());
        assert!(plain_requests[1].input.get("product_name").is_none());
        assert!(plain_requests[1].input.get("product_category").is_none());
    }

    #[test]
    fn autosuggests_web_title_and_description_search_chains_when_requested() {
        let title_policy_requests = autosuggest_skill_requests(
            &role("web"),
            &task("给冷萃咖啡生成淘宝标题，联网查最新标题规则、标题规范和违禁词，最多4条来源"),
            "{}",
        );

        assert_eq!(title_policy_requests.len(), 2);
        assert_eq!(title_policy_requests[0].skill_id, "web_title_generator");
        assert_eq!(title_policy_requests[1].skill_id, "search_platform_policy");
        assert_eq!(title_policy_requests[1].input["platform"], "淘宝");
        assert_eq!(title_policy_requests[1].input["policy_type"], "content");
        assert_eq!(title_policy_requests[1].input["max_results"], 4);
        assert_eq!(title_policy_requests[1].input["_needs_fresh_data"], true);

        let title_market_requests = autosuggest_skill_requests(
            &role("web"),
            &task("给冷萃咖啡生成商品标题，联网查最新搜索量、热搜词和高点击趋势，最多3条来源"),
            "{}",
        );

        assert_eq!(title_market_requests.len(), 2);
        assert_eq!(title_market_requests[0].skill_id, "web_title_generator");
        assert_eq!(title_market_requests[1].skill_id, "search_market_info");
        assert_eq!(title_market_requests[1].input["max_results"], 3);
        assert!(
            title_market_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("商品标题")
                    && query.contains("搜索趋势")
                    && query.contains("CTR"))
        );

        let description_reviews = autosuggest_skill_requests(
            &role("web"),
            &task("写冷萃咖啡详情文案，联网查买家痛点、真实评价和购买顾虑"),
            "{}",
        );

        assert_eq!(description_reviews.len(), 2);
        assert_eq!(description_reviews[0].skill_id, "web_product_description");
        assert_eq!(description_reviews[1].skill_id, "search_product_reviews");
        assert_eq!(description_reviews[1].input["focus"], "negative");
        assert_eq!(description_reviews[1].input["platform"], "淘宝");
        assert_eq!(description_reviews[1].input["_needs_fresh_data"], true);

        let description_market = autosuggest_skill_requests(
            &role("web"),
            &task("写冷萃咖啡详情文案，联网查最新详情页趋势和高转化卖点"),
            "{}",
        );

        assert_eq!(description_market.len(), 2);
        assert_eq!(description_market[0].skill_id, "web_product_description");
        assert_eq!(description_market[1].skill_id, "search_market_info");
        assert!(
            description_market[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("详情页文案")
                    && query.contains("转化趋势")
                    && query.contains("买家痛点"))
        );
    }

    #[test]
    fn web_title_and_description_remain_native_without_search_intent() {
        let title_requests =
            autosuggest_skill_requests(&role("web"), &task("给冷萃咖啡生成淘宝标题"), "{}");

        assert_eq!(title_requests.len(), 1);
        assert_eq!(title_requests[0].skill_id, "web_title_generator");

        let description_requests =
            autosuggest_skill_requests(&role("web"), &task("写商品详情文案和卖点文案"), "{}");

        assert_eq!(description_requests.len(), 1);
        assert_eq!(description_requests[0].skill_id, "web_product_description");
    }

    #[test]
    fn web_title_and_description_search_preserve_selected_product_context() {
        let summary = selected_product_summary();
        let title_requests = autosuggest_skill_requests(
            &role("web"),
            &task_with_metadata(
                "给这个商品生成标题，联网查最新搜索量和高点击标题趋势",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(title_requests.len(), 2);
        assert_eq!(title_requests[0].skill_id, "web_title_generator");
        assert_eq!(
            title_requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(title_requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(title_requests[1].skill_id, "search_market_info");
        assert_eq!(
            title_requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(title_requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(title_requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(title_requests[1].input["platform"], "抖音");

        let description_requests = autosuggest_skill_requests(
            &role("web"),
            &task_with_metadata(
                "给这个商品写详情文案，联网查真实评价和买家痛点",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(description_requests.len(), 2);
        assert_eq!(description_requests[0].skill_id, "web_product_description");
        assert_eq!(
            description_requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(
            description_requests[1].input["product_name"],
            "五一咖啡组合"
        );
        assert_eq!(
            description_requests[1].input["product_category"],
            "咖啡饮品"
        );
        assert_eq!(description_requests[1].skill_id, "search_product_reviews");
    }

    #[test]
    fn autosuggests_design_skill_by_intent() {
        let cases = [
            ("给新品做主图设计方案", "design_main_image"),
            ("规划商品详情页长图设计", "design_detail_page"),
            ("做品牌配色方案，想要高端质感", "design_color_scheme"),
            ("做618活动海报banner", "design_campaign_poster"),
            ("整理抖音信息流广告素材尺寸规格", "design_material_spec"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("design"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn autosuggests_design_visual_trend_search_chain_when_requested() {
        let main_image_requests = autosuggest_skill_requests(
            &role("design"),
            &task("给冷萃咖啡做主图设计方案，联网查最新爆款主图和高点击率视觉趋势，最多4条来源"),
            "{}",
        );

        assert_eq!(main_image_requests.len(), 2);
        assert_eq!(main_image_requests[0].skill_id, "design_main_image");
        assert_eq!(main_image_requests[0].input["platform"], "淘宝");
        assert_eq!(main_image_requests[1].skill_id, "search_trends");
        assert_eq!(main_image_requests[1].input["platform"], "淘宝");
        assert_eq!(main_image_requests[1].input["days"], 60);
        assert_eq!(main_image_requests[1].input["max_results"], 4);
        assert_eq!(main_image_requests[1].input["_needs_fresh_data"], true);
        assert!(
            main_image_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("主图设计")
                    && query.contains("高点击率")
                    && query.contains("视觉趋势"))
        );
        assert!(main_image_requests[1].input.get("product_ref").is_none());
        assert!(main_image_requests[1].input.get("product_name").is_none());
        assert!(
            main_image_requests[1]
                .input
                .get("product_category")
                .is_none()
        );

        let poster_requests = autosuggest_skill_requests(
            &role("design"),
            &task("做618活动海报banner，查最新活动海报设计趋势，最多3条来源"),
            "{}",
        );

        assert_eq!(poster_requests.len(), 2);
        assert_eq!(poster_requests[0].skill_id, "design_campaign_poster");
        assert_eq!(poster_requests[1].skill_id, "search_trends");
        assert_eq!(poster_requests[1].input["days"], 30);
        assert_eq!(poster_requests[1].input["max_results"], 3);
        assert!(
            poster_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("618大促")
                    && query.contains("活动海报")
                    && query.contains("设计趋势"))
        );
    }

    #[test]
    fn design_visual_search_chain_preserves_selected_product_context_only_when_selected() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("design"),
            &task_with_metadata(
                "给这个商品做主图设计方案，联网查最新爆款视觉趋势",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "design_main_image");
        assert_eq!(requests[0].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[1].skill_id, "search_trends");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(requests[1].input["platform"], "抖音");
        assert!(
            requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合"))
        );

        let plain_requests = autosuggest_skill_requests(
            &role("design"),
            &task("给冷萃咖啡做主图设计方案，联网查最新爆款视觉趋势"),
            "{}",
        );

        assert_eq!(plain_requests.len(), 2);
        assert_eq!(plain_requests[1].skill_id, "search_trends");
        assert!(plain_requests[1].input.get("product_ref").is_none());
        assert!(plain_requests[1].input.get("product_name").is_none());
        assert!(plain_requests[1].input.get("product_category").is_none());
    }

    #[test]
    fn pure_design_visual_trend_search_does_not_force_native_design() {
        let requests = autosuggest_skill_requests(
            &role("design"),
            &task("查一下淘宝咖啡类目最新爆款主图视觉趋势，最多5条来源"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "search_trends");
        assert_eq!(requests[0].input["platform"], "淘宝");
        assert_eq!(requests[0].input["days"], 60);
        assert_eq!(requests[0].input["max_results"], 5);
        assert!(requests[0].input["query"].as_str().is_some_and(
            |query| query.contains("咖啡饮品")
                && query.contains("主图设计")
                && query.contains("视觉趋势")
        ));
    }

    #[test]
    fn autosuggests_design_color_and_material_search_chains_when_requested() {
        let color_requests = autosuggest_skill_requests(
            &role("design"),
            &task("给咖啡品牌做配色方案，联网查最新流行色和竞品配色趋势，最多4条来源"),
            "{}",
        );

        assert_eq!(color_requests.len(), 2);
        assert_eq!(color_requests[0].skill_id, "design_color_scheme");
        assert_eq!(color_requests[1].skill_id, "search_trends");
        assert_eq!(color_requests[1].input["platform"], "淘宝");
        assert_eq!(color_requests[1].input["days"], 90);
        assert_eq!(color_requests[1].input["max_results"], 4);
        assert_eq!(color_requests[1].input["_needs_fresh_data"], true);
        assert!(
            color_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("咖啡饮品")
                    && query.contains("品牌配色")
                    && query.contains("流行色")
                    && query.contains("竞品配色"))
        );
        assert!(color_requests[1].input.get("product_ref").is_none());
        assert!(color_requests[1].input.get("product_name").is_none());

        let material_requests = autosuggest_skill_requests(
            &role("design"),
            &task("整理抖音信息流广告素材尺寸规格，联网查最新平台上传规范和审核要求，最多3条来源"),
            "{}",
        );

        assert_eq!(material_requests.len(), 2);
        assert_eq!(material_requests[0].skill_id, "design_material_spec");
        assert_eq!(material_requests[1].skill_id, "search_platform_policy");
        assert_eq!(material_requests[1].input["platform"], "抖音");
        assert_eq!(material_requests[1].input["policy_type"], "material");
        assert_eq!(material_requests[1].input["days"], 30);
        assert_eq!(material_requests[1].input["max_results"], 3);
        assert_eq!(material_requests[1].input["_needs_fresh_data"], true);

        let plain_color = autosuggest_skill_requests(
            &role("design"),
            &task("做品牌配色方案，想要高端质感"),
            "{}",
        );
        assert_eq!(plain_color.len(), 1);
        assert_eq!(plain_color[0].skill_id, "design_color_scheme");
    }

    #[test]
    fn design_color_material_search_chains_preserve_selected_product_only_when_selected() {
        let summary = selected_product_summary();
        let color_requests = autosuggest_skill_requests(
            &role("design"),
            &task_with_metadata(
                "给这个商品做配色方案，联网查最新色彩趋势和竞品配色",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(color_requests.len(), 2);
        assert_eq!(color_requests[0].skill_id, "design_color_scheme");
        assert_eq!(
            color_requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(color_requests[1].skill_id, "search_trends");
        assert_eq!(
            color_requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(color_requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(color_requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(color_requests[1].input["platform"], "抖音");
        assert!(
            color_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合"))
        );

        let material_requests = autosuggest_skill_requests(
            &role("design"),
            &task_with_metadata(
                "整理这个商品的小红书素材规格，联网查最新平台尺寸规范",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(material_requests.len(), 2);
        assert_eq!(material_requests[1].skill_id, "search_platform_policy");
        assert_eq!(
            material_requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(material_requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(material_requests[1].input["product_category"], "咖啡饮品");

        let plain_material = autosuggest_skill_requests(
            &role("design"),
            &task("整理抖音广告素材尺寸规格，联网查最新平台上传规范"),
            "{}",
        );
        assert_eq!(plain_material.len(), 2);
        assert_eq!(plain_material[1].skill_id, "search_platform_policy");
        assert!(plain_material[1].input.get("product_ref").is_none());
        assert!(plain_material[1].input.get("product_name").is_none());
        assert!(plain_material[1].input.get("product_category").is_none());
    }

    #[test]
    fn pure_design_material_policy_search_does_not_force_native_design() {
        let requests = autosuggest_skill_requests(
            &role("design"),
            &task("查一下抖音信息流广告最新素材尺寸规范和审核要求，最多5条来源"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "search_platform_policy");
        assert_eq!(requests[0].input["platform"], "抖音");
        assert_eq!(requests[0].input["policy_type"], "material");
        assert_eq!(requests[0].input["days"], 30);
        assert_eq!(requests[0].input["max_results"], 5);
        assert_eq!(requests[0].input["_needs_fresh_data"], true);
    }

    #[test]
    fn autosuggests_service_support_skill_by_intent() {
        let cases = [
            (
                "客户投诉物流延迟很生气，生成工单处理话术",
                "service_ticket_handler",
            ),
            ("生成客服常见FAQ话术库", "service_faq_playbook"),
            (
                "客户威胁投诉平台，需要主管升级处理流程",
                "service_escalation_flow",
            ),
            ("客户要求退款退货，帮我处理售后", "service_return_handler"),
            ("DSR下降，服务评分低，给提升方案", "service_dsr_improvement"),
            ("分析客户NPS满意度和推荐值", "service_nps_analyzer"),
            (
                "分析投诉分类和满意度驱动因素，看看NPS是被什么拖累的",
                "service_nps_driver_analysis",
            ),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("service"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn autosuggests_service_benchmark_chain_for_dsr_and_nps() {
        let dsr_requests = autosuggest_skill_requests(
            &role("service"),
            &task("DSR下降，服务评分低，联网查最新行业基准并给提升方案，最多4条来源"),
            "{}",
        );

        assert_eq!(dsr_requests.len(), 2);
        assert_eq!(dsr_requests[0].skill_id, "service_dsr_improvement");
        assert_eq!(dsr_requests[0].input["platform"], "淘宝");
        assert_eq!(dsr_requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(dsr_requests[1].input["metric"], "dsr");
        assert_eq!(dsr_requests[1].input["platform"], "电商");
        assert_eq!(dsr_requests[1].input["_needs_fresh_data"], true);
        assert_eq!(dsr_requests[1].input["max_results"], 4);

        let nps_requests = autosuggest_skill_requests(
            &role("service"),
            &task("分析投诉分类和满意度驱动因素，顺便联网查抖音NPS行业基准"),
            "{}",
        );

        assert_eq!(nps_requests.len(), 2);
        assert_eq!(nps_requests[0].skill_id, "service_nps_driver_analysis");
        assert_eq!(nps_requests[0].input["platform"], "抖音");
        assert_eq!(nps_requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(nps_requests[1].input["metric"], "nps");
        assert_eq!(nps_requests[1].input["platform"], "抖音");
        assert_eq!(nps_requests[1].input["_needs_fresh_data"], true);
    }

    #[test]
    fn autosuggests_service_review_search_chain_when_reviews_requested() {
        let sentiment_requests = autosuggest_skill_requests(
            &role("service"),
            &task("分析冷萃咖啡差评和投诉情绪，联网搜最近30天真实用户评价，最多5条来源"),
            "{}",
        );

        assert_eq!(sentiment_requests.len(), 2);
        assert_eq!(sentiment_requests[0].skill_id, "service_sentiment_analyzer");
        assert_eq!(sentiment_requests[1].skill_id, "search_product_reviews");
        assert_eq!(sentiment_requests[1].input["focus"], "negative");
        assert_eq!(sentiment_requests[1].input["platform"], "淘宝");
        assert_eq!(sentiment_requests[1].input["days"], 30);
        assert_eq!(sentiment_requests[1].input["max_results"], 5);
        assert_eq!(sentiment_requests[1].input["_needs_fresh_data"], true);
        assert!(sentiment_requests[1].input.get("product_ref").is_none());
        assert!(sentiment_requests[1].input.get("product_name").is_none());

        let return_requests = autosuggest_skill_requests(
            &role("service"),
            &task("客户要求退款退货，帮我处理售后，并搜这个品类真实差评和口碑"),
            "{}",
        );

        assert_eq!(return_requests.len(), 2);
        assert_eq!(return_requests[0].skill_id, "service_return_handler");
        assert_eq!(return_requests[1].skill_id, "search_product_reviews");
        assert_eq!(return_requests[1].input["focus"], "negative");
    }

    #[test]
    fn service_review_search_chain_keeps_selected_product_explicit() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("service"),
            &task_with_metadata(
                "分析这个商品差评和投诉情绪，联网搜真实用户评价",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "service_sentiment_analyzer");
        assert_eq!(requests[1].skill_id, "search_product_reviews");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
        assert_eq!(requests[1].input["platform"], "抖音");
        assert!(
            requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("五一咖啡组合"))
        );
    }

    #[test]
    fn autosuggests_service_platform_policy_search_chain_when_rules_requested() {
        let return_requests = autosuggest_skill_requests(
            &role("service"),
            &task(
                "客户要求退款退货，帮我处理售后，并联网查抖音最新退换货规则和赔付标准，最多4条来源",
            ),
            "{}",
        );

        assert_eq!(return_requests.len(), 2);
        assert_eq!(return_requests[0].skill_id, "service_return_handler");
        assert_eq!(return_requests[1].skill_id, "search_platform_policy");
        assert_eq!(return_requests[1].input["platform"], "抖音");
        assert_eq!(return_requests[1].input["policy_type"], "general");
        assert_eq!(return_requests[1].input["days"], 30);
        assert_eq!(return_requests[1].input["max_results"], 4);
        assert_eq!(return_requests[1].input["_needs_fresh_data"], true);
        assert!(return_requests[1].input.get("product_ref").is_none());
        assert!(return_requests[1].input.get("product_name").is_none());

        let faq_requests = autosuggest_skill_requests(
            &role("service"),
            &task("整理售后FAQ和标准话术，查淘宝最新七天无理由和运费险规则"),
            "{}",
        );

        assert_eq!(faq_requests.len(), 2);
        assert_eq!(faq_requests[0].skill_id, "service_faq_playbook");
        assert_eq!(faq_requests[1].skill_id, "search_platform_policy");
        assert_eq!(faq_requests[1].input["platform"], "淘宝");
        assert_eq!(faq_requests[1].input["policy_type"], "general");
    }

    #[test]
    fn service_platform_policy_search_chain_keeps_selected_product_explicit() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("service"),
            &task_with_metadata(
                "这个商品需要主管升级和平台介入，生成升级处理流程，并查最新平台介入规则和赔付标准",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "service_escalation_flow");
        assert_eq!(
            requests[0].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].skill_id, "search_platform_policy");
        assert_eq!(requests[1].input["platform"], "抖音");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(requests[1].input["product_category"], "咖啡饮品");
    }

    #[test]
    fn service_platform_policy_chain_preserves_native_without_policy_intent() {
        let requests = autosuggest_skill_requests(
            &role("service"),
            &task("客户要求退款退货，帮我处理售后并判断运费承担"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "service_return_handler");
    }

    #[test]
    fn autosuggest_service_benchmark_chain_keeps_selected_product_explicit() {
        let summary = selected_product_summary();
        let requests = autosuggest_skill_requests(
            &role("service"),
            &task_with_metadata(
                "这个商品DSR下降，查最新行业基准并给提升方案",
                &[("context.selected_summary", &summary)],
            ),
            "{}",
        );

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].skill_id, "service_dsr_improvement");
        assert_eq!(requests[1].skill_id, "search_industry_benchmarks");
        assert_eq!(requests[1].input["industry"], "咖啡饮品");
        assert_eq!(requests[1].input["product_name"], "五一咖啡组合");
        assert_eq!(
            requests[1].input["product_ref"],
            "product:11111111-1111-1111-1111-111111111111"
        );
    }

    #[test]
    fn autosuggests_engineering_skill_by_intent() {
        let cases = [
            (
                "评审一下这个电商系统架构的扩展性和重构方向",
                "engineering_arch_review",
            ),
            (
                "最近支付接口频繁报错，帮我排查bug根因",
                "engineering_bug_analysis",
            ),
            (
                "大促时接口延迟很高，想做性能优化，目标QPS 5000",
                "engineering_perf_optimize",
            ),
            (
                "消息队列技术选型怎么做，团队现在主要是Rust和Postgres",
                "engineering_tech_selection",
            ),
            (
                "检查一下抖音平台API连通性和接口健康情况",
                "engineering_system_check",
            ),
            ("看一下这个平台SLA和可用性情况", "engineering_sla_monitor"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("engineering"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
    }

    #[test]
    fn autosuggests_engineering_system_check_with_platform_boundary() {
        let requests = autosuggest_skill_requests(
            &role("engineering"),
            &task("检查一下淘宝平台API连通性和接口健康情况，需要真实测试连接"),
            "{}",
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "engineering_system_check");
        assert_eq!(requests[0].input["platform"], "淘宝");
        assert_eq!(requests[0].input["test_live"], true);
        assert!(
            requests[0]
                .reason
                .as_deref()
                .is_some_and(|reason| reason.contains("需审批"))
        );
    }

    #[test]
    fn autosuggests_engineering_knowledge_search_chain_when_requested() {
        let bug_requests = autosuggest_skill_requests(
            &role("engineering"),
            &task(
                "最近支付接口频繁报错，帮我排查bug根因，并联网查已知解决方案和最佳实践，最多4条来源",
            ),
            "{}",
        );

        assert_eq!(bug_requests.len(), 2);
        assert_eq!(bug_requests[0].skill_id, "engineering_bug_analysis");
        assert_eq!(bug_requests[1].skill_id, "search_market_info");
        assert_eq!(bug_requests[1].input["topic"], "general");
        assert_eq!(bug_requests[1].input["days"], 90);
        assert_eq!(bug_requests[1].input["max_results"], 4);
        assert_eq!(bug_requests[1].input["_needs_fresh_data"], true);
        assert!(
            bug_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("支付失败")
                    && query.contains("解决方案")
                    && query.contains("最佳实践"))
        );

        let tech_requests = autosuggest_skill_requests(
            &role("engineering"),
            &task(
                "消息队列技术选型怎么做，团队现在主要是Rust和Postgres，查最新对比和性能benchmark，最多5条来源",
            ),
            "{}",
        );

        assert_eq!(tech_requests.len(), 2);
        assert_eq!(tech_requests[0].skill_id, "engineering_tech_selection");
        assert_eq!(tech_requests[1].skill_id, "search_market_info");
        assert_eq!(tech_requests[1].input["days"], 90);
        assert_eq!(tech_requests[1].input["max_results"], 5);
        assert!(
            tech_requests[1].input["query"]
                .as_str()
                .is_some_and(|query| query.contains("消息队列")
                    && query.contains("技术选型")
                    && query.contains("benchmark"))
        );
    }

    #[test]
    fn engineering_search_chain_preserves_native_without_external_intent() {
        let cases = [
            "最近支付接口频繁报错，帮我排查bug根因",
            "消息队列技术选型怎么做，团队现在主要是Rust和Postgres",
            "大促时接口延迟很高，想做性能优化，目标QPS 5000",
        ];

        for prompt in cases {
            let requests = autosuggest_skill_requests(&role("engineering"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert!(requests[0].skill_id.starts_with("engineering_"), "{prompt}");
        }
    }

    #[test]
    fn parses_structured_role_work_output() {
        let parsed = parse_worker_role_output(
            r#"{
              "decision": {"kind": "NoAction"},
              "role_output": {
                "summary": "运营方案需要先拆优先级",
                "findings": ["预算有限", "目标明确"],
                "recommendations": ["P1先做转化链路", "P2补素材", "P3复盘"],
                "evidence": ["assignment.input.data"],
                "risks": ["预算消耗过快"],
                "open_questions": []
              }
            }"#,
        );

        let role_output = parsed.role_output.expect("role output should parse");
        assert_eq!(role_output.summary, "运营方案需要先拆优先级");
        assert_eq!(role_output.recommendations.len(), 3);
        assert_eq!(parsed.evidence, vec!["assignment.input.data"]);
        assert_eq!(parsed.risks, vec!["预算消耗过快"]);
        assert!(parsed.content.contains("P1先做转化链路"));
    }

    #[test]
    fn wraps_legacy_string_role_output_as_summary() {
        let parsed = parse_worker_role_output(
            r#"{
              "decision": {"kind": "NoAction"},
              "role_output": "这是旧格式角色产物",
              "evidence": ["legacy evidence"]
            }"#,
        );

        let role_output = parsed.role_output.expect("legacy role output should parse");
        assert_eq!(role_output.summary, "这是旧格式角色产物");
        assert_eq!(role_output.evidence, vec!["legacy evidence"]);
        assert_eq!(parsed.content, "这是旧格式角色产物");
    }

    #[test]
    fn parses_skill_requests_from_worker_json() {
        let requests = parse_skill_requests(
            r#"{
              "decision": {"kind": "NoAction"},
              "skill_requests": [
                {
                  "skill_id": "accounting_roi_calc",
                  "input": {"investment": 1000, "revenue_generated": 1500},
                  "reason": "需要计算ROI"
                }
              ],
              "role_output": {"summary": "需要财务计算"}
            }"#,
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "accounting_roi_calc");
        assert_eq!(requests[0].input["investment"], 1000);
        assert_eq!(requests[0].reason.as_deref(), Some("需要计算ROI"));
    }

    #[test]
    fn parses_skill_request_arguments_alias_from_llm_json() {
        let requests = parse_skill_requests(
            r#"{
              "decision": {"kind": "NoAction"},
              "skill_requests": [
                {
                  "skill_id": "ops_execution_plan",
                  "arguments": {"goal": "提升五一咖啡转化率", "stage": "五一预热"}
                }
              ],
              "role_output": {"summary": "需要运营计划"}
            }"#,
        );

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].skill_id, "ops_execution_plan");
        assert_eq!(requests[0].input["goal"], "提升五一咖啡转化率");
        assert_eq!(requests[0].input["stage"], "五一预热");
    }

    #[test]
    fn successful_skill_results_become_report_evidence() {
        let parsed = parse_worker_role_output(
            r#"{
              "decision": {"kind": "NoAction"},
              "role_output": {
                "summary": "标题需要评分",
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": [],
                "open_questions": []
              }
            }"#,
        );
        let merged = merge_skill_results(
            parsed,
            &[SkillExecutionResult {
                request_id: "req-1".to_string(),
                skill_id: SkillId::new("web_title_seo_scorer"),
                status: SkillExecutionStatus::Success,
                output: None,
                summary: Some("标题SEO评分已完成".to_string()),
                metadata: serde_json::Map::new(),
                error: None,
            }],
        );

        assert!(
            merged
                .evidence
                .iter()
                .any(|item| item.contains("skill:web_title_seo_scorer status=success"))
        );
        assert!(
            merged
                .role_output
                .expect("role output")
                .evidence
                .iter()
                .any(|item| item.contains("标题SEO评分已完成"))
        );
    }

    #[test]
    fn deferred_skill_results_become_risks_and_questions() {
        let parsed = parse_worker_role_output(
            r#"{
              "decision": {"kind": "NoAction"},
              "role_output": {"summary": "需要搜索验证"}
            }"#,
        );
        let merged = merge_skill_results(
            parsed,
            &[SkillExecutionResult {
                request_id: "req-1".to_string(),
                skill_id: SkillId::new("search_trends"),
                status: SkillExecutionStatus::Deferred,
                output: None,
                summary: Some("搜索技能骨架已返回".to_string()),
                metadata: serde_json::Map::new(),
                error: None,
            }],
        );

        assert!(
            merged
                .risks
                .iter()
                .any(|item| item.contains("skill:search_trends status=Deferred"))
        );
        assert!(
            merged
                .open_questions
                .iter()
                .any(|item| item.contains("尚未产出可用结果"))
        );
    }
}
