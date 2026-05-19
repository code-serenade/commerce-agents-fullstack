use std::collections::{HashMap, HashSet};

use super::{
    DEFAULT_NODE_ACCEPTANCE, chat, planner, policy,
    policy::{EvaluationPolicy, SpecificContentSource},
    util,
};
use crate::{
    agent_session::{RoleMemorySummary, TaskGraphMemorySummary, WorkerDetail},
    core::protocol::{
        Evaluation, RoleWorkOutput, TaskGraph, TaskGraphHandoff, TaskGraphProcessGap,
        TaskGraphProcessPanel, TaskGraphProcessRole, TaskGraphProcessSummary, TaskId, TaskNode,
        TaskNodeId, WorkerAssignment, WorkerReport, WorkerReportStatus,
    },
    roles::RoleRoute,
};

const CATEGORY_LEVEL_BOUNDARY_MARKER: &str = "category_level_boundary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskGraphAnswerMode {
    Learning,
    Quick,
    Execution,
}

impl TaskGraphAnswerMode {
    fn from_metadata(value: Option<&str>) -> Self {
        match value {
            Some("quick") => Self::Quick,
            Some("execution") | Some("collab") => Self::Execution,
            _ => Self::Learning,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskGraphLearningDepth {
    Low,
    Medium,
    High,
}

impl TaskGraphLearningDepth {
    fn from_metadata(value: Option<&str>) -> Self {
        match value {
            Some("low") | Some("beginner") | Some("entry") => Self::Low,
            Some("professional") | Some("high") | Some("advanced") => Self::High,
            _ => Self::Medium,
        }
    }
}

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
    answer_mode: TaskGraphAnswerMode,
    learning_depth: TaskGraphLearningDepth,
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
            answer_mode: TaskGraphAnswerMode::Learning,
            learning_depth: TaskGraphLearningDepth::Medium,
        }
    }

    pub fn with_response_mode(mut self, response_mode: Option<&str>) -> Self {
        self.answer_mode = TaskGraphAnswerMode::from_metadata(response_mode);
        self
    }

    pub fn with_learning_depth(mut self, learning_depth: Option<&str>) -> Self {
        self.learning_depth = TaskGraphLearningDepth::from_metadata(learning_depth);
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
            objective: node.objective.clone(),
            inputs,
            rework_instruction: self.rework_instructions.get(&node.id).cloned(),
        }
    }

    pub fn mark_running(&mut self, assignment: &WorkerAssignment) {
        self.attempts
            .insert(assignment.node_id.clone(), assignment.attempt);
        self.states
            .insert(assignment.node_id.clone(), TaskNodeRuntimeState::Running);
    }

    pub fn apply_report(&mut self, report: WorkerReport) -> EvaluationOutcome {
        let node = self
            .graph
            .nodes
            .iter()
            .find(|node| node.id == report.node_id)
            .cloned();
        let evaluation =
            evaluate_report(&report, node.as_ref(), &self.reports, &self.graph.root_task);
        self.evaluations.push(evaluation.clone());
        self.reports.insert(report.node_id.clone(), report.clone());

        let Some(node) = node else {
            self.states.insert(
                report.node_id.clone(),
                TaskNodeRuntimeState::Failed("node not found".to_string()),
            );
            return EvaluationOutcome::Failed(evaluation);
        };

        if should_accept_boundary_report(&report, &node, &evaluation, &self.graph.root_task) {
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
                .unwrap_or_else(|| "请补齐节点产物，避免空内容和占位内容。".to_string());
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
        let passed_nodes = self
            .graph
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Passed)
                )
            })
            .collect::<Vec<_>>();
        let primary_node = select_primary_answer_node(&self.graph.nodes, &passed_nodes)
            .or_else(|| self.graph.nodes.first().filter(|node| node.role != "chat"));
        let support_count = passed_nodes
            .iter()
            .filter(|node| {
                primary_node
                    .map(|primary| node.id != primary.id && node.role != "chat")
                    .unwrap_or(true)
            })
            .count();
        let mut lines = Vec::new();
        let team_playbook = ecommerce_team_playbook(&self.graph.root_task, &passed_nodes);

        lines.push("# 多智能体协作结论".to_string());
        lines.push(mode_intro_line(self.answer_mode).to_string());
        lines.push(String::new());
        lines.push(format!("**用户任务**：{}", self.graph.root_task));
        if let Some(product_context) = product_context_line(&self.graph.root_task, &self.reports) {
            lines.push(product_context);
        }
        let team_overview = team_overview_line(&passed_nodes, primary_node);
        if !team_overview.is_empty() {
            lines.push(team_overview);
        }
        lines.push(String::new());

        let team_contract_lines = team_collaboration_contract_lines(
            &self.graph.root_task,
            primary_node,
            &passed_nodes,
            self.answer_mode,
        );
        let deterministic_quality_lines =
            deterministic_answer_quality_lines(&self.graph.root_task, primary_node, &passed_nodes);
        let direct_team_answer = direct_team_answer(
            &self.graph.root_task,
            primary_node,
            &passed_nodes,
            &self.reports,
        );
        let primary_passed_node =
            primary_node.filter(|node| passed_nodes.iter().any(|passed| passed.id == node.id));
        let primary_summary =
            primary_answer_summary(primary_passed_node, &passed_nodes, &self.reports);

        lines.push("## 总回答".to_string());
        lines.extend(total_answer_brief_lines(
            &self.graph.root_task,
            primary_node,
            primary_summary.as_deref(),
            support_count,
            self.answer_mode,
        ));
        lines.push(String::new());

        lines.push("## 正文分析".to_string());
        match self.answer_mode {
            TaskGraphAnswerMode::Quick => {
                lines.push("快速模式只保留关键原因和下一步，不展开完整教学过程。".to_string());
                let quick_causes = quick_reason_lines(primary_node, &passed_nodes, &self.reports);
                if quick_causes.is_empty() {
                    lines
                        .push("- 当前可用上下文较少，先按已有岗位产物给出方向性判断。".to_string());
                } else {
                    lines.extend(quick_causes.into_iter().take(3));
                }
            }
            TaskGraphAnswerMode::Learning => {
                let teaching_lines = teaching_explanation_lines(
                    &self.graph.root_task,
                    primary_node,
                    &passed_nodes,
                    self.learning_depth,
                );
                if teaching_lines.is_empty() {
                    lines.push(
                        "先把问题拆成指标、原因、动作和风险四层，再按岗位分工理解答案。"
                            .to_string(),
                    );
                } else {
                    lines.extend(teaching_lines);
                }
            }
            TaskGraphAnswerMode::Execution => {
                lines.push(
                    "丰富执行模式会把判断转成责任岗位、动作、验收方式和风险边界。".to_string(),
                );
                let execution_context = execution_context_lines(&passed_nodes, &self.reports);
                lines.extend(execution_context.into_iter().take(5));
            }
        }
        if !matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
            if !direct_team_answer.is_empty() {
                lines.push(String::new());
                lines.push("### 业务拆解".to_string());
                lines.extend(direct_team_answer.iter().cloned());
            }
            if let Some(summary) = primary_summary.as_deref() {
                lines.push(String::new());
                lines.push("### 主岗摘要".to_string());
                lines.push(summary.to_string());
            }
            if !team_playbook.is_empty() {
                lines.extend(team_playbook.iter().cloned());
            }
            if support_count > 0 {
                let support_scopes = support_scope_summary(&passed_nodes, primary_node);
                lines.push(format!(
                    "本轮由{}牵头，{} \
                     个协作岗位分别从{}补充判断；各岗位先独立完成自己的判断，\
                     再把结论交给主岗汇总，所以回答会同时体现分工和信息互通。",
                    primary_node
                        .map(|node| role_display_name(&node.role))
                        .unwrap_or("主岗位"),
                    support_count,
                    support_scopes
                ));
            }
            if !deterministic_quality_lines.is_empty() {
                lines.push(String::new());
                lines.push("### 团队调度".to_string());
                lines.extend(deterministic_quality_lines.iter().cloned());
            }
            if !team_contract_lines.is_empty() {
                lines.push(String::new());
                lines.push("### 协作边界".to_string());
                lines.extend(team_contract_lines.iter().cloned());
            }
            let executive_brief = executive_team_brief(primary_node, &passed_nodes, &self.reports);
            if !executive_brief.is_empty()
                && matches!(self.answer_mode, TaskGraphAnswerMode::Execution)
            {
                lines.push(String::new());
                lines.push("### 团队合成逻辑".to_string());
                lines.extend(executive_brief);
            }
        }
        lines.push(String::new());

        let missing_data_rows = missing_data_table_rows(&self.graph.root_task, &passed_nodes);
        if !missing_data_rows.is_empty() {
            lines.push("## 待补数据清单".to_string());
            lines.push("| 待补数据 | 用途 | 负责岗位 | 未补齐时的边界 |".to_string());
            lines.push("| --- | --- | --- | --- |".to_string());
            lines.extend(missing_data_rows);
            lines.push(String::new());
        }

        let role_briefs = role_brief_table(&passed_nodes, primary_node, &self.reports);
        if !role_briefs.is_empty() {
            lines.push("## 虚拟员工分工".to_string());
            if matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
                let role_rows = quick_role_rows(&passed_nodes, primary_node, &self.reports);
                lines.push("| 类型 | 岗位 | 核心判断 | 下一步 |".to_string());
                lines.push("| --- | --- | --- | --- |".to_string());
                lines.extend(role_rows);
            } else {
                lines.push("| 类型 | 岗位 | 负责问题 | 独立判断 | 交给团队的内容 |".to_string());
                lines.push("| --- | --- | --- | --- | --- |".to_string());
                lines.extend(role_briefs);
            }
            lines.push(String::new());
        }

        let team_brief = team_decision_brief(&passed_nodes, &self.reports);
        lines.push("## 主岗综合方案".to_string());
        if !team_brief.is_empty() {
            if let Some(node) = primary_node {
                lines.push(format!(
                    "- **主回答岗位**：{}（{}）负责把各岗位输入合成为综合结论、优先级、执行清单和 \
                     final_report。",
                    node.title,
                    role_display_name(&node.role)
                ));
            }
            let team_brief_limit = match self.answer_mode {
                TaskGraphAnswerMode::Quick => 3,
                _ => usize::MAX,
            };
            lines.extend(team_brief.into_iter().take(team_brief_limit));
            if !team_playbook.is_empty() && !matches!(self.answer_mode, TaskGraphAnswerMode::Quick)
            {
                lines.push(
                    "- **团队协作产物**：主岗负责把上述岗位结论压缩成可执行节奏；\
                     协作岗的判断会作为工作区 `final_report` 的过程依据，后续可继续追问或导出。"
                        .to_string(),
                );
            }
        } else {
            lines.push(
                "- 当前没有形成完整主岗方案，建议补充目标、预算、商品或指标后重新运行。"
                    .to_string(),
            );
        }
        lines.push(String::new());

        if matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
            let gaps = runtime_gap_lines(&self.graph.root_task, &self.graph.nodes, &self.states);
            if !gaps.is_empty() {
                lines.push("## 未完成或需复核".to_string());
                lines.extend(gaps.into_iter().take(3));
                lines.push(String::new());
            }
            lines.push("## 总结与交付物".to_string());
            lines.push(
                "- **final_report**：本轮快速结论会作为 Markdown 交付物写入工作区 artifact。"
                    .to_string(),
            );
            lines.push(
                "- **阅读方式**：快速模式只展示结论、关键原因、岗位摘要和主岗方案；\
                 完整岗位过程可在右侧过程面板回看。"
                    .to_string(),
            );
            return lines.join("\n");
        }

        let execution_draft = ecommerce_team_execution_draft(
            &self.graph.root_task,
            primary_node,
            &passed_nodes,
            &self.reports,
            self.answer_mode,
        );
        if !execution_draft.is_empty() && !matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
            lines.push(
                if matches!(self.answer_mode, TaskGraphAnswerMode::Execution) {
                    "## 丰富执行方案"
                } else {
                    "## 团队执行稿"
                }
                .to_string(),
            );
            lines.extend(execution_draft);
            lines.push(String::new());
        }

        lines.push("## 岗位协作过程".to_string());
        if let Some(node) = primary_node {
            lines.push(format!(
                "- **主回答岗位**：{}（{}）负责先给出核心判断。",
                node.title,
                role_display_name(&node.role)
            ));
            let supports = passed_nodes
                .iter()
                .filter(|item| item.id != node.id && item.role != "chat")
                .map(|item| format!("{}（{}）", item.title, role_display_name(&item.role)))
                .collect::<Vec<_>>();
            if !supports.is_empty() {
                lines.push(format!(
                    "- **协作岗位**：{} 分别补充证据、执行动作和风险边界。",
                    supports.join("、")
                ));
            }
        }
        if passed_nodes.is_empty() {
            lines.push("- 暂无通过的岗位产物。".to_string());
        }
        let process_limit = match self.answer_mode {
            TaskGraphAnswerMode::Quick => 2,
            TaskGraphAnswerMode::Learning
                if matches!(self.learning_depth, TaskGraphLearningDepth::Low) =>
            {
                3
            }
            _ => usize::MAX,
        };
        for (index, node) in passed_nodes.iter().take(process_limit).enumerate() {
            let Some(report) = self.reports.get(&node.id) else {
                continue;
            };
            let role_label = role_display_name(&node.role);
            let prefix = format!(
                "{:02}. {}（{}，{}）",
                index + 1,
                node.title,
                role_label,
                node.role
            );
            if let Some(role_output) = &report.role_output {
                let summary =
                    visible_non_empty(role_output.summary.as_str()).unwrap_or("已形成本岗位判断。");
                lines.push(format!("- **{prefix}**：{}", summary));
                if !matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
                    extend_synthesis_items(&mut lines, "  发现", &role_output.findings);
                    extend_synthesis_items(&mut lines, "  建议", &role_output.recommendations);
                    extend_synthesis_items(&mut lines, "  风险", &role_output.risks);
                    if matches!(self.learning_depth, TaskGraphLearningDepth::High)
                        || matches!(self.answer_mode, TaskGraphAnswerMode::Execution)
                    {
                        extend_synthesis_items(&mut lines, "  待补充", &role_output.open_questions);
                    }
                }
            } else {
                let content = visible_non_empty(report.content.as_str()).unwrap_or("");
                lines.push(format!(
                    "- **{prefix}**：{}",
                    if content.is_empty() {
                        "无可展示内容"
                    } else {
                        content
                    }
                ));
            }
        }
        if process_limit != usize::MAX && passed_nodes.len() > process_limit {
            lines.push(format!(
                "- 其余 {} 个岗位的完整过程已保留在右侧过程面板和运行记录中。",
                passed_nodes.len() - process_limit
            ));
        }

        let handoffs = collaboration_handoffs(&self.graph.nodes);
        if !handoffs.is_empty() {
            lines.push(String::new());
            lines.push("## 协同关系".to_string());
            lines.extend(handoffs);
        } else {
            let inferred_handoffs = inferred_collaboration_handoffs(&passed_nodes);
            if !inferred_handoffs.is_empty() {
                lines.push(String::new());
                lines.push("## 协同关系".to_string());
                lines.extend(inferred_handoffs);
            }
        }

        let actions = unique_strings(
            passed_nodes
                .iter()
                .filter_map(|node| self.reports.get(&node.id))
                .filter_map(|report| report.role_output.as_ref())
                .flat_map(|output| output.recommendations.clone())
                .filter_map(|item| clean_visible_answer_item(&item))
                .collect(),
        );
        if !actions.is_empty() && !matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
            lines.push(String::new());
            lines.push("## 下一步".to_string());
            for (index, action) in actions.iter().take(6).enumerate() {
                lines.push(format!(
                    "{}. {}",
                    index + 1,
                    strip_list_marker(action.trim())
                ));
            }
        }

        let check_items = execution_check_items(&passed_nodes, &self.reports);
        if !check_items.is_empty() && matches!(self.answer_mode, TaskGraphAnswerMode::Execution) {
            lines.push(String::new());
            lines.push("## 执行检查清单".to_string());
            for item in check_items.iter().take(6) {
                lines.push(format!("- {item}"));
            }
        }

        let deliverables = deliverable_hints(&passed_nodes, &self.reports);
        lines.push(String::new());
        lines.push("## 总结与交付物".to_string());
        if !deliverables.is_empty() {
            for item in deliverables {
                lines.push(format!("- {item}"));
            }
        }
        lines.push(
            "- **final_report**：本轮完整回答会作为 Markdown 交付物写入工作区 artifact。"
                .to_string(),
        );
        if matches!(self.answer_mode, TaskGraphAnswerMode::Quick) {
            lines.push(
                "- **阅读方式**：快速模式只展示摘要，完整岗位过程可在右侧过程面板回看。"
                    .to_string(),
            );
        } else {
            lines.push(
                "- **过程依据**：岗位独立输出、评估结果、风险边界和待补数据会保留在运行记录中，\
                 后续可继续预览、导出或引用。"
                    .to_string(),
            );
        }

        let gaps = runtime_gap_lines(&self.graph.root_task, &self.graph.nodes, &self.states);
        if !gaps.is_empty() {
            lines.push(String::new());
            lines.push("## 未完成或需复核".to_string());
            lines.extend(gaps);
        }

        lines.join("\n")
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
        if let Some(summary) = report
            .role_output
            .as_ref()
            .map(|output| output.summary.trim())
            .filter(|summary| !summary.is_empty())
        {
            return Some(summary.to_string());
        }

        let content = report.content.trim();
        if !content.is_empty() {
            return Some(content.to_string());
        }

        None
    }

    pub fn process_summary(&self) -> TaskGraphProcessSummary {
        let passed_nodes = self
            .graph
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    self.states.get(&node.id),
                    Some(TaskNodeRuntimeState::Passed)
                )
            })
            .collect::<Vec<_>>();
        let primary_node = select_primary_answer_node(&self.graph.nodes, &passed_nodes)
            .or_else(|| self.graph.nodes.first().filter(|node| node.role != "chat"));
        let roles = self
            .graph
            .nodes
            .iter()
            .map(|node| process_role_summary(node, primary_node, self))
            .collect::<Vec<_>>();
        let primary = primary_node
            .and_then(|node| roles.iter().find(|role| role.node_id == node.id))
            .cloned();
        let handoffs = process_handoffs(&self.graph.nodes);
        let next_actions = unique_strings(
            passed_nodes
                .iter()
                .filter_map(|node| self.reports.get(&node.id))
                .filter_map(|report| report.role_output.as_ref())
                .flat_map(|output| output.recommendations.clone())
                .filter_map(|item| {
                    visible_non_empty(&item).map(|value| strip_list_marker(value).to_string())
                })
                .collect(),
        )
        .into_iter()
        .take(6)
        .collect();
        let check_items = execution_check_items(&passed_nodes, &self.reports)
            .into_iter()
            .take(6)
            .collect();
        let mut deliverables = deliverable_hints(&passed_nodes, &self.reports);
        if !deliverables.is_empty() {
            deliverables.push(
                "final_report：本轮完整回答会作为 Markdown \
                 工作区交付物沉淀，包含综合结论、岗位分工、主岗综合方案和执行稿。"
                    .to_string(),
            );
            deliverables.push(
                "过程依据：岗位独立输出、评估结果、风险边界和待补数据会保留在运行记录中，\
                 可继续预览、导出或引用。"
                    .to_string(),
            );
        }
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

        TaskGraphProcessSummary {
            graph_id: self.graph.graph_id.clone(),
            root_task: self.graph.root_task.clone(),
            primary,
            roles,
            handoffs,
            next_actions,
            check_items,
            deliverables,
            risks,
            open_questions,
            incomplete,
            panel: TaskGraphProcessPanel {
                default_collapsed: true,
                anchor: "answer_scoped_process".to_string(),
                label: "本次回答过程".to_string(),
                reason: "过程面板应绑定本次回答，默认折叠，用户需要回看岗位过程时再展开。"
                    .to_string(),
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
            worker_details,
            risks: unique_strings(
                role_summaries
                    .iter()
                    .flat_map(|summary| summary.risks.clone())
                    .collect(),
            ),
            open_questions: unique_strings(
                role_summaries
                    .iter()
                    .flat_map(|summary| summary.open_questions.clone())
                    .collect(),
            ),
            role_summaries,
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
    if let Some(graph) = planner::plan_configured_task_graph(task_id, task, route) {
        return graph;
    }
    if chat::is_direct_answer_task(task) {
        return chat::plan_direct_answer_chat_graph(task_id, task);
    }
    chat::plan_fallback_chat_graph(task_id, task)
}

pub fn plan_quick_chat_graph(task_id: &TaskId, task: &str) -> Option<TaskGraph> {
    if chat::is_direct_answer_task(task) {
        return Some(chat::plan_direct_answer_chat_graph(task_id, task));
    }

    let graph = planner::plan_configured_task_graph(task_id, task, None)?;
    let [node] = graph.nodes.as_slice() else {
        return None;
    };
    (node.role == "chat" && node.input_refs.is_empty()).then_some(graph)
}

pub fn plan_forced_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    chat::plan_fallback_chat_graph(task_id, task)
}

