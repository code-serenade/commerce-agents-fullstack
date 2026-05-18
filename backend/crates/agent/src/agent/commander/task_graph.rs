use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use crate::{
    agent_session::{RoleMemorySummary, TaskGraphMemorySummary, WorkerDetail},
    core::protocol::{
        Evaluation, RoleWorkOutput, TaskGraph, TaskGraphHandoff, TaskGraphId, TaskGraphProcessGap,
        TaskGraphProcessPanel, TaskGraphProcessRole, TaskGraphProcessSummary, TaskId, TaskNode,
        TaskNodeId, WorkerAssignment, WorkerId, WorkerReport, WorkerReportStatus,
    },
    roles::{RoleCatalog, RoleRoute},
};

const DEFAULT_TASK_GRAPH_RULES_PATH: &str = "config/json/agent/task-graph-rules.json";
const TASK_GRAPH_RULES_ENV: &str = "CM_AGENT_TASK_GRAPH_RULES";
const CATEGORY_LEVEL_BOUNDARY_MARKER: &str = "category_level_boundary";
const DEFAULT_NODE_ACCEPTANCE: &str =
    "输出必须围绕本岗位职责给出可复核的业务判断、依据、风险边界和下一步动作。";

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
    if let Some(roles) = explicit_requested_team_roles(&task.to_lowercase())
        && let Some(graph) = plan_explicit_team_task_graph(task_id, task, roles)
    {
        return graph;
    }
    if route.is_some_and(is_requested_multi_role_route)
        && let Some(graph) = plan_ecommerce_team_task_graph(task_id, task, route)
    {
        return graph;
    }
    if route.is_some_and(is_request_selected_route)
        && let Some(graph) = plan_route_task_graph(task_id, task, route)
    {
        return graph;
    }
    if is_direct_answer_task(task) && !has_explicit_team_request(task) {
        return plan_direct_answer_chat_graph(task_id, task);
    }
    if let Some(graph) = plan_configured_task_graph(task_id, task) {
        return graph;
    }
    if let Some(graph) = plan_ecommerce_team_task_graph(task_id, task, route) {
        return graph;
    }
    if let Some(graph) = plan_route_task_graph(task_id, task, route) {
        return graph;
    }
    plan_fallback_chat_graph(task_id, task)
}

fn is_request_selected_route(route: &RoleRoute) -> bool {
    !route.scores.is_empty() && route.scores.iter().all(|score| score.score >= 90.0)
}

fn is_requested_multi_role_route(route: &RoleRoute) -> bool {
    if !is_request_selected_route(route) {
        return false;
    }
    std::iter::once(route.primary_runtime_role.as_str())
        .chain(route.support_runtime_roles.iter().map(String::as_str))
        .filter(|role| *role != "chat")
        .take(2)
        .count()
        > 1
}

pub fn plan_quick_chat_graph(task_id: &TaskId, task: &str) -> Option<TaskGraph> {
    if is_direct_answer_task(task) {
        return Some(plan_direct_answer_chat_graph(task_id, task));
    }

    let graph = plan_configured_task_graph(task_id, task)?;
    let [node] = graph.nodes.as_slice() else {
        return None;
    };
    (node.role == "chat" && node.input_refs.is_empty()).then_some(graph)
}

pub fn plan_forced_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    plan_fallback_chat_graph(task_id, task)
}

fn plan_route_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    let route = route?;
    let mut roles = Vec::new();
    roles.push(route.primary_runtime_role.clone());
    if is_request_selected_route(route) || should_include_route_support_roles(task, route) {
        roles.extend(route.support_runtime_roles.iter().cloned());
    }
    roles.retain(|role| role != "chat");
    roles.dedup();
    if roles.is_empty() {
        return None;
    }

    let max_attempts = 2;
    let primary = roles.first()?.clone();
    let nodes = roles
        .iter()
        .enumerate()
        .map(|(index, role)| {
            let dependencies = if index == 0 {
                Vec::new()
            } else {
                vec![primary.as_str()]
            };
            node(
                role,
                role,
                &route_node_title(role),
                route_node_objective(role, task, index == 0),
                dependencies,
                max_attempts,
            )
        })
        .collect();

    Some(TaskGraph {
        graph_id: TaskGraphId(format!("graph-{}", task_id.0)),
        root_task: task.to_string(),
        nodes,
    })
}

fn should_include_route_support_roles(task: &str, route: &RoleRoute) -> bool {
    if route.support_runtime_roles.is_empty() {
        return false;
    }
    if route.primary_runtime_role == "data" {
        return true;
    }

    let lower = task.to_lowercase();
    has_explicit_team_request(&lower)
}

fn has_explicit_team_request(task: &str) -> bool {
    contains_any(
        task,
        &[
            "agent集群",
            "agent 集群",
            "多智能体",
            "多岗位",
            "多角色",
            "协作",
            "协同",
            "一起",
            "分工",
            "虚拟员工",
        ],
    )
}

fn plan_ecommerce_team_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    if !is_ecommerce_team_task(task, route) {
        return None;
    }

    let graph_id = TaskGraphId(format!("graph-{}", task_id.0));
    let max_attempts = 2;
    let roles = ecommerce_team_roles(task, route);
    if roles.len() <= 1 {
        return None;
    }

    let mut nodes = Vec::new();
    nodes.push(node(
        "data",
        "data",
        "数据诊断",
        ecommerce_team_role_objective("data", task),
        Vec::new(),
        max_attempts,
    ));

    for role in roles
        .iter()
        .copied()
        .filter(|role| !matches!(*role, "data" | "ops"))
    {
        nodes.push(node(
            role,
            role,
            &route_node_title(role),
            ecommerce_team_role_objective(role, task),
            vec!["data"],
            max_attempts,
        ));
    }

    let ops_input_ids = nodes
        .iter()
        .filter(|node| node.role != "ops")
        .map(|node| node.id.0.clone())
        .collect::<Vec<_>>();
    let ops_inputs = ops_input_ids.iter().map(String::as_str).collect::<Vec<_>>();
    nodes.push(node(
        "ops",
        "ops",
        "运营方案",
        ecommerce_team_role_objective("ops", task),
        ops_inputs,
        max_attempts,
    ));

    Some(TaskGraph {
        graph_id,
        root_task: task.to_string(),
        nodes,
    })
}

fn plan_explicit_team_task_graph(
    task_id: &TaskId,
    task: &str,
    roles: Vec<&'static str>,
) -> Option<TaskGraph> {
    let roles = order_explicit_team_roles(roles);
    if roles.len() <= 1 {
        return None;
    }

    let graph_id = TaskGraphId(format!("graph-{}", task_id.0));
    let max_attempts = 2;
    let has_data = roles.contains(&"data");
    let has_ops = roles.contains(&"ops");
    let mut nodes = Vec::new();

    for role in roles.iter().copied().filter(|role| *role != "ops") {
        let dependencies = if has_data && role != "data" {
            vec!["data"]
        } else {
            Vec::new()
        };
        nodes.push(node(
            role,
            role,
            &route_node_title(role),
            ecommerce_team_role_objective(role, task),
            dependencies,
            max_attempts,
        ));
    }

    if has_ops {
        let ops_input_ids = nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<Vec<_>>();
        let ops_inputs = ops_input_ids.iter().map(String::as_str).collect::<Vec<_>>();
        nodes.push(node(
            "ops",
            "ops",
            "运营方案",
            ecommerce_team_role_objective("ops", task),
            ops_inputs,
            max_attempts,
        ));
    }

    Some(TaskGraph {
        graph_id,
        root_task: task.to_string(),
        nodes,
    })
}

fn order_explicit_team_roles(roles: Vec<&'static str>) -> Vec<&'static str> {
    let unique = roles.into_iter().fold(Vec::new(), |mut unique, role| {
        if !unique.contains(&role) {
            unique.push(role);
        }
        unique
    });
    let mut ordered = Vec::new();
    if unique.contains(&"data") {
        ordered.push("data");
    }
    for role in [
        "creative",
        "design",
        "service",
        "engineering",
        "douyin_ads",
        "accounting",
        "web",
    ] {
        if unique.contains(&role) {
            ordered.push(role);
        }
    }
    if unique.contains(&"ops") {
        ordered.push("ops");
    }
    for role in unique {
        if !ordered.contains(&role) {
            ordered.push(role);
        }
    }
    ordered
}

fn is_ecommerce_team_task(task: &str, route: Option<&RoleRoute>) -> bool {
    let requested_multi_role = route.is_some_and(is_requested_multi_role_route);
    if route.is_some_and(is_request_selected_route) && !requested_multi_role {
        return false;
    }
    let explicit_team_request = has_explicit_team_request(task);
    let category_analysis_request = is_category_analysis_team_request(task);
    if is_direct_answer_task(task) && !explicit_team_request && !requested_multi_role {
        return false;
    }
    let lower = task.to_lowercase();
    let product_or_platform_signal = contains_any(
        &lower,
        &[
            "商品",
            "产品",
            "类目",
            "品类",
            "电商",
            "咖啡",
            "茶饮",
            "食品",
            "服饰",
            "美妆",
            "家居",
            "店铺",
            "货盘",
            "选品",
            "竞品",
            "直播间",
            "抖音",
            "千川",
        ],
    );
    let task_signal = contains_any(
        &lower,
        &[
            "分析",
            "诊断",
            "方案",
            "策略",
            "规划",
            "复盘",
            "判断",
            "健康",
            "提升",
            "增长",
            "转化",
            "营销",
            "运营",
            "启动",
            "怎么做",
        ],
    );
    if !(product_or_platform_signal && task_signal) {
        return false;
    }

    if route.is_some()
        && !explicit_team_request
        && !requested_multi_role
        && !category_analysis_request
        && !route.is_some_and(allows_auto_ecommerce_team_route)
    {
        return false;
    }

    if let Some(route) = route
        && matches!(
            route.primary_runtime_role.as_str(),
            "service" | "engineering" | "web"
        )
        && !has_explicit_team_request(&lower)
        && !contains_any(
            &lower,
            &[
                "商品",
                "产品",
                "类目",
                "品类",
                "电商",
                "咖啡",
                "直播间",
                "抖音",
                "千川",
            ],
        )
    {
        return false;
    }

    true
}

fn allows_auto_ecommerce_team_route(route: &RoleRoute) -> bool {
    matches!(
        route.primary_runtime_role.as_str(),
        "chat" | "ops" | "data" | "creative" | "douyin_ads" | "accounting" | "design"
    )
}

fn is_category_analysis_team_request(task: &str) -> bool {
    let lower = task.to_lowercase();
    contains_any(
        &lower,
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
        ],
    ) && contains_any(&lower, &["分析", "诊断", "策略", "方案", "规划", "复盘"])
}

fn ecommerce_team_roles(task: &str, route: Option<&RoleRoute>) -> Vec<&'static str> {
    let lower = task.to_lowercase();
    if let Some(roles) = explicit_requested_team_roles(&lower)
        && roles.len() > 1
    {
        return roles;
    }

    if let Some(route) = route
        && is_requested_multi_role_route(route)
    {
        let mut roles = Vec::new();
        push_known_ecommerce_team_role(&mut roles, "data");
        for role in std::iter::once(route.primary_runtime_role.as_str())
            .chain(route.support_runtime_roles.iter().map(String::as_str))
        {
            push_known_ecommerce_team_role(&mut roles, role);
        }
        push_known_ecommerce_team_role(&mut roles, "ops");
        return roles;
    }

    let mut roles = vec!["data", "creative", "service", "engineering"];
    if contains_any(
        &lower,
        &["抖音", "直播", "直播间", "短视频", "千川", "投流", "投放"],
    ) {
        roles.push("douyin_ads");
        roles.push("accounting");
    }
    if contains_any(&lower, &["预算", "成本", "利润", "roi", "毛利", "财务"]) {
        roles.push("accounting");
    }
    if contains_any(&lower, &["详情页", "主图", "视觉", "设计", "页面", "素材"]) {
        roles.push("design");
    }
    if contains_any(&lower, &["seo", "关键词", "搜索", "自然流量", "收录"]) {
        roles.push("web");
    }
    if let Some(route) = route {
        for role in std::iter::once(route.primary_runtime_role.as_str())
            .chain(route.support_runtime_roles.iter().map(String::as_str))
        {
            match role {
                "data" => roles.push("data"),
                "creative" => roles.push("creative"),
                "service" => roles.push("service"),
                "engineering" => roles.push("engineering"),
                "douyin_ads" => roles.push("douyin_ads"),
                "accounting" => roles.push("accounting"),
                "design" => roles.push("design"),
                "web" => roles.push("web"),
                _ => {}
            }
        }
    }
    roles.push("ops");
    roles.into_iter().fold(Vec::new(), |mut unique, role| {
        if !unique.contains(&role) {
            unique.push(role);
        }
        unique
    })
}

fn explicit_requested_team_roles(task: &str) -> Option<Vec<&'static str>> {
    if !contains_any(
        task,
        &[
            "只让",
            "只用",
            "仅让",
            "仅用",
            "限定",
            "指定",
            "三个岗位",
            "3个岗位",
            "三岗",
        ],
    ) {
        return None;
    }

    let mut roles = Vec::new();
    for (role, aliases) in [
        ("ops", &["运营", "运营策略", "运营策略师", "主岗"][..]),
        ("data", &["数据", "数据分析", "数据分析师"][..]),
        ("creative", &["内容", "创意", "内容创意", "内容创意师"][..]),
        ("service", &["客服", "客户成功", "服务", "客户成功专员"][..]),
        (
            "engineering",
            &["技术", "工程", "技术架构", "技术架构师"][..],
        ),
        ("douyin_ads", &["千川", "投放", "广告", "千川投放"][..]),
        ("accounting", &["财务", "预算", "成本", "财务分析"][..]),
        ("design", &["设计", "视觉", "设计师"][..]),
        ("web", &["web", "seo", "搜索"][..]),
    ] {
        if aliases.iter().any(|alias| task.contains(alias)) {
            push_known_ecommerce_team_role(&mut roles, role);
        }
    }
    if roles.is_empty() {
        return None;
    }
    if !roles.contains(&"ops") && contains_any(task, &["合成", "整合", "方案", "运营"]) {
        push_known_ecommerce_team_role(&mut roles, "ops");
    }
    Some(roles)
}

fn push_known_ecommerce_team_role(roles: &mut Vec<&'static str>, role: &str) {
    let normalized = match role {
        "data" => "data",
        "creative" => "creative",
        "service" => "service",
        "engineering" => "engineering",
        "douyin_ads" => "douyin_ads",
        "accounting" => "accounting",
        "design" => "design",
        "web" => "web",
        "ops" => "ops",
        _ => return,
    };
    if !roles.contains(&normalized) {
        roles.push(normalized);
    }
}

fn ecommerce_team_role_objective(role: &str, task: &str) -> String {
    match role {
        "data" => format!(
            "作为数据分析师，先定义类目、商品或直播转化问题的指标口径、样本周期、\
             缺失数据和不能编造的边界：{task}"
        ),
        "creative" => format!(
            "作为内容创意师，基于数据口径把业务判断转成标题、脚本、卖点、\
             素材测试方向和可复盘假设：{task}"
        ),
        "service" => format!(
            "作为客户成功专员，补充客服咨询、售后承诺、退款/履约边界和异常升级 \
             SOP，说明这些如何影响转化：{task}"
        ),
        "engineering" => format!(
            "作为技术架构师，确认埋点、接口、报表刷新、系统稳定性和回滚边界，让后续复盘可验证：\
             {task}"
        ),
        "douyin_ads" => format!(
            "作为千川投放专员，拆解人群包、素材组、出价、预算、ROI/CPA \
             止损线和直播/短视频承接：{task}"
        ),
        "accounting" => format!("作为财务分析师，评估预算、毛利、CPA/ROI 止损线和放量风险：{task}"),
        "design" => format!(
            "作为设计师，把卖点、信任证明、页面层级、主图和行动按钮承接成可测试方案：{task}"
        ),
        "web" => format!("作为 Web/SEO 专家，补充关键词、页面结构、FAQ 和自然流量承接方案：{task}"),
        "ops" => format!(
            "作为主回答岗位运营策略师，整合所有协作岗位的输入，形成最终团队结论、执行节奏、\
             风险边界和 final_report 交付物：{task}"
        ),
        _ => format!("围绕本岗位边界完成判断：{task}"),
    }
}

