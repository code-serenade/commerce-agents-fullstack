use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use super::{chat, planner};
use crate::{
    agent::commander::{
        no_user_facing_content_message, report_has_failure_diagnostic,
        report_has_llm_access_denied, report_has_llm_provider_error,
        sanitize_final_user_facing_text, user_facing_report_text,
    },
    agent_session::{RoleMemorySummary, TaskGraphMemorySummary, WorkerDetail},
    core::protocol::{
        Evaluation, RoleWorkOutput, TaskGraph, TaskGraphHandoff, TaskGraphProcessGap,
        TaskGraphProcessPanel, TaskGraphProcessRole, TaskGraphProcessSummary, TaskId, TaskNode,
        TaskNodeId, WorkerAssignment, WorkerReport, WorkerReportStatus,
    },
    roles::RoleRoute,
};

const MIN_MULTI_ROLE_FINAL_CHARS: usize = 220;
const MIN_UPSTREAM_ENRICH_CHARS: usize = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskNodeRuntimeState {
    Pending,
    Running,
    Passed,
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone)]
pub struct TaskGraphRuntime {
    pub graph: TaskGraph,
    pub attempts: HashMap<TaskNodeId, u8>,
    pub states: HashMap<TaskNodeId, TaskNodeRuntimeState>,
    pub reports: HashMap<TaskNodeId, WorkerReport>,
    pub rework_instructions: HashMap<TaskNodeId, String>,
    pub evaluations: Vec<Evaluation>,
    running_since: HashMap<TaskNodeId, Instant>,
    response_mode: Option<String>,
    learning_depth: Option<String>,
}

impl TaskGraphRuntime {
    pub fn new(graph: TaskGraph) -> Self {
        let states = graph
            .nodes
            .iter()
            .map(|node| (node.id.clone(), TaskNodeRuntimeState::Pending))
            .collect();

        Self {
            graph,
            attempts: HashMap::new(),
            states,
            reports: HashMap::new(),
            rework_instructions: HashMap::new(),
            evaluations: Vec::new(),
            running_since: HashMap::new(),
            response_mode: None,
            learning_depth: None,
        }
    }

    pub fn with_response_mode(mut self, response_mode: Option<&str>) -> Self {
        self.response_mode = response_mode
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        self
    }

    pub fn with_learning_depth(mut self, learning_depth: Option<&str>) -> Self {
        self.learning_depth = learning_depth
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        self
    }