fn evaluate_report(
    report: &WorkerReport,
    node: Option<&TaskNode>,
    upstream_reports: &HashMap<TaskNodeId, WorkerReport>,
    root_task: &str,
) -> Evaluation {
    let policy = policy::load_evaluation_policy();
    let content = report.content.trim();
    let mut reasons = Vec::new();
    if report.status == WorkerReportStatus::Failed {
        reasons.push("worker reported failed status".to_string());
    }
    if content.is_empty() {
        reasons.push("content is empty".to_string());
    }
    if open_questions_are_blocking(report.role_output.as_ref(), &report.open_questions) {
        reasons.push("open questions remain".to_string());
    }
    match &report.role_output {
        Some(role_output) => {
            reasons.extend(evaluate_role_output(&report.role, role_output, &policy));
            reasons.extend(evaluate_quality_contract(report, role_output, &policy));
            if let Some(node) = node {
                reasons.extend(evaluate_upstream_evidence(
                    node,
                    role_output,
                    upstream_reports,
                ));
            }
        }
        None => reasons.push("structured role output is missing".to_string()),
    }
    let lower = content.to_lowercase();
    if util::contains_string_any(&lower, &policy.placeholder_keywords) {
        reasons.push("content looks like placeholder or failure text".to_string());
    }
    if lower.contains("无数据")
        && !report
            .role_output
            .as_ref()
            .is_some_and(has_actionable_boundary_output)
    {
        reasons.push("content looks like placeholder or failure text".to_string());
    }
    if is_explicit_unselected_product_task(root_task)
        && report_mentions_forbidden_default_product_context(report)
    {
        reasons.push("selected product context is not allowed for this task".to_string());
    }

    let reasons = downgrade_role_contract_gaps(report, node, root_task, reasons, &policy);
    let passed = reasons.is_empty();
    Evaluation {
        node_id: report.node_id.clone(),
        passed,
        score: if passed { 1.0 } else { 0.0 },
        reasons,
        rework_instruction: (!passed).then_some(
            "请重新提交本节点结果：内容必须具体、非空，不能是占位或失败表达。".to_string(),
        ),
    }
}

fn should_accept_boundary_report(
    report: &WorkerReport,
    node: &TaskNode,
    evaluation: &Evaluation,
    root_task: &str,
) -> bool {
    let policy = policy::load_evaluation_policy();
    if evaluation.passed {
        return true;
    }
    if report.role != node.role {
        return false;
    }
    if report.status == WorkerReportStatus::Failed
        && !report
            .role_output
            .as_ref()
            .is_some_and(|output| is_recoverable_category_failure(output, root_task))
    {
        return false;
    }
    let Some(output) = report.role_output.as_ref() else {
        return false;
    };
    let has_boundary = has_category_level_boundary(output) || is_unselected_product_task(root_task);
    if !has_actionable_boundary_output(output) || !has_boundary {
        return false;
    }
    evaluation.reasons.iter().all(|reason| {
        is_role_contract_or_boundary_gap(reason, &report.role, &policy)
            || reason.starts_with("upstream evidence:")
            || is_downgradable_quality_gap(reason, &report.role, Some(node), &policy)
    })
}

fn downgrade_role_contract_gaps(
    report: &WorkerReport,
    node: Option<&TaskNode>,
    root_task: &str,
    reasons: Vec<String>,
    policy: &EvaluationPolicy,
) -> Vec<String> {
    if reasons.is_empty() {
        return reasons;
    }
    let category_repair = node.is_some_and(|node| {
        is_unselected_product_task(root_task)
            && should_repair_category_boundary_report(report, node)
    });
    if category_repair {
        return reasons
            .into_iter()
            .filter(|reason| {
                !is_role_contract_or_boundary_gap(reason, report.role.as_str(), policy)
            })
            .collect();
    }
    let Some(output) = report.role_output.as_ref() else {
        return reasons;
    };
    if report.status == WorkerReportStatus::Failed
        && !is_recoverable_category_failure(output, root_task)
    {
        return reasons;
    }
    let has_boundary = has_category_level_boundary(output) || is_unselected_product_task(root_task);
    if !has_boundary || !has_actionable_boundary_output(output) {
        return reasons;
    }

    let recoverable_category_failure = report
        .role_output
        .as_ref()
        .is_some_and(|output| is_recoverable_category_failure(output, root_task));

    reasons
        .into_iter()
        .filter(|reason| {
            !(is_downgradable_quality_gap(reason, &report.role, node, policy)
                || (recoverable_category_failure && reason == "worker reported failed status"))
        })
        .collect()
}

fn should_repair_category_boundary_report(report: &WorkerReport, node: &TaskNode) -> bool {
    if !matches!(
        report.role.as_str(),
        "data"
            | "service"
            | "engineering"
            | "creative"
            | "design"
            | "accounting"
            | "douyin_ads"
            | "web"
    ) {
        return false;
    }
    let joined = report_text_fields(report).join(" ");
    let has_category_or_product_scope = contains_any(
        &joined,
        &[
            "类目",
            "品类",
            "咖啡",
            "商品",
            "产品",
            "直播间",
            "转化",
            "未选商品",
            "未指定商品",
            "context.selected_summary",
            CATEGORY_LEVEL_BOUNDARY_MARKER,
        ],
    );
    let has_boundary_or_gap = contains_data_gap(&joined)
        || joined.contains(CATEGORY_LEVEL_BOUNDARY_MARKER)
        || contains_any(
            &joined,
            &["不能编造", "不注入", "无真实数据", "缺少真实", "未手动选择"],
        );
    let dependent_role = !node.input_refs.is_empty();
    has_category_or_product_scope && (has_boundary_or_gap || dependent_role)
}

