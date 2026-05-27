use std::collections::{HashMap, HashSet};

use super::{chat, planner};
use crate::{
    agent::commander::{
        no_user_facing_content_message, sanitize_final_user_facing_text, user_facing_report_text,
    },
    agent_session::{RoleMemorySummary, TaskGraphMemorySummary, WorkerDetail},
    core::protocol::{
        Evaluation, RoleWorkOutput, TaskGraph, TaskGraphHandoff, TaskGraphProcessGap,
        TaskGraphProcessPanel, TaskGraphProcessRole, TaskGraphProcessSummary, TaskId, TaskNode,
        TaskNodeId, WorkerAssignment, WorkerReport, WorkerReportStatus,
    },
    roles::RoleRoute,
};

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
            no_user_facing_content_message()
        } else {
            sanitize_final_user_facing_text(&sections.join("\n\n"))
                .unwrap_or_else(no_user_facing_content_message)
        }
    }

    fn learning_synthesis(&self) -> String {
        self.final_node_output()
            .or_else(|| {
                self.graph
                    .nodes
                    .iter()
                    .rev()
                    .filter_map(|node| self.reports.get(&node.id))
                    .find_map(user_facing_report_text)
            })
            .unwrap_or_else(no_user_facing_content_message)
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

        TaskGraphProcessSummary {
            graph_id: self.graph.graph_id.clone(),
            root_task: self.graph.root_task.clone(),
            primary,
            roles,
            handoffs,
            next_actions: Vec::new(),
            check_items: Vec::new(),
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

fn evaluate_report(
    report: &WorkerReport,
    node: Option<&TaskNode>,
    upstream_reports: &HashMap<TaskNodeId, WorkerReport>,
) -> Evaluation {
    let mut reasons = Vec::new();
    if report.status == WorkerReportStatus::Failed {
        reasons.push("worker reported failed status".to_string());
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
        role_label: node.role.clone(),
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
                    from_role_label: upstream.role.clone(),
                    to_role_label: node.role.clone(),
                    summary: format!("{} depends on {}", node.id.0, upstream.id.0),
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
            role_label: node.role.clone(),
            title: node.title.clone(),
            state: "failed".to_string(),
            reason: reason.clone(),
        }),
        Some(TaskNodeRuntimeState::Skipped(reason)) => Some(TaskGraphProcessGap {
            node_id: node.id.clone(),
            role: node.role.clone(),
            role_label: node.role.clone(),
            title: node.title.clone(),
            state: "skipped".to_string(),
            reason: reason.clone(),
        }),
        _ => None,
    }
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
    use super::{TaskGraphRuntime, TaskNodeRuntimeState};
    use crate::core::protocol::{
        RoleWorkOutput, TaskGraph, TaskGraphId, TaskId, TaskNode, TaskNodeId, WorkerId,
        WorkerReport, WorkerReportStatus,
    };

    #[test]
    fn task_graph_synthesis_hides_internal_report_shell() {
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-user-facing".to_string()),
            root_task: "帮我做一段咖啡商品讲解。".to_string(),
            nodes: vec![node("creative", "内容创意", "creative")],
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
        });

        let output = runtime.synthesize();

        assert!(output.contains("没有拿到可直接展示的完整内容"));
        assert!(!output.contains("none"));
        assert!(!output.contains("Passed worker reports"));
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
    fn task_graph_synthesis_hides_failed_descendant_reason() {
        let data = node("data", "Data", "data");
        let mut ops = node("ops_final", "Final", "ops");
        ops.input_refs = vec![TaskNodeId("data".to_string())];
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-partial".to_string()),
            root_task: "Build a plan.".to_string(),
            nodes: vec![data.clone(), ops.clone()],
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
    fn quick_task_graph_synthesis_dedupes_repeated_cross_worker_blocks() {
        let ops = node("ops", "Ops", "ops");
        let data = node("data", "Data", "data");
        let creative = node("creative", "Creative", "creative");
        let mut runtime = TaskGraphRuntime::new(TaskGraph {
            graph_id: TaskGraphId("graph-quick".to_string()),
            root_task: "快速判断这个冷萃咖啡怎么卖。".to_string(),
            nodes: vec![ops.clone(), data.clone(), creative.clone()],
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

    fn node(id: &str, title: &str, role: &str) -> TaskNode {
        TaskNode {
            id: TaskNodeId(id.to_string()),
            role: role.to_string(),
            worker_id: WorkerId(format!("worker.{role}")),
            title: title.to_string(),
            objective: "输出用户可直接理解的内容。".to_string(),
            llm_usage: None,
            skill_autosuggest: None,
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
            status: WorkerReportStatus::Completed,
        }
    }
}
