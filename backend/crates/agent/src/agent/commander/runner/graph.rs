use std::collections::HashSet;

use tracing::{debug, warn};

use super::{Commander, CommanderPhase, CommanderState, next_message_id, primary_route_score};
use crate::{
    agent::commander::{
        EvaluationOutcome, TaskGraphRuntime, plan_configured_non_chat_task_graph,
        plan_configured_task_graph, plan_task_graph,
        runner::{output::direct_chat_report_output, route::primary_graph_role},
    },
    core::protocol::{
        AgentId, Message, MessageContext, Payload, SessionEvent, TaskGraph, WorkerAssignment,
    },
    roles::RoleRoute,
};

impl Commander {
    pub(super) fn dispatch_assignment_to_worker(&self, assignment: WorkerAssignment) -> bool {
        let Some(worker_tx) = self.session_context.get_worker_tx(&assignment.worker_id) else {
            warn!(
                worker_id = %assignment.worker_id.0,
                "no worker tx available for task graph assignment"
            );
            return false;
        };

        let task_message = Message::new_with_context(
            next_message_id(),
            self.current_context.clone(),
            self.id.clone(),
            AgentId(assignment.worker_id.0.clone()),
            Payload::WorkerAssignment { assignment },
        );
        worker_tx.send(task_message).is_ok()
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
        let graph = if self.context_allows_direct_chat_graph(role_route) {
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

    fn runnable_task_graph(&self, mut graph: TaskGraph) -> Option<TaskGraph> {
        let available = graph
            .nodes
            .iter()
            .filter(|node| {
                self.session_context
                    .get_worker_tx(&node.worker_id)
                    .is_some()
            })
            .map(|node| node.id.clone())
            .collect::<HashSet<_>>();
        graph.nodes.retain(|node| available.contains(&node.id));
        if graph.nodes.is_empty() {
            return None;
        }
        for node in &mut graph.nodes {
            node.input_refs
                .retain(|input_ref| available.contains(input_ref));
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
                let attempt = runtime.attempts.get(&node_id).copied().unwrap_or(1) + 1;
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
                    attempt,
                    instruction,
                });
            }
            EvaluationOutcome::Failed(evaluation) => {
                let reason = evaluation.reasons.join("; ");
                self.emit_event(SessionEvent::TaskNodeEvaluated {
                    session_id: self.session_id_from_context(context),
                    graph_id: graph_id.clone(),
                    evaluation,
                });
                self.emit_event(SessionEvent::TaskNodeFailed {
                    session_id: self.session_id_from_context(context),
                    graph_id,
                    node_id,
                    reason,
                });
            }
        }

        self.schedule_task_graph_runtime(runtime).await;
    }

    async fn schedule_task_graph_runtime(&mut self, mut runtime: TaskGraphRuntime) {
        if runtime.is_finished() {
            self.finish_task_graph(runtime).await;
            return;
        }

        let ready_nodes = runtime.ready_nodes();
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
            }
        }

        self.active_task_graph = Some(runtime);
    }

    async fn finish_task_graph(&mut self, runtime: TaskGraphRuntime) {
        let content = runtime
            .direct_chat_output()
            .unwrap_or_else(|| runtime.synthesize());
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