fn route_node_title(role: &str) -> String {
    match role {
        "ops" => "运营方案",
        "data" => "数据诊断",
        "service" => "服务方案",
        "design" => "体验设计",
        "accounting" => "预算评估",
        "engineering" => "工程判断",
        "web" => "Web/SEO方案",
        "creative" => "创意产出",
        "douyin_ads" => "千川投放优化",
        _ => "角色判断",
    }
    .to_string()
}

fn route_node_objective(role: &str, task: &str, primary: bool) -> String {
    if primary {
        return format!("作为主角色 {role}，先完成本岗位对用户问题的核心判断：{task}");
    }
    format!("作为支持角色 {role}，基于主角色输入补充本岗位判断，并指出证据、风险和下一步：{task}")
}

fn plan_fallback_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    TaskGraph {
        graph_id: TaskGraphId(format!("graph-{}", task_id.0)),
        root_task: task.to_string(),
        nodes: vec![node(
            "chat",
            "chat",
            "通用对话",
            format!("以对话助手身份回答用户问题；如果用户意图不清，先澄清，不启动多智能体：{task}"),
            Vec::new(),
            1,
        )],
    }
}

fn plan_direct_answer_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    TaskGraph {
        graph_id: TaskGraphId(format!("graph-{}", task_id.0)),
        root_task: task.to_string(),
        nodes: vec![node(
            "chat",
            "chat",
            "直接回答",
            format!(
                "直接回答用户原始问题，必须遵守用户对简短、只输出、不要解释的要求，\
                 不启动多智能体：{task}"
            ),
            Vec::new(),
            1,
        )],
    }
}

#[allow(dead_code)]
fn plan_builtin_task_graph(task_id: &TaskId, task: &str, route: Option<&RoleRoute>) -> TaskGraph {
    let lower = task.to_lowercase();
    let graph_id = TaskGraphId(format!("graph-{}", task_id.0));
    let max_attempts = 2;
    let mut nodes = Vec::new();
    let marketing_campaign = contains_any(
        &lower,
        &[
            "抖音",
            "直播",
            "短视频",
            "五一",
            "节日",
            "营销",
            "活动",
            "促销",
            "投流",
            "千川",
            "引流",
        ],
    );

    if contains_any(
        &lower,
        &["客服", "售后", "投诉", "退款", "满意度", "nps", "服务"],
    ) {
        nodes.push(node(
            "service",
            "service",
            "服务方案",
            format!("给出客服、售后、投诉、退款或满意度处理方案：{task}"),
            Vec::new(),
            max_attempts,
        ));
    } else if contains_any(
        &lower,
        &[
            "seo",
            "关键词",
            "搜索",
            "自然流量",
            "收录",
            "排名",
            "标题优化",
            "独立站",
        ],
    ) {
        nodes.push(node(
            "web",
            "web",
            "Web/SEO方案",
            format!("给出 SEO、关键词、收录、标题或自然流量增长方案：{task}"),
            Vec::new(),
            max_attempts,
        ));
    } else if contains_any(
        &lower,
        &[
            "代码",
            "rust",
            "python",
            "架构",
            "接口",
            "性能",
            "sla",
            "故障",
            "系统",
            "发布",
            "回滚",
            "稳定性",
            "bug",
        ],
    ) {
        nodes.push(node(
            "engineering",
            "engineering",
            "工程判断",
            format!("给出工程分析和实现建议：{task}"),
            Vec::new(),
            max_attempts,
        ));
    } else if contains_any(
        &lower,
        &[
            "详情页",
            "设计",
            "预算",
            "风险",
            "归因",
            "漏斗",
            "数据",
            "转化率",
            "指标",
        ],
    ) {
        nodes.push(node(
            "data",
            "data",
            "数据诊断",
            format!("分析任务中的数据、指标、归因和约束：{task}"),
            Vec::new(),
            max_attempts,
        ));
        if contains_any(&lower, &["详情页", "设计", "页面"]) {
            nodes.push(node(
                "design",
                "design",
                "体验设计",
                "基于上游数据诊断，提出详情页或体验优化方案。".to_string(),
                vec!["data"],
                max_attempts,
            ));
        }
        if contains_any(&lower, &["预算", "成本", "财务", "风险"]) {
            nodes.push(node(
                "accounting",
                "accounting",
                "预算评估",
                "基于上游数据诊断，评估预算、成本和经营风险。".to_string(),
                vec!["data"],
                max_attempts,
            ));
        }
        if marketing_campaign && contains_any(&lower, &["转化率", "方案", "运营", "营销", "活动"])
        {
            if !nodes.iter().any(|node| node.role == "accounting") {
                nodes.push(node(
                    "accounting",
                    "accounting",
                    "预算评估",
                    "基于上游数据诊断，评估活动成本、引流SKU、投流预算和经营风险。".to_string(),
                    vec!["data"],
                    max_attempts,
                ));
            }
            nodes.push(node(
                "creative",
                "creative",
                "创意产出",
                "基于上游数据诊断，产出抖音短视频、直播或活动内容创意。".to_string(),
                vec!["data"],
                max_attempts,
            ));
        }
        if contains_any(&lower, &["运营", "调整", "方案", "策略"]) {
            let mut ops_inputs = Vec::new();
            if nodes.iter().any(|node| node.role == "design") {
                ops_inputs.push("design");
            }
            if nodes.iter().any(|node| node.role == "accounting") {
                ops_inputs.push("accounting");
            }
            if nodes.iter().any(|node| node.role == "creative") {
                ops_inputs.push("creative");
            }
            if ops_inputs.is_empty() {
                ops_inputs.push("data");
            }
            nodes.push(node(
                "ops",
                "ops",
                "运营方案",
                "基于上游结果，形成可执行运营调整方案。".to_string(),
                ops_inputs,
                max_attempts,
            ));
        }
        if contains_any(&lower, &["文案", "脚本", "标题", "内容"])
            && !nodes.iter().any(|node| node.role == "creative")
        {
            nodes.push(node(
                "creative",
                "creative",
                "创意产出",
                "基于上游数据诊断，产出内容创意或文案。".to_string(),
                vec!["data"],
                max_attempts,
            ));
        }
    } else if contains_any(&lower, &["文案", "脚本", "标题", "内容"]) {
        nodes.push(node(
            "ops",
            "ops",
            "运营定位",
            format!("梳理内容任务目标、受众和运营约束：{task}"),
            Vec::new(),
            max_attempts,
        ));
        nodes.push(node(
            "creative",
            "creative",
            "创意产出",
            "基于运营定位，产出内容创意或文案。".to_string(),
            vec!["ops"],
            max_attempts,
        ));
    } else {
        let role = route
            .map(|route| route.primary_runtime_role.as_str())
            .unwrap_or("chat");
        nodes.push(node(
            role,
            role,
            "直接回答",
            task.to_string(),
            Vec::new(),
            max_attempts,
        ));
    }

    TaskGraph {
        graph_id,
        root_task: task.to_string(),
        nodes,
    }
}

