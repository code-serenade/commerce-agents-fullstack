use super::{
    Commander, CommanderPhase, CommanderState, next_message_id, output::build_contribution,
    primary_route_score, primary_worker_id,
};
use crate::{
    agent::commander::no_user_facing_content_message,
    core::protocol::{
        AgentId, Message, MessageContext, Payload, SessionEvent, TaskGraph, TaskGraphAdaptiveHint,
        TaskGraphId, TaskId, TaskNode, TaskNodeId, TaskSpec, WorkerAssignment, WorkerId,
        WorkerReport,
    },
    roles::{RoleCollaborationPlan, RoleContribution},
};

impl Commander {
    pub(super) fn try_fast_route(&mut self, role_route: Option<&crate::roles::RoleRoute>) -> bool {
        let Some(role_route) = role_route else {
            return false;
        };
        if self.should_preserve_task_graph_for_manual_multi_role(role_route) {
            return false;
        }
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
        let dispatch_graph = role_route_dispatch_graph(&task, role_route);
        self.emit_dispatch_planned_for_graph("fast_worker", &dispatch_graph);
        let dispatched = if dispatch_graph.nodes.len() > 1 {
            self.dispatch_collaboration_primary_assignment(&task, &dispatch_graph, &target_worker)
        } else {
            self.dispatch_task_to_worker(task, Some(target_worker))
        };
        if !dispatched {
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

    fn should_preserve_task_graph_for_manual_multi_role(
        &self,
        role_route: &crate::roles::RoleRoute,
    ) -> bool {
        if self
            .current_context
            .metadata
            .get("strategy.collaboration_mode")
            .is_none_or(|value| value != "manual")
        {
            return false;
        }

        std::iter::once(role_route.primary_runtime_role.as_str())
            .chain(role_route.support_runtime_roles.iter().map(String::as_str))
            .filter(|role| *role != "chat")
            .count()
            >= 2
    }

    pub(super) fn prepare_collaboration(
        &mut self,
        role_route: Option<&crate::roles::RoleRoute>,
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
        self.collaboration_reports.clear();
    }

    pub(super) async fn handle_collaboration_worker_finished(
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

    fn dispatch_collaboration_primary_assignment(
        &self,
        task: &TaskSpec,
        graph: &TaskGraph,
        target_worker: &WorkerId,
    ) -> bool {
        let Some(node) = graph.nodes.first() else {
            return false;
        };
        let assignment = WorkerAssignment {
            graph_id: graph.graph_id.clone(),
            node_id: node.id.clone(),
            task_id: task.id.clone(),
            worker_id: target_worker.clone(),
            role: node.role.clone(),
            attempt: 1,
            objective: format!(
                "{}\n\n本轮是轻量主岗协作，直接输出可供支持岗位复核的初稿，\
                 不请求外部 Skill 或工具。",
                node.objective
            ),
            llm_usage: Some("fast".to_string()),
            skill_autosuggest: Some(false),
            inputs: Vec::new(),
            rework_instruction: None,
        };
        self.dispatch_assignment_to_worker(assignment)
    }

    pub(super) async fn handle_collaboration_worker_report(
        &mut self,
        report: WorkerReport,
        context: &MessageContext,
    ) -> bool {
        let Some(plan) = self.active_collaboration.clone() else {
            return false;
        };
        if plan.assignment_for(&report.worker_id).is_none() {
            return false;
        }

        self.memory.push_progress(format!(
            "collaboration node {} reported by {}",
            report.node_id.0, report.worker_id.0
        ));
        self.emit_event(SessionEvent::WorkerFinished {
            session_id: self.session_id_from_context(context),
            worker_id: report.worker_id.clone(),
            task_id: report.task_id.clone(),
        });
        let worker_id = report.worker_id.clone();
        let task_id = report.task_id.clone();
        self.collaboration_reports
            .insert(worker_id.clone(), report.clone());
        let output = crate::agent::commander::user_facing_report_text(&report)
            .unwrap_or_else(|| report.content.clone());

        if worker_id == plan.primary.worker_id && !plan.support.is_empty() {
            self.active_collaboration_task_id = Some(task_id.clone());
            self.collaboration_outputs.insert(
                worker_id.clone(),
                build_contribution(&plan.primary, &task_id, &output),
            );
            self.emit_event(SessionEvent::CollaborationWorkerFinished {
                session_id: self.session_id_from_context(context),
                worker_id,
                task_id: task_id.clone(),
                content: Some(output),
            });
            self.dispatch_support_workers(&plan, &task_id).await;
            return true;
        }

        if self
            .pending_support_workers
            .iter()
            .any(|item| item == &worker_id)
        {
            self.pending_support_workers
                .retain(|item| item != &worker_id);
            self.completed_support_workers.push(worker_id.clone());
            if let Some(assignment) = plan.assignment_for(&worker_id) {
                self.collaboration_outputs.insert(
                    worker_id.clone(),
                    build_contribution(assignment, &task_id, &output),
                );
            }
            self.emit_event(SessionEvent::CollaborationWorkerFinished {
                session_id: self.session_id_from_context(context),
                worker_id,
                task_id: task_id.clone(),
                content: Some(output),
            });
            if self.pending_support_workers.is_empty() {
                let primary_task_id = self.active_collaboration_task_id.clone().unwrap_or(task_id);
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
        let primary_report = self
            .collaboration_reports
            .get(&plan.primary.worker_id)
            .cloned();
        let primary_output = self
            .collaboration_outputs
            .get(&plan.primary.worker_id)
            .map(|contribution| contribution.content.as_str())
            .filter(|content| !content.trim().is_empty())
            .unwrap_or("主角色已完成初稿，但没有返回可读摘要。")
            .to_string();
        let graph_id = TaskGraphId(format!("graph-{}", primary_task_id.0));
        for assignment in &plan.support {
            let support_task_id = TaskId(format!(
                "{}:support:{}",
                primary_task_id.0, assignment.runtime_role
            ));
            let worker_assignment = WorkerAssignment {
                graph_id: graph_id.clone(),
                node_id: TaskNodeId(format!("support_{}", assignment.runtime_role)),
                task_id: support_task_id,
                worker_id: assignment.worker_id.clone(),
                role: assignment.runtime_role.clone(),
                attempt: 1,
                objective: format!(
                    "原始用户任务：{root_task}\n\n主角色初稿：{primary_output}\n作为 {} \
                     角色，请基于同一上下文补充本角色的独立判断、证据、风险和下一步建议；\
                     不要只说“接收上游结果”。本轮是轻量协作补充，直接输出可汇总内容，\
                     不请求外部 Skill 或工具。",
                    assignment.runtime_role
                ),
                llm_usage: Some("fast".to_string()),
                skill_autosuggest: Some(false),
                inputs: primary_report.clone().into_iter().collect(),
                rework_instruction: None,
            };
            if self.dispatch_assignment_to_worker(worker_assignment) {
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
        let support_roles = self
            .completed_support_workers
            .iter()
            .filter_map(|worker_id| {
                self.active_collaboration
                    .as_ref()
                    .and_then(|plan| plan.assignment_for(worker_id))
                    .map(|assignment| assignment.runtime_role.clone())
                    .or_else(|| Some(worker_id.0.trim_start_matches("worker.").to_string()))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let content = if support_roles.is_empty() {
            self.active_collaboration
                .as_ref()
                .and_then(|plan| self.collaboration_outputs.get(&plan.primary.worker_id))
                .map(|contribution| contribution.content.clone())
                .unwrap_or_else(no_user_facing_content_message)
        } else {
            self.synthesize_collaboration_output(task_id, &support_roles)
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
        self.collaboration_reports.clear();
        self.phase = CommanderPhase::Idle;
        self.state = CommanderState::Idle;
    }

    fn synthesize_collaboration_output(&self, _task_id: &TaskId, support_roles: &str) -> String {
        let task = self
            .current_task
            .as_ref()
            .map(|task| task.description.clone())
            .unwrap_or_default();
        let Some(plan) = &self.active_collaboration else {
            return synthesize_user_facing_collaboration_report(
                &task,
                None,
                support_roles,
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
            &task,
            Some(plan),
            support_roles,
            &contributions,
            self.current_context
                .metadata
                .get("context.selected_summary"),
        )
    }

    pub(super) fn is_collaboration_worker(&self, worker_id: &WorkerId) -> bool {
        self.active_collaboration
            .as_ref()
            .map(|plan| plan.assignment_for(worker_id).is_some())
            .unwrap_or(false)
    }
}

pub(super) fn synthesize_user_facing_collaboration_report(
    task: &str,
    plan: Option<&RoleCollaborationPlan>,
    support_roles: &str,
    contributions: &[RoleContribution],
    selected_summary: Option<&String>,
) -> String {
    let roles = collaboration_roles(plan, contributions);
    let mut lines = Vec::new();

    lines.push("# 协作结果".to_string());
    lines.push(format!("**本轮请求**：{}", task.trim()));
    if let Some(selected_summary) = selected_summary
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        lines.push(format!(
            "**上下文摘要**：{}",
            compact_text(selected_summary, 140)
        ));
    }
    if !support_roles.trim().is_empty() {
        lines.push(format!("**参与角色**：{}", support_roles.trim()));
    }
    lines.push(String::new());

    lines.push("## 角色输出".to_string());
    let contribution_lines = user_facing_contribution_lines(contributions);
    if contribution_lines.is_empty() {
        lines.push("- 暂无可读角色输出。".to_string());
    } else {
        lines.extend(contribution_lines);
    }
    lines.push(String::new());

    lines.push("## 协作说明".to_string());
    if roles.is_empty() {
        lines.push("- 未记录到参与角色。".to_string());
    } else {
        lines.push(format!("- 参与角色：{}", roles.join(", ")));
    }

    lines.join("\n")
}

fn role_route_dispatch_graph(task: &TaskSpec, role_route: &crate::roles::RoleRoute) -> TaskGraph {
    let primary_role = role_route.primary_runtime_role.trim();
    let primary_id = TaskNodeId(primary_role.to_string());
    let mut nodes = vec![TaskNode {
        id: primary_id.clone(),
        role: primary_role.to_string(),
        worker_id: WorkerId(format!("worker.{primary_role}")),
        title: role_step_title(primary_role, true),
        objective: format!(
            "作为本轮主岗位先完成核心判断，并给支持岗位可复核的初稿：{}",
            task.description
        ),
        llm_usage: None,
        skill_autosuggest: Some(false),
        input_refs: Vec::new(),
        acceptance: vec!["必须先给出本岗位的核心判断、依据、风险和下一步建议。".to_string()],
        max_attempts: 1,
    }];

    for support_role in &role_route.support_runtime_roles {
        let support_role = support_role.trim();
        if support_role.is_empty() || support_role == primary_role {
            continue;
        }
        if nodes.iter().any(|node| node.role == support_role) {
            continue;
        }
        nodes.push(TaskNode {
            id: TaskNodeId(support_role.to_string()),
            role: support_role.to_string(),
            worker_id: WorkerId(format!("worker.{support_role}")),
            title: role_step_title(support_role, false),
            objective: format!(
                "基于主岗位初稿补充本岗位的独立判断、证据、风险和下一步建议：{}",
                task.description
            ),
            llm_usage: None,
            skill_autosuggest: Some(false),
            input_refs: vec![primary_id.clone()],
            acceptance: vec!["必须补充本岗位独立贡献，不能只复述主岗位结论。".to_string()],
            max_attempts: 1,
        });
    }

    let adaptive_hint = (nodes.len() > 1).then(|| TaskGraphAdaptiveHint {
        min_level: Some("complex".to_string()),
        reason: Some(
            "本轮调度已形成主岗位加支持岗位的协作链路，过程面板会展示实际参与岗位。".to_string(),
        ),
    });

    TaskGraph {
        graph_id: crate::core::protocol::TaskGraphId(format!("graph-{}", task.id.0)),
        root_task: task.description.clone(),
        nodes,
        adaptive_hint,
    }
}

fn role_step_title(runtime_role: &str, primary: bool) -> String {
    let role_label = match runtime_role {
        "chat" => "快速对话",
        "data" => "数据诊断",
        "ops" => "运营方案",
        "creative" => "创意产出",
        "service" => "客服承接",
        "engineering" => "资料与系统",
        "douyin_ads" => "投放判断",
        "accounting" => "预算评估",
        "design" => "体验设计",
        "web" => "市场与搜索",
        other => other,
    };
    if primary {
        format!("{role_label}主判断")
    } else {
        format!("{role_label}协作补充")
    }
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
    roles
}

fn push_unique_role(roles: &mut Vec<String>, role: &str) {
    let role = role.trim();
    if !role.is_empty() && !roles.iter().any(|item| item == role) {
        roles.push(role.to_string());
    }
}

fn user_facing_contribution_lines(contributions: &[RoleContribution]) -> Vec<String> {
    contributions
        .iter()
        .filter_map(|contribution| {
            let content = sanitize_contribution_text(&contribution.content);
            (!content.is_empty())
                .then(|| format!("- **{}**：{}", contribution.runtime_role.trim(), content))
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

#[cfg(test)]
mod tests {
    use super::{role_route_dispatch_graph, synthesize_user_facing_collaboration_report};
    use crate::{
        core::protocol::{TaskId, WorkerId},
        roles::{RoleAssignment, RoleCollaborationPlan, RoleContribution, RoleId, RoleRoute},
    };

    #[test]
    fn legacy_collaboration_synthesis_is_generic_summary() {
        let task = "请用多个角色一起判断这个问题。";
        let plan = RoleCollaborationPlan {
            primary: assignment("ops"),
            support: vec![assignment("data"), assignment("creative")],
        };
        let contributions = vec![
            contribution("ops", "先统一目标和优先级。"),
            contribution("data", "补充可复核指标和缺失数据。"),
            contribution("creative", "补充素材测试方向。"),
        ];

        let output = synthesize_user_facing_collaboration_report(
            task,
            Some(&plan),
            "data, creative",
            &contributions,
            None,
        );

        assert!(output.contains("# 协作结果"));
        assert!(output.contains("## 角色输出"));
        assert!(output.contains("**ops**"));
        assert!(output.contains("**data**"));
        assert!(output.contains("**creative**"));
        assert!(output.contains("参与角色：ops, data, creative"));
        assert!(!output.contains("任务编号"));
        assert!(!output.contains("worker"));
        assert!(!output.contains("fast-route"));
        assert!(!output.contains("legacy role table"));
        assert!(!output.contains("legacy primary plan"));
        assert!(!output.contains("final_report"));
    }

    #[test]
    fn legacy_collaboration_synthesis_hides_internal_evaluator_noise() {
        let task = "请用多个角色一起判断这个问题。";
        let plan = RoleCollaborationPlan {
            primary: assignment("ops"),
            support: vec![assignment("data"), assignment("service")],
        };
        let contributions = vec![
            contribution("ops", "任务已完成: task-123"),
            contribution("data", "data role should provide at least two findings"),
            contribution(
                "service",
                "service role output lacks service-specific content",
            ),
        ];

        let output = synthesize_user_facing_collaboration_report(
            task,
            Some(&plan),
            "data, service",
            &contributions,
            None,
        );
        let lower = output.to_ascii_lowercase();

        assert!(output.contains("暂无可读角色输出"));
        assert!(!lower.contains("data role should provide"));
        assert!(!lower.contains("service role output lacks"));
        assert!(!output.contains("任务已完成: task-123"));
        assert!(!lower.contains("worker"));
    }

    #[test]
    fn fast_route_dispatch_graph_keeps_support_roles_visible() {
        let task = crate::core::protocol::TaskSpec {
            id: TaskId("task-1".to_string()),
            description: "请数据、运营、创意三个岗位一起设计校园手机壳上新方案。".to_string(),
        };
        let route = RoleRoute {
            primary_role: RoleId("role.ops-strategist".to_string()),
            primary_runtime_role: "ops".to_string(),
            support_roles: vec![
                RoleId("role.data-analyst".to_string()),
                RoleId("role.creative-studio".to_string()),
            ],
            support_runtime_roles: vec!["data".to_string(), "creative".to_string()],
            scores: Vec::new(),
        };

        let graph = role_route_dispatch_graph(&task, &route);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.nodes[0].role, "ops");
        assert_eq!(graph.nodes[1].role, "data");
        assert_eq!(graph.nodes[2].role, "creative");
        assert_eq!(graph.nodes[1].input_refs, vec![graph.nodes[0].id.clone()]);
        assert_eq!(graph.nodes[2].input_refs, vec![graph.nodes[0].id.clone()]);
        assert_eq!(
            graph.adaptive_hint.and_then(|hint| hint.min_level),
            Some("complex".to_string())
        );
        assert!(
            graph
                .nodes
                .iter()
                .all(|node| node.skill_autosuggest == Some(false))
        );
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
