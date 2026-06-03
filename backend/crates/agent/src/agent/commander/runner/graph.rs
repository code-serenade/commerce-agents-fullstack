use tracing::{debug, warn};

use super::{Commander, CommanderPhase, CommanderState, next_message_id, primary_route_score};
use crate::agent::commander::role_dispatch_debug::{
    log_assignment_dispatch, log_task_graph_planned, log_task_node_failed, log_worker_report,
};
use crate::{
    agent::commander::{
        EvaluationOutcome, TaskGraphRuntime, no_user_facing_content_message,
        plan_configured_non_chat_task_graph, plan_configured_task_graph, plan_forced_role_graph,
        plan_forced_selected_roles_graph, plan_task_graph,
        runner::{output::direct_chat_report_output, route::primary_graph_role},
        sanitize_final_user_facing_text, user_facing_gap_reason,
    },
    core::protocol::{
        AgentId, Message, MessageContext, Payload, RoleWorkOutput, SessionEvent, TaskGraph,
        WorkerAssignment, WorkerDiagnostic, WorkerReport, WorkerReportStatus,
    },
    roles::RoleRoute,
};

impl Commander {
    pub(super) fn dispatch_assignment_to_worker(&self, assignment: WorkerAssignment) -> bool {
        let worker_profiles = self.session_context.list_worker_profiles();
        let Some(worker_tx) = self.session_context.get_worker_tx(&assignment.worker_id) else {
            warn!(
                worker_id = %assignment.worker_id.0,
                "no worker tx available for task graph assignment"
            );
            log_assignment_dispatch(
                &self.current_context,
                &assignment,
                &worker_profiles,
                false,
                Some("worker_tx_missing"),
            );
            return false;
        };

        let task_message = Message::new_with_context(
            next_message_id(),
            self.current_context.clone(),
            self.id.clone(),
            AgentId(assignment.worker_id.0.clone()),
            Payload::WorkerAssignment {
                assignment: assignment.clone(),
            },
        );
        let dispatched = worker_tx.send(task_message).is_ok();
        log_assignment_dispatch(
            &self.current_context,
            &assignment,
            &worker_profiles,
            dispatched,
            (!dispatched).then_some("worker_channel_send_failed"),
        );
        dispatched
    }

    pub(super) async fn try_task_graph_route(&mut self, role_route: Option<&RoleRoute>) -> bool {
        let Some(current_task) = self.current_task.clone() else {
            return false;
        };

        let task_id = crate::core::protocol::TaskId(current_task.id);
        let graph = plan_task_graph(&task_id, &current_task.description, role_route);
        self.schedule_planned_task_graph("task_graph", role_route, graph)
            .await
    }

    pub(super) async fn try_configured_task_graph_route(
        &mut self,
        role_route: Option<&RoleRoute>,
    ) -> bool {
        let Some(current_task) = self.current_task.clone() else {
            return false;
        };

        let task_id = crate::core::protocol::TaskId(current_task.id);
        let graph = if let Some(graph) =
            self.single_role_task_graph(&task_id, &current_task.description, role_route)
        {
            Some(graph)
        } else if let Some(graph) =
            self.manual_role_task_graph(&task_id, &current_task.description, role_route)
        {
            Some(graph)
        } else {
            if self.context_allows_direct_chat_graph(role_route) {
                debug!(
                    task_id = %task_id.0,
                    routing_mode = self
                        .current_context
                        .metadata
                        .get("strategy.routing_mode")
                        .map(String::as_str)
                        .unwrap_or("<missing>"),
                    "commander configured graph lookup allows direct chat graph"
                );
                plan_configured_task_graph(&task_id, &current_task.description, role_route)
            } else {
                debug!(
                    task_id = %task_id.0,
                    routing_mode = self
                        .current_context
                        .metadata
                        .get("strategy.routing_mode")
                        .map(String::as_str)
                        .unwrap_or("<missing>"),
                    "commander configured graph lookup requires non-chat graph"
                );
                plan_configured_non_chat_task_graph(&task_id, &current_task.description, role_route)
            }
        };
        let Some(graph) = graph else {
            debug!(
                task_id = %task_id.0,
                routing_mode = self
                    .current_context
                    .metadata
                    .get("strategy.routing_mode")
                    .map(String::as_str)
                    .unwrap_or("<missing>"),
                "commander configured graph lookup missed"
            );
            return false;
        };
        debug!(
            task_id = %task_id.0,
            graph_id = %graph.graph_id.0,
            node_count = graph.nodes.len(),
            roles = ?graph.nodes.iter().map(|node| node.role.as_str()).collect::<Vec<_>>(),
            "commander configured graph lookup hit"
        );
        self.schedule_planned_task_graph("task_graph", role_route, graph)
            .await
    }