fn is_category_repair_gap(reason: &str, role: &str) -> bool {
    let policy = policy::load_evaluation_policy();
    is_category_repair_gap_with_policy(reason, role, &policy)
}

fn is_category_repair_gap_with_policy(reason: &str, role: &str, policy: &EvaluationPolicy) -> bool {
    reason == "worker reported failed status"
        || reason == "open questions remain"
        || reason == "role output has open questions"
        || reason == "content looks like placeholder or failure text"
        || reason.starts_with("upstream evidence:")
        || is_downgradable_quality_gap(reason, role, None, policy)
}

fn is_role_contract_or_boundary_gap(reason: &str, role: &str, policy: &EvaluationPolicy) -> bool {
    is_category_repair_gap_with_policy(reason, role, policy)
        || policy
            .boundary_gap_reasons
            .iter()
            .any(|configured| configured == reason)
}

fn is_category_level_task(task: &str) -> bool {
    contains_any(
        task,
        &[
            "类目",
            "品类",
            "咖啡类",
            "茶饮类",
            "食品类",
            "服饰类",
            "美妆类",
            "家居类",
            "商品类",
            "类商品",
            "类产品",
            "行业",
            "未指定商品",
            "未选商品",
            "不指定商品",
            "不引用任何预设商品",
            "不要引用任何预设商品",
            "不引用预设商品",
            "不引用任何商品",
            "不使用预设商品",
            "不使用默认商品",
            "不引用默认商品",
            "不选择商品",
            "不绑定商品",
            "without selected product",
            "no selected product",
        ],
    )
}

fn is_unselected_product_task(task: &str) -> bool {
    is_category_level_task(task) || is_explicit_unselected_product_task(task)
}

fn is_explicit_unselected_product_task(task: &str) -> bool {
    contains_any(
        task,
        &[
            "不引用任何预设商品",
            "不要引用任何预设商品",
            "不引用预设商品",
            "不引用任何商品",
            "不使用预设商品",
            "不使用默认商品",
            "不引用默认商品",
            "不选择商品",
            "不绑定商品",
            "未选商品",
            "未指定商品",
            "without selected product",
            "no selected product",
        ],
    )
}

fn has_category_level_boundary(output: &RoleWorkOutput) -> bool {
    let joined = format!(
        "{} {} {} {} {}",
        output.summary,
        output.findings.join(" "),
        output.recommendations.join(" "),
        output.evidence.join(" "),
        output.risks.join(" ")
    );
    contains_any(
        &joined,
        &[
            "类目级",
            CATEGORY_LEVEL_BOUNDARY_MARKER,
            "未手动选择预设商品",
            "不注入任何预设商品",
            "数据边界",
            "不能编造",
            "只按用户文本",
        ],
    )
}

fn has_actionable_boundary_output(output: &RoleWorkOutput) -> bool {
    !output.summary.trim().is_empty()
        && (!output.findings.is_empty() || !output.recommendations.is_empty())
        && (!output.risks.is_empty()
            || !output.open_questions.is_empty()
            || contains_data_gap(&format!(
                "{} {} {}",
                output.summary,
                output.findings.join(" "),
                output.recommendations.join(" ")
            )))
}

fn is_recoverable_category_failure(output: &RoleWorkOutput, root_task: &str) -> bool {
    is_unselected_product_task(root_task)
        && has_actionable_boundary_output(output)
        && (has_category_level_boundary(output)
            || contains_data_gap(&format!(
                "{} {} {} {} {}",
                output.summary,
                output.findings.join(" "),
                output.recommendations.join(" "),
                output.evidence.join(" "),
                output.risks.join(" ")
            )))
}

fn is_downgradable_quality_gap(
    reason: &str,
    role: &str,
    node: Option<&TaskNode>,
    policy: &EvaluationPolicy,
) -> bool {
    if policy
        .downgradable_prefixes
        .iter()
        .any(|prefix| reason.starts_with(prefix))
    {
        return true;
    }

    if let Some(role_policy) = policy.roles.get(role) {
        let role_reasons = &role_policy.reasons;
        if [
            role_reasons.min_findings.as_str(),
            role_reasons.min_recommendations.as_str(),
            role_reasons.require_risks.as_str(),
            role_reasons.specific_content.as_str(),
        ]
        .into_iter()
        .any(|configured| !configured.is_empty() && configured == reason)
        {
            return true;
        }
    }

    node.is_some_and(|node| !node.input_refs.is_empty()) && reason.starts_with("upstream evidence:")
}

fn evaluate_quality_contract(
    report: &WorkerReport,
    output: &RoleWorkOutput,
    policy: &EvaluationPolicy,
) -> Vec<String> {
    let mut reasons = Vec::new();
    let content = format!(
        "{} {} {} {} {}",
        output.summary,
        output.findings.join(" "),
        output.recommendations.join(" "),
        output.risks.join(" "),
        report.content
    );
    let lower = content.to_lowercase();

    if output.summary.chars().count() < policy.min_summary_chars && report.role != "chat" {
        reasons.push("quality completeness: summary is too thin".to_string());
    }
    if report.role != "chat" && output.findings.is_empty() && output.recommendations.is_empty() {
        reasons.push("quality completeness: no findings or recommendations".to_string());
    }
    if util::contains_string_any(&lower, &policy.unsupported_evidence_keywords) {
        reasons.push("quality fabrication: claims unsupported external evidence".to_string());
    }
    if is_action_oriented_role(&report.role, policy) && output.recommendations.is_empty() {
        reasons.push("quality actionability: missing executable recommendations".to_string());
    }
    if is_risk_sensitive_role(&report.role, policy) && output.risks.is_empty() {
        reasons.push("quality risk awareness: missing risk boundary".to_string());
    }
    if util::contains_string_any(&lower, &policy.refusal_keywords)
        && output.open_questions.is_empty()
    {
        reasons
            .push("quality refusal detection: refusal without explicit open question".to_string());
    }

    reasons
}

fn open_questions_are_blocking(output: Option<&RoleWorkOutput>, open_questions: &[String]) -> bool {
    let policy = policy::load_evaluation_policy();
    if open_questions.is_empty() {
        return false;
    }
    if output.is_some_and(has_actionable_boundary_output) {
        return false;
    }
    if open_questions
        .iter()
        .any(|item| util::contains_string_any(item, &policy.blocking_open_question_keywords))
    {
        return true;
    }

    output
        .map(|output| {
            output.summary.trim().is_empty()
                || (output.findings.is_empty() && output.recommendations.is_empty())
        })
        .unwrap_or(true)
}

fn evaluate_upstream_evidence(
    node: &TaskNode,
    output: &RoleWorkOutput,
    upstream_reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    if node.input_refs.is_empty() {
        return Vec::new();
    }

    let mut reasons = Vec::new();
    if output.evidence.is_empty() {
        reasons.push("upstream evidence: dependent role must cite upstream inputs".to_string());
        return reasons;
    }

    for input_ref in &node.input_refs {
        let source_marker = format!("worker.assignment.input.{}", input_ref.0);
        let upstream_role = upstream_reports
            .get(input_ref)
            .map(|report| report.role.as_str())
            .unwrap_or_default();
        let cited = output.evidence.iter().any(|item| {
            let item = item.trim();
            item.contains(&input_ref.0)
                || item.contains(&source_marker)
                || (!upstream_role.is_empty() && item.contains(upstream_role))
        });
        if !cited {
            reasons.push(format!(
                "upstream evidence: missing citation for input {}",
                input_ref.0
            ));
        }
    }

    reasons
}

fn select_primary_answer_node<'a>(
    graph_nodes: &'a [TaskNode],
    passed_nodes: &[&'a TaskNode],
) -> Option<&'a TaskNode> {
    let is_passed = |candidate: &TaskNode| passed_nodes.iter().any(|node| node.id == candidate.id);
    let has_downstream = |candidate: &TaskNode| {
        graph_nodes
            .iter()
            .any(|node| node.input_refs.iter().any(|input| input == &candidate.id))
    };

    passed_nodes
        .iter()
        .copied()
        .find(|node| node.role == "ops")
        .or_else(|| {
            graph_nodes
                .iter()
                .rev()
                .find(|node| is_passed(node) && !has_downstream(node) && node.role != "chat")
        })
        .or_else(|| {
            passed_nodes
                .iter()
                .copied()
                .find(|node| node.role != "chat")
        })
}

fn primary_answer_summary(
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Option<String> {
    primary_node
        .and_then(|node| report_summary(node, reports))
        .or_else(|| primary_summary(nodes, reports))
}

fn report_summary(node: &TaskNode, reports: &HashMap<TaskNodeId, WorkerReport>) -> Option<String> {
    let report = reports.get(&node.id)?;
    report
        .role_output
        .as_ref()
        .and_then(|output| visible_non_empty(output.summary.as_str()).map(str::to_string))
        .or_else(|| visible_non_empty(report.content.as_str()).map(str::to_string))
}

fn primary_summary(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Option<String> {
    nodes
        .iter()
        .rev()
        .find_map(|node| {
            let report = reports.get(&node.id)?;
            let output = report.role_output.as_ref()?;
            visible_non_empty(output.summary.as_str()).map(str::to_string)
        })
        .or_else(|| {
            nodes.iter().rev().find_map(|node| {
                visible_non_empty(reports.get(&node.id)?.content.as_str()).map(str::to_string)
            })
        })
}

fn product_context_line(
    root_task: &str,
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Option<String> {
    if is_explicit_unselected_product_task(root_task) {
        return Some(
            "**商品上下文**：本轮用户明确要求不引用预设商品，不注入任何默认商品或历史商品，\
             只按用户文本、同一会话历史和工作区上下文理解问题。"
                .to_string(),
        );
    }
    if reports
        .values()
        .any(report_mentions_selected_product_context)
    {
        return Some(
            "**商品上下文**：本轮使用用户手动选择的预设商品资料，来源为 context.selected_summary。"
                .to_string(),
        );
    }
    if root_task.contains("不引用任何预设商品")
        || root_task.contains("未指定商品")
        || is_category_level_task(root_task)
        || reports
            .values()
            .any(report_mentions_unselected_product_context)
    {
        return Some(
            "**商品上下文**：本轮未检测到用户手动选择的预设商品资料，不注入默认商品，\
             只按用户文本、同一会话历史和工作区上下文理解问题。"
                .to_string(),
        );
    }
    None
}

fn team_overview_line(nodes: &[&TaskNode], primary_node: Option<&TaskNode>) -> String {
    let roles = unique_strings(
        nodes
            .iter()
            .filter(|node| node.role != "chat")
            .filter(|node| {
                primary_node
                    .map(|primary| primary.id != node.id)
                    .unwrap_or(true)
            })
            .map(|node| role_display_name(&node.role).to_string())
            .collect(),
    );
    if roles.is_empty() {
        return String::new();
    }
    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("主回答岗位");
    format!(
        "**协作方式**：{}牵头，{}共同参与；每个岗位先独立产出，再把发现、建议、\
         风险和交接证据汇总到最终回答。",
        primary,
        roles.join("、")
    )
}

fn report_mentions_selected_product_context(report: &WorkerReport) -> bool {
    report_text_fields(report).iter().any(|item| {
        let item = item.trim();
        (item.contains("用户本轮手动选择的预设商品资料")
            || item.contains("本轮使用用户手动选择的预设商品资料")
            || item.contains("本轮使用用户手动选择的商品")
            || item.contains("来源 context.selected_summary"))
            && !contains_any(
                item,
                &[
                    "未检测到 context.selected_summary",
                    "no context.selected_summary",
                    "没有 context.selected_summary",
                    "缺少 context.selected_summary",
                    "不注入",
                    "不引用",
                    "不能把",
                    "不能使用",
                ],
            )
    })
}

fn report_mentions_unselected_product_context(report: &WorkerReport) -> bool {
    report_text_fields(report).iter().any(|item| {
        item.contains("未检测到 context.selected_summary")
            || item.contains("不注入任何预设商品资料")
            || item.contains("不使用默认商品资料")
    })
}

fn report_mentions_forbidden_default_product_context(report: &WorkerReport) -> bool {
    report_text_fields(report).iter().any(|item| {
        let item = item.trim();
        let negated = contains_any(
            item,
            &[
                "未检测到 context.selected_summary",
                "no context.selected_summary",
                "没有 context.selected_summary",
                "缺少 context.selected_summary",
                "不注入",
                "不引用",
                "不能把",
                "不能使用",
                "不能默认引用",
                "不会默认引用",
                "不要默认引用",
            ],
        );
        (item.contains("千乐咖啡")
            || item.contains("来源 context.selected_summary")
            || item.contains("用户本轮手动选择的预设商品资料")
            || item.contains("用户本轮手动选择的商品")
            || item.contains("本轮使用用户手动选择"))
            && !negated
    })
}

fn report_text_fields(report: &WorkerReport) -> Vec<String> {
    let mut items = vec![report.content.clone()];
    if let Some(output) = &report.role_output {
        items.push(output.summary.clone());
        items.extend(output.findings.clone());
        items.extend(output.recommendations.clone());
        items.extend(output.evidence.clone());
        items.extend(output.risks.clone());
        items.extend(output.open_questions.clone());
    }
    items.extend(report.evidence.clone());
    items.extend(report.risks.clone());
    items.extend(report.open_questions.clone());
    items
}

fn format_runtime_gap_reason(reason: &str) -> String {
    if reason.trim().is_empty() {
        return "本节点没有形成可复核产物，需要补齐输入后重新运行。".to_string();
    }

    let mapped = reason
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(localize_evaluation_reason)
        .collect::<Vec<_>>();
    if mapped.is_empty() {
        reason.to_string()
    } else {
        unique_strings(mapped).join("；")
    }
}

fn localize_evaluation_reason(reason: &str) -> String {
    let policy = policy::load_evaluation_policy();
    if let Some(localized) = policy.reason_localizations.get(reason) {
        return localized.clone();
    }
    for item in &policy.prefix_localizations {
        if reason.starts_with(&item.prefix) {
            return item.message.clone();
        }
    }
    reason.to_string()
}

fn execution_check_items(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let mut items = Vec::new();
    for node in nodes {
        let Some(output) = reports
            .get(&node.id)
            .and_then(|report| report.role_output.as_ref())
        else {
            continue;
        };
        match node.role.as_str() {
            "data" => {
                push_unique(
                    &mut items,
                    "数据：补齐曝光、点击、加购、付款、退款和渠道拆分，先统一口径再判断健康度。"
                        .to_string(),
                );
            }
            "ops" => {
                push_unique(
                    &mut items,
                    "运营：把目标、周期、预算和成功指标写入同一张复盘表，避免同时改动过多变量。"
                        .to_string(),
                );
            }
            "creative" => {
                push_unique(
                    &mut items,
                    "创意：把卖点拆成标题、直播脚本和短视频钩子，按 A/B 结果保留胜出版本。"
                        .to_string(),
                );
            }
            "service" => {
                push_unique(
                    &mut items,
                    "客服：同步咨询话术、售后承诺和异常升级 SOP，避免前端营销承诺超过履约能力。"
                        .to_string(),
                );
            }
            "engineering" => {
                push_unique(
                    &mut items,
                    "工程：确认埋点、数据接口、刷新时间和失败回滚边界，保证结论可复核。"
                        .to_string(),
                );
            }
            "douyin_ads" => {
                push_unique(
                    &mut items,
                    "投放：按人群包、素材组、出价和预算节奏拆分测试，并设置止损阈值。".to_string(),
                );
            }
            _ => {}
        }
        for question in &output.open_questions {
            push_unique(&mut items, format!("待补充：{}", question.trim()));
        }
    }
    items
}

fn deliverable_hints(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let mut items = Vec::new();
    let has_role = |role: &str| nodes.iter().any(|node| node.role == role);

    if has_role("data") {
        items.push("数据诊断报告：记录指标口径、漏斗断点、缺口和复核方式。".to_string());
    }
    if has_role("ops") || has_role("douyin_ads") {
        items.push("运营执行清单：沉淀优先级、责任岗位、预算节奏和复盘节点。".to_string());
    }
    if has_role("creative") || has_role("design") {
        items.push("素材与页面方案：沉淀标题、脚本、主图/详情页调整点和测试假设。".to_string());
    }
    if has_role("service") {
        items.push("客服承接 SOP：沉淀咨询话术、异议处理、售后边界和升级路径。".to_string());
    }

    if items.is_empty()
        && nodes.iter().any(|node| {
            reports
                .get(&node.id)
                .and_then(|report| report.role_output.as_ref())
                .is_some()
        })
    {
        items.push("本轮回答可保存为工作区交付物，供后续对话和复盘继续引用。".to_string());
    }

    items
}

fn process_role_summary(
    node: &TaskNode,
    primary_node: Option<&TaskNode>,
    runtime: &TaskGraphRuntime,
) -> TaskGraphProcessRole {
    let report = runtime.reports.get(&node.id);
    let role_output = report.and_then(|report| report.role_output.as_ref());
    let summary = role_output
        .and_then(|output| visible_non_empty(&output.summary))
        .or_else(|| report.and_then(|report| visible_non_empty(&report.content)))
        .map(ToString::to_string);
    let finding = role_output
        .and_then(|output| first_visible_non_empty(&output.findings))
        .map(strip_list_marker)
        .map(ToString::to_string);
    let recommendation = role_output
        .and_then(|output| first_visible_non_empty(&output.recommendations))
        .map(strip_list_marker)
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
        role_label: role_display_name(&node.role).to_string(),
        title: node.title.clone(),
        objective: node.objective.clone(),
        status,
        attempt: runtime.attempts.get(&node.id).copied().unwrap_or(0),
        is_primary: primary_node.is_some_and(|primary| primary.id == node.id),
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
                    from_role_label: role_display_name(&upstream.role).to_string(),
                    to_role_label: role_display_name(&node.role).to_string(),
                    summary: format!(
                        "{} 读取 {} 的结论、证据和风险边界后继续补充本岗位判断。",
                        role_display_name(&node.role),
                        role_display_name(&upstream.role)
                    ),
                })
            })
        })
        .collect()
}

