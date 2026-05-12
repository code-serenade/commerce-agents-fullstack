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

fn autosuggest_ops(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
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
        let numbers = numbers_in_text(text);
        return Some(autosuggest_request(
            "ops_channel_strategy",
            serde_json::json!({
                "product_type": infer_category(text),
                "budget": numbers.first().copied().unwrap_or(10_000.0).max(0.0),
                "target_audience": infer_audience(text),
            }),
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
                "product_name": compact_task_text(&task.description),
                "features": infer_feature_list(text),
                "platform": infer_platform(text),
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
        let numbers = numbers_in_text(text);
        return Some(autosuggest_request(
            "ops_assortment_planning",
            serde_json::json!({
                "category": infer_category(text),
                "budget": numbers.first().copied().unwrap_or(50_000.0).max(0.0),
            }),
            "根据选品组合/货盘规划意图自动补偿选品规划 skill",
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
                serde_json::json!({
                    "cost": cost,
                    "competitor_prices": numbers.iter().copied().skip(1).collect::<Vec<_>>(),
                    "platform": infer_platform(text),
                }),
                "根据运营定价意图自动补偿定价策略 skill",
            ));
        }
    }

    if contains_any(text, &["inventory", "stock", "库存", "补货", "备货"]) {
        let numbers = numbers_in_text(text);
        if numbers.len() >= 2 {
            return Some(autosuggest_request(
                "ops_inventory_planning",
                serde_json::json!({
                    "avg_daily_demand": numbers[0].max(0.0),
                    "lead_time_days": numbers[1].round().max(1.0) as i64,
                    "platform": infer_platform(text),
                }),
                "根据库存/补货意图自动补偿库存规划 skill",
            ));
        }
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
        if let Some(budget) = numbers.first().copied().filter(|value| *value >= 100.0) {
            return Some(autosuggest_request(
                "ops_promo_planning",
                serde_json::json!({
                    "product": compact_task_text(&task.description),
                    "budget": budget,
                    "platform": infer_platform(text),
                }),
                "根据活动/促销意图自动补偿促销策划 skill",
            ));
        }
    }

    Some(autosuggest_request(
        "ops_execution_plan",
        serde_json::json!({
            "goal": compact_task_text(&task.description),
            "stage": infer_stage(text),
        }),
        "根据运营岗位任务自动补偿执行计划 skill",
    ))
}