    pub(super) fn single_role_task_graph(
        &self,
        task_id: &crate::core::protocol::TaskId,
        task: &str,
        role_route: Option<&RoleRoute>,
    ) -> Option<TaskGraph> {
        if self
            .current_context
            .metadata
            .get("strategy.collaboration_mode")
            .is_none_or(|value| value != "single")
        {
            return None;
        }
        let route = role_route?;
        Some(plan_forced_role_graph(
            task_id,
            task,
            &route.primary_runtime_role,
        ))
    }

    pub(super) fn manual_role_task_graph(
        &self,
        task_id: &crate::core::protocol::TaskId,
        task: &str,
        role_route: Option<&RoleRoute>,
    ) -> Option<TaskGraph> {
        if self
            .current_context
            .metadata
            .get("strategy.collaboration_mode")
            .is_none_or(|value| value != "manual")
        {
            return None;
        }
        let route = role_route?;
        let roles = std::iter::once(route.primary_runtime_role.clone())
            .chain(route.support_runtime_roles.iter().cloned())
            .filter(|role| role != "chat")
            .fold(Vec::<String>::new(), |mut unique, role| {
                if !unique.iter().any(|item| item == &role) {
                    unique.push(role);
                }
                unique
            });
        (!roles.is_empty()).then(|| plan_forced_selected_roles_graph(task_id, task, &roles))
    }

    pub(super) async fn schedule_planned_task_graph(
        &mut self,
        route_mode: &str,
        role_route: Option<&RoleRoute>,
        graph: TaskGraph,
    ) -> bool {
        let Some(graph) = self.runnable_task_graph(graph) else {
            debug!("commander task graph skipped: no runnable nodes");
            return false;
        };

        self.memory
            .push_progress(format!("task graph planned: {} nodes", graph.nodes.len()));
        let display_roles = self.task_graph_display_roles(role_route, &graph);
        debug!(
            route_mode,
            graph_id = %graph.graph_id.0,
            node_count = graph.nodes.len(),
            primary_role = display_roles.0.as_deref().unwrap_or("<none>"),
            support_roles = ?display_roles.1,
            "commander scheduling task graph"
        );
        let worker_profiles = self.session_context.list_worker_profiles();
        log_task_graph_planned(&self.current_context, route_mode, &graph, &worker_profiles);
        self.emit_route_decided(route_mode, display_roles.0.clone(), display_roles.1.clone());
        self.emit_dispatch_planned_for_graph_with_roles(route_mode, &graph, display_roles);
        self.emit_event(SessionEvent::TaskGraphPlanned {
            session_id: self.session_id_from_context(&self.current_context),
            graph: graph.clone(),
        });
        self.active_collaboration = None;
        self.active_collaboration_task_id = None;
        self.pending_support_workers.clear();
        self.completed_support_workers.clear();
        self.collaboration_outputs.clear();
        let runtime = TaskGraphRuntime::new(graph)
            .with_response_mode(
                self.current_context
                    .metadata
                    .get("strategy.response_mode")
                    .map(String::as_str)
                    .or_else(|| {
                        self.current_context
                            .metadata
                            .get("strategy.id")
                            .map(String::as_str)
                    }),
            )
            .with_learning_depth(
                self.current_context
                    .metadata
                    .get("strategy.learning_level")
                    .map(String::as_str),
            );
        self.schedule_task_graph_runtime(runtime).await;
        true
    }

    fn should_run_task_graph(&self, role_route: Option<&RoleRoute>, graph: &TaskGraph) -> bool {
        if self.force_collaboration() {
            return true;
        }
        if graph.nodes.len() > 1 {
            return true;
        }
        if primary_graph_role(graph) == Some("chat") {
            return self
                .current_context
                .metadata
                .get("strategy.routing_mode")
                .is_some_and(|value| value == "fast");
        }
        role_route
            .map(primary_route_score)
            .is_none_or(|score| score >= self.fast_route_min_score)
    }