fn process_gap_summary(node: &TaskNode, runtime: &TaskGraphRuntime) -> Option<TaskGraphProcessGap> {
    match runtime.states.get(&node.id) {
        Some(TaskNodeRuntimeState::Failed(reason)) => {
            if is_unselected_product_task(&runtime.graph.root_task)
                && is_category_repair_gap(reason, &node.role)
            {
                return None;
            }
            Some(TaskGraphProcessGap {
                node_id: node.id.clone(),
                role: node.role.clone(),
                role_label: role_display_name(&node.role).to_string(),
                title: node.title.clone(),
                state: "failed".to_string(),
                reason: format_runtime_gap_reason(reason),
            })
        }
        Some(TaskNodeRuntimeState::Skipped(reason)) => Some(TaskGraphProcessGap {
            node_id: node.id.clone(),
            role: node.role.clone(),
            role_label: role_display_name(&node.role).to_string(),
            title: node.title.clone(),
            state: "skipped".to_string(),
            reason: format_runtime_gap_reason(reason),
        }),
        _ => None,
    }
}

fn runtime_gap_lines(
    root_task: &str,
    nodes: &[TaskNode],
    states: &HashMap<TaskNodeId, TaskNodeRuntimeState>,
) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|node| match states.get(&node.id) {
            Some(TaskNodeRuntimeState::Failed(reason)) => {
                if is_unselected_product_task(root_task)
                    && is_category_repair_gap(reason, &node.role)
                {
                    None
                } else {
                    Some(format!(
                        "- {}（{}）未完成：{}",
                        node.title,
                        role_display_name(&node.role),
                        format_runtime_gap_reason(reason)
                    ))
                }
            }
            Some(TaskNodeRuntimeState::Skipped(reason)) => Some(format!(
                "- {}（{}）已跳过：{}",
                node.title,
                role_display_name(&node.role),
                format_runtime_gap_reason(reason)
            )),
            _ => None,
        })
        .collect()
}

fn team_decision_brief(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let mut items = Vec::new();
    for node in nodes {
        if node.role == "chat" {
            continue;
        }
        let Some(output) = reports
            .get(&node.id)
            .and_then(|report| report.role_output.as_ref())
        else {
            continue;
        };
        let finding = first_visible_non_empty(&output.findings)
            .and_then(clean_visible_answer_item)
            .or_else(|| clean_visible_answer_item(output.summary.as_str()))
            .unwrap_or_else(|| "已形成本岗位判断".to_string());
        let action = first_clean_visible_answer_item(&output.recommendations)
            .unwrap_or_else(|| "继续按本岗位边界补齐执行动作。".to_string());
        items.push(format!(
            "- **{}**：{}；对应动作：{}",
            role_display_name(&node.role),
            finding,
            action
        ));
    }
    let team_narrative = team_decision_narrative(nodes, reports);
    if !team_narrative.is_empty() {
        items.push("- **团队合成**：".to_string());
        items.extend(team_narrative.into_iter().map(|item| format!("  - {item}")));
    }
    items
}

fn team_decision_narrative(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let role_outputs = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .filter_map(|node| {
            reports
                .get(&node.id)
                .and_then(|report| report.role_output.as_ref())
                .map(|output| (*node, output))
        })
        .collect::<Vec<_>>();
    if role_outputs.len() <= 1 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    for (node, output) in &role_outputs {
        let finding = first_visible_non_empty(&output.findings)
            .or_else(|| visible_non_empty(output.summary.as_str()))
            .and_then(clean_visible_answer_item)
            .unwrap_or_else(|| "已完成本岗位判断".to_string());
        let action = first_clean_visible_answer_item(&output.recommendations)
            .unwrap_or_else(|| "把本岗位缺口、证据和风险交给主岗汇总".to_string());
        let risk = first_visible_non_empty(&output.risks)
            .or_else(|| first_visible_non_empty(&output.open_questions))
            .and_then(clean_visible_answer_item)
            .unwrap_or_else(|| "暂无额外风险，继续按主岗节奏复核".to_string());
        lines.push(format!(
            "{}先独立判断“{}”，再把“{}”交给团队；需要同步关注“{}”。",
            role_display_name(&node.role),
            finding,
            action,
            risk
        ));
    }

    let handoff = role_outputs
        .iter()
        .map(|(node, _)| role_display_name(&node.role))
        .collect::<Vec<_>>();
    if handoff.len() >= 3 {
        lines.push(format!(
            "协作顺序不是简单罗列：{}先各自完成判断，再由主回答岗位统一收敛成优先级、执行清单和 \
             final_report 交付物。",
            handoff.join("、")
        ));
    }
    lines
}

fn direct_team_answer(
    root_task: &str,
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.is_empty()
        || (roles.len() <= 1
            && !is_category_level_task(root_task)
            && !is_live_conversion_task(root_task))
    {
        return Vec::new();
    }

    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("主回答岗位");
    let mut lines = Vec::new();
    if is_live_conversion_task(root_task) {
        lines.push(format!(
            "- **结论**：{}牵头判断直播间转化是否健康时，要把曝光、点击、加购、付款、退款、\
             咨询承接和履约风险连成一条漏斗看；任何一段异常，都不能只用最终成交额掩盖。",
            primary
        ));
        if let Some(sequence) = team_judgement_sequence(&roles) {
            lines.push(format!("- **判断顺序**：{sequence}"));
        }
        let funnel_lines = live_conversion_funnel_lines(root_task);
        if !funnel_lines.is_empty() {
            lines.push("- **本题漏斗初判**：".to_string());
            lines.extend(funnel_lines.into_iter().map(|item| format!("  - {item}")));
        }
    } else if contains_any(
        root_task,
        &["咖啡", "饮品", "食品", "类目", "品类", "商品类"],
    ) {
        lines.push(format!(
            "- **结论**：{}牵头先按类目级链路分析，不把历史商品或默认商品当成本轮指定商品；\
             咖啡类商品应优先拆成人群、场景、价格带、内容卖点、渠道承接和复购风险。",
            primary
        ));
        if let Some(direction) = team_execution_direction(&roles) {
            lines.push(format!(
                "- **执行方向**：{direction}；如果后续要精确到某个预设商品，\
                 必须由用户在输入区手动选择商品资料后再进入单品级复核。"
            ));
        }
    } else {
        lines.push(format!(
            "- **结论**：{}负责收敛最终判断，协作岗位分别补齐证据、动作、风险和交付物，\
             使回答不是单个聊天回复，而是一组电商虚拟员工的团队方案。",
            primary
        ));
    }

    let role_summaries = role_summary_lines(nodes, reports);
    if !role_summaries.is_empty() {
        lines.push("- **岗位协作**：".to_string());
        lines.extend(role_summaries.into_iter().map(|item| format!("  - {item}")));
    }
    let role_plans = team_role_workplans(root_task, nodes);
    if !role_plans.is_empty() {
        lines.push("- **岗位独立产出**：".to_string());
        lines.extend(role_plans.into_iter().map(|item| format!("  - {item}")));
    }

    if is_category_level_task(root_task) {
        lines.push(
            "- **边界**：本轮是未选商品或类目级问题，系统只能输出方向、口径和待补数据清单；\
             不会把任何历史商品、测试商品或默认商品注入为本轮商品。"
                .to_string(),
        );
    }
    if roles.len() >= 3 {
        lines.push(
            "- **团队感知**：每个岗位会保留自己的独立判断、交接内容和风险边界；\
             主岗负责把这些输入合成一份可执行结论，而不是把多个岗位压成一段普通聊天回答。"
                .to_string(),
        );
    }
    let category_workflow = category_team_workflow(root_task, nodes);
    if !category_workflow.is_empty() {
        lines.push("- **团队执行顺序**：".to_string());
        lines.extend(
            category_workflow
                .into_iter()
                .map(|item| format!("  - {item}")),
        );
    }
    lines
}