fn node(
    id: &str,
    role: &str,
    title: &str,
    objective: String,
    input_refs: Vec<&str>,
    max_attempts: u8,
) -> TaskNode {
    TaskNode {
        id: TaskNodeId(id.to_string()),
        role: role.to_string(),
        worker_id: WorkerId(format!("worker.{role}")),
        title: title.to_string(),
        objective,
        input_refs: input_refs
            .into_iter()
            .map(|input_ref| TaskNodeId(input_ref.to_string()))
            .collect(),
        acceptance: vec![DEFAULT_NODE_ACCEPTANCE.to_string()],
        max_attempts,
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TaskGraphRuleSet {
    #[serde(default)]
    rules: Vec<TaskGraphRule>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TaskGraphRule {
    id: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    match_any: Vec<String>,
    #[serde(default)]
    match_all: Vec<String>,
    #[serde(default)]
    match_all_any: Vec<Vec<String>>,
    #[serde(default)]
    nodes: Vec<TaskGraphRuleNode>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TaskGraphRuleNode {
    id: String,
    role: String,
    title: String,
    objective: String,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default = "default_rule_node_max_attempts")]
    max_attempts: u8,
    #[serde(default)]
    acceptance: Vec<String>,
}

fn default_rule_node_max_attempts() -> u8 {
    2
}

fn plan_configured_task_graph(task_id: &TaskId, task: &str) -> Option<TaskGraph> {
    let path = env::var(TASK_GRAPH_RULES_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_TASK_GRAPH_RULES_PATH.to_string());
    let path = resolve_task_graph_rules_path(&path)?;

    let rules = match load_task_graph_rules(&path) {
        Ok(rules) => rules,
        Err(error) => {
            crate::log_error_msg!(&format!(
                "task graph rules ignored: path={}, error={error}",
                path.display()
            ));
            return None;
        }
    };
    let graph = plan_task_graph_from_rules(task_id, task, &rules)?;
    if is_quick_chat_graph(&graph) && !is_quick_chat_task(task) {
        return None;
    }
    Some(graph)
}

fn resolve_task_graph_rules_path(path: &str) -> Option<PathBuf> {
    let candidate = Path::new(path);
    if candidate.exists() {
        return Some(candidate.to_path_buf());
    }
    if path != DEFAULT_TASK_GRAPH_RULES_PATH {
        return None;
    }

    let repo_config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(DEFAULT_TASK_GRAPH_RULES_PATH);
    repo_config_path.exists().then_some(repo_config_path)
}

fn load_task_graph_rules(path: impl AsRef<Path>) -> Result<TaskGraphRuleSet, String> {
    let content = fs::read_to_string(path.as_ref()).map_err(|error| error.to_string())?;
    parse_task_graph_rules(&content)
}

fn parse_task_graph_rules(content: &str) -> Result<TaskGraphRuleSet, String> {
    let rules = serde_json::from_str::<TaskGraphRuleSet>(content)
        .map_err(|error| format!("invalid task graph rules json: {error}"))?;
    validate_task_graph_rules(&rules)?;
    Ok(rules)
}

fn plan_task_graph_from_rules(
    task_id: &TaskId,
    task: &str,
    rules: &TaskGraphRuleSet,
) -> Option<TaskGraph> {
    let rule = rules.rules.iter().find(|rule| rule.matches(task))?;
    Some(build_configured_task_graph(task_id, task, rule))
}

fn is_quick_chat_graph(graph: &TaskGraph) -> bool {
    let [node] = graph.nodes.as_slice() else {
        return false;
    };
    node.role == "chat" && node.input_refs.is_empty()
}

fn is_quick_chat_task(task: &str) -> bool {
    let task = task.trim().to_lowercase();
    if task.is_empty() {
        return false;
    }
    if is_direct_answer_task(&task) {
        return true;
    }
    let exact = [
        "hi",
        "hello",
        "hey",
        "你好",
        "您好",
        "嗨",
        "哈喽",
        "在吗",
        "早上好",
        "中午好",
        "下午好",
        "晚上好",
    ];
    if exact.iter().any(|value| task == *value) {
        return true;
    }
    task.chars().count() <= 8 && exact.iter().any(|value| task.contains(value))
}

fn is_direct_answer_task(task: &str) -> bool {
    let trimmed = task.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let explicit_direct = [
        "只输出",
        "只给",
        "只回答",
        "只回复",
        "不要解释",
        "不用解释",
        "无需解释",
        "直接回答",
        "直接给",
        "简短回答",
        "简要回答",
        "一句话",
        "only output",
        "answer only",
        "no explanation",
    ]
    .iter()
    .any(|term| trimmed.contains(term) || lower.contains(term));
    let has_digit = trimmed.chars().any(|ch| ch.is_ascii_digit());
    let char_count = trimmed.chars().count();
    let arithmetic_request = has_digit
        && [
            "+", "-", "*", "/", "×", "÷", "加", "减", "乘", "除", "计算", "等于", "结果",
        ]
        .iter()
        .any(|term| trimmed.contains(term))
        && char_count <= 40
        && !contains_any(
            trimmed,
            &[
                "分析", "方案", "策略", "运营", "电商", "漏斗", "转化", "点击", "曝光", "加购",
                "下单", "付款",
            ],
        );
    let short_question = ["是多少", "多少", "几个", "几"]
        .iter()
        .any(|term| trimmed.contains(term))
        && char_count <= 40;

    explicit_direct || arithmetic_request || short_question
}

impl TaskGraphRule {
    fn matches(&self, task: &str) -> bool {
        let lower = task.to_lowercase();
        let contains = |token: &str| lower.contains(&token.to_lowercase());
        if !self.match_all.iter().all(|token| contains(token)) {
            return false;
        }
        if !self.match_any.is_empty() && !self.match_any.iter().any(|token| contains(token)) {
            return false;
        }
        self.match_all_any
            .iter()
            .all(|group| group.iter().any(|token| contains(token)))
    }
}

fn build_configured_task_graph(task_id: &TaskId, task: &str, rule: &TaskGraphRule) -> TaskGraph {
    let graph_id = TaskGraphId(format!("graph-{}", task_id.0));
    let nodes = rule
        .nodes
        .iter()
        .map(|node| TaskNode {
            id: TaskNodeId(node.id.clone()),
            role: node.role.clone(),
            worker_id: WorkerId(format!("worker.{}", node.role)),
            title: node.title.clone(),
            objective: render_rule_template(&node.objective, task, rule),
            input_refs: node
                .depends_on
                .iter()
                .map(|input_ref| TaskNodeId(input_ref.clone()))
                .collect(),
            acceptance: if node.acceptance.is_empty() {
                vec![DEFAULT_NODE_ACCEPTANCE.to_string()]
            } else {
                node.acceptance.clone()
            },
            max_attempts: node.max_attempts.max(1),
        })
        .collect();

    TaskGraph {
        graph_id,
        root_task: task.to_string(),
        nodes,
    }
}

fn render_rule_template(template: &str, task: &str, rule: &TaskGraphRule) -> String {
    template
        .replace("{{task}}", task)
        .replace("{{rule_id}}", &rule.id)
        .replace("{{rule_description}}", &rule.description)
}

fn validate_task_graph_rules(rules: &TaskGraphRuleSet) -> Result<(), String> {
    validate_task_graph_rules_with_catalog(rules, &RoleCatalog::configured())
}

fn validate_task_graph_rules_with_catalog(
    rules: &TaskGraphRuleSet,
    catalog: &RoleCatalog,
) -> Result<(), String> {
    let valid_roles = catalog
        .roles()
        .iter()
        .map(|role| role.runtime_role.clone())
        .collect::<HashSet<_>>();
    let mut rule_ids = HashSet::new();

    for rule in &rules.rules {
        validate_rule(rule, &valid_roles)?;
        if !rule_ids.insert(rule.id.clone()) {
            return Err(format!("duplicate rule id '{}'", rule.id));
        }
    }

    Ok(())
}

fn validate_rule(rule: &TaskGraphRule, valid_roles: &HashSet<String>) -> Result<(), String> {
    if rule.id.trim().is_empty() {
        return Err("rule id is required".to_string());
    }
    if rule.nodes.is_empty() {
        return Err(format!("rule '{}' must declare at least one node", rule.id));
    }

    let mut node_ids = HashSet::new();
    for node in &rule.nodes {
        if node.id.trim().is_empty() {
            return Err(format!("rule '{}' has node without id", rule.id));
        }
        if !node_ids.insert(node.id.clone()) {
            return Err(format!(
                "rule '{}' has duplicate node '{}'",
                rule.id, node.id
            ));
        }
        if !valid_roles.contains(&node.role) {
            return Err(format!(
                "rule '{}' node '{}' uses unknown role '{}'",
                rule.id, node.id, node.role
            ));
        }
        if node.title.trim().is_empty() {
            return Err(format!(
                "rule '{}' node '{}' must declare title",
                rule.id, node.id
            ));
        }
        if node.objective.trim().is_empty() {
            return Err(format!(
                "rule '{}' node '{}' must declare objective",
                rule.id, node.id
            ));
        }
    }

    for node in &rule.nodes {
        for dep in &node.depends_on {
            if !node_ids.contains(dep) {
                return Err(format!(
                    "rule '{}' node '{}' depends on unknown node '{}'",
                    rule.id, node.id, dep
                ));
            }
            if dep == &node.id {
                return Err(format!(
                    "rule '{}' node '{}' cannot depend on itself",
                    rule.id, node.id
                ));
            }
        }
    }

    if !rule.nodes.iter().any(|node| node.depends_on.is_empty()) {
        return Err(format!(
            "rule '{}' must have at least one root node",
            rule.id
        ));
    }
    validate_rule_has_no_cycles(rule)
}

fn validate_rule_has_no_cycles(rule: &TaskGraphRule) -> Result<(), String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Done,
    }

    fn visit(
        node_id: &str,
        deps: &HashMap<&str, Vec<&str>>,
        states: &mut HashMap<String, VisitState>,
        rule_id: &str,
    ) -> Result<(), String> {
        match states.get(node_id).copied() {
            Some(VisitState::Visiting) => {
                return Err(format!(
                    "rule '{rule_id}' contains dependency cycle at node '{node_id}'"
                ));
            }
            Some(VisitState::Done) => return Ok(()),
            None => {}
        }

        states.insert(node_id.to_string(), VisitState::Visiting);
        for dep in deps.get(node_id).into_iter().flatten() {
            visit(dep, deps, states, rule_id)?;
        }
        states.insert(node_id.to_string(), VisitState::Done);
        Ok(())
    }

    let deps = rule
        .nodes
        .iter()
        .map(|node| {
            (
                node.id.as_str(),
                node.depends_on
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut states = HashMap::new();
    for node in &rule.nodes {
        visit(&node.id, &deps, &mut states, &rule.id)?;
    }
    Ok(())
}

fn evaluate_report(
    report: &WorkerReport,
    node: Option<&TaskNode>,
    upstream_reports: &HashMap<TaskNodeId, WorkerReport>,
    root_task: &str,
) -> Evaluation {
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
            reasons.extend(evaluate_role_output(&report.role, role_output));
            reasons.extend(evaluate_quality_contract(report, role_output));
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
    if contains_any(
        &lower,
        &[
            "todo",
            "tbd",
            "placeholder",
            "无法处理",
            "不知道",
            "作为ai",
            "as an ai",
        ],
    ) {
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

    let reasons = downgrade_role_contract_gaps(report, node, root_task, reasons);
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
        is_role_contract_or_boundary_gap(reason, &report.role)
            || reason.starts_with("upstream evidence:")
            || is_downgradable_quality_gap(reason, &report.role, Some(node))
    })
}

fn downgrade_role_contract_gaps(
    report: &WorkerReport,
    node: Option<&TaskNode>,
    root_task: &str,
    reasons: Vec<String>,
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
            .filter(|reason| !is_role_contract_or_boundary_gap(reason, report.role.as_str()))
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
            !(is_downgradable_quality_gap(reason, &report.role, node)
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
    reason == "worker reported failed status"
        || reason == "open questions remain"
        || reason == "role output has open questions"
        || reason == "content looks like placeholder or failure text"
        || reason.starts_with("upstream evidence:")
        || is_downgradable_quality_gap(reason, role, None)
}

fn is_role_contract_or_boundary_gap(reason: &str, role: &str) -> bool {
    is_category_repair_gap(reason, role)
        || reason == "structured role output is missing"
        || reason == "content is empty"
        || reason == "quality refusal detection: refusal without explicit open question"
        || reason == "role output claims external tool execution"
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

fn is_downgradable_quality_gap(reason: &str, role: &str, node: Option<&TaskNode>) -> bool {
    if reason.starts_with("quality completeness:")
        || reason.starts_with("quality actionability:")
        || reason.starts_with("quality risk awareness:")
    {
        return true;
    }

    match role {
        "data" => matches!(
            reason,
            "data role should provide at least two findings"
                | "data role should mark data or interpretation risks"
        ),
        "ops" => reason == "ops role should provide at least three recommendations",
        "service" => {
            reason == "service role output lacks service-specific content"
                || reason == "service role should provide scripts, SOP, or escalation steps"
        }
        "engineering" => {
            reason == "engineering role output lacks technical content"
                || reason
                    == "engineering role should provide technical risks or rollback boundaries"
        }
        "design" => {
            reason == "design role output lacks design-specific content"
                || reason == "design role should provide visual or page recommendations"
        }
        "creative" => {
            reason == "creative role output lacks creative-specific content"
                || reason == "creative role should provide usable creative directions"
        }
        "accounting" => {
            reason == "accounting role output lacks financial content"
                || reason == "accounting role should provide budget or financial risks"
        }
        "web" => {
            reason == "web role output lacks web or SEO-specific content"
                || reason == "web role should provide SEO, keyword, or conversion recommendations"
        }
        _ => {
            node.is_some_and(|node| !node.input_refs.is_empty())
                && reason.starts_with("upstream evidence:")
        }
    }
}

fn evaluate_quality_contract(report: &WorkerReport, output: &RoleWorkOutput) -> Vec<String> {
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

    if output.summary.chars().count() < 8 && report.role != "chat" {
        reasons.push("quality completeness: summary is too thin".to_string());
    }
    if report.role != "chat" && output.findings.is_empty() && output.recommendations.is_empty() {
        reasons.push("quality completeness: no findings or recommendations".to_string());
    }
    if contains_any(
        &lower,
        &[
            "已查询",
            "已检索",
            "查询到",
            "搜索结果显示",
            "数据库显示",
            "调用工具",
            "tool result",
        ],
    ) {
        reasons.push("quality fabrication: claims unsupported external evidence".to_string());
    }
    if is_action_oriented_role(&report.role) && output.recommendations.is_empty() {
        reasons.push("quality actionability: missing executable recommendations".to_string());
    }
    if is_risk_sensitive_role(&report.role) && output.risks.is_empty() {
        reasons.push("quality risk awareness: missing risk boundary".to_string());
    }
    if contains_any(
        &lower,
        &[
            "不能回答",
            "无法回答",
            "无法完成任务",
            "无法继续",
            "无法推进",
            "我不能",
            "i cannot",
            "can't help",
        ],
    ) && output.open_questions.is_empty()
    {
        reasons
            .push("quality refusal detection: refusal without explicit open question".to_string());
    }

    reasons
}

fn open_questions_are_blocking(output: Option<&RoleWorkOutput>, open_questions: &[String]) -> bool {
    if open_questions.is_empty() {
        return false;
    }
    if output.is_some_and(has_actionable_boundary_output) {
        return false;
    }
    if open_questions.iter().any(|item| {
        contains_any(
            item,
            &[
                "无法继续",
                "无法判断",
                "无法完成任务",
                "阻止",
                "必须提供",
                "必须补充",
            ],
        )
    }) {
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
    match reason {
        "data role should provide at least two findings" => {
            "数据岗位需要补齐至少两个可复核发现。".to_string()
        }
        "data role should mark data or interpretation risks" => {
            "数据岗位需要说明数据口径或解释风险。".to_string()
        }
        "service role output lacks service-specific content" => {
            "服务岗位需要补充客服承接、售后边界或升级 SOP。".to_string()
        }
        "service role should provide scripts, SOP, or escalation steps" => {
            "服务岗位需要给出话术、SOP 或升级步骤。".to_string()
        }
        "engineering role output lacks technical content" => {
            "工程岗位需要补充埋点、接口、稳定性或回滚边界。".to_string()
        }
        "engineering role should provide technical risks or rollback boundaries" => {
            "工程岗位需要说明技术风险或回滚边界。".to_string()
        }
        "structured role output is missing" => "岗位没有返回结构化产物。".to_string(),
        "content is empty" => "岗位产物为空。".to_string(),
        "open questions remain" => "仍有阻塞性待确认问题。".to_string(),
        "worker reported failed status" => "worker 报告执行失败。".to_string(),
        "selected product context is not allowed for this task" => {
            "本轮明确要求不引用预设商品，岗位不能把历史商品或默认商品当成本轮商品。".to_string()
        }
        "content looks like placeholder or failure text" => {
            "岗位产物像占位或失败文本。".to_string()
        }
        _ if reason.starts_with("quality completeness: summary is too thin") => {
            "岗位摘要过薄，需要补充具体判断。".to_string()
        }
        _ if reason.starts_with("quality completeness: no findings or recommendations") => {
            "岗位缺少发现或行动建议。".to_string()
        }
        _ if reason.starts_with("quality actionability:") => "岗位缺少可执行建议。".to_string(),
        _ if reason.starts_with("quality risk awareness:") => "岗位缺少风险边界。".to_string(),
        _ if reason.starts_with("upstream evidence: missing citation for input") => {
            "协作岗位没有引用上游岗位产物，需要补齐交接证据。".to_string()
        }
        _ if reason.starts_with("upstream evidence: dependent role must cite upstream inputs") => {
            "协作岗位需要引用上游输入。".to_string()
        }
        _ => reason.to_string(),
    }
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

fn evaluate_role_output(role: &str, output: &RoleWorkOutput) -> Vec<String> {
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

    match role {
        "data" => {
            if output.findings.len() < 2 {
                reasons.push("data role should provide at least two findings".to_string());
            }
            if output.risks.is_empty() {
                reasons.push("data role should mark data or interpretation risks".to_string());
            }
        }
        "ops" if output.recommendations.len() < 3 => {
            reasons.push("ops role should provide at least three recommendations".to_string());
        }
        "design" => {
            if output.recommendations.is_empty() {
                reasons
                    .push("design role should provide visual or page recommendations".to_string());
            }
            if !contains_any(
                &format!("{} {}", output.summary, output.recommendations.join(" ")),
                &["视觉", "页面", "素材", "版式", "主图", "详情页", "设计"],
            ) {
                reasons.push("design role output lacks design-specific content".to_string());
            }
        }
        "accounting" => {
            if output.risks.is_empty() {
                reasons
                    .push("accounting role should provide budget or financial risks".to_string());
            }
            if !contains_any(
                &format!("{} {}", output.summary, output.findings.join(" ")),
                &["预算", "成本", "利润", "roi", "ROI", "现金流", "财务"],
            ) {
                reasons.push("accounting role output lacks financial content".to_string());
            }
        }
        "creative" => {
            if output.recommendations.is_empty() {
                reasons.push("creative role should provide usable creative directions".to_string());
            }
            if !contains_any(
                &format!("{} {}", output.summary, output.recommendations.join(" ")),
                &["文案", "标题", "脚本", "创意", "内容", "短视频"],
            ) {
                reasons.push("creative role output lacks creative-specific content".to_string());
            }
        }
        "service" => {
            if output.recommendations.is_empty() {
                reasons.push(
                    "service role should provide scripts, SOP, or escalation steps".to_string(),
                );
            }
            if !contains_any(
                &format!("{} {}", output.summary, output.recommendations.join(" ")),
                &[
                    "客服",
                    "售后",
                    "投诉",
                    "退款",
                    "sop",
                    "SOP",
                    "升级",
                    "满意度",
                ],
            ) {
                reasons.push("service role output lacks service-specific content".to_string());
            }
        }
        "engineering" => {
            if output.risks.is_empty() {
                reasons.push(
                    "engineering role should provide technical risks or rollback boundaries"
                        .to_string(),
                );
            }
            if !contains_any(
                &format!("{} {}", output.summary, output.findings.join(" ")),
                &[
                    "技术",
                    "架构",
                    "接口",
                    "性能",
                    "发布",
                    "回滚",
                    "稳定性",
                    "系统",
                ],
            ) {
                reasons.push("engineering role output lacks technical content".to_string());
            }
        }
        "web" => {
            if output.recommendations.is_empty() {
                reasons.push(
                    "web role should provide SEO, keyword, or conversion recommendations"
                        .to_string(),
                );
            }
            if !contains_any(
                &format!("{} {}", output.summary, output.recommendations.join(" ")),
                &["seo", "SEO", "关键词", "收录", "标题", "自然流量", "页面"],
            ) {
                reasons.push("web role output lacks web or SEO-specific content".to_string());
            }
        }
        _ => {}
    }

    reasons
}

fn is_action_oriented_role(role: &str) -> bool {
    matches!(
        role,
        "ops" | "design" | "creative" | "service" | "engineering" | "web"
    )
}

fn is_risk_sensitive_role(role: &str) -> bool {
    matches!(
        role,
        "ops" | "data" | "accounting" | "engineering" | "service" | "web"
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_creates_single_chat_node_for_plain_question() {
        let graph = plan_task_graph(
            &TaskId("task-1".to_string()),
            "解释一下 agent session 是什么",
            None,
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "chat");
        assert!(graph.nodes[0].input_refs.is_empty());
    }

    #[test]
    fn planner_adds_finance_and_creative_for_douyin_holiday_conversion_plan() {
        let rules_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/json/agent/task-graph-rules.json");
        let rules = load_task_graph_rules(rules_path).expect("configured graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-coffee".to_string()),
            "我是做咖啡运营的，主要营销平台是抖音，现在马上就五一了，\
             可以帮我想一个运营方案吗提升我的转化率",
            &rules,
        )
        .expect("douyin conversion task should match configured graph");

        let data = graph.nodes.iter().find(|node| node.role == "data").unwrap();
        let accounting = graph
            .nodes
            .iter()
            .find(|node| node.role == "accounting")
            .unwrap();
        let creative = graph
            .nodes
            .iter()
            .find(|node| node.role == "creative")
            .unwrap();
        let ops = graph.nodes.iter().find(|node| node.role == "ops").unwrap();

        assert_eq!(accounting.input_refs, vec![data.id.clone()]);
        assert_eq!(creative.input_refs, vec![data.id.clone()]);
        assert!(ops.input_refs.contains(&accounting.id));
        assert!(ops.input_refs.contains(&creative.id));
    }

    #[test]
    fn role_split_task_matches_ops_data_creative_rule() {
        let rules_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/json/agent/task-graph-rules.json");
        let rules = load_task_graph_rules(rules_path).expect("configured graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-role-split".to_string()),
            "请按运营、数据、内容三个虚拟员工分工，说明咖啡新品首周该怎么启动。",
            &rules,
        )
        .expect("role split task should match configured graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert_eq!(roles, vec!["ops", "data", "creative"]);

        let ops = graph.nodes.iter().find(|node| node.role == "ops").unwrap();
        let data = graph.nodes.iter().find(|node| node.role == "data").unwrap();
        let creative = graph
            .nodes
            .iter()
            .find(|node| node.role == "creative")
            .unwrap();

        assert!(data.input_refs.contains(&ops.id));
        assert!(creative.input_refs.contains(&ops.id));
        assert!(ops.objective.contains("咖啡新品首周"));
    }

    #[test]
    fn configurable_planner_builds_task_graph_from_matching_rule() {
        let rules = parse_task_graph_rules(
            r#"{
              "rules": [
                {
                  "id": "douyin_holiday_conversion",
                  "description": "抖音节日转化率",
                  "match_any": ["抖音", "五一"],
                  "match_all_any": [["转化率", "转化"], ["方案", "运营"]],
                  "nodes": [
                    {
                      "id": "data",
                      "role": "data",
                      "title": "数据诊断",
                      "objective": "分析：{{task}}"
                    },
                    {
                      "id": "creative",
                      "role": "creative",
                      "title": "创意产出",
                      "depends_on": ["data"],
                      "objective": "规则={{rule_id}}；{{rule_description}}"
                    },
                    {
                      "id": "ops",
                      "role": "ops",
                      "title": "运营方案",
                      "depends_on": ["creative"],
                      "objective": "整合输出"
                    }
                  ]
                }
              ]
            }"#,
        )
        .expect("valid configured rules");

        let graph = plan_task_graph_from_rules(
            &TaskId("task-config".to_string()),
            "五一抖音咖啡转化率运营方案",
            &rules,
        )
        .expect("rule should match");

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.nodes[0].role, "data");
        assert!(graph.nodes[0].objective.contains("五一抖音咖啡"));
        assert!(
            graph.nodes[0]
                .acceptance
                .contains(&DEFAULT_NODE_ACCEPTANCE.to_string())
        );
        assert!(
            !graph.nodes[0]
                .acceptance
                .iter()
                .any(|item| item.contains("content must be specific"))
        );
        assert_eq!(graph.nodes[1].role, "creative");
        assert_eq!(graph.nodes[1].input_refs, vec![graph.nodes[0].id.clone()]);
        assert!(
            graph.nodes[1]
                .objective
                .contains("douyin_holiday_conversion")
        );
        assert_eq!(graph.nodes[2].input_refs, vec![graph.nodes[1].id.clone()]);
    }

    #[test]
    fn configured_chat_quick_reply_uses_single_chat_node() {
        let rules_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/json/agent/task-graph-rules.json");
        let rules = load_task_graph_rules(rules_path).expect("configured graph rules should load");
        let graph = plan_task_graph_from_rules(&TaskId("task-chat".to_string()), "hi", &rules)
            .expect("hi should match configured chat quick reply graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "chat");
        assert_eq!(graph.nodes[0].role, "chat");
        assert!(graph.nodes[0].input_refs.is_empty());
        assert_eq!(graph.nodes[0].max_attempts, 1);
        assert!(
            !graph.nodes[0]
                .acceptance
                .iter()
                .any(|item| item.contains("reply should be brief"))
        );
        assert!(
            graph.nodes[0]
                .acceptance
                .iter()
                .any(|item| item.contains("回复应简短自然"))
        );
    }

    #[test]
    fn quick_chat_helper_only_matches_configured_chat_rules() {
        let chat_graph = plan_quick_chat_graph(&TaskId("task-chat".to_string()), "hi")
            .expect("hi should use quick chat graph");
        assert_eq!(chat_graph.nodes.len(), 1);
        assert_eq!(chat_graph.nodes[0].role, "chat");

        let business_graph = plan_quick_chat_graph(
            &TaskId("task-business".to_string()),
            "请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54",
        );
        assert!(business_graph.is_none());

        let greeting_business_graph = plan_quick_chat_graph(
            &TaskId("task-greeting-business".to_string()),
            "你好，请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54",
        );
        assert!(greeting_business_graph.is_none());
    }

    #[test]
    fn configurable_planner_returns_none_when_no_rule_matches() {
        let rules = parse_task_graph_rules(
            r#"{
              "rules": [
                {
                  "id": "seo_only",
                  "match_all": ["SEO"],
                  "nodes": [
                    {
                      "id": "web",
                      "role": "web",
                      "title": "SEO方案",
                      "objective": "优化搜索"
                    }
                  ]
                }
              ]
            }"#,
        )
        .expect("valid configured rules");

        let graph = plan_task_graph_from_rules(
            &TaskId("task-no-match".to_string()),
            "五一抖音咖啡转化率运营方案",
            &rules,
        );

        assert!(graph.is_none());
    }

    #[test]
    fn configurable_planner_rejects_unknown_role_and_cycles() {
        let unknown_role = parse_task_graph_rules(
            r#"{
              "rules": [
                {
                  "id": "bad_role",
                  "nodes": [
                    {
                      "id": "x",
                      "role": "not_a_role",
                      "title": "坏节点",
                      "objective": "test"
                    }
                  ]
                }
              ]
            }"#,
        )
        .expect_err("unknown role should fail");
        assert!(unknown_role.contains("unknown role"));

        let cycle = parse_task_graph_rules(
            r#"{
              "rules": [
                {
                  "id": "cycle",
                  "nodes": [
                    {
                      "id": "a",
                      "role": "data",
                      "title": "A",
                      "objective": "a",
                      "depends_on": ["b"]
                    },
                    {
                      "id": "b",
                      "role": "ops",
                      "title": "B",
                      "objective": "b",
                      "depends_on": ["a"]
                    }
                  ]
                }
              ]
            }"#,
        )
        .expect_err("cycle should fail");
        assert!(cycle.contains("root node") || cycle.contains("cycle"));
    }

    #[test]
    fn configurable_planner_accepts_roles_from_configured_catalog() {
        let rules = serde_json::from_str::<TaskGraphRuleSet>(
            r#"{
              "rules": [
                {
                  "id": "douyin_ads_plan",
                  "match_all": ["千川"],
                  "nodes": [
                    {
                      "id": "ads",
                      "role": "douyin_ads",
                      "title": "千川投放",
                      "objective": "优化千川计划：{{task}}"
                    }
                  ]
                }
              ]
            }"#,
        )
        .expect("rules json should parse");
        let mut roles = RoleCatalog::builtin().roles().to_vec();
        roles.push(crate::roles::RoleProfile {
            id: crate::roles::RoleId("role.douyin-ads".to_string()),
            name: "千川投放专员".to_string(),
            runtime_role: "douyin_ads".to_string(),
            priority: 25,
            domains: vec!["domain.ecommerce".to_string()],
            keywords: vec!["千川".to_string(), "投流".to_string()],
            preferred_actions: vec![crate::roles::RoleAction::Optimize],
            required_capabilities: vec!["ads.optimization".to_string()],
            optional_capabilities: vec!["data.analysis".to_string()],
        });
        let catalog = RoleCatalog::new(roles);

        validate_task_graph_rules_with_catalog(&rules, &catalog)
            .expect("configured role should be accepted");

        let graph =
            plan_task_graph_from_rules(&TaskId("task-ads".to_string()), "千川投流优化", &rules)
                .expect("rule should match");
        assert_eq!(graph.nodes[0].role, "douyin_ads");
    }

    #[test]
    fn planner_prioritizes_service_role_for_support_workflows() {
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: "客服团队近期投诉和退款增加，请设计一套售后SOP".to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 3,
        });
        let graph = plan_task_graph(
            &TaskId("task-service".to_string()),
            "客服团队近期投诉和退款增加，请设计一套售后SOP",
            Some(&route),
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "service");
    }

    #[test]
    fn planner_prioritizes_engineering_role_for_sla_incidents() {
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: "系统接口延迟升高并影响SLA，请给出架构排查和回滚方案".to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 3,
        });
        let graph = plan_task_graph(
            &TaskId("task-engineering".to_string()),
            "系统接口延迟升高并影响SLA，请给出架构排查和回滚方案",
            Some(&route),
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "engineering");
    }

    #[test]
    fn planner_prioritizes_web_role_for_seo_workflows() {
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: "为跨境电商独立站制定SEO关键词集群、标题优化和收录策略".to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 3,
        });
        let graph = plan_task_graph(
            &TaskId("task-web".to_string()),
            "为跨境电商独立站制定SEO关键词集群、标题优化和收录策略",
            Some(&route),
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "web");
    }

    #[test]
    fn planner_upgrades_category_commerce_analysis_to_team_graph() {
        let task = "请分析咖啡类商品";
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: task.to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 6,
        });
        let graph = plan_task_graph(
            &TaskId("task-category-team".to_string()),
            task,
            Some(&route),
        );
        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(roles.first().copied(), Some("data"));
        assert!(roles.contains(&"creative"));
        assert!(roles.contains(&"service"));
        assert!(roles.contains(&"engineering"));
        assert_eq!(roles.last().copied(), Some("ops"));

        let ops = graph.nodes.iter().find(|node| node.role == "ops").unwrap();
        assert!(ops.input_refs.iter().any(|input| input.0 == "data"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "service"));
        assert!(ops.objective.contains("final_report"));
    }

    #[test]
    fn planner_preserves_direct_one_sentence_answer_without_team_graph() {
        let task = "请用一句话说明如何判断直播间转化是否健康，不引用任何预设商品。";
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: task.to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 6,
        });
        let graph = plan_task_graph(
            &TaskId("task-direct-one-line".to_string()),
            task,
            Some(&route),
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "chat");
        assert!(graph.nodes[0].objective.contains("直接回答"));
    }

    #[test]
    fn planner_respects_manual_multi_role_route_before_direct_answer() {
        let task = "请用一句话说明如何判断直播间转化是否健康，不引用任何预设商品。";
        let route = crate::roles::RoleRouter::default()
            .route_for_runtime_roles(
                "ops",
                &[
                    "data".to_string(),
                    "creative".to_string(),
                    "service".to_string(),
                    "engineering".to_string(),
                ],
            )
            .expect("manual route should resolve built-in roles");

        let graph = plan_task_graph(
            &TaskId("task-manual-direct-team".to_string()),
            task,
            Some(&route),
        );
        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert!(roles.len() > 1, "{roles:?}");
        assert!(roles.contains(&"ops"), "{roles:?}");
        assert!(roles.contains(&"data"), "{roles:?}");
        assert!(roles.contains(&"creative"), "{roles:?}");
        assert!(!roles.contains(&"chat"), "{roles:?}");
    }

    #[test]
    fn planner_routes_manual_live_metric_health_question_to_ecommerce_team_graph() {
        let task = "请用Agent集群方式判断：直播间曝光1000、点击80、加购20、下单8、付款7是否健康，\
                    不引用任何预设商品，并说明各虚拟员工如何协作。";
        let route = crate::roles::RoleRouter::default()
            .route_for_runtime_roles(
                "ops",
                &[
                    "data".to_string(),
                    "creative".to_string(),
                    "service".to_string(),
                    "engineering".to_string(),
                ],
            )
            .expect("manual route should resolve built-in roles");

        let graph = plan_task_graph(
            &TaskId("task-live-health-team".to_string()),
            task,
            Some(&route),
        );
        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(roles.first().copied(), Some("data"), "{roles:?}");
        assert_eq!(roles.last().copied(), Some("ops"), "{roles:?}");
        assert!(roles.contains(&"creative"), "{roles:?}");
        assert!(roles.contains(&"service"), "{roles:?}");
        assert!(roles.contains(&"engineering"), "{roles:?}");
        let ops = graph.nodes.iter().find(|node| node.role == "ops").unwrap();
        assert!(ops.input_refs.iter().any(|input| input.0 == "data"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "creative"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "service"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "engineering"));
    }

    #[test]
    fn planner_manual_live_team_uses_requested_roles_without_auto_unregistered_roles() {
        let task = "请用Agent集群方式判断：直播间曝光1000、点击80、加购20、下单8、付款7是否健康，\
                    不引用任何预设商品，并说明各虚拟员工如何协作。";
        let route = crate::roles::RoleRouter::default()
            .route_for_runtime_roles(
                "ops",
                &[
                    "data".to_string(),
                    "creative".to_string(),
                    "service".to_string(),
                    "engineering".to_string(),
                ],
            )
            .expect("manual route should resolve built-in roles");

        let graph = plan_task_graph(
            &TaskId("task-live-health-requested-team".to_string()),
            task,
            Some(&route),
        );
        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            roles,
            vec!["data", "creative", "service", "engineering", "ops"]
        );
        assert!(!roles.contains(&"douyin_ads"), "{roles:?}");
        assert!(!roles.contains(&"accounting"), "{roles:?}");
    }

    #[test]
    fn planner_respects_text_explicit_three_role_limit() {
        let task = "请用教学模式，只让运营、数据、内容三个岗位协作，分析咖啡类商品直播转化；\
                    不要引用任何预设商品。";
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: task.to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 4,
        });

        let graph = plan_task_graph(
            &TaskId("task-explicit-three-role-limit".to_string()),
            task,
            Some(&route),
        );
        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(roles, vec!["data", "creative", "ops"], "{roles:?}");
        assert!(!roles.contains(&"accounting"), "{roles:?}");
        assert!(!roles.contains(&"douyin_ads"), "{roles:?}");
        assert!(!roles.contains(&"service"), "{roles:?}");
        assert!(!roles.contains(&"engineering"), "{roles:?}");
    }

    #[test]
    fn planner_routes_single_data_funnel_task_to_data_role() {
        let task = "请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54";
        let route = crate::roles::RoleRouter::default().route(crate::roles::RoleRouteInput {
            message: task.to_string(),
            domain_id: Some("domain.general".to_string()),
            action: None,
            max_roles: 3,
        });
        let graph = plan_task_graph(&TaskId("task-funnel".to_string()), task, Some(&route));

        assert_eq!(graph.nodes[0].role, "data");
        assert_eq!(
            graph.nodes[0].worker_id,
            WorkerId("worker.data".to_string())
        );
        assert!(graph.nodes[0].input_refs.is_empty());
        assert!(graph.nodes.iter().any(|node| node.role == "ops"));
    }

    #[test]
    fn scheduler_returns_only_one_ready_node_per_worker() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-same-worker".to_string()),
            root_task: "same worker".to_string(),
            nodes: vec![
                node(
                    "chat-a",
                    "chat",
                    "直接回答 A",
                    "A".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "chat-b",
                    "chat",
                    "直接回答 B",
                    "B".to_string(),
                    Vec::new(),
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let ready = runtime.ready_nodes();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].worker_id, WorkerId("worker.chat".to_string()));

        let assignment = runtime.build_assignment(&ready[0]);
        runtime.mark_running(&assignment);
        assert!(runtime.ready_nodes().is_empty());

        let outcome = runtime.apply_report(report_from_assignment(&assignment, "first done"));
        assert!(matches!(outcome, EvaluationOutcome::Passed(_)));
        let next_ready = runtime.ready_nodes();
        assert_eq!(next_ready.len(), 1);
        assert_ne!(next_ready[0].id, ready[0].id);
    }

    #[test]
    fn runtime_requests_rework_for_empty_report() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-4".to_string()),
            root_task: "解释一下 agent".to_string(),
            nodes: vec![node(
                "chat",
                "chat",
                "通用对话",
                "解释一下 agent".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);

        let outcome = runtime.apply_report(report_from_assignment(&assignment, ""));

        assert!(matches!(outcome, EvaluationOutcome::Rework { .. }));
        assert!(matches!(
            runtime.states.get(&node.id),
            Some(TaskNodeRuntimeState::Pending)
        ));
        let next_assignment = runtime.build_assignment(&node);
        assert_eq!(next_assignment.attempt, 2);
        assert!(next_assignment.rework_instruction.is_some());
    }

    #[test]
    fn runtime_fails_after_attempts_are_exhausted() {
        let mut graph = plan_task_graph(&TaskId("task-5".to_string()), "解释一下 agent", None);
        graph.nodes[0].max_attempts = 1;
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);

        let outcome = runtime.apply_report(report_from_assignment(&assignment, ""));

        assert!(matches!(outcome, EvaluationOutcome::Failed(_)));
        assert!(runtime.is_finished());
        assert!(runtime.synthesize().contains("未完成或需复核"));
    }

    #[test]
    fn runtime_requests_rework_when_open_questions_remain() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-8".to_string()),
            root_task: "解释一下 agent".to_string(),
            nodes: vec![node(
                "chat",
                "chat",
                "通用对话",
                "解释一下 agent".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "需要更多上下文");
        report.open_questions = vec!["必须补充业务指标口径后才能继续".to_string()];

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Rework { .. }));
        assert!(matches!(
            runtime.states.get(&node.id),
            Some(TaskNodeRuntimeState::Pending)
        ));
    }

    #[test]
    fn evaluator_allows_non_blocking_open_questions_with_substantive_output() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-design-gaps".to_string()),
            root_task: "做详情页设计建议".to_string(),
            nodes: vec![node(
                "design",
                "design",
                "体验设计",
                "做详情页设计建议".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "设计建议完整");
        let role_output = report.role_output.as_mut().expect("role output");
        role_output.open_questions = vec!["需要当前详情页截图以便进一步细化设计。".to_string()];
        report.open_questions = role_output.open_questions.clone();

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Passed(_)));
    }

    #[test]
    fn evaluator_rejects_incomplete_role_output_for_data_role() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-10".to_string()),
            root_task: "分析转化率下降".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析转化率下降".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "数据结论不足");
        report.role_output = Some(RoleWorkOutput {
            summary: "数据结论不足".to_string(),
            findings: vec!["转化下降".to_string()],
            recommendations: Vec::new(),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Rework { .. }));
        assert!(matches!(
            runtime.states.get(&node.id),
            Some(TaskNodeRuntimeState::Pending)
        ));
    }

    #[test]
    fn evaluator_accepts_actionable_data_gap_as_support_output() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-data-gap".to_string()),
            root_task: "请用一句话说明如何判断直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "作为数据支持角色判断直播间转化健康度".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "数据支持说明");
        report.role_output = Some(RoleWorkOutput {
            summary: "没有指定商品和真实漏斗数据时，只能给出健康判断框架。".to_string(),
            findings: vec!["需要同时看曝光、点击、加购、付款和退款链路。".to_string()],
            recommendations: vec!["补齐直播间漏斗指标后，再判断转化率是否健康。".to_string()],
            evidence: vec!["原始任务明确不引用任何预设商品。".to_string()],
            risks: vec!["缺少真实指标时不能编造具体转化率或行业排名。".to_string()],
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Passed(_)));
    }

    #[test]
    fn evaluator_accepts_actionable_service_support_output_without_service_scene() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-service-support".to_string()),
            root_task: "请用一句话说明如何判断直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![node(
                "service",
                "service",
                "服务方案",
                "作为服务支持角色补充转化健康风险".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "服务支持说明");
        report.role_output = Some(RoleWorkOutput {
            summary: "服务支持角色关注咨询承接、异议处理和售后承诺对转化的影响。".to_string(),
            findings: vec!["如果咨询响应慢或承诺不清，点击到付款会出现流失。".to_string()],
            recommendations: vec![
                "同步准备高频异议话术和售后边界，避免用户在付款前反复犹豫。".to_string(),
            ],
            evidence: vec!["原始任务要求判断直播间转化健康度。".to_string()],
            risks: vec!["不能用未确认的政策或库存承诺刺激转化。".to_string()],
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Passed(_)));
    }

    #[test]
    fn evaluator_rejects_missing_structured_role_output() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-15".to_string()),
            root_task: "分析转化率下降".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析转化率下降".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "只有纯文本，不含结构化角色产物");
        report.role_output = None;

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Rework { .. }));
    }

    #[test]
    fn evaluator_passes_complete_ops_role_output() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-11".to_string()),
            root_task: "给运营调整方案".to_string(),
            nodes: vec![node(
                "ops",
                "ops",
                "运营方案",
                "给运营调整方案".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "运营方案完整");
        report.role_output = Some(RoleWorkOutput {
            summary: "先用三档优先级推进运营调整。".to_string(),
            findings: vec!["当前目标是提升转化".to_string()],
            recommendations: vec![
                "P1 修正首屏卖点".to_string(),
                "P2 调整投放人群".to_string(),
                "P3 每日复盘转化".to_string(),
            ],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["预算消耗需设置止损线".to_string()],
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Passed(_)));
    }

    #[test]
    fn synthesis_uses_structured_role_output_sections() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-12".to_string()),
            root_task: "给运营调整方案".to_string(),
            nodes: vec![node(
                "ops",
                "ops",
                "运营方案",
                "给运营调整方案".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "运营方案完整");
        report.role_output = Some(RoleWorkOutput {
            summary: "运营先聚焦转化链路。".to_string(),
            findings: vec!["首屏承接弱".to_string()],
            recommendations: vec![
                "P1 优化首屏".to_string(),
                "P2 调整人群".to_string(),
                "P3 复盘素材".to_string(),
            ],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["不要同时改太多变量".to_string()],
            open_questions: Vec::new(),
        });
        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();

        assert!(output.contains("# 多智能体协作结论"));
        assert!(output.contains("运营先聚焦转化链路"));
        assert!(output.contains("## 总回答"));
        assert!(output.contains("## 主岗综合方案"));
        assert!(output.contains("## 虚拟员工分工"));
        assert!(output.contains("## 岗位协作过程"));
        assert!(output.contains("| 类型 | 岗位 | 负责问题 | 独立判断 | 交给团队的内容 |"));
        assert!(output.contains("发现: 首屏承接弱"));
        assert!(output.contains("建议: P1 优化首屏"));
        assert!(output.contains("风险: 不要同时改太多变量"));
        assert!(output.contains("## 下一步"));
        assert!(output.contains("1. P1 优化首屏"));
        assert!(!output.contains("1. 1."));
        assert!(!output.contains("## 执行检查清单"));
        assert!(output.contains("## 总结与交付物"));
        assert!(output.contains("主回答岗位"));
    }

    #[test]
    fn manual_ecommerce_multi_role_graph_uses_team_flow_with_ops_synthesis() {
        let route = crate::roles::RoleRouter::default()
            .route_for_runtime_roles(
                "ops",
                &[
                    "data".to_string(),
                    "service".to_string(),
                    "creative".to_string(),
                    "engineering".to_string(),
                ],
            )
            .expect("manual role route");
        let graph = plan_task_graph(
            &TaskId("task-coffee-category".to_string()),
            "请分析咖啡类商品",
            Some(&route),
        );

        assert_eq!(
            graph.nodes.first().map(|node| node.role.as_str()),
            Some("data")
        );
        assert_eq!(
            graph.nodes.last().map(|node| node.role.as_str()),
            Some("ops")
        );
        assert!(!graph.nodes.iter().any(|node| node.role == "chat"));
        let ops = graph.nodes.iter().find(|node| node.role == "ops").unwrap();
        assert!(ops.input_refs.iter().any(|input| input.0 == "data"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "service"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "creative"));
        assert!(ops.input_refs.iter().any(|input| input.0 == "engineering"));

        let mut runtime = TaskGraphRuntime::new(graph);
        while let Some(node) = runtime.ready_nodes().into_iter().next() {
            let assignment = runtime.build_assignment(&node);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "咖啡类目分析完成");
            if let Some(output) = report.role_output.as_mut() {
                output.evidence.push(
                    "本轮未检测到 context.selected_summary；不注入任何预设商品资料，只按用户文本、\
                     会话历史和工作区上下文理解问题。"
                        .to_string(),
                );
                if !assignment.inputs.is_empty() {
                    output.evidence.extend(
                        assignment
                            .inputs
                            .iter()
                            .map(|input| format!("worker.assignment.input.{}", input.node_id.0)),
                    );
                }
            }
            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        assert!(runtime.is_finished());
        let output = runtime.synthesize();

        assert!(output.contains("主回答岗位**：运营方案（运营策略师）"));
        assert!(output.contains("本轮未检测到用户手动选择的预设商品资料"));
        assert!(output.contains("**协作方式**：运营策略师牵头"));
        assert!(output.contains("## 总回答"));
        assert!(output.contains("## 虚拟员工分工"));
        assert!(output.contains("## 主岗综合方案"));
        assert!(output.contains("**团队判断**"));
        assert!(output.contains("先判健康度"));
        assert!(output.contains("再做转化承接"));
        assert!(output.contains("最后保留可验证性"));
        assert!(output.contains("| 运营策略师 |"));
        assert!(output.contains("| 数据分析师 |"));
        assert!(output.contains("| 客户成功专员 |"));
        assert!(output.contains("| 内容创意师 |"));
        assert!(output.contains("| 技术架构师 |"));
        assert!(output.contains("final_report"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_level_boundary_reports_are_accepted_as_team_inputs() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-boundary".to_string()),
            root_task: "请分析咖啡类商品，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "先给出运营判断".to_string(),
                    Vec::new(),
                    1,
                ),
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "基于运营输入补充指标判断".to_string(),
                    vec!["ops"],
                    1,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "基于运营输入补充服务判断".to_string(),
                    vec!["ops"],
                    1,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&ops_assignment, "运营方案完整")),
            EvaluationOutcome::Passed(_)
        ));

        for node in runtime.ready_nodes() {
            let assignment = runtime.build_assignment(&node);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "类目级边界分析完成");
            if let Some(output) = report.role_output.as_mut() {
                output.evidence.push(
                    "本轮未检测到 context.selected_summary；不注入任何预设商品资料，只按用户文本、\
                     会话历史和工作区上下文理解问题。"
                        .to_string(),
                );
                output.risks.push(
                    "数据边界：本轮未手动选择预设商品，按类目级分析降级处理，不能编造商品指标。"
                        .to_string(),
                );
                output
                    .open_questions
                    .push("如果后续要做单品级诊断，再由用户手动选择预设商品。".to_string());
            }

            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        assert!(runtime.is_finished());
        let output = runtime.synthesize();
        assert!(output.contains("## 虚拟员工分工"));
        assert!(output.contains("数据分析师"));
        assert!(output.contains("客户成功专员"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_level_quality_gaps_are_downgraded_when_report_has_boundary_output() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-thin-boundary".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析咖啡类商品".to_string(),
                Vec::new(),
                1,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "类目级数据边界说明");
        report.role_output = Some(RoleWorkOutput {
            summary: "数据岗按类目级链路给出边界判断。".to_string(),
            findings: vec!["当前未选商品，只能先定义曝光、点击、加购、付款漏斗口径。".to_string()],
            recommendations: Vec::new(),
            evidence: vec![
                "category_level_boundary".to_string(),
                "用户本轮没有手动选择预设商品。".to_string(),
            ],
            risks: vec!["数据边界：不能编造咖啡类商品的真实转化率。".to_string()],
            open_questions: Vec::new(),
        });

        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));
        assert!(runtime.is_finished());
        let output = runtime.synthesize();
        assert!(!output.contains("data role should provide at least two findings"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_level_failed_status_with_boundary_output_is_accepted() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-runtime-boundary".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析咖啡类商品".to_string(),
                Vec::new(),
                1,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "类目级数据边界说明");
        report.status = WorkerReportStatus::Failed;
        report.role_output = Some(RoleWorkOutput {
            summary: "数据岗已按类目级链路给出可执行边界。".to_string(),
            findings: vec![
                "当前未手动选择预设商品，只能先定义曝光、点击、加购、付款和退款漏斗口径。"
                    .to_string(),
                "没有真实漏斗数据时不能编造转化率或行业排名。".to_string(),
            ],
            recommendations: vec![
                "先按类目通用链路输出判断框架，再由用户手动选择商品进入单品复核。".to_string(),
            ],
            evidence: vec![
                "category_level_boundary".to_string(),
                "本轮未检测到 context.selected_summary。".to_string(),
            ],
            risks: vec!["数据边界：缺少单品指标不阻塞类目级回答。".to_string()],
            open_questions: Vec::new(),
        });

        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));
        assert!(runtime.is_finished());
        let output = runtime.synthesize();
        assert!(output.contains("数据岗已按类目级链路"));
        assert!(!output.contains("worker reported failed status"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_level_root_task_accepts_actionable_boundary_without_explicit_marker() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-root-boundary".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![node(
                "service",
                "service",
                "服务方案",
                "分析咖啡类商品的服务承接".to_string(),
                Vec::new(),
                1,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let service = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&service);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "类目级服务承接边界说明");
        report.role_output = Some(RoleWorkOutput {
            summary: "服务岗按类目级链路判断咖啡类商品咨询承接。".to_string(),
            findings: vec!["当前未选择具体商品，只能先识别咨询、售后和履约承诺边界。".to_string()],
            recommendations: vec!["先准备类目级客服答疑、售后承诺和异常升级 SOP。".to_string()],
            evidence: vec!["用户本轮没有手动选择预设商品。".to_string()],
            risks: vec!["类目级服务结论不能替代具体商品库存、物流和退款政策复核。".to_string()],
            open_questions: Vec::new(),
        });

        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));
        let output = runtime.synthesize();
        assert!(output.contains("团队执行稿") || output.contains("服务岗"));
        assert!(!output.contains("service role output lacks service-specific content"));
        assert!(!output.contains("未完成或需复核"));
        assert!(
            output.contains("咖啡类商品应优先拆成人群、场景、价格带、内容卖点、渠道承接和复购风险")
        );
        assert!(output.contains("不会把任何历史商品、测试商品或默认商品注入为本轮商品"));
    }

    #[test]
    fn category_team_thin_support_reports_are_repaired_as_boundaries() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-team-repair".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先识别咖啡类商品指标口径".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "基于数据输入补充服务承接".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合数据和服务输入".to_string(),
                    vec!["data", "service"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        let mut data_report = report_from_assignment(&data_assignment, "数据边界");
        data_report.role_output = Some(RoleWorkOutput {
            summary: "数据岗确认咖啡类商品缺少真实曝光、点击、加购和付款指标。".to_string(),
            findings: vec!["当前没有真实漏斗数据，不能编造行业排名。".to_string()],
            recommendations: Vec::new(),
            evidence: vec!["用户本轮没有手动选择预设商品。".to_string()],
            risks: vec!["缺少真实数据时只能输出类目级框架。".to_string()],
            open_questions: Vec::new(),
        });
        assert!(matches!(
            runtime.apply_report(data_report),
            EvaluationOutcome::Passed(_)
        ));

        let service = runtime.ready_nodes().remove(0);
        let service_assignment = runtime.build_assignment(&service);
        runtime.mark_running(&service_assignment);
        let mut service_report = report_from_assignment(&service_assignment, "服务承接边界");
        service_report.role_output = Some(RoleWorkOutput {
            summary: "服务岗确认咖啡类商品应补咨询承接和售后承诺边界。".to_string(),
            findings: vec!["未指定商品时只能先按类目服务风险处理。".to_string()],
            recommendations: Vec::new(),
            evidence: vec!["worker.assignment.input.data".to_string()],
            risks: Vec::new(),
            open_questions: Vec::new(),
        });
        assert!(matches!(
            runtime.apply_report(service_report),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合完成");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec![
                "worker.assignment.input.data".to_string(),
                "worker.assignment.input.service".to_string(),
            ];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        assert!(runtime.is_finished());
        let output = runtime.synthesize();
        assert!(!output.contains("data role should provide at least two findings"));
        assert!(!output.contains("service role output lacks service-specific content"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_team_failed_support_reports_are_accepted_without_raw_quality_leak() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-team-failed-support".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先识别咖啡类商品指标口径".to_string(),
                    Vec::new(),
                    1,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "作为支持角色 service，基于主角色输入补充本岗位判断，并指出证据、\
                     风险和下一步：请分析咖啡类商品"
                        .to_string(),
                    vec!["data"],
                    1,
                ),
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "把类目判断转成内容测试方向".to_string(),
                    vec!["data"],
                    1,
                ),
                node(
                    "engineering",
                    "engineering",
                    "工程判断",
                    "补充埋点和数据复盘边界".to_string(),
                    vec!["data"],
                    1,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合所有岗位形成 final_report".to_string(),
                    vec!["data", "service", "creative", "engineering"],
                    1,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        let mut data_report = report_from_assignment(&data_assignment, "数据诊断");
        data_report.status = WorkerReportStatus::Failed;
        data_report.role_output = Some(RoleWorkOutput {
            summary: "数据岗确认咖啡类商品缺少真实曝光、点击、加购和付款指标。".to_string(),
            findings: vec!["当前没有真实漏斗数据，不能编造行业排名。".to_string()],
            recommendations: Vec::new(),
            evidence: vec!["用户本轮没有手动选择预设商品。".to_string()],
            risks: Vec::new(),
            open_questions: vec!["请提供咖啡类商品的具体指标来源。".to_string()],
        });
        assert!(matches!(
            runtime.apply_report(data_report),
            EvaluationOutcome::Passed(_)
        ));

        for node_id in ["service", "creative", "engineering"] {
            let ready = runtime
                .ready_nodes()
                .into_iter()
                .find(|node| node.id.0 == node_id)
                .expect("support node should be ready");
            let assignment = runtime.build_assignment(&ready);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "支持岗位边界");
            report.status = WorkerReportStatus::Failed;
            report.role_output = Some(RoleWorkOutput {
                summary: format!(
                    "{}按类目级链路给出支持判断，当前缺少真实商品和业务数据。",
                    role_display_name(&assignment.role)
                ),
                findings: vec!["未指定商品时只能先按类目服务、内容或工程风险处理。".to_string()],
                recommendations: Vec::new(),
                evidence: vec!["worker.assignment.input.data".to_string()],
                risks: Vec::new(),
                open_questions: vec!["待用户补充具体商品或真实指标后再做单品级复核。".to_string()],
            });
            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合完成");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec![
                "worker.assignment.input.data".to_string(),
                "worker.assignment.input.service".to_string(),
                "worker.assignment.input.creative".to_string(),
                "worker.assignment.input.engineering".to_string(),
            ];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        assert!(runtime.is_finished());
        let output = runtime.synthesize();
        assert!(output.contains("岗位独立产出"));
        assert!(output.contains("团队感知"));
        assert!(output.contains("团队合成"));
        assert!(output.contains("各自完成判断"));
        assert!(output.contains("虚拟员工分工"));
        assert!(output.contains("final_report"));
        assert!(output.contains("不会把任何历史商品、测试商品或默认商品注入为本轮商品"));
        assert!(!output.contains("data role should provide at least two findings"));
        assert!(!output.contains("service role output lacks service-specific content"));
        assert!(!output.contains("engineering role output lacks technical content"));
        assert!(!output.contains("worker reported failed status"));
        assert!(!output.contains("未完成或需复核"));
        assert!(!output.contains("千乐咖啡"));
    }

    #[test]
    fn live_conversion_no_preset_product_failed_support_reports_are_boundary() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-live-no-product-boundary".to_string()),
            root_task: "请用一句话说明如何判断直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先识别直播间漏斗口径".to_string(),
                    Vec::new(),
                    1,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "补充咨询承接和售后边界".to_string(),
                    vec!["data"],
                    1,
                ),
                node(
                    "engineering",
                    "engineering",
                    "工程判断",
                    "补充埋点和复盘边界".to_string(),
                    vec!["data"],
                    1,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合所有岗位形成 final_report".to_string(),
                    vec!["data", "service", "engineering"],
                    1,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        let mut data_report = report_from_assignment(&data_assignment, "直播间数据边界");
        data_report.status = WorkerReportStatus::Failed;
        data_report.role_output = Some(RoleWorkOutput {
            summary: "未选择预设商品时，直播间转化健康度只能先按曝光、点击、加购、\
                      付款和退款链路定义口径。"
                .to_string(),
            findings: vec!["缺少真实曝光、点击、加购和付款指标，不能编造转化率。".to_string()],
            recommendations: vec!["补齐直播间漏斗指标后，再判断各段转化是否健康。".to_string()],
            evidence: vec!["原始任务明确不引用任何预设商品。".to_string()],
            risks: vec!["没有真实指标时不能承诺健康或不健康。".to_string()],
            open_questions: Vec::new(),
        });
        assert!(matches!(
            runtime.apply_report(data_report),
            EvaluationOutcome::Passed(_)
        ));

        for node_id in ["service", "engineering"] {
            let ready = runtime
                .ready_nodes()
                .into_iter()
                .find(|node| node.id.0 == node_id)
                .expect("support node should be ready");
            let assignment = runtime.build_assignment(&ready);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "支持岗位边界");
            report.status = WorkerReportStatus::Failed;
            report.role_output = Some(RoleWorkOutput {
                summary: format!(
                    "{}补充直播间转化判断的协作边界，当前缺少真实商品和业务数据。",
                    role_display_name(&assignment.role)
                ),
                findings: vec!["未选择预设商品时，只能按直播转化链路给出岗位判断。".to_string()],
                recommendations: Vec::new(),
                evidence: vec!["worker.assignment.input.data".to_string()],
                risks: vec!["缺少真实数据会影响咨询承接或技术复盘判断。".to_string()],
                open_questions: vec!["待用户补充真实直播间漏斗数据后再复核。".to_string()],
            });
            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合完成");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec![
                "worker.assignment.input.data".to_string(),
                "worker.assignment.input.service".to_string(),
                "worker.assignment.input.engineering".to_string(),
            ];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();
        assert!(output.contains("直播间转化是否健康"));
        assert!(output.contains("final_report"));
        assert!(output.contains("不会把任何历史商品、测试商品或默认商品注入为本轮商品"));
        assert!(!output.contains("data role should provide at least two findings"));
        assert!(!output.contains("service role output lacks service-specific content"));
        assert!(!output.contains("engineering role output lacks technical content"));
        assert!(!output.contains("未完成或需复核"));
        assert!(!output.contains("千乐咖啡"));
    }

    #[test]
    fn multi_role_live_conversion_answer_is_substantial_without_default_product() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-live-multi-role-quality".to_string()),
            root_task: "请用Agent集群方式判断：直播间曝光1000、点击80、加购20、下单8、\
                        付款7是否健康，不引用任何预设商品，并说明各虚拟员工如何协作。"
                .to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "计算直播间漏斗指标并说明数据边界".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "补充咨询承接和售后边界".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "补充直播内容与素材测试方向".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "engineering",
                    "engineering",
                    "工程判断",
                    "补充埋点和复盘边界".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合所有岗位形成 final_report".to_string(),
                    vec!["data", "service", "creative", "engineering"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        for node_id in ["data", "service", "creative", "engineering", "ops"] {
            let ready = runtime
                .ready_nodes()
                .into_iter()
                .find(|node| node.id.0 == node_id)
                .expect("node should become ready");
            let assignment = runtime.build_assignment(&ready);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "直播间漏斗岗位产物");
            if let Some(output) = report.role_output.as_mut() {
                output.summary = match node_id {
                    "data" => "数据岗计算到曝光到点击、点击到加购、加购到下单、\
                               下单到付款和总转化，需要结合行业基准判断健康度。"
                        .to_string(),
                    "service" => "客服岗认为需要同步咨询承接、优惠解释、退款边界和异常升级 \
                                  SOP，避免支付前阻力被误判成流量问题。"
                        .to_string(),
                    "creative" => "创意岗把漏斗问题转成直播开场钩子、利益点标题、短视频素材和 A/B \
                                   测试方向。"
                        .to_string(),
                    "engineering" => "工程岗补充埋点、接口刷新、报表口径和回滚边界，\
                                      保证后续复盘可追踪。"
                        .to_string(),
                    _ => "运营主岗把数据、内容、服务和工程输入合成为优先动作、复盘节奏和 \
                          final_report。"
                        .to_string(),
                };
                if node_id != "data" {
                    output
                        .evidence
                        .push("worker.assignment.input.data".to_string());
                }
                if node_id == "ops" {
                    output.evidence.extend(
                        [
                            "worker.assignment.input.service",
                            "worker.assignment.input.creative",
                            "worker.assignment.input.engineering",
                        ]
                        .map(String::from),
                    );
                }
            }
            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        let output = runtime.synthesize();

        assert!(output.contains("直播间转化是否健康"));
        assert!(output.contains("直播漏斗确定性初判"));
        assert!(output.contains("曝光到点击约 8.0%"));
        assert!(output.contains("点击到加购约 25.0%"));
        assert!(output.contains("加购到下单约 40.0%"));
        assert!(output.contains("下单到付款约 87.5%"));
        assert!(output.contains("曝光到付款总转化约 0.70%"));
        assert!(output.contains("团队调度方式"));
        assert!(output.contains("团队合成"));
        assert!(output.contains("团队感知"));
        assert!(output.contains("虚拟员工分工"));
        assert!(output.contains("数据分析师"));
        assert!(output.contains("运营策略师"));
        assert!(output.contains("客户成功专员"));
        assert!(output.contains("技术架构师"));
        assert!(output.contains("final_report"));
        assert!(output.contains("本轮用户明确要求不引用预设商品"));
        assert!(!output.contains("data role should provide at least two findings"));
        assert!(!output.contains("service role output lacks service-specific content"));
        assert!(!output.contains("engineering role output lacks technical content"));
        assert!(!output.contains("worker reported failed status"));
        assert!(!output.contains("千乐咖啡"));
    }

    #[test]
    fn explicit_no_preset_product_rejects_default_product_leak() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-no-preset-product-leak".to_string()),
            root_task: "请判断直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析直播间转化".to_string(),
                Vec::new(),
                1,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "错误引用了默认商品");
        report.role_output = Some(RoleWorkOutput {
            summary: "本轮使用千乐咖啡的 context.selected_summary 做直播间判断。".to_string(),
            findings: vec![
                "千乐咖啡被当成本轮默认商品。".to_string(),
                "直播漏斗需要复核。".to_string(),
            ],
            recommendations: vec!["继续分析千乐咖啡。".to_string()],
            evidence: vec!["来源 context.selected_summary".to_string()],
            risks: vec!["默认商品污染本轮回答。".to_string()],
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Failed(_)));
        let output = runtime.synthesize();
        assert!(output.contains("不能把历史商品或默认商品当成本轮商品"));
        assert!(!output.contains("selected product context is not allowed for this task"));
    }

    #[test]
    fn explicit_no_preset_product_allows_absent_selected_summary_boundary() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-no-selected-summary-boundary".to_string()),
            root_task: "请判断直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析直播间转化".to_string(),
                Vec::new(),
                1,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "未选商品边界");
        report.status = WorkerReportStatus::Failed;
        report.role_output = Some(RoleWorkOutput {
            summary: "本轮未检测到 context.selected_summary；不注入任何预设商品资料，\
                      只按用户文本和会话历史判断。"
                .to_string(),
            findings: vec!["直播间健康度需要曝光、点击、加购、下单、付款等真实指标。".to_string()],
            recommendations: vec![
                "先按用户提供的漏斗数字给出初判，缺行业基准时只列复核项。".to_string(),
            ],
            evidence: vec![CATEGORY_LEVEL_BOUNDARY_MARKER.to_string()],
            risks: vec!["未选商品链路不能默认引用千乐咖啡或任何历史商品。".to_string()],
            open_questions: Vec::new(),
        });

        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));
        let output = runtime.synthesize();
        assert!(output.contains("不注入任何默认商品或历史商品"));
        assert!(!output.contains("本轮使用用户手动选择的预设商品资料"));
        assert!(!output.contains("worker reported failed status"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_team_synthesis_explains_per_role_handoff_and_artifact() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-team-synthesis".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先识别咖啡类商品指标口径".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "把类目判断转成素材测试方向".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "补充客服承接边界".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "engineering",
                    "engineering",
                    "工程判断",
                    "补充埋点和数据复盘边界".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合所有岗位形成 final_report".to_string(),
                    vec!["data", "creative", "service", "engineering"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        for node_id in ["data", "creative", "service", "engineering", "ops"] {
            let ready = runtime
                .ready_nodes()
                .into_iter()
                .find(|node| node.id.0 == node_id)
                .expect("node should become ready");
            let assignment = runtime.build_assignment(&ready);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "岗位产物");
            if let Some(output) = report.role_output.as_mut() {
                output.evidence.push("category_level_boundary".to_string());
                if node_id != "data" {
                    output
                        .evidence
                        .push("worker.assignment.input.data".to_string());
                }
                if node_id == "ops" {
                    output.evidence.extend(
                        [
                            "worker.assignment.input.creative",
                            "worker.assignment.input.service",
                            "worker.assignment.input.engineering",
                        ]
                        .map(String::from),
                    );
                }
            }
            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        let output = runtime.synthesize();
        assert!(output.contains("团队执行顺序"));
        assert!(output.contains("数据诊断先给出类目级漏斗口径"));
        assert!(output.contains("内容创意师把类目判断转成卖点测试"));
        assert!(output.contains("客户成功专员补齐咨询承接"));
        assert!(output.contains("技术架构师补齐埋点复盘"));
        assert!(!output.contains("内容/服务/工程岗位"));
        assert!(output.contains("final_report"));
        assert!(output.contains("不会把任何历史商品"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn category_team_thin_role_outputs_become_boundaries_not_failures() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-category-thin-boundary".to_string()),
            root_task: "请分析咖啡类商品".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先识别咖啡类商品指标口径".to_string(),
                    Vec::new(),
                    1,
                ),
                node(
                    "service",
                    "service",
                    "服务方案",
                    "补充客服承接边界".to_string(),
                    vec!["data"],
                    1,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合岗位边界形成 final_report".to_string(),
                    vec!["data", "service"],
                    1,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        let mut data_report =
            report_from_assignment(&data_assignment, "咖啡类商品需要先补充真实指标。");
        data_report.role_output = Some(RoleWorkOutput {
            summary: "数据边界：当前是类目级问题，未选择预设商品，不能编造咖啡类商品真实指标。"
                .to_string(),
            findings: vec!["未检测到 context.selected_summary，不能进入单品级漏斗。".to_string()],
            recommendations: vec![
                "先输出类目诊断框架，并列出需要补齐的曝光、点击、加购、付款数据。".to_string(),
            ],
            evidence: vec![CATEGORY_LEVEL_BOUNDARY_MARKER.to_string()],
            risks: Vec::new(),
            open_questions: Vec::new(),
        });
        assert!(matches!(
            runtime.apply_report(data_report),
            EvaluationOutcome::Passed(_)
        ));

        let service = runtime.ready_nodes().remove(0);
        let service_assignment = runtime.build_assignment(&service);
        runtime.mark_running(&service_assignment);
        let mut service_report = report_from_assignment(&service_assignment, "咖啡类服务边界。");
        service_report.role_output = Some(RoleWorkOutput {
            summary: "服务边界：未手动选择预设商品时，只能给咖啡类咨询承接方向。".to_string(),
            findings: Vec::new(),
            recommendations: vec!["先准备咖啡风味、物流、售后和优惠解释话术。".to_string()],
            evidence: vec![
                CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
                "worker.assignment.input.data".to_string(),
            ],
            risks: Vec::new(),
            open_questions: Vec::new(),
        });
        assert!(matches!(
            runtime.apply_report(service_report),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合边界。");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec![
                "worker.assignment.input.data".to_string(),
                "worker.assignment.input.service".to_string(),
                CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
            ];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();
        assert!(output.contains("团队报告结构"));
        assert!(output.contains("商品链路"));
        assert!(output.contains("会话上下文"));
        assert!(output.contains("过程文件"));
        assert!(output.contains("final_report"));
        assert!(!output.contains("data role should provide at least two findings"));
        assert!(!output.contains("service role output lacks service-specific content"));
        assert!(!output.contains("未完成或需复核"));
    }

    #[test]
    fn synthesis_explains_team_handoff_for_multiple_roles() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-team-answer".to_string()),
            root_task: "请分析直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "先给出运营判断".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "基于运营输入补充指标判断".to_string(),
                    vec!["ops"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&ops_assignment, "运营方案完整")),
            EvaluationOutcome::Passed(_)
        ));

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        let mut data_report = report_from_assignment(&data_assignment, "数据诊断完成");
        if let Some(output) = data_report.role_output.as_mut() {
            output.evidence = vec!["worker.assignment.input.ops".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(data_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();

        assert!(!output.contains("2 个协作岗位"));
        assert!(output.contains("1 个协作岗位"));
        assert!(output.contains("协作岗位"));
        assert!(output.contains("分别从指标口径补充判断"));
        assert!(!output.contains("分别从指标、服务、内容、技术或预算边界补充判断"));
        assert!(output.contains("先读取 运营策略师 的结论、证据和风险边界"));
        assert!(!output.contains("接收 运营方案 的结果继续处理"));
        assert!(output.contains("数据诊断报告"));
        assert!(output.contains("## 团队执行稿"));
        assert!(output.contains(
            "判断直播间转化是否健康时，要把曝光、点击、加购、付款、退款、\
             咨询承接和履约风险连成一条漏斗看"
        ));
        assert!(output.contains("不会把任何历史商品、测试商品或默认商品注入为本轮商品"));
        assert!(output.contains("交付物"));
    }

    #[test]
    fn synthesis_filters_heading_only_fragments_from_visible_answer() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-heading-only-fragments".to_string()),
            root_task: "请用多岗协作分析咖啡类商品直播内容，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "给出直播内容方向".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合创意为执行方案".to_string(),
                    vec!["creative"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let creative = runtime.ready_nodes().remove(0);
        let creative_assignment = runtime.build_assignment(&creative);
        runtime.mark_running(&creative_assignment);
        let mut creative_report = report_from_assignment(&creative_assignment, "创意方向完成");
        if let Some(output) = creative_report.role_output.as_mut() {
            output.recommendations = vec![
                "标题方向：".to_string(),
                "脚本方向：".to_string(),
                "围绕低负担、早八通勤和冷萃场景写 3 条标题。".to_string(),
            ];
            output.evidence = vec![CATEGORY_LEVEL_BOUNDARY_MARKER.to_string()];
        }
        assert!(matches!(
            runtime.apply_report(creative_report),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合完成");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec!["worker.assignment.input.creative".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();

        assert!(!output.contains("标题方向："));
        assert!(!output.contains("脚本方向："));
        assert!(output.contains("围绕低负担、早八通勤和冷萃场景写 3 条标题"));
    }

    #[test]
    fn synthesis_filters_unscoped_role_claims_from_visible_answer() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-unscoped-role-claim".to_string()),
            root_task: "请用运营、数据、内容三个岗位分析咖啡类直播转化，不引用任何预设商品。"
                .to_string(),
            nodes: vec![
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合运营方案".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "补充指标口径".to_string(),
                    vec!["ops"],
                    2,
                ),
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "补充素材方向".to_string(),
                    vec!["ops"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        for node_id in ["ops", "data", "creative"] {
            let ready = runtime
                .ready_nodes()
                .into_iter()
                .find(|node| node.id.0 == node_id)
                .expect("node should become ready");
            let assignment = runtime.build_assignment(&ready);
            runtime.mark_running(&assignment);
            let mut report = report_from_assignment(&assignment, "岗位产物");
            if let Some(output) = report.role_output.as_mut() {
                if node_id == "ops" {
                    output.recommendations = vec![
                        "与客服/技术团队确认支付、物流、售后流程，确保下单到付款环节顺畅。"
                            .to_string(),
                        "先补齐直播漏斗口径，再把内容测试和复盘节奏排到首周。".to_string(),
                    ];
                }
                if node_id != "ops" {
                    output.evidence = vec!["worker.assignment.input.ops".to_string()];
                }
            }
            assert!(matches!(
                runtime.apply_report(report),
                EvaluationOutcome::Passed(_)
            ));
        }

        let output = runtime.synthesize();

        assert!(!output.contains("客服/技术团队"));
        assert!(!output.contains("支付、物流、售后流程"));
        assert!(output.contains("先补齐直播漏斗口径"));
    }

    #[test]
    fn synthesis_quick_mode_returns_compact_team_answer() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-quick-mode".to_string()),
            root_task: "请快速判断咖啡类商品直播转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "快速收敛运营判断".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "补充漏斗指标判断".to_string(),
                    vec!["ops"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph).with_response_mode(Some("quick"));
        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&ops_assignment, "运营判断完整")),
            EvaluationOutcome::Passed(_)
        ));

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        let mut data_report = report_from_assignment(&data_assignment, "数据诊断完成");
        if let Some(output) = data_report.role_output.as_mut() {
            output.evidence = vec!["worker.assignment.input.ops".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(data_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();
        assert!(output.contains("# 多智能体协作结论"));
        assert!(output.contains("快速简答已完成"));
        assert!(output.contains("## 总回答"));
        assert!(output.contains("## 正文分析"));
        assert!(output.contains("| 类型 | 岗位 | 核心判断 | 下一步 |"));
        assert!(output.contains("主岗"));
        assert!(output.contains("协作岗"));
        assert!(output.contains("## 主岗综合方案"));
        assert!(output.contains("## 总结与交付物"));
        assert!(!output.contains("## 下一步"));
        assert!(!output.contains("## 岗位协作过程"));
        assert!(!output.contains("## 协同关系"));
        assert!(!output.contains("## 丰富执行方案"));
    }

    #[test]
    fn synthesis_execution_mode_emits_action_table_and_role_ownership() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-execution-mode".to_string()),
            root_task: "请给出咖啡类商品直播转化提升的丰富执行方案，不引用任何预设商品。"
                .to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "给出漏斗口径和数据缺口".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "基于数据诊断给出内容方向".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合数据和创意形成执行方案".to_string(),
                    vec!["creative"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph).with_response_mode(Some("execution"));
        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&data_assignment, "数据诊断完整")),
            EvaluationOutcome::Passed(_)
        ));

        let creative = runtime.ready_nodes().remove(0);
        let creative_assignment = runtime.build_assignment(&creative);
        runtime.mark_running(&creative_assignment);
        let mut creative_report = report_from_assignment(&creative_assignment, "创意方案完整");
        if let Some(output) = creative_report.role_output.as_mut() {
            output.evidence = vec!["worker.assignment.input.data".to_string()];
            output.findings = vec!["咖啡卖点需要拆成场景、口味和复购理由。".to_string()];
            output.recommendations = vec!["产出三套短视频钩子和直播间开场脚本。".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(creative_report),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营执行方案完整");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec![
                "worker.assignment.input.creative".to_string(),
                "worker.assignment.input.data".to_string(),
            ];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();
        assert!(output.contains("丰富执行模式已完成"));
        assert!(output.contains("## 丰富执行方案"));
        assert!(output.contains("| 优先级 | 负责人 | 动作 | 验收方式 | 风险边界 |"));
        assert!(output.contains("主回答岗位"));
        assert!(output.contains("协作岗位"));
        assert!(output.contains("丰富执行要求"));
        assert!(!output.contains("## 教学讲解"));
    }

    #[test]
    fn synthesis_learning_mode_keeps_student_friendly_explanation_by_default() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-learning-mode".to_string()),
            root_task: "请用教学方式解释咖啡类商品直播转化怎么分析，不引用任何预设商品。"
                .to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "解释漏斗指标".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合为教学讲解".to_string(),
                    vec!["data"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&data_assignment, "数据教学诊断完整")),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营教学方案完整");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.evidence = vec!["worker.assignment.input.data".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();
        assert!(output.contains("教学模式已完成"));
        assert!(output.contains("## 正文分析"));
        assert!(output.contains("怎么读这份回答"));
        assert!(output.contains("商品边界"));
        assert!(output.contains("## 团队执行稿"));
        assert!(!output.contains("## 丰富执行方案"));
    }

    #[test]
    fn synthesis_learning_depth_changes_explanation_detail_without_changing_layout() {
        let make_runtime = |depth: Option<&str>| {
            let graph = TaskGraph {
                graph_id: TaskGraphId(format!(
                    "graph-learning-depth-{}",
                    depth.unwrap_or("default")
                )),
                root_task: "请用教学方式解释咖啡类商品直播转化怎么分析，不引用任何预设商品。"
                    .to_string(),
                nodes: vec![
                    node(
                        "data",
                        "data",
                        "数据诊断",
                        "解释漏斗指标".to_string(),
                        Vec::new(),
                        2,
                    ),
                    node(
                        "ops",
                        "ops",
                        "运营方案",
                        "整合为教学讲解".to_string(),
                        vec!["data"],
                        2,
                    ),
                ],
            };
            let mut runtime = TaskGraphRuntime::new(graph).with_learning_depth(depth);
            while let Some(node) = runtime.ready_nodes().into_iter().next() {
                let assignment = runtime.build_assignment(&node);
                runtime.mark_running(&assignment);
                let mut report = report_from_assignment(&assignment, "教学深度测试产物");
                if let Some(output) = report.role_output.as_mut() {
                    output
                        .evidence
                        .push("worker.assignment.input.data".to_string());
                }
                assert!(matches!(
                    runtime.apply_report(report),
                    EvaluationOutcome::Passed(_)
                ));
            }
            runtime.synthesize()
        };

        let low_output = make_runtime(Some("low"));
        let high_output = make_runtime(Some("high"));

        for section in [
            "# 多智能体协作结论",
            "## 总回答",
            "## 正文分析",
            "## 虚拟员工分工",
            "## 主岗综合方案",
            "## 总结与交付物",
        ] {
            assert!(low_output.contains(section));
            assert!(high_output.contains(section));
        }
        assert!(low_output.contains("学习重点"));
        assert!(!low_output.contains("判断逻辑"));
        assert!(high_output.contains("判断逻辑"));
        assert!(high_output.contains("按预算拆执行节奏"));
        assert!(high_output.len() > low_output.len());
    }

    #[test]
    fn synthesis_filters_unsupported_benchmark_claims_but_keeps_benchmark_gaps() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-benchmark-claim".to_string()),
            root_task: "请判断直播间曝光1000、点击80、加购20、下单8、付款7是否健康，\
                        不引用任何预设商品。"
                .to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "判断直播间漏斗".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "数据诊断完成");
        if let Some(output) = report.role_output.as_mut() {
            output.summary = "数据岗只基于用户给出的本场漏斗做初判。".to_string();
            output.findings = vec![
                "曝光到点击率约8%，高于多数直播场景平均水平。".to_string(),
                "点击到加购率（CVR）约25%，属于中等水平。".to_string(),
                "点击到加购率约25.0%，符合多数电商直播的加购转化率。".to_string(),
                "曝光到点击率约 8.0%，符合一般直播封面/标题吸引力预期。".to_string(),
                "下单到付款率约87.5%，支付环节存在一定阻力。".to_string(),
                "点击到加购约25%，需要收集行业基准后再判断是否健康。".to_string(),
            ];
            output.recommendations = vec!["收集同类直播行业基准，再做复核。".to_string()];
            output.risks = vec!["缺乏行业基准导致健康评估不够客观。".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();

        assert!(!output.contains("高于多数直播场景平均水平"));
        assert!(!output.contains("属于中等水平"));
        assert!(!output.contains("符合多数电商直播"));
        assert!(!output.contains("符合一般直播"));
        assert!(!output.contains("存在一定阻力"));
        assert!(output.contains("需要收集行业基准后再判断是否健康"));
        assert!(output.contains("缺乏行业基准导致健康评估不够客观"));
    }

    #[test]
    fn synthesis_filters_unsupported_external_tool_claims() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-external-tool-claim".to_string()),
            root_task: "请判断直播间曝光1000、点击80、加购20、下单8、付款7是否健康，\
                        不引用任何预设商品。"
                .to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "判断直播间漏斗".to_string(),
                Vec::new(),
                2,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "数据诊断完成");
        if let Some(output) = report.role_output.as_mut() {
            output.summary = "数据岗只基于用户给出的本场漏斗做初判。".to_string();
            output.findings = vec![
                "曝光到点击率约8%，只能说明本场漏斗表现。".to_string(),
                "点击到加购约25%，需要补齐行业基准后再判断是否健康。".to_string(),
            ];
            output.recommendations = vec![
                "完成 data_funnel_analysis 后，获取同类直播或历史数据做基准对比。".to_string(),
                "使用 search_market_info skill 获取行业基准数据。".to_string(),
                "通过 `search_industry_benchmarks` 获取同类直播转化率区间，作为对照基准。"
                    .to_string(),
                "先按用户给出的漏斗数字复核曝光、点击、加购、下单和付款口径。".to_string(),
            ];
            output.evidence = vec![CATEGORY_LEVEL_BOUNDARY_MARKER.to_string()];
            output.risks = vec![
                "未接入真实外部行业基准时，不输出行业区间、行业均值或行业百分位。".to_string(),
            ];
        }
        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();

        assert!(!output.contains("data_funnel_analysis"));
        assert!(!output.contains("search_market_info"));
        assert!(!output.contains("search_industry_benchmarks"));
        assert!(!output.contains("获取同类直播"));
        assert!(output.contains("先按用户给出的漏斗数字复核"));
        assert!(output.contains("未接入真实外部行业基准"));
    }

    #[test]
    fn synthesis_uses_terminal_ops_node_as_primary_answer() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-terminal-primary".to_string()),
            root_task: "请分析直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先判断指标口径".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "creative",
                    "creative",
                    "创意产出",
                    "基于数据诊断补充创意方向".to_string(),
                    vec!["data"],
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合数据和创意，形成最终运营判断".to_string(),
                    vec!["creative"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&data_assignment, "数据指标口径完整")),
            EvaluationOutcome::Passed(_)
        ));

        let creative = runtime.ready_nodes().remove(0);
        let creative_assignment = runtime.build_assignment(&creative);
        runtime.mark_running(&creative_assignment);
        let mut creative_report = report_from_assignment(&creative_assignment, "创意承接完整");
        if let Some(output) = creative_report.role_output.as_mut() {
            output.evidence = vec!["worker.assignment.input.data".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(creative_report),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合结论完整");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.summary = "运营主岗结论：健康直播间要同时看点击率、加购率、付款率和退款/\
                              咨询风险，而不是只看单一转化率。"
                .to_string();
            output.evidence = vec!["worker.assignment.input.creative".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.synthesize();

        assert!(output.contains("运营主岗结论"));
        assert!(output.contains("主回答岗位**：运营方案（运营策略师）"));
        assert!(output.contains("本轮用户明确要求不引用预设商品"));
    }

    #[test]
    fn process_summary_exposes_answer_scoped_team_process() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-process-summary".to_string()),
            root_task: "请分析直播间转化是否健康，不引用任何预设商品。".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "先判断指标口径".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "ops",
                    "ops",
                    "运营方案",
                    "整合数据诊断，形成最终运营判断".to_string(),
                    vec!["data"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);

        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&data_assignment, "数据指标口径完整")),
            EvaluationOutcome::Passed(_)
        ));

        let ops = runtime.ready_nodes().remove(0);
        let ops_assignment = runtime.build_assignment(&ops);
        runtime.mark_running(&ops_assignment);
        let mut ops_report = report_from_assignment(&ops_assignment, "运营整合结论完整");
        if let Some(output) = ops_report.role_output.as_mut() {
            output.summary =
                "运营主岗结论：先看点击、加购、付款和售后风险，再决定优化顺序。".to_string();
            output.recommendations =
                vec!["P1 先补齐真实漏斗指标，再排查首屏和客服承接。".to_string()];
            output.evidence = vec!["worker.assignment.input.data".to_string()];
        }
        assert!(matches!(
            runtime.apply_report(ops_report),
            EvaluationOutcome::Passed(_)
        ));

        let process = runtime.process_summary();

        assert_eq!(process.graph_id.0, "graph-process-summary");
        assert_eq!(
            process.primary.as_ref().map(|role| role.role.as_str()),
            Some("ops")
        );
        assert!(process.panel.default_collapsed);
        assert_eq!(process.panel.anchor, "answer_scoped_process");
        assert_eq!(process.roles.len(), 2);
        assert!(process.roles.iter().any(|role| {
            role.role == "data" && role.status == "passed" && role.role_label == "数据分析师"
        }));
        assert!(process.handoffs.iter().any(|handoff| {
            handoff.from_role == "data"
                && handoff.to_role == "ops"
                && handoff.summary.contains("运营策略师")
                && handoff.summary.contains("数据分析师")
        }));
        assert!(
            process
                .next_actions
                .iter()
                .any(|action| action.contains("补齐真实漏斗指标"))
        );
        assert!(
            process
                .deliverables
                .iter()
                .any(|item| item.contains("final_report"))
        );
        assert!(process.incomplete.is_empty());
    }

    #[test]
    fn synthesis_localizes_runtime_gap_reasons() {
        let mut graph = TaskGraph {
            graph_id: TaskGraphId("graph-localized-gap".to_string()),
            root_task: "请分析直播间转化是否健康".to_string(),
            nodes: vec![node(
                "data",
                "data",
                "数据诊断",
                "分析直播间转化".to_string(),
                Vec::new(),
                1,
            )],
        };
        graph.nodes[0].max_attempts = 1;
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&data);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(&assignment, "数据产物不足");
        report.role_output = Some(RoleWorkOutput {
            summary: "数据结论不足".to_string(),
            findings: vec!["只看到转化下降".to_string()],
            recommendations: Vec::new(),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
        });

        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Failed(_)
        ));

        let output = runtime.synthesize();

        assert!(output.contains("数据岗位需要补齐至少两个可复核发现"));
        assert!(!output.contains("data role should provide at least two findings"));
    }

    #[test]
    fn direct_chat_output_returns_summary_without_graph_synthesis() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-chat-direct".to_string()),
            root_task: "hi".to_string(),
            nodes: vec![node(
                "chat",
                "chat",
                "快速对话",
                "自然回复 hi".to_string(),
                Vec::new(),
                1,
            )],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let node = runtime.ready_nodes().remove(0);
        let assignment = runtime.build_assignment(&node);
        runtime.mark_running(&assignment);
        let mut report = report_from_assignment(
            &assignment,
            "你好！有什么我可以帮你的吗？\n建议: 直接告诉我你的目标或问题。",
        );
        report.role_output = Some(RoleWorkOutput {
            summary: "你好！有什么我可以帮你的吗？".to_string(),
            findings: vec!["用户是简单问候".to_string()],
            recommendations: vec!["等待用户提出具体问题".to_string()],
            evidence: vec!["原始任务".to_string()],
            risks: Vec::new(),
            open_questions: Vec::new(),
        });

        assert!(matches!(
            runtime.apply_report(report),
            EvaluationOutcome::Passed(_)
        ));

        let output = runtime.direct_chat_output().expect("direct chat output");

        assert_eq!(output, "你好！有什么我可以帮你的吗？");
        assert!(!output.contains("建议: 直接告诉我你的目标或问题。"));
        assert!(!output.contains("多智能体任务图已完成"));
    }

    #[test]
    fn evaluator_requires_dependent_role_to_cite_upstream_report() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-13".to_string()),
            root_task: "基于数据诊断做详情页设计".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "分析转化率下降".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "design",
                    "design",
                    "体验设计",
                    "基于数据诊断优化详情页".to_string(),
                    vec!["data"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&data_assignment, "数据诊断完成")),
            EvaluationOutcome::Passed(_)
        ));

        let design = runtime.ready_nodes().remove(0);
        let design_assignment = runtime.build_assignment(&design);
        runtime.mark_running(&design_assignment);
        let mut report = report_from_assignment(&design_assignment, "设计方案完整");
        report.role_output = Some(RoleWorkOutput {
            summary: "详情页设计应围绕首屏信任和转化路径收敛。".to_string(),
            findings: vec!["页面首屏承接弱".to_string()],
            recommendations: vec!["重排主图、卖点和行动按钮的页面层级。".to_string()],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["缺少真实热区和点击数据，方案需小流量验证。".to_string()],
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Rework { .. }));
    }

    #[test]
    fn evaluator_accepts_dependent_role_with_upstream_citation() {
        let graph = TaskGraph {
            graph_id: TaskGraphId("graph-task-14".to_string()),
            root_task: "基于数据诊断做详情页设计".to_string(),
            nodes: vec![
                node(
                    "data",
                    "data",
                    "数据诊断",
                    "分析转化率下降".to_string(),
                    Vec::new(),
                    2,
                ),
                node(
                    "design",
                    "design",
                    "体验设计",
                    "基于数据诊断优化详情页".to_string(),
                    vec!["data"],
                    2,
                ),
            ],
        };
        let mut runtime = TaskGraphRuntime::new(graph);
        let data = runtime.ready_nodes().remove(0);
        let data_assignment = runtime.build_assignment(&data);
        runtime.mark_running(&data_assignment);
        assert!(matches!(
            runtime.apply_report(report_from_assignment(&data_assignment, "数据诊断完成")),
            EvaluationOutcome::Passed(_)
        ));

        let design = runtime.ready_nodes().remove(0);
        let design_assignment = runtime.build_assignment(&design);
        runtime.mark_running(&design_assignment);
        let mut report = report_from_assignment(&design_assignment, "设计方案完整");
        report.role_output = Some(RoleWorkOutput {
            summary: "详情页设计应围绕首屏信任和转化路径收敛。".to_string(),
            findings: vec!["页面首屏承接弱".to_string()],
            recommendations: vec!["重排主图、卖点和行动按钮的页面层级。".to_string()],
            evidence: vec!["worker.assignment.input.data".to_string()],
            risks: vec!["缺少真实热区和点击数据，方案需小流量验证。".to_string()],
            open_questions: Vec::new(),
        });

        let outcome = runtime.apply_report(report);

        assert!(matches!(outcome, EvaluationOutcome::Passed(_)));
    }

    #[test]
    fn evaluator_covers_role_contract_scenarios_from_old_python_effects() {
        let cases = vec![
            (
                "data",
                RoleWorkOutput {
                    summary: "数据诊断先锁定漏斗、口径和归因边界。".to_string(),
                    findings: vec![
                        "点击到成交漏斗存在断点".to_string(),
                        "活动口径需要统一".to_string(),
                    ],
                    recommendations: vec!["按渠道拆分转化漏斗并标记异常段。".to_string()],
                    evidence: vec!["原始任务".to_string()],
                    risks: vec!["样本量不足会导致归因误判。".to_string()],
                    open_questions: Vec::new(),
                },
            ),
            (
                "service",
                RoleWorkOutput {
                    summary: "客服 SOP 应先稳定投诉响应和升级边界。".to_string(),
                    findings: vec!["退款争议需要统一口径".to_string()],
                    recommendations: vec!["建立售后首响、补偿、升级三段 SOP。".to_string()],
                    evidence: vec!["原始任务".to_string()],
                    risks: vec!["超政策承诺会扩大财务和履约风险。".to_string()],
                    open_questions: Vec::new(),
                },
            ),
            (
                "accounting",
                RoleWorkOutput {
                    summary: "预算风险控制要先看成本、利润和 ROI 止损线。".to_string(),
                    findings: vec!["预算消耗需要绑定毛利空间".to_string()],
                    recommendations: vec!["按 ROI 阈值设置日预算上限。".to_string()],
                    evidence: vec!["原始任务".to_string()],
                    risks: vec!["现金流不足时不能扩大投放。".to_string()],
                    open_questions: Vec::new(),
                },
            ),
            (
                "engineering",
                RoleWorkOutput {
                    summary: "工程发布计划应围绕稳定性、回滚和接口兼容。".to_string(),
                    findings: vec!["系统发布需要保护核心接口".to_string()],
                    recommendations: vec!["先灰度发布并准备回滚检查单。".to_string()],
                    evidence: vec!["原始任务".to_string()],
                    risks: vec!["缺少监控会放大发布故障。".to_string()],
                    open_questions: Vec::new(),
                },
            ),
            (
                "web",
                RoleWorkOutput {
                    summary: "SEO 页面应围绕关键词集群和收录路径优化。".to_string(),
                    findings: vec!["页面标题和关键词意图需要对齐".to_string()],
                    recommendations: vec!["建立核心词、长尾词和内容页面的 SEO 集群。".to_string()],
                    evidence: vec!["原始任务".to_string()],
                    risks: vec!["未验证搜索量时不能承诺自然流量结果。".to_string()],
                    open_questions: Vec::new(),
                },
            ),
            (
                "creative",
                RoleWorkOutput {
                    summary: "短视频创意应先产出可测试的标题和脚本钩子。".to_string(),
                    findings: vec!["内容需要突出首三秒冲突".to_string()],
                    recommendations: vec!["准备三组标题和短视频脚本开头做 A/B 测试。".to_string()],
                    evidence: vec!["原始任务".to_string()],
                    risks: vec!["不能编造产品功效或用户反馈。".to_string()],
                    open_questions: Vec::new(),
                },
            ),
        ];

        for (role, role_output) in cases {
            let report = WorkerReport {
                graph_id: TaskGraphId(format!("graph-{role}")),
                node_id: TaskNodeId(role.to_string()),
                task_id: TaskId(format!("task-{role}")),
                worker_id: WorkerId(format!("worker.{role}")),
                role: role.to_string(),
                content: role_output.summary.clone(),
                role_output: Some(role_output),
                evidence: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
                status: WorkerReportStatus::Completed,
            };

            let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

            assert!(
                evaluation.passed,
                "{role} should pass, got {:?}",
                evaluation.reasons
            );
        }
    }

    #[test]
    fn evaluator_allows_risk_marked_missing_precision_without_refusal() {
        let role_output = RoleWorkOutput {
            summary: "预算风险需要先用 CPA、ROI 和毛利边界判断。".to_string(),
            findings: vec!["CPC 上升叠加 CVR 下降会推高 CPA。".to_string()],
            recommendations: vec!["先按客单价补齐盈亏平衡测算。".to_string()],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["缺乏客单价数据，无法完成精确盈亏平衡点量化计算。".to_string()],
            open_questions: Vec::new(),
        };
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-accounting-precision".to_string()),
            node_id: TaskNodeId("accounting".to_string()),
            task_id: TaskId("task-accounting-precision".to_string()),
            worker_id: WorkerId("worker.accounting".to_string()),
            role: "accounting".to_string(),
            content: role_output.summary.clone(),
            role_output: Some(role_output),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

        assert!(evaluation.passed, "{:?}", evaluation.reasons);
    }

    #[test]
    fn evaluator_allows_web_seo_terms_without_claiming_external_search() {
        let role_output = RoleWorkOutput {
            summary: "SEO 页面应围绕关键词集群和搜索结果展示结构优化。".to_string(),
            findings: vec!["自然流量增长依赖页面主题集群和标题匹配。".to_string()],
            recommendations: vec![
                "建立核心词、长尾词和 FAQ 页面的关键词集群。".to_string(),
                "优化标题、描述和结构化数据以改善搜索结果展示。".to_string(),
            ],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["未验证搜索量和收录状态时不能承诺排名结果。".to_string()],
            open_questions: Vec::new(),
        };
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-web-seo".to_string()),
            node_id: TaskNodeId("web".to_string()),
            task_id: TaskId("task-web-seo".to_string()),
            worker_id: WorkerId("worker.web".to_string()),
            role: "web".to_string(),
            content: role_output.summary.clone(),
            role_output: Some(role_output),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

        assert!(evaluation.passed, "{:?}", evaluation.reasons);
    }

    #[test]
    fn chat_role_allows_direct_answer_without_findings() {
        let role_output = RoleWorkOutput {
            summary: "可以回答。本轮建议由运营、数据和内容创意三个虚拟员工参与。".to_string(),
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: vec!["还需要补充首周预算、目标平台和库存边界。".to_string()],
        };
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-chat-direct".to_string()),
            node_id: TaskNodeId("chat".to_string()),
            task_id: TaskId("task-chat-direct".to_string()),
            worker_id: WorkerId("worker.chat".to_string()),
            role: "chat".to_string(),
            content: role_output.summary.clone(),
            role_output: Some(role_output),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

        assert!(evaluation.passed, "{:?}", evaluation.reasons);
    }

    #[test]
    fn chat_role_allows_concise_direct_answer() {
        let role_output = RoleWorkOutput {
            summary: "42".to_string(),
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
        };
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-chat-short-answer".to_string()),
            node_id: TaskNodeId("chat".to_string()),
            task_id: TaskId("task-chat-short-answer".to_string()),
            worker_id: WorkerId("worker.chat".to_string()),
            role: "chat".to_string(),
            content: role_output.summary.clone(),
            role_output: Some(role_output),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

        assert!(evaluation.passed, "{:?}", evaluation.reasons);
        assert!(
            !evaluation
                .reasons
                .contains(&"quality completeness: summary is too thin".to_string())
        );
    }

    #[test]
    fn evaluator_rejects_claimed_external_search_results() {
        let role_output = RoleWorkOutput {
            summary: "已检索到实时搜索结果显示该关键词排名第一。".to_string(),
            findings: vec!["搜索结果显示当前页面排名第一。".to_string()],
            recommendations: vec!["继续扩大该关键词页面。".to_string()],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["需要验证数据来源。".to_string()],
            open_questions: Vec::new(),
        };
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-web-fabrication".to_string()),
            node_id: TaskNodeId("web".to_string()),
            task_id: TaskId("task-web-fabrication".to_string()),
            worker_id: WorkerId("worker.web".to_string()),
            role: "web".to_string(),
            content: role_output.summary.clone(),
            role_output: Some(role_output),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

        assert!(!evaluation.passed);
        assert!(
            evaluation
                .reasons
                .contains(&"quality fabrication: claims unsupported external evidence".to_string())
        );
    }

    #[test]
    fn evaluator_allows_realtime_data_risk_boundary_without_claiming_evidence() {
        let role_output = RoleWorkOutput {
            summary: "直播间转化健康需要用漏斗转化率和基准对比。".to_string(),
            findings: vec!["需要记录观看、加购和付款的链路指标。".to_string()],
            recommendations: vec![
                "先补齐观看、加购、付款数据，再与历史或行业基准对比。".to_string(),
                "设置异常波动提醒。".to_string(),
                "按日复盘转化断点。".to_string(),
            ],
            evidence: vec!["原始任务".to_string()],
            risks: vec!["缺乏实时数据源会导致监控不完整。".to_string()],
            open_questions: Vec::new(),
        };
        let report = WorkerReport {
            graph_id: TaskGraphId("graph-ops-risk-boundary".to_string()),
            node_id: TaskNodeId("ops".to_string()),
            task_id: TaskId("task-ops-risk-boundary".to_string()),
            worker_id: WorkerId("worker.ops".to_string()),
            role: "ops".to_string(),
            content: role_output.summary.clone(),
            role_output: Some(role_output),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

        let evaluation = evaluate_report(&report, None, &HashMap::new(), "");

        assert!(evaluation.passed);
        assert!(
            !evaluation
                .reasons
                .contains(&"quality fabrication: claims unsupported external evidence".to_string())
        );
    }

    #[test]
    fn planner_can_make_ops_depend_on_design_and_accounting() {
        let rules = parse_task_graph_rules(
            r#"{
              "rules": [
                {
                  "id": "design_budget_ops",
                  "match_all": ["详情页", "预算", "运营方案"],
                  "nodes": [
                    {
                      "id": "data",
                      "role": "data",
                      "title": "数据诊断",
                      "objective": "分析数据"
                    },
                    {
                      "id": "design",
                      "role": "design",
                      "title": "体验设计",
                      "depends_on": ["data"],
                      "objective": "设计方案"
                    },
                    {
                      "id": "accounting",
                      "role": "accounting",
                      "title": "预算评估",
                      "depends_on": ["data"],
                      "objective": "预算评估"
                    },
                    {
                      "id": "ops",
                      "role": "ops",
                      "title": "运营方案",
                      "depends_on": ["design", "accounting"],
                      "objective": "整合运营方案"
                    }
                  ]
                }
              ]
            }"#,
        )
        .expect("valid configured rules");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-9".to_string()),
            "基于数据诊断做详情页设计和预算风险评估，并给运营方案",
            &rules,
        )
        .expect("task should match configured graph");

        let design = graph
            .nodes
            .iter()
            .find(|node| node.role == "design")
            .unwrap();
        let accounting = graph
            .nodes
            .iter()
            .find(|node| node.role == "accounting")
            .unwrap();
        let ops = graph.nodes.iter().find(|node| node.role == "ops").unwrap();

        assert!(ops.input_refs.contains(&design.id));
        assert!(ops.input_refs.contains(&accounting.id));
    }

    fn report_from_assignment(assignment: &WorkerAssignment, content: &str) -> WorkerReport {
        WorkerReport {
            graph_id: assignment.graph_id.clone(),
            node_id: assignment.node_id.clone(),
            task_id: assignment.task_id.clone(),
            worker_id: assignment.worker_id.clone(),
            role: assignment.role.clone(),
            content: content.to_string(),
            role_output: valid_role_output_for_role(&assignment.role, content),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        }
    }

    fn valid_role_output_for_role(role: &str, content: &str) -> Option<RoleWorkOutput> {
        if content.trim().is_empty() {
            return None;
        }

        let summary = format!("测试角色产物：{}，已满足当前节点目标。", content.trim());
        let output = match role {
            "data" => RoleWorkOutput {
                summary: summary.clone(),
                findings: vec![
                    "漏斗指标需要拆解".to_string(),
                    "归因口径需要统一".to_string(),
                ],
                recommendations: vec!["按渠道和页面段落复核转化断点。".to_string()],
                evidence: vec!["原始任务".to_string()],
                risks: vec!["数据口径不一致会导致误判。".to_string()],
                open_questions: Vec::new(),
            },
            "design" => RoleWorkOutput {
                summary: summary.clone(),
                findings: vec!["页面首屏承接需要优化".to_string()],
                recommendations: vec!["调整详情页视觉层级和主图卖点。".to_string()],
                evidence: vec!["worker.assignment.input.data".to_string()],
                risks: vec!["缺少真实点击数据时需要小流量验证。".to_string()],
                open_questions: Vec::new(),
            },
            "accounting" => RoleWorkOutput {
                summary: summary.clone(),
                findings: vec!["预算和成本需要绑定 ROI 判断。".to_string()],
                recommendations: vec!["设置预算止损线。".to_string()],
                evidence: vec!["worker.assignment.input.data".to_string()],
                risks: vec!["现金流不足会放大投放风险。".to_string()],
                open_questions: Vec::new(),
            },
            "ops" => RoleWorkOutput {
                summary: summary.clone(),
                findings: vec!["运营目标需要转化链路承接。".to_string()],
                recommendations: vec![
                    "P1 优化首屏卖点。".to_string(),
                    "P2 调整投放人群。".to_string(),
                    "P3 每日复盘转化。".to_string(),
                ],
                evidence: vec![
                    "worker.assignment.input.design".to_string(),
                    "worker.assignment.input.accounting".to_string(),
                    "worker.assignment.input.data".to_string(),
                ],
                risks: vec!["预算消耗需设置止损线。".to_string()],
                open_questions: Vec::new(),
            },
            "chat" => RoleWorkOutput {
                summary: summary.clone(),
                findings: vec!["已基于当前任务直接回答。".to_string()],
                recommendations: Vec::new(),
                evidence: vec!["原始任务".to_string()],
                risks: Vec::new(),
                open_questions: Vec::new(),
            },
            _ => RoleWorkOutput {
                summary,
                findings: vec!["已形成角色判断。".to_string()],
                recommendations: vec!["按角色边界推进下一步。".to_string()],
                evidence: vec!["原始任务".to_string()],
                risks: vec!["需要持续确认边界。".to_string()],
                open_questions: Vec::new(),
            },
        };

        Some(output)
    }
}