    fn force_collaboration(&self) -> bool {
        self.current_context
            .metadata
            .get("strategy.collaboration_mode")
            .is_some_and(|value| value == "manual" || value == "single")
            || self
                .current_context
                .metadata
                .get("strategy.id")
                .is_some_and(|value| value == "collab")
    }

    fn context_allows_direct_chat_graph(&self, _role_route: Option<&RoleRoute>) -> bool {
        self.current_context
            .metadata
            .get("strategy.routing_mode")
            .is_some_and(|value| value == "fast")
            || self
                .current_context
                .metadata
                .get("strategy.selected_roles")
                .is_some_and(|value| value.split(',').map(str::trim).all(|role| role == "chat"))
    }

    fn runnable_task_graph(&self, graph: TaskGraph) -> Option<TaskGraph> {
        if graph.nodes.is_empty() {
            return None;
        }
        Some(graph)
    }

    pub(super) async fn handle_task_graph_report(
        &mut self,
        report: crate::core::protocol::WorkerReport,
        context: &MessageContext,
    ) {
        self.memory.push_progress(format!(
            "task graph node {} reported by {}",
            report.node_id.0, report.worker_id.0
        ));
        self.emit_event(SessionEvent::WorkerFinished {
            session_id: self.session_id_from_context(context),
            worker_id: report.worker_id.clone(),
            task_id: report.task_id.clone(),
        });
        let Some(mut runtime) = self.active_task_graph.take() else {
            if report.role == "chat" {
                let content = direct_chat_report_output(&report);
                let _ = self.sender.send(Message::new_with_context(
                    next_message_id(),
                    context.clone(),
                    self.id.clone(),
                    self.host_id.clone(),
                    Payload::Text { content },
                ));
                self.phase = CommanderPhase::Idle;
                self.state = CommanderState::Idle;
            }
            return;
        };
        self.emit_and_apply_task_graph_report(&mut runtime, report, context);

        self.schedule_task_graph_runtime(runtime).await;
    }

    async fn schedule_task_graph_runtime(&mut self, mut runtime: TaskGraphRuntime) {
        loop {
            if runtime.is_finished() {
                self.finish_task_graph(runtime).await;
                return;
            }

            let ready_nodes = runtime.ready_nodes();
            if ready_nodes.is_empty() {
                self.active_task_graph = Some(runtime);
                return;
            }

            let mut had_inline_failure = false;
            for node in ready_nodes {
                self.emit_event(SessionEvent::TaskNodeReady {
                    session_id: self.session_id_from_context(&self.current_context),
                    graph_id: runtime.graph.graph_id.clone(),
                    node_id: node.id.clone(),
                    worker_id: node.worker_id.clone(),
                });
                let assignment = runtime.build_assignment(&node);
                if self.dispatch_assignment_to_worker(assignment.clone()) {
                    runtime.mark_running(&assignment);
                    self.emit_event(SessionEvent::WorkerStarted {
                        session_id: self.session_id_from_context(&self.current_context),
                        worker_id: assignment.worker_id.clone(),
                        task_id: assignment.task_id.clone(),
                    });
                    self.emit_event(SessionEvent::TaskNodeStarted {
                        session_id: self.session_id_from_context(&self.current_context),
                        graph_id: assignment.graph_id,
                        node_id: assignment.node_id,
                        worker_id: assignment.worker_id,
                        attempt: assignment.attempt,
                    });
                } else {
                    had_inline_failure = true;
                    runtime
                        .attempts
                        .insert(node.id.clone(), node.max_attempts.max(assignment.attempt));
                    let context = self.current_context.clone();
                    let report = worker_unavailable_report(&assignment);
                    self.emit_and_apply_task_graph_report(&mut runtime, report, &context);
                }
            }

            if !had_inline_failure {
                self.active_task_graph = Some(runtime);
                return;
            }
        }
    }

    fn emit_and_apply_task_graph_report(
        &mut self,
        runtime: &mut TaskGraphRuntime,
        report: WorkerReport,
        context: &MessageContext,
    ) {
        log_worker_report(context, &report);
        self.emit_event(SessionEvent::TaskNodeReported {
            session_id: self.session_id_from_context(context),
            graph_id: report.graph_id.clone(),
            node_id: report.node_id.clone(),
            worker_id: report.worker_id.clone(),
            report: Box::new(report.clone()),
        });
        let graph_id = runtime.graph.graph_id.clone();
        let worker_id = report.worker_id.clone();
        let node_id = report.node_id.clone();
        let outcome = runtime.apply_report(report);
        let rework_attempt = runtime.attempts.get(&node_id).copied().unwrap_or(1) + 1;
        self.emit_task_graph_evaluation_outcome(
            context,
            graph_id,
            worker_id,
            node_id,
            rework_attempt,
            outcome,
        );
    }