fn live_conversion_funnel_lines(root_task: &str) -> Vec<String> {
    let metrics = extract_live_conversion_metrics(root_task);
    if metrics.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let exposure = metrics.get("曝光").copied();
    let click = metrics.get("点击").copied();
    let cart = metrics.get("加购").copied();
    let order = metrics.get("下单").copied();
    let paid = metrics.get("付款").copied();

    if let (Some(exposure), Some(click)) = (exposure, click) {
        lines.push(format!(
            "曝光到点击约 {:.1}%，先看直播封面、标题、主播开场和人群匹配是否足够吸引。",
            ratio_percent(click, exposure)
        ));
    }
    if let (Some(click), Some(cart)) = (click, cart) {
        lines.push(format!(
            "点击到加购约 {:.1}%，重点检查卖点、价格利益、信任证明和讲解节奏是否承接住兴趣。",
            ratio_percent(cart, click)
        ));
    }
    if let (Some(cart), Some(order)) = (cart, order) {
        lines.push(format!(
            "加购到下单约 {:.1}%，需要排查优惠门槛、库存、运费、活动规则和犹豫点。",
            ratio_percent(order, cart)
        ));
    }
    if let (Some(order), Some(paid)) = (order, paid) {
        lines.push(format!(
            "下单到付款约 {:.1}%，如果接近 100% 说明支付环节阻力不大；后续还要看退款、投诉和履约。",
            ratio_percent(paid, order)
        ));
    }
    if let (Some(exposure), Some(paid)) = (exposure, paid) {
        lines.push(format!(
            "曝光到付款总转化约 {:.2}%，单看该链路不是结论，必须结合行业基准、客单价、\
             毛利和投放成本判断健康度。",
            ratio_percent(paid, exposure)
        ));
    }

    lines
}

fn extract_live_conversion_metrics(root_task: &str) -> HashMap<&'static str, f64> {
    const LABELS: [&str; 5] = ["曝光", "点击", "加购", "下单", "付款"];
    let mut metrics = HashMap::new();
    for label in LABELS {
        if let Some(value) = extract_number_after_label(root_task, label) {
            metrics.insert(label, value);
        }
    }
    metrics
}

fn extract_number_after_label(text: &str, label: &str) -> Option<f64> {
    let start = text.find(label)? + label.len();
    let tail = &text[start ..];
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

fn ratio_percent(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        return 0.0;
    }
    numerator / denominator * 100.0
}

fn deterministic_answer_quality_lines(
    root_task: &str,
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.is_empty() {
        return Vec::new();
    }

    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("运营策略师");
    let mut lines = Vec::new();
    if is_live_conversion_task(root_task) {
        let funnel_lines = live_conversion_funnel_lines(root_task);
        if !funnel_lines.is_empty() {
            lines.push(format!(
                "- **直播漏斗确定性初判**：{}先把用户给出的原始数字转成固定口径指标，再交给内容、\
                 服务、工程等岗位补充原因和执行边界。",
                primary
            ));
            lines.extend(funnel_lines.into_iter().map(|item| format!("  - {item}")));
            lines.push(
                "- **健康度判断**：这组数据只能说明本轮漏斗各段表现，不能单独判定行业健康度；\
                 还需要行业基准、历史同类直播、客单价、毛利、投放成本、\
                 退款和咨询承接数据一起复核。"
                    .to_string(),
            );
        }
    }

    if roles.len() >= 3 {
        lines.push(format!(
            "- **团队调度方式**：{}是主回答岗位；{}会先独立完成各自判断，再把结论、证据、\
             风险和待补数据交给主岗合成 `final_report`。",
            primary,
            roles
                .iter()
                .map(|role| role_display_name(role))
                .collect::<Vec<_>>()
                .join("、")
        ));
    }
    lines
}

fn team_role_workplans(root_task: &str, nodes: &[&TaskNode]) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.len() <= 1 {
        return Vec::new();
    }

    let category_scope = is_category_level_task(root_task)
        || contains_any(root_task, &["咖啡", "饮品", "食品", "类目", "品类"]);
    let mut items = Vec::new();
    if roles.contains(&"data") {
        items.push(
            if category_scope {
                "数据分析师：先定义类目级漏斗口径，拆曝光、点击、加购、付款、退款、\
                 咨询量和渠道来源；没有用户手动选择商品时，只给待补数据清单和复核公式。"
            } else {
                "数据分析师：先统一指标口径、样本周期和归因方式，再把异常段落交给运营判断优先级。"
            }
            .to_string(),
        );
    }
    if roles.contains(&"ops") {
        items.push(
            "运营策略师：作为主岗把各岗位输入收敛成 P0/P1/P2 动作、负责岗位、验证指标和复盘节奏。"
                .to_string(),
        );
    }
    if roles.contains(&"creative") {
        items.push(
            "内容创意师：把类目判断转成直播开场钩子、短视频脚本、标题方向和 A/B 测试假设。"
                .to_string(),
        );
    }
    if roles.contains(&"service") {
        items.push(
            "客户成功专员：补客服问答、异议处理、退款/物流承诺和异常升级 \
             SOP，避免营销承诺超过履约能力。"
                .to_string(),
        );
    }
    if roles.contains(&"engineering") {
        items.push(
            "技术架构师：确认埋点、接口、报表刷新、数据回放和失败回滚边界，\
             让团队结论可以持续复核。"
                .to_string(),
        );
    }
    if roles.contains(&"douyin_ads") {
        items.push(
            "千川投放专员：拆人群包、素材组、出价、预算节奏和止损线，\
             等真实数据补齐后再给放量判断。"
                .to_string(),
        );
    }
    if roles.contains(&"accounting") {
        items.push(
            "财务分析师：把毛利、优惠成本、广告费、退款率和 ROI 止损线绑定到运营动作。".to_string(),
        );
    }
    if roles.contains(&"web") {
        items.push(
            "Web/SEO 专家：补关键词、FAQ、页面承接和自然流量入口，避免站外曝光落到低转化页面。"
                .to_string(),
        );
    }
    items
}

fn role_summary_lines(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.role != "chat")
        .filter_map(|node| {
            let output = reports.get(&node.id)?.role_output.as_ref()?;
            let signal = first_clean_visible_answer_item(&output.recommendations)
                .or_else(|| first_clean_visible_answer_item(&output.findings))
                .or_else(|| clean_visible_answer_item(output.summary.as_str()))?;
            Some(format!(
                "{}：{}",
                role_display_name(&node.role),
                compact_for_brief(&signal, 120)
            ))
        })
        .take(5)
        .collect()
}

fn ecommerce_team_playbook(root_task: &str, nodes: &[&TaskNode]) -> Vec<String> {
    let mut lines = Vec::new();
    if nodes.iter().filter(|node| node.role != "chat").count() <= 1 {
        return lines;
    }

    if is_category_level_task(root_task) {
        lines.push(
            "- **本轮链路**：用户没有手动选择预设商品，所以系统按类目级/未指定商品链路回答；\
             历史商品和默认商品不会被注入为本轮商品。"
                .to_string(),
        );
    }

    if is_live_conversion_task(root_task) {
        lines.push(
            "- **判断框架**：直播间转化是否健康，先看曝光到点击的吸引力，再看点击到加购的承接力，\
             接着看加购到付款的购买阻力，最后看退款、投诉、客服咨询和履约风险。"
                .to_string(),
        );
    } else if contains_any(root_task, &["咖啡", "饮品", "食品", "类目", "品类"]) {
        lines.push(
            "- **类目框架**：先按人群、场景、价格带、内容卖点、渠道承接和复购风险拆解，\
             不编造单品指标。"
                .to_string(),
        );
    }

    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.contains(&"data") {
        lines.push(
            "- **数据岗接力**：负责定义指标口径、样本周期和漏斗节点；\
             没有真实数据时只输出诊断框架和待补齐数据，不把缺数据包装成已查询结果。"
                .to_string(),
        );
    }
    if roles.contains(&"ops") {
        lines.push(
            "- **运营岗接力**：负责把岗位输入收敛为今日可执行动作、复盘节奏和优先级，\
             避免只给泛泛建议。"
                .to_string(),
        );
    }
    if roles.contains(&"creative") {
        lines.push(
            "- **内容岗接力**：把判断转成标题、直播脚本、短视频钩子和素材 A/B 测试方向。"
                .to_string(),
        );
    }
    if roles.contains(&"design") {
        lines.push(
            "- **设计岗接力**：把卖点、信任证明、价格利益和行动按钮落到主图、\
             详情页或直播视觉承接。"
                .to_string(),
        );
    }
    if roles.contains(&"service") {
        lines.push(
            "- **服务岗接力**：同步客服答疑、售后承诺和异常升级 \
             SOP，防止营销承诺超过真实履约能力。"
                .to_string(),
        );
    }
    if roles.contains(&"engineering") {
        lines.push(
            "- **工程岗接力**：确认埋点、数据接口、刷新时间和回滚边界，让后续复盘有可验证依据。"
                .to_string(),
        );
    }

    lines
}

fn mode_intro_line(mode: TaskGraphAnswerMode) -> &'static str {
    match mode {
        TaskGraphAnswerMode::Quick => "快速简答已完成。",
        TaskGraphAnswerMode::Execution => "丰富执行模式已完成，多智能体任务图已形成可落地方案。",
        TaskGraphAnswerMode::Learning => "教学模式已完成，多智能体任务图已形成可讲解的业务结论。",
    }
}

fn total_answer_brief_lines(
    root_task: &str,
    primary_node: Option<&TaskNode>,
    primary_summary: Option<&str>,
    support_count: usize,
    mode: TaskGraphAnswerMode,
) -> Vec<String> {
    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("主回答岗位");
    let mut lines = Vec::new();

    if is_live_conversion_task(root_task) {
        lines.push(format!(
            "- **一句话结论**：{}先按直播漏斗回答，重点看曝光、点击、加购、付款、\
             退款和咨询承接是否逐段稳定；当前没有真实完整数据时，只能先给判断框架和待补数据清单。",
            primary
        ));
    } else if contains_any(
        root_task,
        &["咖啡", "饮品", "食品", "类目", "品类", "商品类"],
    ) {
        lines.push(format!(
            "- **一句话结论**：{}先按类目/未指定商品链路回答，拆人群、场景、价格带、内容卖点、\
             渠道承接和复购风险；不会把历史商品或默认商品当成本轮指定商品。",
            primary
        ));
    } else {
        lines.push(format!(
            "- **一句话结论**：{}负责收敛最终判断，协作岗位补齐证据、动作、风险和交付物。",
            primary
        ));
    }

    if let Some(summary) = primary_summary
        .map(strip_list_marker)
        .map(|summary| compact_for_brief(summary, 130))
        .filter(|summary| !summary.is_empty())
    {
        lines.push(format!("- **主岗摘要**：{summary}"));
    }

    if support_count > 0 {
        lines.push(format!(
            "- **团队协作**：{} 牵头，{} 个协作岗位分别独立判断后交给主岗合成 `final_report`。",
            primary, support_count
        ));
    }

    if is_unselected_product_task(root_task) || is_category_level_task(root_task) {
        lines.push(
            "- **商品边界**：只有用户手动勾选预设商品后，后端才注入该商品资料；\
             本轮未选商品时走普通类目/未指定商品链路。"
                .to_string(),
        );
    }

    match mode {
        TaskGraphAnswerMode::Quick => {
            lines.push(
                "- **阅读方式**：这里给结论和关键原因，完整岗位过程在右侧过程面板和运行记录中。"
                    .to_string(),
            );
        }
        TaskGraphAnswerMode::Learning => {
            lines.push(
                "- **阅读方式**：先看这一段拿结论，\
                 再看“正文分析”的解释和“虚拟员工分工”的岗位协作。"
                    .to_string(),
            );
        }
        TaskGraphAnswerMode::Execution => {
            lines.push(
                "- **阅读方式**：先看结论，再看“丰富执行方案”的负责人、动作、验收方式和风险边界。"
                    .to_string(),
            );
        }
    }

    lines
}

fn quick_role_rows(
    nodes: &[&TaskNode],
    primary_node: Option<&TaskNode>,
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.role != "chat")
        .filter_map(|node| {
            let output = reports.get(&node.id)?.role_output.as_ref()?;
            let finding = first_visible_non_empty(&output.findings)
                .or_else(|| visible_non_empty(output.summary.as_str()))
                .unwrap_or("已形成本岗位判断");
            let action = first_visible_non_empty(&output.recommendations)
                .or_else(|| first_visible_non_empty(&output.risks))
                .unwrap_or("交给主岗整合");
            let role_type = if primary_node.is_some_and(|primary| primary.id == node.id) {
                "主岗"
            } else {
                "协作岗"
            };
            Some(format!(
                "| {} | {} | {} | {} |",
                role_type,
                role_display_name(&node.role),
                markdown_table_cell(strip_list_marker(finding)),
                markdown_table_cell(strip_list_marker(action))
            ))
        })
        .collect()
}

fn teaching_explanation_lines(
    root_task: &str,
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    depth: TaskGraphLearningDepth,
) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("主回答岗位");
    lines.push(format!(
        "- **怎么读这份回答**：先看“总回答”拿结论，再看“虚拟员工分工”理解每个岗位为什么这么判断，\
         最后看主岗方案决定下一步先做什么。{}负责把团队意见收口。",
        primary
    ));
    if !matches!(depth, TaskGraphLearningDepth::Low) && is_live_conversion_task(root_task) {
        lines.push(
            "- **关键概念**：直播漏斗要逐段看，曝光到点击看吸引力，点击到加购看商品承接，\
             加购到付款看价格、信任和下单阻力。"
                .to_string(),
        );
    }
    if is_category_level_task(root_task) {
        lines.push(
            "- **商品边界**：未手动选择预设商品时，只能做类目级判断；选择了具体商品后，\
             才进入单品资料注入链路。"
                .to_string(),
        );
    }
    match depth {
        TaskGraphLearningDepth::Low => {
            lines.push(
                "- **学习重点**：先抓结论、负责岗位和下一步动作，暂时不展开太多术语。".to_string(),
            );
        }
        TaskGraphLearningDepth::Medium => {
            lines.push(
                "- **继续追问建议**：可以继续补充预算、平台、商品资料、历史转化或素材方向，\
                 系统会在同一会话里把这些信息作为上下文继续使用。"
                    .to_string(),
            );
        }
        TaskGraphLearningDepth::High => {
            lines.push(
                "- **判断逻辑**：先确认数据和商品边界，再让各岗位独立输出证据、动作、风险，\
                 最后由主岗把不同岗位的判断合成为可执行方案。"
                    .to_string(),
            );
            lines.push(
                "- **继续追问建议**：\
                 可以继续追问“按预算拆执行节奏”“按人群写直播话术”“按岗位导出执行表”，\
                 系统会沿用同一会话上下文继续推演。"
                    .to_string(),
            );
        }
    }
    lines
}