fn autosuggest_data(text: &str, _task: &Task) -> Option<ParsedSkillRequest> {
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
        return Some(autosuggest_request(
            "data_competitor_analysis",
            serde_json::json!({
                "our_product": infer_product_name(text),
                "our_price": numbers.first().copied().unwrap_or(99.0),
            }),
            "根据竞品/对标意图自动补偿竞品分析 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "channel roi",
            "渠道roi",
            "渠道 roi",
            "投放渠道",
            "归因",
            "渠道投产",
        ],
    ) {
        return Some(autosuggest_request(
            "data_channel_roi",
            serde_json::json!({
                "days": infer_period_days(text),
            }),
            "根据渠道投产/归因意图自动补偿渠道ROI skill",
        ));
    }

    if contains_any(
        text,
        &["refund", "退款", "售后占比", "退货率", "帕累托", "退款分解"],
    ) {
        return Some(autosuggest_request(
            "data_refund_decomposition",
            serde_json::json!({
                "days": infer_period_days(text),
                "top_n": 5,
            }),
            "根据退款分解/帕累托意图自动补偿退款分析 skill",
        ));
    }

    if contains_any(text, &["anomaly", "异常", "波动", "下滑", "暴涨", "暴跌"]) {
        let values = numbers_in_text(text);
        return Some(autosuggest_request(
            "data_anomaly_diagnosis",
            serde_json::json!({
                "metric_name": infer_metric(text),
                "values": values,
            }),
            "根据异常诊断意图自动补偿数据异常 skill",
        ));
    }

    if contains_any(text, &["trend", "forecast", "预测", "趋势"]) {
        let values = numbers_in_text(text);
        return Some(autosuggest_request(
            "data_trend_forecast",
            serde_json::json!({
                "metric_name": infer_metric(text),
                "values": values,
                "forecast_periods": 7,
            }),
            "根据趋势/预测意图自动补偿趋势预测 skill",
        ));
    }

    if contains_any(
        text,
        &["customer", "segment", "rfm", "客户", "人群", "分层"],
    ) {
        return Some(autosuggest_request(
            "data_customer_segmentation",
            serde_json::json!({}),
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
        serde_json::json!(infer_platform(text)),
    );
    Some(autosuggest_request(
        "data_funnel_analysis",
        serde_json::Value::Object(input),
        "根据数据/转化意图自动补偿漏斗分析 skill",
    ))
}

fn autosuggest_accounting(text: &str, _task: &Task) -> Option<ParsedSkillRequest> {
    let numbers = numbers_in_text(text);
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

    Some(autosuggest_request(
        "accounting_budget_plan",
        serde_json::json!({}),
        "根据财务岗位任务自动补偿预算编制 skill",
    ))
}

fn autosuggest_service(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
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
            }),
            "根据FAQ/标准话术意图自动补偿FAQ话术库 skill",
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

fn autosuggest_creative(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["live", "直播"]) {
        return Some(autosuggest_request(
            "creative_live_script",
            serde_json::json!({
                "duration_hours": numbers_in_text(text).first().copied().unwrap_or(3.0).clamp(0.5, 12.0),
                "product_count": 6,
                "live_type": if contains_any(text, &["new", "新品"]) { "新品首发" } else { "日常带货" },
                "platform": infer_platform(text),
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
                "product_name": compact_task_text(&task.description),
                "video_type": infer_video_type(text),
                "duration": numbers_in_text(text).first().copied().unwrap_or(30.0).round().clamp(10.0, 180.0) as i64,
                "platform": infer_platform(text),
            }),
            "根据短视频/脚本意图自动补偿短视频脚本 skill",
        ));
    }

    if contains_any(text, &["seeding", "种草", "笔记", "小红书", "分享文案"]) {
        return Some(autosuggest_request(
            "creative_seeding_copy",
            serde_json::json!({
                "product_name": compact_task_text(&task.description),
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
                "brand_name": compact_task_text(&task.description),
                "category": infer_category(text),
                "target_audience": infer_audience(text),
                "differentiator": "专业+有温度",
                "platform": infer_platform(text),
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

    if contains_any(text, &["trend", "hot", "趋势", "热点", "爆款", "流行"]) {
        return Some(autosuggest_request(
            "creative_trend_catch",
            serde_json::json!({
                "category": infer_category(text),
                "platform": infer_platform(text),
            }),
            "根据趋势/热点意图自动补偿趋势捕捉 skill",
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
                "product_name": compact_task_text(&task.description),
                "article_type": if contains_any(text, &["对比"]) { "对比评测" } else if contains_any(text, &["攻略"]) { "使用攻略" } else { "测评软文" },
                "platform": if text.contains("小红书") { "小红书" } else { "微信公众号" },
            }),
            "根据软文/推广文章意图自动补偿产品软文 skill",
        ));
    }

    Some(autosuggest_request(
        "creative_title_ctr_scorer",
        serde_json::json!({
            "title": extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description)),
            "platform": infer_platform(text),
        }),
        "根据创意岗位任务自动补偿标题 CTR 评分 skill",
    ))
}

fn autosuggest_web(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["keyword", "关键词", "搜索词", "长尾词", "蓝海词"]) {
        return Some(autosuggest_request(
            "web_keyword_research",
            serde_json::json!({
                "seed_keyword": extract_keyword_candidates(text).first().cloned().unwrap_or_else(|| infer_category(text).to_owned()),
                "category": infer_category(text),
                "platform": infer_platform(text),
            }),
            "根据关键词/SEO研究意图自动补偿关键词矩阵 skill",
        ));
    }

    if contains_any(text, &["seo", "SEO", "搜索优化", "排名", "提排"]) {
        return Some(autosuggest_request(
            "web_seo_optimize",
            serde_json::json!({
                "target_keywords": extract_keyword_candidates(text),
                "current_rank": numbers_in_text(text).first().copied().map(|value| value.round() as i64),
                "platform": infer_platform(text),
            }),
            "根据 SEO/提排意图自动补偿 SEO 优化 skill",
        ));
    }

    if contains_any(text, &["标题", "title", "标题生成", "起标题"]) {
        return Some(autosuggest_request(
            "web_title_generator",
            serde_json::json!({
                "product_name": compact_task_text(&task.description),
                "keywords": extract_keyword_candidates(text),
                "platform": infer_platform(text),
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
                "category": infer_category(text),
                "platform": infer_platform(text),
                "brand_tone": if contains_any(text, &["高端"]) { "高端奢华" } else if contains_any(text, &["年轻"]) { "年轻活力" } else { "专业可信" },
            }),
            "根据店铺装修意图自动补偿店铺设计 skill",
        ));
    }

    if contains_any(text, &["详情文案", "产品文案", "卖点文案", "description"]) {
        return Some(autosuggest_request(
            "web_product_description",
            serde_json::json!({
                "product_name": compact_task_text(&task.description),
                "platform": infer_platform(text),
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
                "platform": infer_platform(text),
            }),
            "根据页面转化意图自动补偿页面转化 skill",
        ));
    }

    Some(autosuggest_request(
        "web_title_seo_scorer",
        serde_json::json!({
            "title": extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description)),
            "target_keywords": extract_keyword_candidates(text),
            "platform": infer_platform(text),
        }),
        "根据 Web/SEO 岗位任务自动补偿标题 SEO 评分 skill",
    ))
}