    fn emit_task_graph_evaluation_outcome(
        &mut self,
        context: &MessageContext,
        graph_id: crate::core::protocol::TaskGraphId,
        worker_id: crate::core::protocol::WorkerId,
        node_id: crate::core::protocol::TaskNodeId,
        rework_attempt: u8,
        outcome: EvaluationOutcome,
    ) {
        match outcome {
            EvaluationOutcome::Passed(evaluation) => {
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodePassed {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                });
            }
            EvaluationOutcome::Rework {
                evaluation,
                instruction,
            } => {
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodeReworkRequested {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                    worker_id,
                    attempt: rework_attempt,
                    instruction,
                });
            }
            EvaluationOutcome::Failed(evaluation) => {
                let reason = evaluation.reasons.join("; ");
                log_task_node_failed(context, &graph_id.0, &node_id.0, &reason);
                let user_facing_reason = user_facing_gap_reason(&reason);
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodeFailed {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                    reason: user_facing_reason,
                });
            }
        }
    }

    async fn finish_task_graph(&mut self, runtime: TaskGraphRuntime) {
        let raw_content = runtime
            .direct_chat_output()
            .unwrap_or_else(|| runtime.synthesize());
        let content = sanitize_final_user_facing_text(&raw_content)
            .unwrap_or_else(no_user_facing_content_message);
        let process = runtime.process_summary();
        self.session_context
            .record_task_graph_memory(runtime.memory_summary());
        self.emit_event(SessionEvent::TaskGraphFinished {
            session_id: self.session_id_from_context(&self.current_context),
            graph_id: runtime.graph.graph_id.clone(),
            process: Box::new(process),
        });
        let _ = self.sender.send(Message::new_with_context(
            next_message_id(),
            self.current_context.clone(),
            self.id.clone(),
            self.host_id.clone(),
            Payload::Text { content },
        ));
        self.active_task_graph = None;
        self.phase = CommanderPhase::Idle;
        self.state = CommanderState::Idle;
    }
}

fn worker_unavailable_report(assignment: &WorkerAssignment) -> WorkerReport {
    let failure_kind = "worker_unavailable";
    let summary = "本轮有一个协作步骤没有成功启动，不能把结果当成完整协作结论。".to_string();
    let risk = "部分协作步骤没有实际完成，最终回答不能代表完整协作结果。".to_string();
    let open_question =
        "请稍后重试；如果问题很急，可以先切到普通问答或只指定一个角色继续问。".to_string();
    WorkerReport {
        graph_id: assignment.graph_id.clone(),
        node_id: assignment.node_id.clone(),
        task_id: assignment.task_id.clone(),
        worker_id: assignment.worker_id.clone(),
        role: assignment.role.clone(),
        content: summary.clone(),
        role_output: Some(RoleWorkOutput {
            summary,
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: vec!["failure_kind:worker_unavailable".to_string()],
            risks: vec![risk.clone()],
            open_questions: vec![open_question.clone()],
        }),
        evidence: vec![format!("failure_kind:{failure_kind}")],
        risks: vec![risk],
        open_questions: vec![open_question],
        diagnostics: vec![worker_unavailable_diagnostic(assignment, failure_kind)],
        status: WorkerReportStatus::Failed,
    }
}

fn worker_unavailable_diagnostic(
    assignment: &WorkerAssignment,
    failure_kind: &str,
) -> WorkerDiagnostic {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("failure_kind".to_string(), failure_kind.to_string());
    fields.insert("status".to_string(), "failed".to_string());
    fields.insert("role".to_string(), assignment.role.clone());
    fields.insert("node_id".to_string(), assignment.node_id.0.clone());
    fields.insert("graph_id".to_string(), assignment.graph_id.0.clone());
    fields.insert("worker_id".to_string(), assignment.worker_id.0.clone());
    fields.insert(
        "reason".to_string(),
        "no registered executor for selected role".to_string(),
    );

    WorkerDiagnostic {
        kind: "failure".to_string(),
        source: failure_kind.to_string(),
        fields,
    }
}