fn quick_reason_lines(
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(summary) = primary_answer_summary(primary_node, nodes, reports) {
        lines.push(format!("- **关键原因**：{}", strip_list_marker(&summary)));
    }
    for node in nodes.iter().filter(|node| node.role != "chat") {
        let Some(output) = reports
            .get(&node.id)
            .and_then(|report| report.role_output.as_ref())
        else {
            continue;
        };
        if let Some(finding) = first_visible_non_empty(&output.findings) {
            lines.push(format!(
                "- **{}**：{}",
                role_display_name(&node.role),
                strip_list_marker(finding)
            ));
        }
    }
    unique_strings(lines)
}

fn execution_context_lines(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.role != "chat")
        .filter_map(|node| {
            let output = reports.get(&node.id)?.role_output.as_ref()?;
            let finding = first_visible_non_empty(&output.findings)
                .or_else(|| visible_non_empty(output.summary.as_str()))?;
            Some(format!(
                "- **{}**：{}",
                role_display_name(&node.role),
                strip_list_marker(finding)
            ))
        })
        .collect()
}

fn team_collaboration_contract_lines(
    root_task: &str,
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    mode: TaskGraphAnswerMode,
) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.len() <= 1 {
        return Vec::new();
    }

    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("主回答岗位");
    let mut lines = Vec::new();
    lines.push(format!(
        "- **团队报告结构**：本轮不是把多个岗位回复简单拼接，而是由{}先收口主结论，\
         其他虚拟员工各自完成独立判断，再把证据、动作、风险和待补数据交给主岗统一合成。",
        primary
    ));
    if is_unselected_product_task(root_task) {
        lines.push(
            "- **商品链路**：本轮没有使用任何默认商品或历史商品；\
             只有用户在输入区手动勾选预设商品后，后端才会把该商品资料注入单品级分析。"
                .to_string(),
        );
    }
    lines.push(
        "- **会话上下文**：同一会话内的上一轮问答会作为任务背景参与理解，\
         但不会替代用户本轮的商品选择；这保证上下文连续，同时避免把旧商品误当成本轮商品。"
            .to_string(),
    );

    for role in unique_role_order(&roles) {
        if let Some(line) = role_contract_line(role, &roles) {
            lines.push(format!("- **{}**：{}", role_display_name(role), line));
        }
    }

    if matches!(mode, TaskGraphAnswerMode::Execution) {
        lines.push(
            "- **丰富执行要求**：主回答岗位必须把岗位判断转成可执行清单、负责人、\
             验收指标和风险边界；协作岗输出不是装饰，而是最终方案的证据来源。"
                .to_string(),
        );
    }
    lines.push(format!(
        "- **过程文件**：后端会把最终答复沉淀为 \
         `final_report`；{}如果产出结构化过程，也会进入对应 artifact，供后续预览、追问和导出。",
        artifact_role_summary(&roles)
    ));
    lines
}

fn unique_role_order<'a>(roles: &'a [&'a str]) -> Vec<&'a str> {
    let mut result = Vec::new();
    for role in [
        "data",
        "ops",
        "creative",
        "design",
        "service",
        "engineering",
        "douyin_ads",
        "accounting",
        "web",
    ] {
        if roles.contains(&role) {
            result.push(role);
        }
    }
    for &role in roles {
        if !result.contains(&role) {
            result.push(role);
        }
    }
    result
}

fn team_judgement_sequence(roles: &[&str]) -> Option<String> {
    let mut steps = Vec::new();
    if roles.contains(&"data") {
        steps.push("数据岗先统一指标口径");
    }
    if roles.contains(&"ops") {
        steps.push("运营岗再判断动作优先级");
    }
    if roles.contains(&"creative") {
        steps.push("内容岗把卖点转成标题、脚本和素材测试");
    }
    if roles.contains(&"design") {
        steps.push("设计岗承接主图、详情页和直播视觉");
    }
    if roles.contains(&"service") {
        steps.push("客服岗兜底咨询、售后和履约承诺");
    }
    if roles.contains(&"engineering") {
        steps.push("工程岗兜底埋点、接口和复盘可验证性");
    }
    if roles.contains(&"douyin_ads") {
        steps.push("投放岗校准人群、素材、出价和预算节奏");
    }
    if roles.contains(&"accounting") {
        steps.push("财务岗校准毛利、预算和 ROI 止损线");
    }
    if roles.contains(&"web") {
        steps.push("Web/SEO 岗补搜索入口和页面承接");
    }
    if steps.is_empty() {
        return None;
    }
    Some(format!("{}。", steps.join("，")))
}

fn team_execution_direction(roles: &[&str]) -> Option<String> {
    let mut parts = Vec::new();
    if roles.contains(&"data") {
        parts.push("先形成类目画像和指标口径");
    }
    if roles.contains(&"creative") {
        parts.push("再做标题、脚本和素材测试");
    }
    if roles.contains(&"design") {
        parts.push("再做主图、详情页和直播视觉承接");
    }
    if roles.contains(&"service") {
        parts.push("同步客服问答和售后承接");
    }
    if roles.contains(&"engineering") {
        parts.push("补齐埋点和复盘口径");
    }
    if roles.contains(&"douyin_ads") {
        parts.push("按真实预算和人群包校准投放节奏");
    }
    if roles.contains(&"accounting") {
        parts.push("用毛利和 ROI 约束放量边界");
    }
    if roles.contains(&"web") {
        parts.push("补搜索关键词、FAQ 和页面承接");
    }
    if roles.contains(&"ops") {
        parts.push("最后由运营岗合成执行节奏");
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("，"))
}

fn support_scope_summary_from_roles(roles: &[&str], exclude_role: Option<&str>) -> String {
    let scopes = unique_strings(
        roles
            .iter()
            .filter(|role| **role != "chat")
            .filter(|role| {
                exclude_role
                    .map(|exclude| **role != exclude)
                    .unwrap_or(true)
            })
            .filter_map(|role| role_support_scope(role).map(str::to_string))
            .collect(),
    );
    if scopes.is_empty() {
        return "本轮岗位输入".to_string();
    }
    scopes.join("、")
}

fn artifact_role_summary(roles: &[&str]) -> String {
    let role_names = unique_strings(
        roles
            .iter()
            .filter(|role| **role != "chat")
            .map(|role| role_display_name(role).to_string())
            .collect(),
    );
    if role_names.is_empty() {
        return "参与岗位".to_string();
    }
    format!("{}等参与岗位", role_names.join("、"))
}

fn role_contract_line(role: &str, roles: &[&str]) -> Option<String> {
    match role {
        "data" => Some(
            "先把曝光、点击、加购、付款、咨询、退款等指标口径拆清楚；\
             没有真实指标时输出待补数据清单和判断框架，不编造查询结果。"
                .to_string(),
        ),
        "ops" => Some(format!(
            "把{}收敛成优先级、执行节奏、复盘节点和可落地清单。",
            support_scope_summary_from_roles(roles, Some("ops"))
        )),
        "creative" => Some(
            "把策略转成直播开场钩子、短视频脚本、标题利益点和 A/B 测试假设，负责让方案能变成素材。"
                .to_string(),
        ),
        "design" => Some(
            "把卖点、信任证明、价格利益和行动按钮落到主图、详情页或直播间视觉承接上。".to_string(),
        ),
        "service" => Some(
            "补齐客服问答、异议处理、售后承诺、退款物流边界和异常升级 \
             SOP，防止营销承诺超过履约能力。"
                .to_string(),
        ),
        "engineering" => Some(
            "确认埋点、接口、数据刷新、报表口径和失败回滚，让后续复盘可以验证每个动作。"
                .to_string(),
        ),
        "douyin_ads" => Some(
            "拆解人群、素材组、出价、预算节奏和止损阈值，避免把投放结果只归因给单一因素。"
                .to_string(),
        ),
        "accounting" => {
            Some("把毛利、优惠成本、广告费、退款率和现金流约束转成 CPA/ROI 止损线。".to_string())
        }
        "web" => Some(
            "补充搜索关键词、FAQ、页面结构和自然流量承接，避免站外流量进入低转化页面。".to_string(),
        ),
        _ => None,
    }
}

fn category_team_workflow(root_task: &str, nodes: &[&TaskNode]) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.len() <= 1 || !is_category_level_task(root_task) {
        return Vec::new();
    }

    let mut items = Vec::new();
    if roles.contains(&"data") {
        items.push(
            "数据诊断先给出类目级漏斗口径和待补指标，不把任何历史商品当成默认样本。".to_string(),
        );
    }
    for role in &roles {
        match *role {
            "creative" => items.push(
                "内容创意师把类目判断转成卖点测试、直播钩子、标题方向和素材 A/B 假设。".to_string(),
            ),
            "service" => items
                .push("客户成功专员补齐咨询承接、售后承诺、退款物流和异常升级边界。".to_string()),
            "engineering" => {
                items.push("技术架构师补齐埋点复盘、接口刷新、报表口径和失败回滚边界。".to_string())
            }
            "design" => items.push(
                "设计师把卖点、信任证明和行动按钮落到主图、详情页或直播间视觉承接。".to_string(),
            ),
            "web" => {
                items.push("Web/SEO 专家补充搜索关键词、FAQ、页面结构和自然流量承接。".to_string())
            }
            _ => {}
        }
    }
    if roles.contains(&"ops") {
        items.push(
            "运营策略师最后把上述输入合成执行节奏、风险边界和 `final_report` 交付物。".to_string(),
        );
    }
    if roles.contains(&"douyin_ads") {
        items.push("千川投放岗只在真实预算、人群包和素材数据补齐后计算放量节奏。".to_string());
    }
    if roles.contains(&"accounting") {
        items.push("财务岗负责把毛利、优惠成本、广告费和退款率变成 CPA/ROI 止损线。".to_string());
    }
    items
}

fn ecommerce_team_execution_draft(
    root_task: &str,
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
    mode: TaskGraphAnswerMode,
) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.len() <= 1 {
        return Vec::new();
    }

    let primary = primary_node
        .map(|node| role_display_name(&node.role))
        .unwrap_or("主回答岗位");
    let category_line = if is_category_level_task(root_task) {
        "本轮未选择预设商品，先按类目级链路处理；不能把历史商品或默认商品当成本轮商品。"
    } else {
        "本轮按用户当前问题和已选择上下文处理；若需要单品级指标，必须以用户显式选择的商品资料为准。"
    };
    let first_sentence = if is_live_conversion_task(root_task) {
        "判断直播间转化是否健康，不是只看最终付款，而是看曝光、点击、加购、付款、\
         退款和咨询承接是否逐段稳定。"
    } else if contains_any(root_task, &["咖啡", "饮品", "食品", "类目", "品类"]) {
        "咖啡类商品应先按人群、场景、价格带、内容卖点、渠道承接和复购风险拆解，\
         再进入单品级指标复核。"
    } else {
        "先把用户目标拆成指标、动作、风险和交付物四个层级，再由对应岗位分头判断。"
    };

    let mut lines = vec![format!(
        "- **主结论**：{first_sentence}{category_line} \
         {primary}负责把各岗位结论收敛为最终行动节奏。"
    )];
    if matches!(mode, TaskGraphAnswerMode::Execution) {
        let rows = execution_action_table_rows(nodes, reports);
        if !rows.is_empty() {
            lines.push(String::new());
            lines.push("| 优先级 | 负责人 | 动作 | 验收方式 | 风险边界 |".to_string());
            lines.push("| --- | --- | --- | --- | --- |".to_string());
            lines.extend(rows);
            lines.push(String::new());
        }
    }
    if roles.contains(&"data") {
        lines.push(
            "- **数据分析师**：先定义可复核口径，至少包括曝光、点击、加购、付款、退款、\
             咨询量和渠道拆分；没有真实数据时输出诊断框架和待补齐清单，不编造数值。"
                .to_string(),
        );
    }
    if roles.contains(&"ops") {
        lines.push(format!(
            "- **运营策略师**：把{}合并为优先动作：先补关键口径，再做已参与岗位能承接的测试，\
             最后建立日复盘节奏。",
            support_scope_summary_from_roles(&roles, Some("ops"))
        ));
    }
    if roles.contains(&"creative") {
        lines.push(
            "- **内容创意师**：把判断转成可执行素材，例如直播开场钩子、利益点标题、短视频脚本和 \
             A/B 测试假设。"
                .to_string(),
        );
    }
    if roles.contains(&"design") {
        lines.push(
            "- **设计师**：把卖点、信任证明、价格利益和行动按钮放进主图、详情页或直播间视觉承接，\
             不让用户理解成本过高。"
                .to_string(),
        );
    }
    if roles.contains(&"service") {
        lines.push(
            "- **客户成功专员**：同步客服问答、售后承诺、退款/物流边界和异常升级 \
             SOP，避免营销话术超出履约能力。"
                .to_string(),
        );
    }
    if roles.contains(&"engineering") {
        lines.push(
            "- **技术架构师**：确认埋点、接口、数据刷新、报表口径和失败回滚，\
             让后续复盘可以追踪到每一次改动。"
                .to_string(),
        );
    }
    if roles.contains(&"douyin_ads") {
        lines.push(
            "- **千川投放专员**：把人群、素材组、出价、预算节奏和止损阈值拆开，\
             避免把投放问题全部归因给素材或主播。"
                .to_string(),
        );
    }
    if roles.contains(&"accounting") {
        lines.push(
            "- **财务分析师**：先确认毛利、预算、CPA/ROI 止损线，再判断动作是否值得放量。"
                .to_string(),
        );
    }
    if roles.contains(&"web") {
        lines.push(
            "- **Web/SEO \
             专家**：补充搜索关键词、FAQ、页面结构和自然流量承接，不把站外曝光浪费在低承接页面。"
                .to_string(),
        );
    }
    lines.push(
        "- **交付物**：本轮会沉淀为 \
         `final_report`，报告中保留岗位分工、过程依据、风险边界和下一步清单，\
         方便后续在同一会话继续追问。"
            .to_string(),
    );
    lines
}