    pub fn ready_nodes(&self) -> Vec<TaskNode> {
        let busy_workers = self
            .graph
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Running)
                )
            })
            .map(|node| node.worker_id.clone())
            .collect::<HashSet<_>>();
        let mut scheduled_workers = HashSet::new();

        self.graph
            .nodes
            .iter()
            .filter(|node| {
                let ready = matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Pending)
                ) && node.input_refs.iter().all(|input_ref| {
                    matches!(
                        self.states.get(input_ref),
                        Some(TaskNodeRuntimeState::Passed)
                    )
                });
                if !ready || busy_workers.contains(&node.worker_id) {
                    return false;
                }
                scheduled_workers.insert(node.worker_id.clone())
            })
            .cloned()
            .collect()
    }

    pub fn build_assignment(&self, node: &TaskNode) -> WorkerAssignment {
        let attempt = self.attempts.get(&node.id).copied().unwrap_or(0) + 1;
        let inputs = node
            .input_refs
            .iter()
            .filter_map(|input_ref| self.reports.get(input_ref).cloned())
            .collect();

        WorkerAssignment {
            graph_id: self.graph.graph_id.clone(),
            node_id: node.id.clone(),
            task_id: TaskId(format!("{}:{}", self.graph.graph_id.0, node.id.0)),
            worker_id: node.worker_id.clone(),
            role: node.role.clone(),
            attempt,
            root_task: self.graph.root_task.clone(),
            objective: node.objective.clone(),
            llm_usage: node.llm_usage.clone(),
            skill_autosuggest: node.skill_autosuggest,
            timeout_sec: node.timeout_sec,
            inputs,
            rework_instruction: self.rework_instructions.get(&node.id).cloned(),
        }
    }

    pub fn mark_running(&mut self, assignment: &WorkerAssignment) {
        self.attempts
            .insert(assignment.node_id.clone(), assignment.attempt);
        self.states
            .insert(assignment.node_id.clone(), TaskNodeRuntimeState::Running);
        self.running_since
            .insert(assignment.node_id.clone(), Instant::now());
    }

    pub fn apply_report(&mut self, report: WorkerReport) -> EvaluationOutcome {
        self.running_since.remove(&report.node_id);
        let node = self
            .graph
            .nodes
            .iter()
            .find(|node| node.id == report.node_id)
            .cloned();
        let evaluation = evaluate_report(&report, node.as_ref(), &self.reports);
        self.evaluations.push(evaluation.clone());
        self.reports.insert(report.node_id.clone(), report.clone());

        let Some(node) = node else {
            self.states.insert(
                report.node_id.clone(),
                TaskNodeRuntimeState::Failed("node not found".to_string()),
            );
            return EvaluationOutcome::Failed(evaluation);
        };

        if evaluation.passed {
            self.rework_instructions.remove(&report.node_id);
            self.states
                .insert(report.node_id.clone(), TaskNodeRuntimeState::Passed);
            return EvaluationOutcome::Passed(evaluation);
        }

        let attempt = self.attempts.get(&node.id).copied().unwrap_or(1);
        if attempt < node.max_attempts {
            let instruction = evaluation
                .rework_instruction
                .clone()
                .unwrap_or_else(|| "content must be specific and non-empty".to_string());
            self.rework_instructions
                .insert(node.id.clone(), instruction.clone());
            self.states
                .insert(node.id.clone(), TaskNodeRuntimeState::Pending);
            return EvaluationOutcome::Rework {
                evaluation,
                instruction,
            };
        }

        let reason = evaluation.reasons.join("; ");
        self.states
            .insert(node.id.clone(), TaskNodeRuntimeState::Failed(reason));
        self.skip_blocked_descendants();
        EvaluationOutcome::Failed(evaluation)
    }

    pub fn next_running_timeout_after(&self, timeout: Duration) -> Option<Duration> {
        let timeout = normalize_timeout(timeout);
        let now = Instant::now();

        self.graph
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Running)
                )
            })
            .map(|node| {
                self.running_since
                    .get(&node.id)
                    .map(|started| timeout.saturating_sub(now.saturating_duration_since(*started)))
                    .unwrap_or(Duration::ZERO)
            })
            .min()
    }

    pub fn stalled_running_assignments(&self, timeout: Duration) -> Vec<WorkerAssignment> {
        let timeout = normalize_timeout(timeout);
        let now = Instant::now();

        self.graph
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Running)
                )
            })
            .filter(|node| {
                self.running_since
                    .get(&node.id)
                    .is_none_or(|started| now.saturating_duration_since(*started) >= timeout)
            })
            .map(|node| {
                let mut assignment = self.build_assignment(node);
                assignment.attempt = self.attempts.get(&node.id).copied().unwrap_or(1);
                assignment
            })
            .collect()
    }

    pub fn has_running_nodes(&self) -> bool {
        self.states
            .values()
            .any(|state| matches!(state, TaskNodeRuntimeState::Running))
    }

    pub fn first_pending_assignment(&self) -> Option<WorkerAssignment> {
        self.graph
            .nodes
            .iter()
            .find(|node| {
                matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Pending)
                )
            })
            .map(|node| self.build_assignment(node))
    }

    pub fn exhaust_attempts(&mut self, node_id: &TaskNodeId, attempt: u8) {
        if let Some(node) = self.graph.nodes.iter().find(|node| &node.id == node_id) {
            self.attempts
                .insert(node_id.clone(), node.max_attempts.max(attempt));
        }
    }

    pub fn is_finished(&self) -> bool {
        self.states.values().all(|state| {
            matches!(
                state,
                TaskNodeRuntimeState::Passed
                    | TaskNodeRuntimeState::Failed(_)
                    | TaskNodeRuntimeState::Skipped(_)
            )
        })
    }

    pub fn synthesize(&self) -> String {
        if let Some(message) = self.llm_access_denied_message() {
            return message;
        }
        if let Some(message) = self.llm_provider_error_message() {
            return message;
        }
        if self.response_mode.as_deref() != Some("quick") && self.graph.nodes.len() > 1 {
            return self.learning_synthesis();
        }
        if let Some(final_output) = self.final_node_output() {
            return final_output;
        }

        let mut sections = Vec::new();

        for node in &self.graph.nodes {
            if !matches!(
                self.states.get(&node.id),
                Some(TaskNodeRuntimeState::Passed)
            ) {
                continue;
            }
            let Some(report) = self.reports.get(&node.id) else {
                continue;
            };
            if let Some(summary) = user_facing_report_text(report) {
                sections.push(summary);
            }
        }

        if sections.is_empty() {
            self.execution_gap_message()
                .unwrap_or_else(no_user_facing_content_message)
        } else {
            sanitize_final_user_facing_text(&sections.join("\n\n"))
                .unwrap_or_else(no_user_facing_content_message)
        }
    }

    fn learning_synthesis(&self) -> String {
        if let Some(message) = self.llm_access_denied_message() {
            return message;
        }
        if let Some(message) = self.llm_provider_error_message() {
            return message;
        }
        if let Some(final_output) = self.final_node_output() {
            return self.enrich_thin_final_output(final_output);
        }
        if self.final_node_has_blocking_material_gap() {
            return self.structured_fallback_with_upstream();
        }
        self.completed_role_sections()
            .or_else(|| self.execution_gap_message())
            .unwrap_or_else(no_user_facing_content_message)
    }

    fn enrich_thin_final_output(&self, final_output: String) -> String {
        if self.graph.nodes.len() <= 1 {
            return final_output;
        }
        if looks_like_blocking_material_gap_answer(&final_output)
            || (task_requests_table(&self.graph.root_task)
                && !contains_markdown_table(&final_output))
        {
            let fallback = fallback_structured_synthesis(&self.graph.root_task);
            let Some(upstream_output) = self.completed_role_sections_excluding_final() else {
                return fallback;
            };
            if looks_like_blocking_material_gap_answer(&upstream_output)
                || upstream_output.chars().count() < MIN_UPSTREAM_ENRICH_CHARS
            {
                return fallback;
            }
            return sanitize_final_user_facing_text(&format!(
                "{fallback}\n\n## 已有补充\n\n{upstream_output}"
            ))
            .unwrap_or(fallback);
        }
        if final_output.chars().count() >= MIN_MULTI_ROLE_FINAL_CHARS {
            return final_output;
        }

        let Some(upstream_output) = self.completed_role_sections_excluding_final() else {
            return final_output;
        };
        if upstream_output.chars().count() < MIN_UPSTREAM_ENRICH_CHARS {
            return final_output;
        }
        if upstream_output.contains(final_output.trim()) {
            return upstream_output;
        }

        sanitize_final_user_facing_text(&format!("{final_output}\n\n{upstream_output}"))
            .unwrap_or(final_output)
    }

    fn completed_role_sections(&self) -> Option<String> {
        self.completed_role_sections_with_filter(|_| true)
    }

    fn completed_role_sections_excluding_final(&self) -> Option<String> {
        self.completed_role_sections_with_filter(|node| !looks_like_final_node(node))
    }

    fn completed_role_sections_with_filter(
        &self,
        mut keep_node: impl FnMut(&TaskNode) -> bool,
    ) -> Option<String> {
        let mut sections = Vec::new();
        for node in &self.graph.nodes {
            if !keep_node(node)
                || !matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Passed)
                )
            {
                continue;
            }
            let Some(report) = self.reports.get(&node.id) else {
                continue;
            };
            if let Some(text) = user_facing_report_text(report) {
                push_unique_text(&mut sections, text);
            }
        }
        if sections.is_empty() {
            return None;
        }
        sanitize_final_user_facing_text(&sections.join("\n\n"))
    }

    fn llm_access_denied_message(&self) -> Option<String> {
        self.graph
            .nodes
            .iter()
            .filter_map(|node| self.reports.get(&node.id))
            .find(|report| report_has_llm_access_denied(report))
            .and_then(user_facing_report_text)
    }

    fn llm_provider_error_message(&self) -> Option<String> {
        self.graph
            .nodes
            .iter()
            .filter_map(|node| self.reports.get(&node.id))
            .find(|report| report_has_llm_provider_error(report))
            .and_then(user_facing_report_text)
    }

    pub fn final_node_output(&self) -> Option<String> {
        let final_nodes = self
            .graph
            .nodes
            .iter()
            .filter(|candidate| {
                looks_like_final_node(candidate)
                    && !candidate.input_refs.is_empty()
                    && matches!(
                        self.states.get(&candidate.id),
                        Some(TaskNodeRuntimeState::Passed)
                    )
                    && !self
                        .graph
                        .nodes
                        .iter()
                        .any(|node| node.input_refs.iter().any(|input| input == &candidate.id))
            })
            .collect::<Vec<_>>();
        let [final_node] = final_nodes.as_slice() else {
            return None;
        };

        let report = self.reports.get(&final_node.id)?;
        user_facing_report_text(report)
    }

    fn final_node_has_blocking_material_gap(&self) -> bool {
        self.terminal_final_node_report()
            .is_some_and(report_contains_blocking_material_gap)
    }

    fn terminal_final_node_report(&self) -> Option<&WorkerReport> {
        let final_nodes = self
            .graph
            .nodes
            .iter()
            .filter(|candidate| {
                looks_like_final_node(candidate)
                    && !candidate.input_refs.is_empty()
                    && matches!(
                        self.states.get(&candidate.id),
                        Some(TaskNodeRuntimeState::Passed)
                    )
                    && !self
                        .graph
                        .nodes
                        .iter()
                        .any(|node| node.input_refs.iter().any(|input| input == &candidate.id))
            })
            .collect::<Vec<_>>();
        let [final_node] = final_nodes.as_slice() else {
            return None;
        };
        self.reports.get(&final_node.id)
    }

    fn structured_fallback_with_upstream(&self) -> String {
        let fallback = fallback_structured_synthesis(&self.graph.root_task);
        let Some(upstream_output) = self.completed_role_sections_excluding_final() else {
            return fallback;
        };
        if looks_like_blocking_material_gap_answer(&upstream_output)
            || upstream_output.chars().count() < MIN_UPSTREAM_ENRICH_CHARS
        {
            return fallback;
        }
        sanitize_final_user_facing_text(&format!("{fallback}\n\n## 已有补充\n\n{upstream_output}"))
            .unwrap_or(fallback)
    }

    pub fn direct_chat_output(&self) -> Option<String> {
        let [node] = self.graph.nodes.as_slice() else {
            return None;
        };
        if node.role != "chat"
            || !node.input_refs.is_empty()
            || !matches!(
                self.states.get(&node.id),
                Some(TaskNodeRuntimeState::Passed)
            )
        {
            return None;
        }

        let report = self.reports.get(&node.id)?;
        user_facing_report_text(report)
    }

    fn execution_gap_message(&self) -> Option<String> {
        let gaps = self.execution_gaps();
        if gaps.is_empty() {
            return None;
        }

        let fallback = fallback_structured_synthesis(&self.graph.root_task);
        let mut lines = vec![
            fallback,
            String::new(),
            "## 本轮边界".to_string(),
            String::new(),
            "- 部分协作步骤没有形成可用结果，所以具体销量、转化率、成本、库存、平台规则或实时来源不能写成确定事实。".to_string(),
            "- 上面的内容是基于当前问题和已进入上下文的信息整理出的低风险可执行版本，后续可以用真实资料继续复核。".to_string(),
        ];
        if gaps.len() > 1 {
            lines.push("- 本轮有多个协作步骤未完成，最终答案已避免暴露内部失败细节。".to_string());
        }

        sanitize_final_user_facing_text(&lines.join("\n")).or_else(|| Some(lines.join("\n")))
    }

    pub fn process_summary(&self) -> TaskGraphProcessSummary {
        let roles = self
            .graph
            .nodes
            .iter()
            .map(|node| process_role_summary(node, self))
            .collect::<Vec<_>>();
        let primary = roles
            .iter()
            .find(|role| role.status == "passed")
            .cloned()
            .or_else(|| roles.first().cloned());
        let handoffs = process_handoffs(&self.graph.nodes);
        let risks = unique_strings(
            self.graph
                .nodes
                .iter()
                .filter_map(|node| self.reports.get(&node.id))
                .flat_map(|report| report.risks.clone())
                .collect(),
        );
        let open_questions = unique_strings(
            self.graph
                .nodes
                .iter()
                .filter_map(|node| self.reports.get(&node.id))
                .flat_map(|report| report.open_questions.clone())
                .collect(),
        );
        let incomplete = self
            .graph
            .nodes
            .iter()
            .filter_map(|node| process_gap_summary(node, self))
            .collect();
        let check_items = process_check_items(&self.graph.nodes, &self.reports, self);

        TaskGraphProcessSummary {
            graph_id: self.graph.graph_id.clone(),
            root_task: self.graph.root_task.clone(),
            primary,
            roles,
            handoffs,
            next_actions: Vec::new(),
            check_items,
            deliverables: Vec::new(),
            risks,
            open_questions,
            incomplete,
            panel: TaskGraphProcessPanel {
                default_collapsed: true,
                anchor: "answer_scoped_process".to_string(),
                label: "Task graph process".to_string(),
                reason: "Task graph runtime process details.".to_string(),
            },
        }
    }

    pub fn memory_summary(&self) -> TaskGraphMemorySummary {
        let worker_details = self
            .graph
            .nodes
            .iter()
            .filter_map(|node| {
                let report = self.reports.get(&node.id)?;
                let evaluation = self
                    .evaluations
                    .iter()
                    .rev()
                    .find(|evaluation| evaluation.node_id == node.id)
                    .cloned();
                Some(WorkerDetail {
                    graph_id: report.graph_id.clone(),
                    node_id: report.node_id.clone(),
                    task_id: report.task_id.clone(),
                    worker_id: report.worker_id.clone(),
                    role: report.role.clone(),
                    title: node.title.clone(),
                    objective: node.objective.clone(),
                    attempt: self.attempts.get(&node.id).copied().unwrap_or(1),
                    status: report.status.clone(),
                    content: report.content.clone(),
                    role_output: report.role_output.clone(),
                    evidence: report.evidence.clone(),
                    risks: report.risks.clone(),
                    open_questions: report.open_questions.clone(),
                    diagnostics: report.diagnostics.clone(),
                    evaluation,
                })
            })
            .collect::<Vec<_>>();
        let role_summaries = self
            .graph
            .nodes
            .iter()
            .filter_map(|node| self.reports.get(&node.id))
            .map(|report| {
                let role_output = report.role_output.clone().unwrap_or_default();
                RoleMemorySummary {
                    graph_id: report.graph_id.clone(),
                    node_id: report.node_id.clone(),
                    worker_id: report.worker_id.clone(),
                    role: report.role.clone(),
                    summary: if role_output.summary.trim().is_empty() {
                        report.content.clone()
                    } else {
                        role_output.summary
                    },
                    findings: role_output.findings,
                    recommendations: role_output.recommendations,
                    evidence: if role_output.evidence.is_empty() {
                        report.evidence.clone()
                    } else {
                        role_output.evidence
                    },
                    risks: merge_unique(role_output.risks, report.risks.clone()),
                    open_questions: merge_unique(
                        role_output.open_questions,
                        report.open_questions.clone(),
                    ),
                    status: report.status.clone(),
                }
            })
            .collect::<Vec<_>>();
        let roles = self
            .graph
            .nodes
            .iter()
            .map(|node| node.role.clone())
            .collect::<Vec<_>>();
        let evaluation_summary = (!self.evaluations.is_empty()).then(|| {
            self.evaluations
                .iter()
                .map(|evaluation| {
                    format!(
                        "{} passed={} score={:.2}: {}",
                        evaluation.node_id.0,
                        evaluation.passed,
                        evaluation.score,
                        evaluation.reasons.join("; ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        });

        TaskGraphMemorySummary {
            graph_id: self.graph.graph_id.clone(),
            root_task: self.graph.root_task.clone(),
            roles,
            process: Some(self.process_summary()),
            role_summaries,
            worker_details,
            risks: unique_strings(
                self.reports
                    .values()
                    .flat_map(|report| report.risks.clone())
                    .collect(),
            ),
            open_questions: unique_strings(
                self.reports
                    .values()
                    .flat_map(|report| report.open_questions.clone())
                    .collect(),
            ),
            evaluation_summary,
        }
    }

    fn skip_blocked_descendants(&mut self) {
        let failed = self
            .states
            .iter()
            .filter_map(|(node_id, state)| {
                matches!(state, TaskNodeRuntimeState::Failed(_)).then_some(node_id.clone())
            })
            .collect::<HashSet<_>>();

        let mut changed = true;
        while changed {
            changed = false;
            for node in &self.graph.nodes {
                if !matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Pending)
                ) {
                    continue;
                }
                if node.input_refs.iter().any(|input_ref| {
                    failed.contains(input_ref)
                        || matches!(
                            self.states.get(input_ref),
                            Some(TaskNodeRuntimeState::Skipped(_))
                        )
                }) {
                    self.states.insert(
                        node.id.clone(),
                        TaskNodeRuntimeState::Skipped("dependency did not pass".to_string()),
                    );
                    changed = true;
                }
            }
        }
    }

    fn execution_gaps(&self) -> Vec<TaskGraphProcessGap> {
        self.graph
            .nodes
            .iter()
            .filter_map(|node| process_gap_summary(node, self))
            .collect()
    }
}

fn normalize_timeout(timeout: Duration) -> Duration {
    if timeout.is_zero() {
        Duration::from_millis(1)
    } else {
        timeout
    }
}

fn push_unique_text(items: &mut Vec<String>, text: String) {
    let text = text.trim();
    if text.is_empty() || items.iter().any(|existing| existing == text) {
        return;
    }
    items.push(text.to_string());
}

fn looks_like_final_node(node: &TaskNode) -> bool {
    let id = node.id.0.to_ascii_lowercase();
    id.contains("final")
        || node.title.contains("综合")
        || node.title.contains("总方案")
        || node.title.contains("运营方案")
        || node.objective.contains("综合成")
        || node.objective.contains("形成可执行")
}

fn looks_like_blocking_material_gap_answer(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }
    let gap_hits = [
        "等待用户提供附件或表格数据",
        "等待用户提供",
        "建议用户提供附件或表格数据",
        "建议用户提供附件或表格",
        "建议用户提供附件",
        "建议用户提供表格",
        "当前没有可用的附件或表格",
        "没有可用的附件或表格",
        "未提供附件或表格",
        "未上传表格",
        "没有上传表格",
        "未上传任何表格",
        "未提供具体商品信息",
        "未提供销售数据",
        "未提供库存情况",
        "未提供平台规则",
        "无法提供真实数据",
        "无法提供真实数据或指标",
        "由于未上传表格",
        "由于未上传任何表格",
        "用户未上传任何表格",
        "用户是否上传了表格",
        "是否上传了表格",
        "未找到附件或表格数据",
        "未找到可用的附件或表格数据",
        "无法直接抽取",
        "无法直接分析",
        "无法直接分析数据",
        "无法直接生成具体数据表格",
        "无法直接生成表格",
        "无法生成完整的促销方案",
        "无法生成完整促销方案",
        "无法进行进一步分析",
        "无法提供具体的表格",
        "缺少必要的数据",
        "该岗位本轮未形成有效结论",
    ]
    .iter()
    .filter(|marker| text.contains(*marker))
    .count();
    if gap_hits == 0 {
        return false;
    }
    let actionable_hits = [
        "可先",
        "先按",
        "建议",
        "下一步",
        "动作",
        "清单",
        "表格",
        "风险边界",
        "复核",
        "验证",
    ]
    .iter()
    .filter(|marker| text.contains(*marker))
    .count();
    gap_hits >= 2 || actionable_hits == 0 || text.chars().count() < 220
}

fn report_contains_blocking_material_gap(report: &WorkerReport) -> bool {
    let mut parts = vec![report.content.as_str()];
    if let Some(output) = report.role_output.as_ref() {
        parts.push(output.summary.as_str());
        parts.extend(output.findings.iter().map(String::as_str));
        parts.extend(output.recommendations.iter().map(String::as_str));
        parts.extend(output.risks.iter().map(String::as_str));
        parts.extend(output.open_questions.iter().map(String::as_str));
    }
    looks_like_blocking_material_gap_answer(&parts.join("\n"))
}

fn fallback_structured_synthesis(task: &str) -> String {
    let wants_table = task_requests_table(task);
    let wants_metric_table =
        task.contains("列出指标") || (task.contains("怎么看") && task.contains("下一步"));
    let lead = if wants_table {
        "先按当前问题、已选上下文和上游岗位输出给出可执行表格草案；表格里的真实销量、转化率、成本、\
         库存、平台规则或实时来源，后续再用真实数据复核。"
    } else {
        "先按当前问题、已选上下文和上游岗位输出给出可执行草案；涉及真实销量、转化率、成本、库存、\
         平台规则或实时来源的部分，后续再用真实数据复核。"
    };
    let mut sections = vec![
        "## 当前可先交付".to_string(),
        String::new(),
        lead.to_string(),
        String::new(),
        "## 先做什么".to_string(),
        String::new(),
        "- 先明确分析对象、目标人群、目标动作和当前可用信息。".to_string(),
        "- 把需要验证的判断拆成可观察字段，再用小样本、后台数据或后续上传资料复核。".to_string(),
        "- 对缺少真实数据的部分只写成假设或待验证项，不写成确定事实。".to_string(),
    ];
    if wants_table {
        sections.extend([String::new(), "## 执行表".to_string(), String::new()]);
        if wants_metric_table {
            sections.extend([
                "| 指标 | 怎么看 | 可能问题 | 下一步动作 |".to_string(),
                "| --- | --- | --- | --- |".to_string(),
                "| 触达/点击 | 用户是否愿意进入详情或继续了解 | 入口表达不清、对象不准 | 同时测试 \
                 2-3 个入口表达 |"
                    .to_string(),
                "| 意向/加购 | 用户看完后是否有进一步购买或咨询意愿 | 权益、规格、信任感不足 | \
                 补清楚适用场景、权益和高频疑问 |"
                    .to_string(),
                "| 转化/付款 | 用户是否完成目标动作 | 价格、库存、流程或承接卡住 | \
                 小范围试跑一版低风险方案 |"
                    .to_string(),
                "| 反馈/售后 | 负向反馈集中在哪些点 | 描述不清、承诺过满或预期不一致 | \
                 把高频问题写进页面、话术或复盘字段 |"
                    .to_string(),
            ]);
        } else {
            sections.extend([
                "| 环节 | 当前可做 | 复核信息 | 风险边界 |".to_string(),
                "| --- | --- | --- | --- |".to_string(),
                "| 对象 | 明确本轮分析对象和适用场景 | 已选上下文、用户补充、历史记录 | \
                 不借用无关对象的数据 |"
                    .to_string(),
                "| 方案 | 输出一版可试跑动作或内容草案 | 点击、咨询、加购、转化等反馈 | \
                 不编造价格、销量、库存或规则 |"
                    .to_string(),
                "| 验证 | 每轮只改少量关键变量 | 前后对比、样本周期、异常反馈 | \
                 样本不足时只做方向判断 |"
                    .to_string(),
                "| 复盘 | 记录本轮假设、结果和下轮动作 | 真实数据或后续上传资料 | \
                 缺资料时保留待确认项 |"
                    .to_string(),
            ]);
        }
    }
    sections.extend([
        String::new(),
        "## 风险边界".to_string(),
        String::new(),
        "- 没有真实资料时，不能编造销量、转化率、退款率、成本、库存、平台规则或实时搜索结论。"
            .to_string(),
        "- 如果后续上传 Excel、CSV、图片、活动或后台数据，再按真实字段重新复核结论。".to_string(),
    ]);
    sanitize_final_user_facing_text(&sections.join("\n")).unwrap_or_else(|| sections.join("\n"))
}

fn task_requests_table(task: &str) -> bool {
    [
        "有表格",
        "输出表格",
        "输出成表格",
        "输出一个表格",
        "表格形式",
        "整理成表格",
        "用表格",
        "用表格输出",
        "包含表格",
        "做成表格",
        "列出指标",
    ]
    .iter()
    .any(|marker| task.contains(marker))
}

fn contains_markdown_table(text: &str) -> bool {
    let lines = text.lines().collect::<Vec<_>>();
    lines.windows(2).any(|pair| {
        pair[0].trim().starts_with('|')
            && pair[0].trim().ends_with('|')
            && pair[1].trim().starts_with('|')
            && pair[1].contains("---")
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationOutcome {
    Passed(Evaluation),
    Rework {
        evaluation: Evaluation,
        instruction: String,
    },
    Failed(Evaluation),
}

pub fn plan_task_graph(task_id: &TaskId, task: &str, route: Option<&RoleRoute>) -> TaskGraph {
    plan_configured_task_graph(task_id, task, route)
        .or_else(|| {
            route.map(|route| {
                chat::plan_single_chat_graph(task_id, task, &route.primary_runtime_role)
            })
        })
        .unwrap_or_else(|| chat::plan_single_chat_graph(task_id, task, "chat"))
}

pub fn plan_configured_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    planner::plan_configured_task_graph(task_id, task, route)
}

pub fn plan_configured_non_chat_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    planner::plan_configured_non_chat_task_graph(task_id, task, route)
}

pub fn plan_quick_chat_graph(task_id: &TaskId, task: &str) -> Option<TaskGraph> {
    let graph = planner::plan_configured_task_graph(task_id, task, None)?;
    let [node] = graph.nodes.as_slice() else {
        return None;
    };
    (node.role == "chat" && node.input_refs.is_empty()).then_some(graph)
}

pub fn plan_forced_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    chat::plan_single_chat_graph(task_id, task, "chat")
}

pub fn plan_forced_role_graph(task_id: &TaskId, task: &str, role: &str) -> TaskGraph {
    chat::plan_single_chat_graph(task_id, task, role)
}

pub fn plan_forced_selected_roles_graph(
    task_id: &TaskId,
    task: &str,
    roles: &[String],
) -> TaskGraph {
    chat::plan_selected_roles_graph(task_id, task, roles)
}

fn evaluate_report(
    report: &WorkerReport,
    node: Option<&TaskNode>,
    upstream_reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Evaluation {
    let mut reasons = Vec::new();
    match report.status {
        WorkerReportStatus::Completed => {}
        WorkerReportStatus::Failed => {
            reasons.push("worker reported failed status".to_string());
        }
        WorkerReportStatus::TimedOut => {
            reasons.push("worker cognition timed out".to_string());
        }
    }
    if report_has_llm_access_denied(report) {
        reasons.push("llm access denied: model configuration unavailable".to_string());
    }
    if report_has_llm_provider_error(report) {
        reasons.push("llm provider error: upstream model service unavailable".to_string());
    }
    if report_has_failure_diagnostic(report, "worker_unavailable")
        || report_has_evidence_marker(report, "failure_kind:worker_unavailable")
    {
        reasons.push("worker unavailable: no registered executor for selected role".to_string());
    }
    if report_has_failure_diagnostic(report, "task_graph_stalled")
        || report_has_evidence_marker(report, "failure_kind:task_graph_stalled")
    {
        reasons.push("task graph stalled: no schedulable node".to_string());
    }
    if report.content.trim().is_empty()
        && report
            .role_output
            .as_ref()
            .is_none_or(|output| output.summary.trim().is_empty())
    {
        reasons.push("content is empty".to_string());
    }
    if report.role_output.is_none() {
        reasons.push("structured role output is missing".to_string());
    }
    if let (Some(node), Some(output)) = (node, report.role_output.as_ref()) {
        reasons.extend(evaluate_upstream_evidence(node, output, upstream_reports));
    }

    let passed = reasons.is_empty();
    Evaluation {
        node_id: report.node_id.clone(),
        passed,
        score: if passed { 1.0 } else { 0.0 },
        reasons,
        rework_instruction: (!passed)
            .then_some("content must be specific and non-empty".to_string()),
    }
}

fn report_has_evidence_marker(report: &WorkerReport, marker: &str) -> bool {
    report.evidence.iter().any(|item| item.contains(marker))
        || report
            .role_output
            .as_ref()
            .is_some_and(|output| output.evidence.iter().any(|item| item.contains(marker)))
}

fn evaluate_upstream_evidence(
    node: &TaskNode,
    output: &RoleWorkOutput,
    upstream_reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    if node.input_refs.is_empty() || output.evidence.is_empty() {
        return Vec::new();
    }

    node.input_refs
        .iter()
        .filter(|input_ref| upstream_reports.contains_key(input_ref))
        .filter(|input_ref| {
            let source_marker = format!("worker.assignment.input.{}", input_ref.0);
            !output.evidence.iter().any(|item| {
                item.contains(&input_ref.0)
                    || item.contains(&source_marker)
                    || upstream_reports
                        .get(input_ref)
                        .is_some_and(|report| item.contains(&report.role))
            })
        })
        .map(|input_ref| {
            format!(
                "upstream evidence: missing citation for input {}",
                input_ref.0
            )
        })
        .collect()
}

fn process_role_summary(node: &TaskNode, runtime: &TaskGraphRuntime) -> TaskGraphProcessRole {
    let report = runtime.reports.get(&node.id);
    let role_output = report.and_then(|report| report.role_output.as_ref());
    let summary = role_output
        .and_then(|output| non_empty(&output.summary))
        .or_else(|| report.and_then(|report| non_empty(&report.content)))
        .map(ToString::to_string);
    let finding = role_output
        .and_then(|output| output.findings.iter().find_map(|item| non_empty(item)))
        .map(ToString::to_string);
    let recommendation = role_output
        .and_then(|output| {
            output
                .recommendations
                .iter()
                .find_map(|item| non_empty(item))
        })
        .map(ToString::to_string);
    let status = match runtime.states.get(&node.id) {
        Some(TaskNodeRuntimeState::Pending) => "pending",
        Some(TaskNodeRuntimeState::Running) => "running",
        Some(TaskNodeRuntimeState::Passed) => "passed",
        Some(TaskNodeRuntimeState::Failed(_)) => "failed",
        Some(TaskNodeRuntimeState::Skipped(_)) => "skipped",
        None => "unknown",
    }
    .to_string();

    TaskGraphProcessRole {
        node_id: node.id.clone(),
        task_id: report.map(|report| report.task_id.clone()),
        worker_id: node.worker_id.clone(),
        role: node.role.clone(),
        role_label: process_node_label(node),
        title: node.title.clone(),
        objective: node.objective.clone(),
        status,
        attempt: runtime.attempts.get(&node.id).copied().unwrap_or(0),
        is_primary: false,
        summary,
        finding,
        recommendation,
        evidence: report
            .map(|report| report.evidence.clone())
            .unwrap_or_default(),
        risks: report
            .map(|report| report.risks.clone())
            .unwrap_or_default(),
        open_questions: report
            .map(|report| report.open_questions.clone())
            .unwrap_or_default(),
    }
}

fn process_handoffs(nodes: &[TaskNode]) -> Vec<TaskGraphHandoff> {
    let node_by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();

    nodes
        .iter()
        .flat_map(|node| {
            let node_by_id = &node_by_id;
            node.input_refs.iter().filter_map(move |input| {
                let upstream = node_by_id.get(input)?;
                Some(TaskGraphHandoff {
                    from_node_id: upstream.id.clone(),
                    to_node_id: node.id.clone(),
                    from_role: upstream.role.clone(),
                    to_role: node.role.clone(),
                    from_role_label: process_node_label(upstream),
                    to_role_label: process_node_label(node),
                    summary: format!(
                        "{} 需要参考 {} 的结果",
                        process_node_label(node),
                        process_node_label(upstream)
                    ),
                })
            })
        })
        .collect()
}

fn process_gap_summary(node: &TaskNode, runtime: &TaskGraphRuntime) -> Option<TaskGraphProcessGap> {
    match runtime.states.get(&node.id) {
        Some(TaskNodeRuntimeState::Failed(reason)) => Some(TaskGraphProcessGap {
            node_id: node.id.clone(),
            role: node.role.clone(),
            role_label: process_node_label(node),
            title: node.title.clone(),
            state: "failed".to_string(),
            reason: user_facing_gap_reason(reason),
        }),
        Some(TaskNodeRuntimeState::Skipped(reason)) => Some(TaskGraphProcessGap {
            node_id: node.id.clone(),
            role: node.role.clone(),
            role_label: process_node_label(node),
            title: node.title.clone(),
            state: "skipped".to_string(),
            reason: user_facing_gap_reason(reason),
        }),
        _ => None,
    }
}

fn process_check_items(
    nodes: &[TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
    runtime: &TaskGraphRuntime,
) -> Vec<String> {
    let mut items = Vec::new();
    for node in nodes {
        let Some(report) = reports.get(&node.id) else {
            continue;
        };
        let mut has_search_diagnostic = false;
        for diagnostic in report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == "search_evidence")
        {
            has_search_diagnostic = true;
            let item = search_diagnostic_check_item(node, diagnostic);
            if !items.contains(&item) {
                items.push(item);
            }
        }
        if has_search_diagnostic {
            continue;
        }
        for evidence in report_evidence_items(report) {
            if let Some(item) = search_trace_check_item(node, &evidence)
                && !items.contains(&item)
            {
                items.push(item);
            }
        }
    }
    for node in nodes {
        let Some(report) = reports.get(&node.id) else {
            continue;
        };
        for diagnostic in report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == "failure")
            .filter(|diagnostic| {
                diagnostic
                    .fields
                    .get("failure_kind")
                    .is_some_and(|value| value.starts_with("context_pack_"))
            })
        {
            let item = context_pack_failure_check_item(node, diagnostic);
            if !items.contains(&item) {
                items.push(item);
            }
        }
    }
    for node in nodes {
        let Some(report) = reports.get(&node.id) else {
            continue;
        };
        for diagnostic in report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == "failure")
            .filter(|diagnostic| {
                diagnostic
                    .fields
                    .get("failure_kind")
                    .is_some_and(|value| value.starts_with("skill_"))
            })
        {
            let item = skill_failure_check_item(node, diagnostic);
            if !items.contains(&item) {
                items.push(item);
            }
        }
    }
    for (index, gap) in runtime.execution_gaps().into_iter().enumerate() {
        let item = format!(
            "协作核对：{}未完成，原因 {}",
            user_facing_gap_label(&gap, index),
            user_facing_gap_reason(&gap.reason)
        );
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

fn process_node_label(node: &TaskNode) -> String {
    let title = node.title.trim();
    if !title.is_empty() && contains_cjk(title) && !looks_like_internal_runtime_label(title) {
        return title.to_string();
    }
    "协作步骤".to_string()
}

fn user_facing_gap_label(gap: &TaskGraphProcessGap, index: usize) -> String {
    let label = gap.role_label.trim();
    if !label.is_empty() && !looks_like_internal_runtime_label(label) {
        return format!("{label} ");
    }
    format!("协作步骤{} ", index + 1)
}

fn looks_like_internal_runtime_label(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "chat"
            | "ops"
            | "data"
            | "creative"
            | "service"
            | "engineering"
            | "accounting"
            | "design"
            | "web"
            | "search"
            | "douyin_ads"
    ) || lower.ends_with("_final")
        || lower.starts_with("worker.")
        || lower.starts_with("role.")
}

fn contains_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
}

fn context_pack_failure_check_item(
    node: &TaskNode,
    diagnostic: &crate::core::protocol::WorkerDiagnostic,
) -> String {
    let readiness = diagnostic
        .fields
        .get("readiness")
        .map(String::as_str)
        .unwrap_or("unknown");
    let item_count = diagnostic
        .fields
        .get("item_count")
        .map(String::as_str)
        .unwrap_or("0");
    let omitted_count = diagnostic
        .fields
        .get("omitted_count")
        .map(String::as_str)
        .unwrap_or("0");
    let pending_count = diagnostic
        .fields
        .get("extraction_pending_count")
        .map(String::as_str)
        .unwrap_or("0");
    let failed_count = diagnostic
        .fields
        .get("extraction_failed_count")
        .map(String::as_str)
        .unwrap_or("0");
    let unsupported_count = diagnostic
        .fields
        .get("extraction_unsupported_count")
        .map(String::as_str)
        .unwrap_or("0");
    let reason = diagnostic
        .fields
        .get("failure_kind")
        .map(|value| user_facing_gap_reason(value))
        .unwrap_or_else(|| user_facing_gap_reason(&diagnostic.source));

    format!(
        "资料核对：{} 的资料包状态 {}，进入上下文 {} 项，省略 {} 项，解析中 {} 项，解析失败 {} \
         项，不支持 {} 项，原因 {}",
        process_node_label(node),
        readiness,
        item_count,
        omitted_count,
        pending_count,
        failed_count,
        unsupported_count,
        reason
    )
}

fn skill_failure_check_item(
    node: &TaskNode,
    diagnostic: &crate::core::protocol::WorkerDiagnostic,
) -> String {
    let skill = diagnostic
        .fields
        .get("skill")
        .map(String::as_str)
        .unwrap_or("unknown_skill");
    let status = diagnostic
        .fields
        .get("status")
        .map(String::as_str)
        .unwrap_or("unknown");
    let reason = diagnostic
        .fields
        .get("failure_kind")
        .map(|value| user_facing_gap_reason(value))
        .unwrap_or_else(|| user_facing_gap_reason(&diagnostic.source));

    format!(
        "Skill核对：{} / {} 未完成，状态 {}，原因 {}",
        process_node_label(node),
        skill,
        status,
        reason
    )
}

fn search_diagnostic_check_item(
    node: &TaskNode,
    diagnostic: &crate::core::protocol::WorkerDiagnostic,
) -> String {
    let skill = diagnostic
        .fields
        .get("skill")
        .map(String::as_str)
        .or_else(|| non_empty(&diagnostic.source))
        .unwrap_or("search");
    let status = diagnostic
        .fields
        .get("adapter_status")
        .or_else(|| diagnostic.fields.get("status"))
        .map(String::as_str)
        .unwrap_or("unknown");
    let intent = diagnostic
        .fields
        .get("intent")
        .map(String::as_str)
        .unwrap_or("general_search_evidence");
    let strength = diagnostic
        .fields
        .get("strength")
        .map(String::as_str)
        .unwrap_or("unknown");
    let sources = diagnostic
        .fields
        .get("sources")
        .map(String::as_str)
        .unwrap_or("0");
    let usable_claims = diagnostic
        .fields
        .get("usable_claims")
        .map(String::as_str)
        .unwrap_or("0");
    let gaps = diagnostic
        .fields
        .get("gaps")
        .map(String::as_str)
        .unwrap_or("0");
    let confidence = diagnostic
        .fields
        .get("confidence")
        .map(String::as_str)
        .unwrap_or("unknown");

    format!(
        "搜索核对：{} / {}，状态 {}，意图 {}，强度 {}，来源 {}，可用结论 {}，缺口 {}，置信度 {}",
        process_node_label(node),
        skill,
        status,
        intent,
        strength,
        sources,
        usable_claims,
        gaps,
        confidence
    )
}

pub(crate) fn user_facing_gap_reason(reason: &str) -> String {
    let reason = reason.trim();
    let lower = reason.to_ascii_lowercase();
    if lower.contains("llm_provider_error") || lower.contains("provider error") {
        "模型服务临时不可用".to_string()
    } else if lower.contains("worker_unavailable")
        || lower.contains("worker unavailable")
        || reason.contains("没有注册可用")
    {
        "协作步骤没有成功启动".to_string()
    } else if lower.contains("task_graph_stalled")
        || lower.contains("task graph stalled")
        || lower.contains("no schedulable node")
    {
        "协作流程没有找到可继续执行的步骤".to_string()
    } else if lower.contains("worker reported failed status") {
        "协作步骤没有成功完成".to_string()
    } else if lower.contains("llm access denied") || lower.contains("llm_access_denied") {
        "模型服务配置不可用".to_string()
    } else if lower.contains("skill_adapter_timeout") {
        "Skill 适配器执行超时".to_string()
    } else if lower.contains("skill_adapter_unavailable") {
        "Skill 适配器未接入或当前不可用".to_string()
    } else if lower.contains("skill_pending_approval") {
        "Skill 需要审批后才能执行".to_string()
    } else if lower.contains("skill_not_found") {
        "Skill 未注册或不存在".to_string()
    } else if lower.contains("skill_validation_error") {
        "Skill 入参不完整或校验失败".to_string()
    } else if lower.contains("skill_execution_failed") {
        "Skill 执行失败".to_string()
    } else if lower.contains("skill_deferred") {
        "Skill 当前只能返回待执行或缺数据状态".to_string()
    } else if lower.contains("context_pack_extraction_failed") {
        "部分资料抽取失败，不能当作已读取正文".to_string()
    } else if lower.contains("context_pack_extraction_unsupported") {
        "部分资料类型当前不支持抽取正文".to_string()
    } else if lower.contains("context_pack_extraction_pending") {
        "部分资料仍在解析中，当前只能使用已解析内容或元数据".to_string()
    } else if lower.contains("context_pack_omitted") {
        "部分资料因条数或字数限制未进入本轮上下文".to_string()
    } else if lower.contains("context_pack_empty") {
        "本轮没有可读资料进入上下文".to_string()
    } else if lower.contains("context_pack_partial") {
        "本轮资料包不完整，只能基于已进入上下文的内容回答".to_string()
    } else if lower.contains("timed out") || lower.contains("timeout") {
        "协作步骤执行超时".to_string()
    } else if lower.contains("dependency did not pass") || reason.contains("依赖") {
        "前置协作步骤没有完成".to_string()
    } else if lower.contains("content is empty") {
        "协作步骤没有产出可用内容".to_string()
    } else if reason.is_empty() {
        "未记录具体原因".to_string()
    } else if looks_like_internal_failure_reason(reason) {
        "协作步骤没有成功完成".to_string()
    } else {
        reason.to_string()
    }
}

fn looks_like_internal_failure_reason(reason: &str) -> bool {
    let lower = reason.to_ascii_lowercase();
    lower.contains("failure_kind")
        || lower.contains("worker")
        || lower.contains("executor")
        || lower.contains("diagnostic")
        || lower.contains("role")
        || lower.contains("node")
        || lower.contains("graph")
}

fn report_evidence_items(report: &WorkerReport) -> Vec<String> {
    let mut items = report.evidence.clone();
    if let Some(output) = report.role_output.as_ref() {
        items.extend(output.evidence.clone());
    }
    unique_strings(items)
}

fn search_trace_check_item(node: &TaskNode, evidence: &str) -> Option<String> {
    let fields = parse_search_trace_fields(evidence)?;
    let skill = fields.get("skill").map(String::as_str).unwrap_or("search");
    let status = fields
        .get("adapter_status")
        .or_else(|| fields.get("status"))
        .map(String::as_str)
        .unwrap_or("unknown");
    let intent = fields
        .get("intent")
        .map(String::as_str)
        .unwrap_or("general_search_evidence");
    let strength = fields
        .get("strength")
        .map(String::as_str)
        .unwrap_or("unknown");
    let sources = fields.get("sources").map(String::as_str).unwrap_or("0");
    let usable_claims = fields
        .get("usable_claims")
        .map(String::as_str)
        .unwrap_or("0");
    let gaps = fields.get("gaps").map(String::as_str).unwrap_or("0");
    let confidence = fields
        .get("confidence")
        .map(String::as_str)
        .unwrap_or("unknown");

    Some(format!(
        "搜索核对：{} / {}，状态 {}，意图 {}，强度 {}，来源 {}，可用结论 {}，缺口 {}，置信度 {}",
        process_node_label(node),
        skill,
        status,
        intent,
        strength,
        sources,
        usable_claims,
        gaps,
        confidence
    ))
}

fn parse_search_trace_fields(trace: &str) -> Option<HashMap<String, String>> {
    let rest = trace.trim().strip_prefix("search_evidence_trace:")?;
    let fields = rest
        .split_whitespace()
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| {
            (
                key.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            )
        })
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .collect::<HashMap<_, _>>();
    (!fields.is_empty()).then_some(fields)
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

fn merge_unique(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    for item in right {
        if !item.trim().is_empty() && !left.contains(&item) {
            left.push(item);
        }
    }
    left
}

fn unique_strings(items: Vec<String>) -> Vec<String> {
    items.into_iter().fold(Vec::new(), |mut unique, item| {
        if !item.trim().is_empty() && !unique.contains(&item) {
            unique.push(item);
        }
        unique
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{TaskGraphRuntime, TaskNodeRuntimeState, plan_forced_selected_roles_graph};
    use crate::core::protocol::{
        RoleWorkOutput, TaskGraph, TaskGraphId, TaskId, TaskNode, TaskNodeId, WorkerDiagnostic,
        WorkerId, WorkerReport, WorkerReportStatus,
    };

    #[test]
    fn task_graph_synthesis_hides_internal_report_shell() {
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-user-facing".to_string()),
            root_task: "帮我做一段咖啡商品讲解。".to_string(),
            nodes: vec![node("creative", "内容创意", "creative")],
            adaptive_hint: None,
        });
        runtime.states.insert(
            TaskNodeId("creative".to_string()),
            TaskNodeRuntimeState::Passed,
        );
        runtime.reports.insert(
            TaskNodeId("creative".to_string()),
            report(
                "creative",
                "内容创意(creative): \
                 【可直接测试的讲解（占位符需替换为真实信息）】突出【真实品类，如挂耳/速溶/\
                 胶囊】和【真实规格/到手价】。\n备注：这是教学模式。",
            ),
        );

        let output = runtime.synthesize();
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("突出真实品类和真实规格、到手价。"));
        assert!(output.contains("说明：这是教学模式。"));
        assert!(!lower.contains("task graph"));
        assert!(!lower.contains("passed worker reports"));
        assert!(!lower.contains("worker"));
        assert!(!lower.contains("role_output"));
        assert!(!output.contains("内容创意(creative)"));
        assert!(!output.contains("占位符"));
        assert!(!output.contains("备注："));
        assert!(!output.contains('【'));
        assert!(!output.contains('】'));
    }

    #[test]
    fn direct_chat_output_uses_same_user_facing_cleanup() {
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-chat".to_string()),
            root_task: "帮我写一句自然回复。".to_string(),
            nodes: vec![node("chat", "直接回复", "chat")],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(TaskNodeId("chat".to_string()), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            TaskNodeId("chat".to_string()),
            report(
                "chat",
                "chat(worker.chat): \
                 【可直接发送的回复（占位符需替换为真实信息）】先确认用户问题。\n备注：\
                 教学模式下可以解释原因。",
            ),
        );

        let output = runtime
            .direct_chat_output()
            .expect("chat output should be available");

        assert!(output.contains("先确认用户问题。"));
        assert!(output.contains("说明：教学模式下可以解释原因。"));
        assert!(!output.contains("chat(worker.chat)"));
        assert!(!output.contains("占位符"));
        assert!(!output.contains("备注："));
    }

    #[test]
    fn task_graph_synthesis_uses_readable_empty_fallback() {
        let runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-empty".to_string()),
            root_task: "帮我回答问题。".to_string(),
            nodes: vec![node("creative", "内容创意", "creative")],
            adaptive_hint: None,
        });

        let output = runtime.synthesize();

        assert!(output.contains("没有拿到可直接展示的完整内容"));
        assert!(output.contains("## 当前状态"));
        assert!(output.contains("## 可能原因"));
        assert!(output.contains("## 下一步"));
        assert!(output.contains("- 问题目标还不够明确。"));
        assert!(!output.contains("none"));
        assert!(!output.contains("Passed worker reports"));
    }

    #[test]
    fn task_graph_synthesis_surfaces_llm_access_denied_before_empty_fallback() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-access-denied".to_string()),
            root_task: "Build a plan.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime.states.insert(
            data.id.clone(),
            TaskNodeRuntimeState::Failed(
                "worker reported failed status; structured role output is missing".to_string(),
            ),
        );
        runtime.states.insert(
            ops.id.clone(),
            TaskNodeRuntimeState::Skipped("dependency did not pass".to_string()),
        );
        runtime.reports.insert(
            data.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-access-denied".to_string()),
                node_id: data.id.clone(),
                task_id: TaskId("task-data".to_string()),
                worker_id: WorkerId("worker.data".to_string()),
                role: "data".to_string(),
                content: "llm_access_denied: 当前 AI 模型服务配置不可用".to_string(),
                role_output: None,
                evidence: vec!["failure_kind:llm_access_denied".to_string()],
                risks: Vec::new(),
                open_questions: vec!["llm_access_denied: 当前 AI 模型服务配置不可用".to_string()],
                diagnostics: Vec::new(),
                status: WorkerReportStatus::Failed,
            },
        );

        let output = runtime.synthesize();

        assert!(output.contains("模型服务配置不可用"));
        assert!(output.contains("## 当前状态"));
        assert!(output.contains("## 对本轮回答的影响"));
        assert!(output.contains("## 下一步"));
        assert!(output.contains("- 已收到你的问题和本轮选择的资料。"));
        assert!(!output.contains("services.toml"));
        assert!(!output.contains("[agent_llm_usage]"));
        assert!(!output.contains("API Key"));
        assert!(!output.contains("没有拿到可直接展示的完整内容"));
    }

    #[test]
    fn task_graph_synthesis_surfaces_llm_provider_error_before_empty_fallback() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-provider-error".to_string()),
            root_task: "Build a plan.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime.states.insert(
            data.id.clone(),
            TaskNodeRuntimeState::Failed(
                "worker reported failed status; llm provider error: upstream model service \
                 unavailable; structured role output is missing"
                    .to_string(),
            ),
        );
        runtime.states.insert(
            ops.id.clone(),
            TaskNodeRuntimeState::Skipped("dependency did not pass".to_string()),
        );
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("failure_kind".to_string(), "llm_provider_error".to_string());
        fields.insert("status".to_string(), "failed".to_string());
        fields.insert("role".to_string(), "data".to_string());
        runtime.reports.insert(
            data.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-provider-error".to_string()),
                node_id: data.id.clone(),
                task_id: TaskId("task-data".to_string()),
                worker_id: WorkerId("worker.data".to_string()),
                role: "data".to_string(),
                content: "llm_provider_error: 当前 AI 模型服务临时不可用".to_string(),
                role_output: None,
                evidence: vec!["failure_kind:llm_provider_error".to_string()],
                risks: Vec::new(),
                open_questions: vec!["llm_provider_error: 当前 AI 模型服务临时不可用".to_string()],
                diagnostics: vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "llm_provider_error".to_string(),
                    fields,
                }],
                status: WorkerReportStatus::Failed,
            },
        );

        let output = runtime.synthesize();

        assert!(output.contains("模型服务临时不可用"));
        assert!(output.contains("## 当前状态"));
        assert!(output.contains("## 对本轮回答的影响"));
        assert!(output.contains("## 下一步"));
        assert!(output.contains("- 已收到你的问题和本轮选择的资料。"));
        assert!(!output.contains("services.toml"));
        assert!(!output.contains("[agent_llm_usage]"));
        assert!(!output.contains("base_url"));
        assert!(!output.contains("structured role output"));
        assert!(!output.contains("没有拿到可直接展示的完整内容"));
    }

    #[test]
    fn task_graph_synthesis_uses_safe_gap_message_when_roles_do_not_run() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-worker-unavailable".to_string()),
            root_task: "请数据和运营一起判断这个商品怎么卖。".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime.states.insert(
            data.id.clone(),
            TaskNodeRuntimeState::Failed(
                "worker reported failed status; failure_kind:worker_unavailable".to_string(),
            ),
        );
        runtime.states.insert(
            ops.id.clone(),
            TaskNodeRuntimeState::Skipped("dependency did not pass".to_string()),
        );
        runtime.reports.insert(
            data.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-worker-unavailable".to_string()),
                node_id: data.id.clone(),
                task_id: TaskId("task-data".to_string()),
                worker_id: WorkerId("worker.data".to_string()),
                role: "data".to_string(),
                content: "data 岗位未能执行：当前服务端没有注册可用的该岗位执行器。".to_string(),
                role_output: Some(RoleWorkOutput {
                    summary: "data 岗位未能执行：当前服务端没有注册可用的该岗位执行器。"
                        .to_string(),
                    evidence: vec!["failure_kind:worker_unavailable".to_string()],
                    ..RoleWorkOutput::default()
                }),
                evidence: vec!["failure_kind:worker_unavailable".to_string()],
                risks: Vec::new(),
                open_questions: Vec::new(),
                diagnostics: Vec::new(),
                status: WorkerReportStatus::Failed,
            },
        );

        let output = runtime.synthesize();

        assert!(output.contains("## 当前可先交付"));
        assert!(output.contains("## 先做什么"));
        assert!(output.contains("## 本轮边界"));
        assert!(output.contains("低风险可执行版本"));
        assert!(output.contains("部分协作步骤没有形成可用结果"));
        assert!(!output.contains("没有形成完整答案"));
        assert!(!output.contains("data"));
        assert!(!output.contains("服务端没有可用的该岗位执行器"));
        assert!(!output.contains("ops"));
        assert!(!output.contains("上游依赖岗位未成功完成"));
        assert!(!output.contains("没有拿到可直接展示的完整内容"));
        assert!(!output.contains("failure_kind"));
    }

    #[test]
    fn task_graph_gap_message_still_gives_actionable_draft_for_rich_followup() {
        let data = node("data", "资料整理", "data");
        let mut ops = node("ops_final", "运营总方案", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-rich-followup-gap".to_string()),
            root_task: "我不会给你提供任何其他信息，但是你必须根据我所给资料进行丰富回答"
                .to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime.states.insert(
            data.id.clone(),
            TaskNodeRuntimeState::Failed("worker cognition timed out".to_string()),
        );
        runtime.states.insert(
            ops.id.clone(),
            TaskNodeRuntimeState::Skipped("dependency did not pass".to_string()),
        );

        let output = runtime.synthesize();

        assert!(output.contains("## 当前可先交付"), "{output}");
        assert!(output.contains("## 先做什么"), "{output}");
        assert!(output.contains("## 风险边界"), "{output}");
        assert!(output.contains("## 本轮边界"), "{output}");
        assert!(output.contains("可执行草案"), "{output}");
        assert!(output.contains("真实资料"), "{output}");
        assert!(!output.contains("请直接补充新的要求"), "{output}");
        assert!(!output.contains("没有形成完整答案"), "{output}");
        assert!(!output.contains("dependency did not pass"), "{output}");
        assert!(!output.contains("worker cognition timed out"), "{output}");
    }

    #[test]
    fn forced_selected_roles_graph_only_uses_manual_roles() {
        let roles = vec!["creative".to_string(), "design".to_string()];
        let graph = plan_forced_selected_roles_graph(
            &TaskId("task-manual-image".to_string()),
            "只按创意和设计岗位分析这张主图。",
            &roles,
        );

        let graph_roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(graph_roles, vec!["creative", "design", "creative"]);
        assert!(graph.nodes.iter().all(|node| node.role != "data"));
        assert!(graph.nodes.iter().all(|node| node.role != "accounting"));
        assert_eq!(graph.nodes[2].id.0, "creative_final");
        assert_eq!(
            graph.nodes[2]
                .input_refs
                .iter()
                .map(|input| input.0.as_str())
                .collect::<Vec<_>>(),
            vec!["creative", "design"]
        );
    }

    #[test]
    fn forced_selected_roles_graph_uses_business_role_as_final_when_search_selected() {
        let roles = vec!["search".to_string(), "ops".to_string()];
        let graph = plan_forced_selected_roles_graph(
            &TaskId("task-manual-search-ops".to_string()),
            "手动指定 search 和 ops，搜索竞品后给运营建议。",
            &roles,
        );

        let graph_roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(graph_roles, vec!["search", "ops", "ops"]);
        assert_eq!(graph.nodes[2].id.0, "ops_final");
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(true));
        assert_eq!(
            graph.nodes[2]
                .input_refs
                .iter()
                .map(|input| input.0.as_str())
                .collect::<Vec<_>>(),
            vec!["search", "ops"]
        );
    }

    #[test]
    fn task_graph_synthesis_prefers_terminal_final_node() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-final".to_string()),
            root_task: "Build a plan.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(data.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .states
            .insert(ops.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .reports
            .insert(data.id.clone(), report("data", "upstream draft"));
        runtime
            .reports
            .insert(ops.id.clone(), report("ops_final", "final user plan"));

        let output = runtime.synthesize();

        assert_eq!(output, "final user plan");
        assert!(!output.contains("upstream draft"));
    }

    #[test]
    fn task_graph_synthesis_replaces_material_gap_final_with_structured_answer() {
        let data = node("data", "Data", "data");
        let creative = {
            let mut node = node("creative", "Creative", "creative");
            node.input_refs = vec![TaskNodeId("data".to_string())];
            node
        };
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![
            TaskNodeId("data".to_string()),
            TaskNodeId("creative".to_string()),
        ];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-material-gap-final".to_string()),
            root_task: "学生书包开学季小促销方案，要求有表格、有标题示例、有风险边界。".to_string(),
            nodes: vec![data.clone(), creative.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(data.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .states
            .insert(creative.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .states
            .insert(ops.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            data.id.clone(),
            report("data", "先看点击、加购、付款和退款反馈。"),
        );
        runtime.reports.insert(
            creative.id.clone(),
            report(
                "creative",
                "标题先突出开学收纳、分区和减负，但不编材质规格。",
            ),
        );
        runtime.reports.insert(
            ops.id.clone(),
            report(
                "ops_final",
                "该岗位本轮未形成有效结论。未找到可用的附件或表格数据。\
                 等待用户提供附件或表格数据。无法进行进一步分析，因为缺少必要的数据。",
            ),
        );

        let output = runtime.synthesize();

        assert!(output.contains("## 当前可先交付"));
        assert!(output.contains("## 执行表"));
        assert!(output.contains("| 环节 | 当前可做 | 复核信息 | 风险边界 |"));
        assert!(output.contains("可执行表格草案"));
        assert!(!output.contains("本轮没有可读取的上传表格或附件"));
        assert!(!output.contains("等待用户提供附件或表格数据"));
        assert!(!output.contains("未找到可用的附件或表格数据"));
        assert!(!output.contains("无法进行进一步分析"));
    }

    #[test]
    fn task_graph_synthesis_replaces_new_refusal_gap_with_markdown_table() {
        let data = node("data", "Data", "data");
        let creative = {
            let mut node = node("creative", "Creative", "creative");
            node.input_refs = vec![TaskNodeId("data".to_string())];
            node
        };
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![
            TaskNodeId("data".to_string()),
            TaskNodeId("creative".to_string()),
        ];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-new-gap-final".to_string()),
            root_task: "现在切换成手动多智能体：数据、创意、\
                        运营一起给一个学生书包开学季小促销方案，要求有表格、有标题示例、\
                        有风险边界。"
                .to_string(),
            nodes: vec![data.clone(), creative.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(data.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .states
            .insert(creative.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .states
            .insert(ops.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            data.id.clone(),
            report("data", "先按曝光、点击、加购、付款和退款复核。"),
        );
        runtime.reports.insert(
            creative.id.clone(),
            report("creative", "标题示例：开学装书不乱，一包分区搞定。"),
        );
        runtime.reports.insert(
            ops.id.clone(),
            report(
                "ops_final",
                "未提供具体商品信息、销售数据、库存情况或平台规则，因此无法生成完整促销方案。\
                 建议用户提供附件或表格后再继续。",
            ),
        );

        let output = runtime.synthesize();

        assert!(output.contains("## 当前可先交付"), "{output}");
        assert!(output.contains("## 执行表"), "{output}");
        assert!(
            output.contains("| 环节 | 当前可做 | 复核信息 | 风险边界 |"),
            "{output}"
        );
        assert!(output.contains("可执行表格草案"), "{output}");
        assert!(!output.contains("本轮没有可读取的上传表格或附件"));
        assert!(!output.contains("无法生成完整促销方案"));
        assert!(!output.contains("建议用户提供附件"));
    }

    #[test]
    fn task_graph_synthesis_outputs_table_for_no_upload_table_format_request() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-format-table-gap".to_string()),
            root_task: "不上传任何表格，只要求你把学生书包的分析输出成表格：列出指标、怎么看、\
                        可能问题、下一步动作。"
                .to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(data.id.clone(), TaskNodeRuntimeState::Passed);
        runtime
            .states
            .insert(ops.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            data.id.clone(),
            report("data", "先用指标表说明怎么观察学生书包。"),
        );
        runtime.reports.insert(
            ops.id.clone(),
            report(
                "ops_final",
                "由于未上传任何表格、Excel、CSV \
                 或附件，无法直接生成具体数据表格。用户是否上传了表格？",
            ),
        );

        let output = runtime.synthesize();

        assert!(output.contains("## 执行表"), "{output}");
        assert!(
            output.contains("| 指标 | 怎么看 | 可能问题 | 下一步动作 |"),
            "{output}"
        );
        assert!(output.contains("可执行表格草案"), "{output}");
        assert!(!output.contains("本轮没有可读取的上传表格或附件"));
        assert!(!output.contains("由于未上传任何表格"));
        assert!(!output.contains("用户是否上传了表格"));
    }

    #[test]
    fn task_graph_synthesis_hides_failed_descendant_reason() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-partial".to_string()),
            root_task: "Build a plan.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(data.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.states.insert(
            ops.id.clone(),
            TaskNodeRuntimeState::Failed("dependency did not pass".to_string()),
        );
        runtime
            .reports
            .insert(data.id.clone(), report("data", "usable upstream answer"));

        let output = runtime.synthesize();

        assert_eq!(output, "usable upstream answer");
        assert!(!output.contains("dependency did not pass"));
        assert!(!output.contains("还有这些部分需要补齐"));
    }

    #[test]
    fn timed_out_worker_report_fails_node_and_skips_dependents() {
        let search = node("search", "Search", "search");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("search".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-timeout".to_string()),
            root_task: "Search then synthesize.".to_string(),
            nodes: vec![search.clone(), ops.clone()],
            adaptive_hint: None,
        });
        let assignment = runtime.build_assignment(&search);
        runtime.mark_running(&assignment);

        let outcome = runtime.apply_report(WorkerReport {
            graph_id: TaskGraphId("graph-timeout".to_string()),
            node_id: search.id.clone(),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: "worker cognition timed out after 35s".to_string(),
            role_output: None,
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: vec!["worker cognition timed out after 35s".to_string()],
            diagnostics: Vec::new(),
            status: WorkerReportStatus::TimedOut,
        });

        assert!(matches!(outcome, super::EvaluationOutcome::Failed(_)));
        assert!(matches!(
            runtime.states.get(&search.id),
            Some(TaskNodeRuntimeState::Failed(reason)) if reason.contains("timed out")
        ));
        assert!(matches!(
            runtime.states.get(&ops.id),
            Some(TaskNodeRuntimeState::Skipped(reason)) if reason.contains("dependency")
        ));

        let process = runtime.process_summary();
        assert!(
            process
                .incomplete
                .iter()
                .any(|gap| gap.node_id == search.id && gap.reason == "协作步骤执行超时")
        );
    }

    #[test]
    fn process_summary_adds_readable_search_check_item_from_trace() {
        let search = node("search", "搜索证据", "search");
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-search-process".to_string()),
            root_task: "Search competitor prices.".to_string(),
            nodes: vec![search.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(search.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            search.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-search-process".to_string()),
                node_id: search.id.clone(),
                task_id: TaskId("task-search".to_string()),
                worker_id: WorkerId("worker.search".to_string()),
                role: "search".to_string(),
                content: String::new(),
                role_output: Some(RoleWorkOutput {
                    summary: "本轮已完成搜索核对。".to_string(),
                    evidence: vec![
                        "search_evidence_packet:{\"sources\":[{\"url\":\"https://example.com/raw\"}]}".to_string(),
                        "search_evidence_trace: skill=search_competitor status=success \
                         adapter_status=live intent=competitor_price_or_benchmark strength=standard \
                         sources=2 usable_claims=2 gaps=0 confidence=0.72 query=\"冷萃咖啡 竞品价格\""
                            .to_string(),
                    ],
                    ..RoleWorkOutput::default()
                }),
                evidence: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
                diagnostics: Vec::new(),
                status: WorkerReportStatus::Completed,
            },
        );

        let process = runtime.process_summary();
        let item = process
            .check_items
            .iter()
            .find(|item| item.contains("搜索核对"))
            .expect("search check item should be exposed in process summary");

        assert!(item.contains("搜索证据 / search_competitor"));
        assert!(item.contains("状态 live"));
        assert!(item.contains("来源 2"));
        assert!(item.contains("可用结论 2"));
        assert!(item.contains("缺口 0"));
        assert!(item.contains("置信度 0.72"));
        assert!(!item.contains("search_evidence_packet"));
        assert!(!item.contains("search_evidence_trace"));
        assert!(!item.contains("https://example.com/raw"));
    }

    #[test]
    fn process_summary_prefers_typed_search_diagnostic_check_item() {
        let search = node("search", "搜索证据", "search");
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-search-diagnostic-process".to_string()),
            root_task: "Search competitor prices.".to_string(),
            nodes: vec![search.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(search.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            search.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-search-diagnostic-process".to_string()),
                node_id: search.id.clone(),
                task_id: TaskId("task-search".to_string()),
                worker_id: WorkerId("worker.search".to_string()),
                role: "search".to_string(),
                content: String::new(),
                role_output: Some(RoleWorkOutput {
                    summary: "本轮已完成搜索核对。".to_string(),
                    evidence: vec![
                        "search_evidence_packet:{\"sources\":[{\"url\":\"https://example.com/raw\"}]}".to_string(),
                    ],
                    ..RoleWorkOutput::default()
                }),
                evidence: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
                diagnostics: vec![WorkerDiagnostic {
                    kind: "search_evidence".to_string(),
                    source: "search_competitor".to_string(),
                    fields: BTreeMap::from([
                        ("skill".to_string(), "search_competitor".to_string()),
                        ("adapter_status".to_string(), "degraded".to_string()),
                        (
                            "intent".to_string(),
                            "competitor_price_or_benchmark".to_string(),
                        ),
                        ("strength".to_string(), "standard".to_string()),
                        ("sources".to_string(), "0".to_string()),
                        ("usable_claims".to_string(), "0".to_string()),
                        ("gaps".to_string(), "1".to_string()),
                        ("confidence".to_string(), "0.00".to_string()),
                    ]),
                }],
                status: WorkerReportStatus::Completed,
            },
        );

        let process = runtime.process_summary();
        let item = process
            .check_items
            .iter()
            .find(|item| item.contains("搜索核对"))
            .expect("typed search diagnostic should be exposed in process summary");

        assert!(item.contains("搜索证据 / search_competitor"));
        assert!(item.contains("状态 degraded"));
        assert!(item.contains("来源 0"));
        assert!(item.contains("可用结论 0"));
        assert!(item.contains("缺口 1"));
        assert!(item.contains("置信度 0.00"));
        assert!(!item.contains("search_evidence_packet"));
        assert!(!item.contains("search_evidence_trace"));
        assert!(!item.contains("https://example.com/raw"));
        assert_eq!(
            process
                .check_items
                .iter()
                .filter(|item| item.contains("搜索核对"))
                .count(),
            1,
            "typed diagnostic should be the primary path and avoid duplicate trace fallback items"
        );
    }

    #[test]
    fn process_summary_adds_readable_execution_gap_check_items_without_internal_roles() {
        let data = node("data", "数据诊断", "data");
        let mut ops = node("ops_final", "运营总方案", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-role-gap-process".to_string()),
            root_task: "Data then ops.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });
        runtime.states.insert(
            data.id.clone(),
            TaskNodeRuntimeState::Failed(
                "worker reported failed status; failure_kind:worker_unavailable".to_string(),
            ),
        );
        runtime.states.insert(
            ops.id.clone(),
            TaskNodeRuntimeState::Skipped("dependency did not pass".to_string()),
        );

        let process = runtime.process_summary();

        assert!(
            process
                .check_items
                .iter()
                .any(|item| item.contains("协作核对：数据诊断 未完成")
                    && item.contains("协作步骤没有成功启动"))
        );
        assert!(
            process
                .check_items
                .iter()
                .any(|item| item.contains("协作核对：运营总方案 未完成")
                    && item.contains("前置协作步骤没有完成"))
        );
        assert!(process.incomplete.iter().any(|gap| gap.role == "data"
            && gap.role_label == "数据诊断"
            && gap.state == "failed"
            && gap.reason == "协作步骤没有成功启动"));
        let rendered = process.check_items.join("\n");
        assert!(!rendered.contains("岗位核对"));
        assert!(!rendered.contains("data"));
        assert!(!rendered.contains("ops"));
        assert!(!rendered.contains("岗位执行器"));
        assert!(!rendered.contains("failure_kind"));
    }

    #[test]
    fn typed_worker_unavailable_diagnostic_fails_node_and_adds_check_item() {
        let data = node("data", "数据诊断", "data");
        let mut ops = node("ops_final", "运营总方案", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-typed-worker-unavailable".to_string()),
            root_task: "Data then ops.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
            adaptive_hint: None,
        });

        let outcome = runtime.apply_report(WorkerReport {
            graph_id: TaskGraphId("graph-typed-worker-unavailable".to_string()),
            node_id: data.id.clone(),
            task_id: TaskId("task-data".to_string()),
            worker_id: WorkerId("worker.data".to_string()),
            role: "data".to_string(),
            content: "data 岗位未能执行。".to_string(),
            role_output: Some(RoleWorkOutput {
                summary: "data 岗位未能执行。".to_string(),
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: vec![WorkerDiagnostic {
                kind: "failure".to_string(),
                source: "worker_unavailable".to_string(),
                fields: BTreeMap::from([
                    ("failure_kind".to_string(), "worker_unavailable".to_string()),
                    ("status".to_string(), "failed".to_string()),
                    ("role".to_string(), "data".to_string()),
                ]),
            }],
            status: WorkerReportStatus::Failed,
        });

        assert!(matches!(outcome, super::EvaluationOutcome::Failed(_)));
        assert!(matches!(
            runtime.states.get(&data.id),
            Some(TaskNodeRuntimeState::Failed(reason))
                if reason.contains("worker unavailable")
        ));
        assert!(matches!(
            runtime.states.get(&ops.id),
            Some(TaskNodeRuntimeState::Skipped(reason)) if reason.contains("dependency")
        ));

        let process = runtime.process_summary();
        assert!(
            process
                .check_items
                .iter()
                .any(|item| item.contains("协作核对：数据诊断 未完成")
                    && item.contains("协作步骤没有成功启动"))
        );
        assert!(process.incomplete.iter().any(|gap| {
            gap.role == "data"
                && gap.role_label == "数据诊断"
                && gap.reason == "协作步骤没有成功启动"
        }));
    }

    #[test]
    fn process_summary_adds_readable_skill_failure_check_item_from_typed_diagnostic() {
        let ops = node("ops", "运营方案", "ops");
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-skill-failure-process".to_string()),
            root_task: "Create an execution plan.".to_string(),
            nodes: vec![ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(ops.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            ops.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-skill-failure-process".to_string()),
                node_id: ops.id.clone(),
                task_id: TaskId("task-ops".to_string()),
                worker_id: WorkerId("worker.ops".to_string()),
                role: "ops".to_string(),
                content: "运营方案已生成，但平台执行 Skill 未完成。".to_string(),
                role_output: Some(RoleWorkOutput {
                    summary: "运营方案已生成，但平台执行 Skill 未完成。".to_string(),
                    ..RoleWorkOutput::default()
                }),
                evidence: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
                diagnostics: vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "skill_adapter_unavailable".to_string(),
                    fields: BTreeMap::from([
                        (
                            "failure_kind".to_string(),
                            "skill_adapter_unavailable".to_string(),
                        ),
                        ("status".to_string(), "deferred".to_string()),
                        ("skill".to_string(), "platform_update_price".to_string()),
                        ("request_id".to_string(), "task-ops-skill-1".to_string()),
                        (
                            "reason".to_string(),
                            "Platform execution adapter is not implemented yet.".to_string(),
                        ),
                    ]),
                }],
                status: WorkerReportStatus::Completed,
            },
        );

        let process = runtime.process_summary();

        assert!(process.check_items.iter().any(|item| {
            item.contains("Skill核对：运营方案 / platform_update_price 未完成")
                && item.contains("状态 deferred")
                && item.contains("Skill 适配器未接入或当前不可用")
        }));
        assert!(
            process
                .check_items
                .iter()
                .all(|item| !item.contains("Platform execution adapter"))
        );
    }

    #[test]
    fn process_summary_adds_readable_context_pack_failure_check_item_from_typed_diagnostic() {
        let ops = node("ops", "运营方案", "ops");
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-context-pack-process".to_string()),
            root_task: "读取资料并给出运营建议。".to_string(),
            nodes: vec![ops.clone()],
            adaptive_hint: None,
        });
        runtime
            .states
            .insert(ops.id.clone(), TaskNodeRuntimeState::Passed);
        runtime.reports.insert(
            ops.id.clone(),
            WorkerReport {
                graph_id: TaskGraphId("graph-context-pack-process".to_string()),
                node_id: ops.id.clone(),
                task_id: TaskId("task-ops".to_string()),
                worker_id: WorkerId("worker.ops".to_string()),
                role: "ops".to_string(),
                content: "已基于可读资料生成运营建议。".to_string(),
                role_output: Some(RoleWorkOutput {
                    summary: "已基于可读资料生成运营建议。".to_string(),
                    ..RoleWorkOutput::default()
                }),
                evidence: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
                diagnostics: vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "context_pack_extraction_failed".to_string(),
                    fields: BTreeMap::from([
                        (
                            "failure_kind".to_string(),
                            "context_pack_extraction_failed".to_string(),
                        ),
                        ("readiness".to_string(), "partial".to_string()),
                        ("item_count".to_string(), "2".to_string()),
                        ("omitted_count".to_string(), "1".to_string()),
                        ("extraction_pending_count".to_string(), "0".to_string()),
                        ("extraction_failed_count".to_string(), "1".to_string()),
                        ("extraction_unsupported_count".to_string(), "0".to_string()),
                        (
                            "warning_preview".to_string(),
                            "adapter raw failed: object-key=secret".to_string(),
                        ),
                    ]),
                }],
                status: WorkerReportStatus::Completed,
            },
        );

        let process = runtime.process_summary();

        assert!(process.check_items.iter().any(|item| {
            item.contains("资料核对：运营方案 的资料包状态 partial")
                && item.contains("进入上下文 2 项")
                && item.contains("省略 1 项")
                && item.contains("解析失败 1 项")
                && item.contains("部分资料抽取失败，不能当作已读取正文")
        }));
        assert!(
            process
                .check_items
                .iter()
                .all(|item| !item.contains("object-key=secret"))
        );
    }

    #[test]
    fn quick_task_graph_synthesis_dedupes_repeated_cross_worker_blocks() {
        let ops = node("ops", "Ops", "ops");
        let data = node("data", "Data", "data");
        let creative = node("creative", "Creative", "creative");
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-quick".to_string()),
            root_task: "快速判断这个冷萃咖啡怎么卖。".to_string(),
            nodes: vec![ops.clone(), data.clone(), creative.clone()],
            adaptive_hint: None,
        })
        .with_response_mode(Some("quick"));
        for node in [&ops, &data, &creative] {
            runtime
                .states
                .insert(node.id.clone(), TaskNodeRuntimeState::Passed);
        }
        runtime.reports.insert(
            ops.id.clone(),
            report(
                "ops",
                "先看毛利，再看销量。\n- 补齐成本字段。\n\n运营上先做小范围试卖。",
            ),
        );
        runtime.reports.insert(
            data.id.clone(),
            report("data", "先看毛利，再看销量。\n- 补齐成本字段。"),
        );
        runtime.reports.insert(
            creative.id.clone(),
            report("creative", "标题先突出低糖冷萃和校园场景。"),
        );

        let output = runtime.synthesize();

        assert_eq!(
            output.matches("先看毛利，再看销量。").count(),
            1,
            "{output}"
        );
        assert!(output.contains("运营上先做小范围试卖。"));
        assert!(output.contains("标题先突出低糖冷萃和校园场景。"));
    }

    #[test]
    fn non_quick_graph_enriches_thin_final_with_upstream_role_outputs() {
        let data = node("data", "数据诊断", "data");
        let design = node("design", "主图诊断", "design");
        let mut ops_final = node("ops_final", "运营总方案", "ops");
        ops_final.input_refs = vec![data.id.clone(), design.id.clone()];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-thin-final".to_string()),
            root_task: "给出复盘：稳定部分、风险部分、下一步。".to_string(),
            nodes: vec![data.clone(), design.clone(), ops_final.clone()],
            adaptive_hint: None,
        })
        .with_response_mode(Some("learning"));
        for node in [&data, &design, &ops_final] {
            runtime
                .states
                .insert(node.id.clone(), TaskNodeRuntimeState::Passed);
        }
        runtime.reports.insert(
            data.id.clone(),
            WorkerReport {
                role_output: Some(RoleWorkOutput {
                    summary: "已经稳定：销量、转化率、用户评价这三个观察指标已经明确。".to_string(),
                    findings: vec![
                        "数据风险：仍缺少真实曝光、点击、成交和退款数据，不能验证优化效果。"
                            .to_string(),
                    ],
                    recommendations: vec![
                        "下一步：先补齐一周样本数据，再按指标做对比复盘。".to_string(),
                    ],
                    ..RoleWorkOutput::default()
                }),
                ..report("data", "")
            },
        );
        runtime.reports.insert(
            design.id.clone(),
            WorkerReport {
                role_output: Some(RoleWorkOutput {
                    summary: "主图已能看出商品和赠品，但礼盒感、文字大小仍需要核对。".to_string(),
                    recommendations: vec!["下一步：用两版主图做点击率小测试。".to_string()],
                    ..RoleWorkOutput::default()
                }),
                ..report("design", "")
            },
        );
        runtime.reports.insert(
            ops_final.id.clone(),
            report("ops_final", "学生下一步：收集实际数据并验证优化效果。"),
        );

        let output = runtime.synthesize();

        assert!(output.contains("学生下一步："), "{output}");
        assert!(output.contains("收集实际数据并验证优化效果。"), "{output}");
        assert!(output.contains("已经稳定：销量、转化率、用户评价这三个观察指标已经明确。"));
        assert!(output.contains("数据风险："));
        assert!(output.contains("仍缺少真实曝光、点击、成交和退款数据"));
        assert!(output.contains("主图已能看出商品和赠品"));
        assert!(output.contains("用两版主图做点击率小测试。"));
        assert!(!output.contains("role_output"));
        assert!(!output.contains("worker."));
    }

    #[test]
    fn non_quick_graph_keeps_complete_final_without_upstream_duplication() {
        let data = node("data", "数据诊断", "data");
        let mut ops_final = node("ops_final", "运营总方案", "ops");
        ops_final.input_refs = vec![data.id.clone()];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-complete-final".to_string()),
            root_task: "整理完整方案。".to_string(),
            nodes: vec![data.clone(), ops_final.clone()],
            adaptive_hint: None,
        })
        .with_response_mode(Some("execution"));
        for node in [&data, &ops_final] {
            runtime
                .states
                .insert(node.id.clone(), TaskNodeRuntimeState::Passed);
        }
        runtime.reports.insert(
            data.id.clone(),
            report("data", "上游数据草稿不应重复出现在完整最终答案里。"),
        );
        runtime.reports.insert(
            ops_final.id.clone(),
            report(
                "ops_final",
                "## 已经稳定的部分\n\n商品指标、活动预算和目标已经明确，可以进入小范围验证。\n\n## 仍有风险的部分\n\n真实曝光、点击、成交、退款数据还不足，不能直接判断方案一定有效。\n\n## 学生下一步怎么做\n\n先补齐一周样本数据，再用两个版本主图和活动话术做对比测试，最后按销量、转化率和评价复盘。",
            ),
        );

        let output = runtime.synthesize();

        assert!(output.contains("## 已经稳定的部分"));
        assert!(output.contains("## 仍有风险的部分"));
        assert!(output.contains("## 学生下一步怎么做"));
        assert!(!output.contains("上游数据草稿不应重复"));
    }

    fn node(id: &str, title: &str, role: &str) -> TaskNode {
        TaskNode {
            id: TaskNodeId(id.to_string()),
            role: role.to_string(),
            worker_id: WorkerId(format!("worker.{role}")),
            title: title.to_string(),
            objective: "输出用户可直接理解的内容。".to_string(),
            llm_usage: None,
            skill_autosuggest: None,
            timeout_sec: None,
            input_refs: Vec::new(),
            acceptance: Vec::new(),
            max_attempts: 1,
        }
    }

    fn report(node_id: &str, summary: &str) -> WorkerReport {
        WorkerReport {
            graph_id: TaskGraphId("graph-user-facing".to_string()),
            node_id: TaskNodeId(node_id.to_string()),
            task_id: TaskId(format!("task-{node_id}")),
            worker_id: WorkerId(format!("worker.{node_id}")),
            role: node_id.to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: summary.to_string(),
                findings: Vec::new(),
                recommendations: Vec::new(),
                evidence: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        }
    }
}