fn autosuggest_design(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["material", "asset", "素材", "规格", "尺寸"]) {
        return Some(autosuggest_request(
            "design_material_spec",
            serde_json::json!({
                "usage": infer_design_usage(text),
                "platforms": [infer_platform(text)],
            }),
            "根据素材/规格意图自动补偿素材规范 skill",
        ));
    }

    if contains_any(text, &["详情页", "详情", "长图"]) {
        return Some(autosuggest_request(
            "design_detail_page",
            serde_json::json!({
                "product_name": compact_task_text(&task.description),
                "platform": infer_platform(text),
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
                "category": infer_category(text),
                "platform": infer_platform(text),
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
                "platform": infer_platform(text),
                "style": infer_design_style(text),
            }),
            "根据活动海报意图自动补偿海报设计 skill",
        ));
    }

    Some(autosuggest_request(
        "design_main_image",
        serde_json::json!({
            "product_name": compact_task_text(&task.description),
            "platform": infer_platform(text),
        }),
        "根据设计岗位任务自动补偿主图设计 skill",
    ))
}

fn autosuggest_engineering(text: &str, _task: &Task) -> Option<ParsedSkillRequest> {
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
    ) {
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

    if contains_any(
        text,
        &[
            "tech selection",
            "技术选型",
            "框架选择",
            "数据库选型",
            "选型",
        ],
    ) {
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

fn autosuggest_request(
    skill_id: impl Into<String>,
    input: serde_json::Value,
    reason: impl Into<String>,
) -> ParsedSkillRequest {
    ParsedSkillRequest {
        skill_id: skill_id.into(),
        input,
        reason: Some(reason.into()),
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

fn infer_period_days(text: &str) -> i64 {
    let numbers = numbers_in_text(text);
    numbers
        .iter()
        .find(|value| **value >= 1.0 && **value <= 365.0)
        .map(|value| value.round() as i64)
        .unwrap_or(30)
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

fn infer_tech_domain(text: &str) -> &'static str {
    if contains_any(text, &["database", "数据库", "mysql", "postgres"]) {
        "数据库"
    } else if contains_any(text, &["queue", "mq", "消息队列"]) {
        "消息队列"
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

fn extend_context_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    if let Some(summary) = metadata
        .get("context.summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        facts.push(Fact {
            source: "commerce.product_context".to_string(),
            content: summary.to_string(),
            reliability: 1.0,
        });
    }
    if let Some(sources) = metadata
        .get("context.sources_json")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        facts.push(Fact {
            source: "commerce.product_context.sources".to_string(),
            content: sources.to_string(),
            reliability: 1.0,
        });
    }
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
            (
                "帮这个保温杯写上架文案和标题卖点，突出便携防漏",
                "ops_listing_copy",
            ),
            (
                "做一份零食货盘和SKU组合规划，选品预算80000",
                "ops_assortment_planning",
            ),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("ops"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
        }
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
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("data"), &task(prompt), "{}");
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
    fn autosuggests_creative_content_skill_by_intent() {
        let cases = [
            ("帮冷萃咖啡写一条抖音短视频脚本", "creative_video_script"),
            ("生成小红书种草笔记文案", "creative_seeding_copy"),
            ("给千乐咖啡做品牌IP人设定位", "creative_ip_branding"),
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
            ("看一下这个平台SLA和可用性情况", "engineering_sla_monitor"),
        ];

        for (prompt, expected_skill) in cases {
            let requests = autosuggest_skill_requests(&role("engineering"), &task(prompt), "{}");
            assert_eq!(requests.len(), 1, "{prompt}");
            assert_eq!(requests[0].skill_id, expected_skill, "{prompt}");
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