fn execution_action_table_rows(
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.role != "chat")
        .filter_map(|node| {
            let output = reports.get(&node.id)?.role_output.as_ref()?;
            let action = first_visible_non_empty(&output.recommendations)
                .or_else(|| visible_non_empty(output.summary.as_str()))
                .unwrap_or("按本岗位边界补齐执行动作");
            let risk = first_visible_non_empty(&output.risks)
                .or_else(|| first_visible_non_empty(&output.open_questions))
                .unwrap_or("持续复核数据和执行边界");
            Some(format!(
                "| {} | {} | {} | {} | {} |",
                role_priority_label(&node.role),
                role_display_name(&node.role),
                markdown_table_cell(strip_list_marker(action)),
                markdown_table_cell(role_acceptance_hint(&node.role)),
                markdown_table_cell(strip_list_marker(risk))
            ))
        })
        .collect()
}

fn role_priority_label(role: &str) -> &'static str {
    match role {
        "data" => "P0",
        "ops" => "P0",
        "douyin_ads" | "accounting" => "P1",
        "creative" | "design" | "service" | "engineering" => "P1",
        _ => "P2",
    }
}

fn role_acceptance_hint(role: &str) -> &'static str {
    match role {
        "data" => "漏斗口径、样本周期和异常段可复核",
        "ops" => "形成优先级、节奏、负责人和复盘节点",
        "creative" => "至少产出可测试标题/脚本/素材方向",
        "design" => "主图、详情页或直播视觉承接点明确",
        "service" => "客服话术、售后承诺和升级 SOP 明确",
        "engineering" => "埋点、报表刷新和回滚边界可验证",
        "douyin_ads" => "人群、素材、出价、预算和止损线明确",
        "accounting" => "毛利、CPA/ROI 和现金流边界明确",
        "web" => "关键词、FAQ 和页面转化路径明确",
        _ => "本岗位产物可被主岗继续引用",
    }
}

fn is_live_conversion_task(task: &str) -> bool {
    contains_any(
        task,
        &["直播间", "直播", "转化", "转化率", "点击", "加购", "付款"],
    )
}

fn executive_team_brief(
    primary_node: Option<&TaskNode>,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.len() <= 1 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    if let Some(primary) = primary_node {
        lines.push(format!(
            "**团队判断**：{}先收敛主结论，其他岗位分别补充自己负责的证据、动作和风险；\
             下面可以按“数据先定口径、业务再定动作、服务/工程兜底边界”的顺序执行。",
            role_display_name(&primary.role)
        ));
    }

    let data_point = first_role_recommendation("data", nodes, reports)
        .or_else(|| first_role_finding("data", nodes, reports));
    let ops_point = first_role_recommendation("ops", nodes, reports)
        .or_else(|| first_role_finding("ops", nodes, reports));
    let creative_point = first_role_recommendation("creative", nodes, reports)
        .or_else(|| first_role_finding("creative", nodes, reports));
    let service_point = first_role_recommendation("service", nodes, reports)
        .or_else(|| first_role_finding("service", nodes, reports));
    let engineering_point = first_role_recommendation("engineering", nodes, reports)
        .or_else(|| first_role_finding("engineering", nodes, reports));

    if data_point.is_some() || ops_point.is_some() {
        lines.push(format!(
            "- **先判健康度**：{}{}",
            data_point
                .as_deref()
                .map(strip_list_marker)
                .unwrap_or("先统一曝光、点击、加购、付款、退款等漏斗口径。"),
            ops_point
                .as_deref()
                .map(|point| format!("；运营侧据此执行：{}", strip_list_marker(point)))
                .unwrap_or_default()
        ));
    }
    if creative_point.is_some() || service_point.is_some() {
        lines.push(format!(
            "- **再做转化承接**：{}{}",
            creative_point
                .as_deref()
                .map(strip_list_marker)
                .unwrap_or("把类目判断转成标题、脚本、素材或直播间话术。"),
            service_point
                .as_deref()
                .map(|point| format!("；客服侧同步：{}", strip_list_marker(point)))
                .unwrap_or_default()
        ));
    }
    if let Some(point) = engineering_point {
        lines.push(format!(
            "- **最后保留可验证性**：{}",
            strip_list_marker(&point)
        ));
    }

    lines
}

fn first_role_finding(
    role: &str,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Option<String> {
    first_role_item(role, nodes, reports, |output| &output.findings)
}

fn first_role_recommendation(
    role: &str,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Option<String> {
    first_role_item(role, nodes, reports, |output| &output.recommendations)
}

fn first_role_item(
    role: &str,
    nodes: &[&TaskNode],
    reports: &HashMap<TaskNodeId, WorkerReport>,
    select: impl Fn(&RoleWorkOutput) -> &Vec<String>,
) -> Option<String> {
    nodes
        .iter()
        .filter(|node| node.role == role)
        .filter_map(|node| reports.get(&node.id)?.role_output.as_ref())
        .flat_map(select)
        .find(|item| visible_non_empty(item).is_some())
        .cloned()
}

fn collaboration_handoffs(nodes: &[TaskNode]) -> Vec<String> {
    let node_by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();

    nodes
        .iter()
        .filter(|node| !node.input_refs.is_empty())
        .map(|node| {
            let inputs = node
                .input_refs
                .iter()
                .filter_map(|input| node_by_id.get(input))
                .map(|input| role_display_name(&input.role))
                .collect::<Vec<_>>()
                .join("、");
            format!(
                "- **{}**：先读取 {} \
                 的结论、证据和风险边界，再补充自己的岗位判断，最后交给主回答岗位统一收敛。",
                role_display_name(&node.role),
                inputs
            )
        })
        .collect()
}

fn inferred_collaboration_handoffs(nodes: &[&TaskNode]) -> Vec<String> {
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    if roles.len() <= 1 {
        return Vec::new();
    }

    let mut items = Vec::new();
    if roles.contains(&"ops") {
        items.push("- 运营策略师负责把各岗位判断收敛成主结论、优先级和复盘节奏。".to_string());
    }
    if roles.contains(&"data") && roles.contains(&"ops") {
        items.push(
            "- 数据分析师先给指标口径和漏斗边界，运营策略师再据此决定动作优先级。".to_string(),
        );
    }
    if roles.contains(&"creative") && roles.contains(&"ops") {
        items.push(
            "- 内容创意师把运营目标转成标题、脚本和素材测试方向，避免只停留在策略层。".to_string(),
        );
    }
    if roles.contains(&"service") {
        items.push(
            "- 客户成功专员校验客服话术、售后承诺和异常升级，防止营销承诺超过履约能力。"
                .to_string(),
        );
    }
    if roles.contains(&"engineering") {
        items.push(
            "- 技术架构师校验埋点、接口、数据刷新和回滚边界，确保团队结论可追踪、可复核。"
                .to_string(),
        );
    }
    if roles.contains(&"accounting") || roles.contains(&"douyin_ads") {
        items.push(
            "- 财务或投放岗位补充预算、ROI、出价和止损边界，帮助团队把方案落到成本约束内。"
                .to_string(),
        );
    }
    items
}

fn role_brief_table(
    nodes: &[&TaskNode],
    primary_node: Option<&TaskNode>,
    reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Vec<String> {
    nodes
        .iter()
        .filter(|node| node.role != "chat")
        .filter_map(|node| {
            let output = reports.get(&node.id)?.role_output.as_ref()?;
            let finding = first_visible_non_empty(&output.findings)
                .and_then(clean_visible_answer_item)
                .or_else(|| clean_visible_answer_item(output.summary.as_str()))
                .unwrap_or_else(|| "已形成本岗位判断".to_string());
            let recommendation = first_clean_visible_answer_item(&output.recommendations)
                .or_else(|| first_clean_visible_answer_item(&output.risks))
                .or_else(|| first_clean_visible_answer_item(&output.open_questions))
                .unwrap_or_else(|| "继续交给主岗整合".to_string());
            let role_type = if primary_node.is_some_and(|primary| primary.id == node.id) {
                "主回答岗位"
            } else {
                "协作岗位"
            };
            Some(format!(
                "| {} | {} | {} | {} | {} |",
                role_type,
                role_display_name(&node.role),
                markdown_table_cell(role_responsibility(&node.role)),
                markdown_table_cell(&format!(
                    "{}；{}",
                    finding,
                    role_team_contribution(&node.role)
                )),
                markdown_table_cell(&recommendation)
            ))
        })
        .collect()
}

fn missing_data_table_rows(root_task: &str, nodes: &[&TaskNode]) -> Vec<String> {
    if !is_live_conversion_task(root_task) && !is_category_level_task(root_task) {
        return Vec::new();
    }
    let roles = nodes
        .iter()
        .filter(|node| node.role != "chat")
        .map(|node| node.role.as_str())
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    rows.push(
        "| 曝光、点击、加购、付款、退款、咨询量 | 定位直播漏斗断点 | 数据分析师 | \
         只能输出诊断框架，不编造转化率 |"
            .to_string(),
    );
    rows.push(
        "| 样本周期、渠道来源、直播 ID | 判断波动是否来自时间段或流量来源 | 数据分析师 | \
         只能做类目级方向判断 |"
            .to_string(),
    );
    if roles.contains(&"creative") {
        rows.push(
            "| 标题、脚本、短视频钩子、直播间话术 | 复盘内容是否承接人群和卖点 | 内容创意师 | \
             只能给素材假设，不能判断真实素材优劣 |"
                .to_string(),
        );
    }
    if roles.contains(&"ops") {
        rows.push(
            "| 平台、预算、促销、库存和价格带 | 排执行优先级和复盘节奏 | 运营策略师 | \
             只能给执行顺序，不能替代单品级决策 |"
                .to_string(),
        );
    }
    if is_unselected_product_task(root_task) || is_category_level_task(root_task) {
        rows.push(
            "| 用户手动选择的预设商品资料 | 进入单品级回答链路 | 运营策略师 | \
             未选择时不注入默认商品或历史商品 |"
                .to_string(),
        );
    }
    rows
}

fn role_responsibility(role: &str) -> &'static str {
    match role {
        "data" => "指标口径、样本周期、异常段和可复核证据",
        "ops" => "业务目标、优先级、执行节奏和最终汇总",
        "service" => "客服问答、售后承诺、退款物流和服务风险",
        "creative" => "卖点、标题、脚本、素材测试和内容承接",
        "engineering" => "埋点、接口、报表刷新、链路验证和回滚",
        "design" => "主图、详情页、直播视觉和行动按钮承接",
        "accounting" => "毛利、预算、CPA、ROI 和现金流边界",
        "douyin_ads" => "千川人群、素材组、出价、预算和止损线",
        "web" => "搜索入口、FAQ、页面结构和自然流量承接",
        _ => "本岗位职责内的判断、证据、风险和交接内容",
    }
}

fn role_team_contribution(role: &str) -> &'static str {
    match role {
        "data" => "交给团队的是指标口径、数据缺口和复核方式",
        "ops" => "交给团队的是优先级、执行节奏和最终收敛",
        "service" => "交给团队的是客服承接、售后边界和升级 SOP",
        "creative" => "交给团队的是内容钩子、脚本方向和素材测试假设",
        "engineering" => "交给团队的是埋点、接口、刷新和回滚边界",
        "design" => "交给团队的是视觉层级、页面承接和素材版式",
        "accounting" => "交给团队的是预算、毛利、ROI 和止损约束",
        "douyin_ads" => "交给团队的是人群、素材、出价和预算测试节奏",
        "web" => "交给团队的是关键词、FAQ 和页面自然流量承接",
        _ => "交给团队的是本岗位边界内的判断和待复核事项",
    }
}

fn first_non_empty(items: &[String]) -> Option<&str> {
    items.iter().find_map(|item| non_empty(item.as_str()))
}

fn first_visible_non_empty(items: &[String]) -> Option<&str> {
    items
        .iter()
        .find_map(|item| visible_non_empty(item.as_str()))
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

fn visible_non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()
        && !is_heading_only_fragment(value)
        && !is_unscoped_role_claim(value)
        && !is_unsupported_benchmark_claim(value)
        && !is_unsupported_external_tool_claim(value))
    .then_some(value)
}

