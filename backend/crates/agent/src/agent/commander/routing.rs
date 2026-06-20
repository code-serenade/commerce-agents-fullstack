use serde_json::Value;

use super::CommanderTask;
use crate::{
    cognition::CognitionOutput,
    core::protocol::{
        DecisionIntent, TaskGraph, TaskGraphId, TaskNode, TaskNodeId, TaskSpec, WorkerId,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedDecision {
    pub intent: DecisionIntent,
    pub target_worker_id: Option<WorkerId>,
    pub task_graph: Option<TaskGraph>,
    pub clarification: Option<String>,
}

pub fn decision_intent_from_json(
    output: &CognitionOutput,
    current_task: Option<CommanderTask>,
) -> RoutedDecision {
    let decision = output.get("decision");
    let kind = decision
        .and_then(|value| value.get("kind"))
        .and_then(|value| value.as_str());

    match kind {
        Some("RouteTask") => route_task(decision, current_task),
        Some("ExecuteTaskGraph") => RoutedDecision {
            intent: DecisionIntent::ExecuteTask {
                task: current_task
                    .clone()
                    .map(task_to_spec)
                    .unwrap_or_else(|| TaskSpec {
                        id: crate::core::protocol::TaskId("commander-graph".to_string()),
                        description: parse_task_graph(decision)
                            .map(|graph| graph.root_task)
                            .unwrap_or_default(),
                    }),
            },
            target_worker_id: None,
            task_graph: parse_task_graph(decision),
            clarification: None,
        },
        Some("KeepCurrentAssignment") => RoutedDecision {
            intent: current_task
                .map(task_to_spec)
                .map(|task| DecisionIntent::ExecuteTask { task })
                .unwrap_or(DecisionIntent::Ignore),
            target_worker_id: parse_target_worker_id(decision),
            task_graph: None,
            clarification: None,
        },
        Some("AskForClarification") => RoutedDecision {
            intent: DecisionIntent::Ignore,
            target_worker_id: None,
            task_graph: None,
            clarification: decision
                .and_then(|value| value.get("clarification"))
                .and_then(|value| value.get("question"))
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned),
        },
        Some("NoRoute") => RoutedDecision {
            intent: DecisionIntent::IgnoreNewTask,
            target_worker_id: None,
            task_graph: None,
            clarification: None,
        },
        Some("NoAction") => RoutedDecision {
            intent: DecisionIntent::IgnoreNewTask,
            target_worker_id: None,
            task_graph: None,
            clarification: None,
        },
        Some("ActionIntent") => RoutedDecision {
            intent: current_task
                .map(task_to_spec)
                .map(|task| DecisionIntent::ExecuteTask { task })
                .unwrap_or(DecisionIntent::Ignore),
            target_worker_id: parse_target_worker_id(decision),
            task_graph: None,
            clarification: None,
        },
        Some("StateProposal") => {
            let key = decision
                .and_then(|value| value.get("state_change"))
                .and_then(|value| value.get("key"))
                .and_then(|value| value.as_str());
            let intent = if key == Some("replace_current_task") {
                current_task
                    .map(task_to_spec)
                    .map(|task| DecisionIntent::ReplaceCurrentTask { task })
                    .unwrap_or(DecisionIntent::Ignore)
            } else {
                current_task
                    .map(task_to_spec)
                    .map(|task| DecisionIntent::ExecuteTask { task })
                    .unwrap_or(DecisionIntent::Ignore)
            };

            RoutedDecision {
                intent,
                target_worker_id: None,
                task_graph: None,
                clarification: None,
            }
        }
        Some("StrategyHint") => RoutedDecision {
            intent: DecisionIntent::StrategyHint {
                hint: decision
                    .and_then(|value| value.get("strategy"))
                    .and_then(|value| value.get("hint"))
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
            },
            target_worker_id: None,
            task_graph: None,
            clarification: None,
        },
        _ => RoutedDecision {
            intent: DecisionIntent::Ignore,
            target_worker_id: None,
            task_graph: None,
            clarification: None,
        },
    }
}

fn route_task(decision: Option<&Value>, current_task: Option<CommanderTask>) -> RoutedDecision {
    let intent = current_task
        .map(task_to_spec)
        .map(|task| DecisionIntent::ExecuteTask { task })
        .unwrap_or(DecisionIntent::Ignore);

    RoutedDecision {
        intent,
        target_worker_id: parse_target_worker_id(decision),
        task_graph: None,
        clarification: None,
    }
}

fn parse_target_worker_id(decision: Option<&Value>) -> Option<WorkerId> {
    let route = decision.and_then(|value: &Value| value.get("route"))?;
    let target = route
        .get("target_agent_id")
        .and_then(|value: &Value| value.as_str())
        .map(str::trim)
        .filter(|value: &&str| !value.is_empty())?;

    Some(WorkerId(target.to_string()))
}

fn task_to_spec(task: CommanderTask) -> TaskSpec {
    TaskSpec {
        id: crate::core::protocol::TaskId(task.id),
        description: task.description,
    }
}

fn parse_task_graph(decision: Option<&Value>) -> Option<TaskGraph> {
    let graph = decision.and_then(|value| value.get("graph"))?;
    let nodes = graph
        .get("nodes")?
        .as_array()?
        .iter()
        .filter_map(parse_task_node)
        .collect::<Vec<_>>();
    if nodes.is_empty() {
        return None;
    }
    Some(TaskGraph {
        graph_id: TaskGraphId(
            graph
                .get("graph_id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("commander-generated-graph")
                .to_string(),
        ),
        root_task: graph
            .get("root_task")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        nodes,
        adaptive_hint: None,
    })
}

fn parse_task_node(value: &Value) -> Option<TaskNode> {
    let id = value.get("id").and_then(Value::as_str)?.trim();
    let role = value.get("role").and_then(Value::as_str)?.trim();
    if id.is_empty() || role.is_empty() {
        return None;
    }
    Some(TaskNode {
        id: TaskNodeId(id.to_string()),
        role: role.to_string(),
        worker_id: value
            .get("worker_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|worker_id| !worker_id.is_empty())
            .map(|worker_id| WorkerId(worker_id.to_string()))
            .unwrap_or_else(|| WorkerId(format!("worker.{role}"))),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(role)
            .to_string(),
        objective: value
            .get("objective")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        llm_usage: value
            .get("llm_usage")
            .and_then(Value::as_str)
            .map(str::to_string),
        skill_autosuggest: value.get("skill_autosuggest").and_then(Value::as_bool),
        timeout_sec: value.get("timeout_sec").and_then(Value::as_u64),
        input_refs: value
            .get("input_refs")
            .and_then(Value::as_array)
            .map(|refs| {
                refs.iter()
                    .filter_map(Value::as_str)
                    .map(|item| TaskNodeId(item.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        acceptance: value
            .get("acceptance")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        max_attempts: value
            .get("max_attempts")
            .and_then(Value::as_u64)
            .and_then(|value| u8::try_from(value).ok())
            .unwrap_or(1)
            .max(1),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::decision_intent_from_json;
    use crate::{agent::commander::CommanderTask, cognition::CognitionOutput};

    #[test]
    fn parses_route_task_and_target_worker() {
        let output: CognitionOutput = json!({
            "decision": {
                "kind": "RouteTask",
                "route": {
                    "target_agent_id": "worker-2",
                    "task_summary": "sum",
                    "goal": "finish",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "best fit",
                    "expected_output": "done"
                },
                "clarification": {
                    "question": ""
                }
            },
            "confidence": 0.9,
            "rationale": {
                "primary": "fit",
                "evidence": [],
                "alternatives_considered": []
            }
        });

        let decision = decision_intent_from_json(
            &output,
            Some(CommanderTask {
                id: "task-1".to_string(),
                description: "test".to_string(),
            }),
        );

        assert_eq!(decision.target_worker_id.unwrap().0, "worker-2");
        assert!(matches!(
            decision.intent,
            crate::core::protocol::DecisionIntent::ExecuteTask { .. }
        ));
    }

    #[test]
    fn parses_clarification_question() {
        let output: CognitionOutput = json!({
            "decision": {
                "kind": "AskForClarification",
                "route": {
                    "target_agent_id": "",
                    "task_summary": "",
                    "goal": "",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "",
                    "expected_output": ""
                },
                "clarification": {
                    "question": "需要先确认目标环境"
                }
            },
            "confidence": 0.4,
            "rationale": {
                "primary": "missing info",
                "evidence": [],
                "alternatives_considered": []
            }
        });

        let decision = decision_intent_from_json(&output, None);

        assert_eq!(
            decision.clarification.as_deref(),
            Some("需要先确认目标环境")
        );
    }

    #[test]
    fn keep_current_assignment_executes_current_user_request() {
        let output: CognitionOutput = json!({
            "decision": {
                "kind": "KeepCurrentAssignment",
                "route": {
                    "target_agent_id": "worker.chat",
                    "task_summary": "沿用现有资料继续回答",
                    "goal": "给出丰富干货回答",
                    "constraints": []
                }
            },
            "confidence": 0.8
        });

        let decision = decision_intent_from_json(
            &output,
            Some(CommanderTask {
                id: "task-rich-followup".to_string(),
                description: "我不会给你提供任何其他信息，但是你必须根据我所给资料进行丰富回答"
                    .to_string(),
            }),
        );

        match decision.intent {
            crate::core::protocol::DecisionIntent::ExecuteTask { task } => {
                assert_eq!(task.id.0, "task-rich-followup");
                assert!(task.description.contains("丰富回答"));
            }
            other => panic!("expected current request to execute, got {other:?}"),
        }
        assert_eq!(decision.target_worker_id.unwrap().0, "worker.chat");
    }
}