fn clean_visible_answer_item(value: &str) -> Option<String> {
    let value = visible_non_empty(value)?;
    let value = strip_list_marker(value.trim()).trim();
    if value.is_empty()
        || is_heading_only_fragment(value)
        || is_raw_markdown_table_fragment(value)
        || is_markdown_heading_fragment(value)
    {
        return None;
    }
    let normalized = value
        .replace('\r', "")
        .split('\n')
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !is_raw_markdown_table_fragment(line)
                && !is_markdown_heading_fragment(line)
                && !is_heading_only_fragment(line)
        })
        .collect::<Vec<_>>()
        .join("；");
    let normalized = normalized
        .split('；')
        .map(str::trim)
        .filter(|part| {
            !part.is_empty()
                && !is_raw_markdown_table_fragment(part)
                && !is_markdown_heading_fragment(part)
                && !is_heading_only_fragment(part)
        })
        .collect::<Vec<_>>()
        .join("；");
    (!normalized.is_empty()).then_some(normalized)
}

fn clean_visible_answer_items(items: &[String]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| clean_visible_answer_item(item))
        .collect()
}

fn first_clean_visible_answer_item(items: &[String]) -> Option<String> {
    items
        .iter()
        .find_map(|item| clean_visible_answer_item(item))
}

fn is_markdown_heading_fragment(item: &str) -> bool {
    let value = strip_list_marker(item.trim()).trim();
    value.starts_with('#') || value.contains("Markdown 表格")
}

fn is_raw_markdown_table_fragment(item: &str) -> bool {
    let value = item.trim();
    if !value.contains('|') {
        return false;
    }
    value.starts_with('|')
        || value.ends_with('|')
        || value.contains("|---")
        || value.contains("---|")
        || value.matches('|').count() >= 2
}

fn support_scope_summary(nodes: &[&TaskNode], primary_node: Option<&TaskNode>) -> String {
    let scopes = unique_strings(
        nodes
            .iter()
            .filter(|node| node.role != "chat")
            .filter(|node| {
                primary_node
                    .map(|primary| primary.id != node.id)
                    .unwrap_or(true)
            })
            .filter_map(|node| role_support_scope(&node.role).map(str::to_string))
            .collect(),
    );
    if scopes.is_empty() {
        return "各自岗位边界".to_string();
    }
    scopes.join("、")
}

fn role_support_scope(role: &str) -> Option<&'static str> {
    match role {
        "data" => Some("指标口径"),
        "service" => Some("服务承接"),
        "creative" => Some("内容素材"),
        "engineering" => Some("技术复盘"),
        "design" => Some("视觉承接"),
        "accounting" => Some("预算与利润边界"),
        "douyin_ads" => Some("投放节奏"),
        "web" => Some("搜索与页面承接"),
        "ops" => Some("运营节奏"),
        _ => None,
    }
}

fn is_heading_only_fragment(item: &str) -> bool {
    let value = strip_list_marker(item.trim())
        .trim_matches(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | '*' | '#'))
        .trim();
    if value.is_empty() {
        return false;
    }
    let normalized = value.trim_end_matches(['：', ':']).trim();
    matches!(
        normalized,
        "标题方向"
            | "脚本方向"
            | "直播钩子"
            | "卖点方向"
            | "脚本/标题方向"
            | "素材方向"
            | "推荐动作"
            | "行动建议"
            | "下一步"
            | "风险"
            | "待补充"
            | "发现"
            | "结论"
            | "建议"
    )
}

fn is_unscoped_role_claim(item: &str) -> bool {
    contains_any(
        item,
        &[
            "内容/设计岗",
            "内容/服务/工程岗位",
            "客服和工程岗",
            "客服/技术团队",
            "服务和工程岗位",
            "数据、内容、服务和工程",
            "数据、内容、服务、投放和工程",
            "技术支持检查",
            "财务确认预算",
            "客服准备 FAQ",
            "与客服/技术团队确认",
            "支付通道",
        ],
    )
}

fn markdown_table_cell(value: &str) -> String {
    value
        .replace('|', " / ")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_string()
}

fn role_display_name(role: &str) -> &'static str {
    match role {
        "ops" => "运营策略师",
        "data" => "数据分析师",
        "service" => "客户成功专员",
        "design" => "设计师",
        "accounting" => "财务分析师",
        "engineering" => "技术架构师",
        "web" => "Web/SEO 专家",
        "creative" => "内容创意师",
        "douyin_ads" => "千川投放专员",
        "chat" => "对话助手",
        _ => "虚拟员工",
    }
}

fn contains_data_gap(value: &str) -> bool {
    contains_any(
        value,
        &[
            "缺少",
            "缺乏",
            "未提供",
            "未获取",
            "未检索",
            "未查询",
            "未拿到",
            "当前没有",
            "没有真实",
            "没有指定",
            "缺",
            "需要补充",
            "需要提供",
            "需要获取",
            "需要查询",
            "待补充",
            "无法量化",
            "不能承诺",
            "不能编造",
        ],
    )
}

fn evaluate_role_output(
    role: &str,
    output: &RoleWorkOutput,
    policy: &EvaluationPolicy,
) -> Vec<String> {
    let mut reasons = Vec::new();
    let role = role.trim();

    if output.summary.trim().is_empty() {
        reasons.push("role output summary is empty".to_string());
    }
    if role != "chat" && open_questions_are_blocking(Some(output), &output.open_questions) {
        reasons.push("role output has open questions".to_string());
    }
    if output.summary.contains("已调用")
        || is_unsupported_external_tool_claim(&output.summary)
        || output
            .findings
            .iter()
            .any(|item| item.contains("已调用") || is_unsupported_external_tool_claim(item))
        || output
            .recommendations
            .iter()
            .any(|item| item.contains("已调用") || is_unsupported_external_tool_claim(item))
    {
        reasons.push("role output claims external tool execution".to_string());
    }

    let Some(role_policy) = policy.roles.get(role) else {
        return reasons;
    };
    if let Some(min_findings) = role_policy.min_findings
        && output.findings.len() < min_findings
        && !role_policy.reasons.min_findings.is_empty()
    {
        reasons.push(role_policy.reasons.min_findings.clone());
    }
    if let Some(min_recommendations) = role_policy.min_recommendations
        && output.recommendations.len() < min_recommendations
        && !role_policy.reasons.min_recommendations.is_empty()
    {
        reasons.push(role_policy.reasons.min_recommendations.clone());
    }
    if role_policy.require_risks
        && output.risks.is_empty()
        && !role_policy.reasons.require_risks.is_empty()
    {
        reasons.push(role_policy.reasons.require_risks.clone());
    }
    if !role_policy.specific_content_keywords.is_empty()
        && !util::contains_string_any(
            &specific_content_text(output, &role_policy.specific_content_source),
            &role_policy.specific_content_keywords,
        )
        && !role_policy.reasons.specific_content.is_empty()
    {
        reasons.push(role_policy.reasons.specific_content.clone());
    }

    reasons
}

fn specific_content_text(output: &RoleWorkOutput, source: &SpecificContentSource) -> String {
    match source {
        SpecificContentSource::Findings => {
            format!("{} {}", output.summary, output.findings.join(" "))
        }
        SpecificContentSource::Recommendations => {
            format!("{} {}", output.summary, output.recommendations.join(" "))
        }
        SpecificContentSource::All => format!(
            "{} {} {} {}",
            output.summary,
            output.findings.join(" "),
            output.recommendations.join(" "),
            output.risks.join(" ")
        ),
    }
}

fn is_action_oriented_role(role: &str, policy: &EvaluationPolicy) -> bool {
    policy
        .roles
        .get(role)
        .is_some_and(|role_policy| role_policy.action_oriented)
}

fn is_risk_sensitive_role(role: &str, policy: &EvaluationPolicy) -> bool {
    policy
        .roles
        .get(role)
        .is_some_and(|role_policy| role_policy.risk_sensitive)
}

fn extend_synthesis_items(lines: &mut Vec<String>, label: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    let visible_items = clean_visible_answer_items(items);
    if visible_items.is_empty() {
        return;
    }
    lines.push(format!("{label}: {}", visible_items.join("；")));
}

fn is_unsupported_external_tool_claim(item: &str) -> bool {
    let item = item.trim();
    if item.is_empty() {
        return false;
    }
    if contains_any(
        item,
        &[
            "已保存真实外部搜索证据",
            "外部搜索未返回来源",
            "外部搜索 adapter 未",
            "未接入真实外部",
            "未产出可信实时来源",
            "没有可采信实时来源",
            "不能伪造实时来源",
        ],
    ) {
        return false;
    }

    let lower = item.to_ascii_lowercase();
    let mentions_internal_skill = lower.contains("search_")
        || lower.contains("data_funnel_analysis")
        || lower.contains(" skill")
        || item.contains("skill ");
    if mentions_internal_skill && !contains_data_gap(item) {
        return true;
    }

    if contains_any(
        item,
        &[
            "已查询",
            "已检索",
            "已抓取",
            "已获取",
            "查询到",
            "检索到",
            "抓取到",
            "获取到",
            "通过 `search",
            "使用 search",
            "调用 search",
            "运行 search",
            "获取同类直播",
            "获取行业基准",
            "获取同类商品",
            "获取市场基准",
        ],
    ) && !contains_data_gap(item)
    {
        return true;
    }

    false
}

fn is_unsupported_benchmark_claim(item: &str) -> bool {
    let item = item.trim();
    if item.is_empty() {
        return false;
    }
    let has_source = contains_any(item, &["来源", "引用", "证据", "已保存真实外部搜索证据"]);
    if has_source {
        return false;
    }
    let has_assertive_comparison = contains_any(
        item,
        &[
            "整体偏好",
            "真正偏好",
            "相对优异",
            "符合行业",
            "符合一般直播",
            "符合多数",
            "匹配度良好",
            "优于",
            "高于",
            "低于",
            "属于",
            "略高",
            "略低",
            "接近",
            "接近或略高",
            "接近或略低",
            "偏高",
            "偏低",
            "较高",
            "较低",
            "中等水平",
            "表现良好",
            "表现较好",
            "表现优秀",
            "转化率高",
            "点击率低",
            "点击率高",
            "加购率低",
            "加购率高",
            "下单率低",
            "下单率高",
            "付款率低",
            "付款率高",
            "阻力极低",
            "存在一定阻力",
            "处于常见水平",
            "健康区间",
            "健康水平",
        ],
    );
    if has_assertive_comparison {
        return true;
    }
    let has_benchmark = contains_any(
        item,
        &[
            "行业平均",
            "行业均值",
            "行业基准",
            "行业水平",
            "行业常规",
            "同类直播",
            "同类商品",
            "同类场景",
            "一般直播",
            "多数电商直播",
            "多数直播",
            "平均水平",
            "常见范围",
            "常见水平",
            "行业区间",
            "行业范围",
            "行业",
        ],
    );
    if !has_benchmark {
        return false;
    }
    let asks_for_benchmark = contains_any(
        item,
        &[
            "需要",
            "待补",
            "缺乏",
            "缺少",
            "收集",
            "对比",
            "复核",
            "补齐",
            "接入",
            "未接入",
            "没有真实",
            "不能",
            "不得",
            "不输出",
            "不可",
            "无法客观判断",
        ],
    );
    if asks_for_benchmark && !contains_benchmark_number_claim(item) {
        return false;
    }
    if contains_any(
        item,
        &[
            "符合",
            "处于",
            "属于",
            "常见水平",
            "平均水平",
            "常规水平",
            "健康水平",
            "健康区间",
            "吸引力预期",
            "良好",
            "优秀",
            "正常",
            "合理",
        ],
    ) {
        return true;
    }
    contains_benchmark_number_claim(item)
}

fn contains_benchmark_number_claim(value: &str) -> bool {
    const TERMS: [&str; 9] = [
        "行业", "基准", "均值", "平均", "常规", "常见", "区间", "范围", "水平",
    ];
    TERMS.iter().any(|term| {
        value.find(term).is_some_and(|index| {
            let after = &value[index ..];
            let before = &value[.. index];
            segment_after_contains_percent_or_range(after)
                || segment_before_contains_percent_or_range(before)
        })
    })
}

fn segment_after_contains_percent_or_range(value: &str) -> bool {
    let segment = value
        .split(['，', '。', '；', ';', ',', '\n'])
        .next()
        .unwrap_or(value);
    contains_percent_or_range(segment)
}

fn segment_before_contains_percent_or_range(value: &str) -> bool {
    let segment = value
        .rsplit(['，', '。', '；', ';', ',', '\n'])
        .next()
        .unwrap_or(value);
    contains_percent_or_range(segment)
}

fn contains_percent_or_range(value: &str) -> bool {
    let mut previous_was_digit = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            previous_was_digit = true;
            continue;
        }
        if previous_was_digit && matches!(ch, '%' | '％' | '-' | '~' | '—' | '至') {
            return true;
        }
        if !matches!(ch, '.' | ' ' | '\t') {
            previous_was_digit = false;
        }
    }
    false
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if value.trim().is_empty() || items.iter().any(|item| item == &value) {
        return;
    }
    items.push(value);
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

fn strip_list_marker(value: &str) -> &str {
    let value = value.trim();
    let mut saw_digit = false;

    for (index, ch) in value.char_indices() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        if saw_digit && matches!(ch, '.' | '、' | ')') {
            return value[index + ch.len_utf8() ..].trim_start();
        }
        return value;
    }
    value
}

fn compact_for_brief(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }
    let mut output = normalized.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
