use crate::{
    agent::worker::{Task, TaskMemory, WorkerPhase},
    cognition::{CognitionInput, Context, Fact, Intent, IntentKind},
    core::protocol::AgentId,
    roles::RoleProfile,
};

pub(super) fn build_cognition_input(
    worker_id: AgentId,
    phase: WorkerPhase,
    memory: &TaskMemory,
    role: &RoleProfile,
    task: &Task,
) -> CognitionInput {
    CognitionInput {
        intent: Intent {
            id: task.id.clone(),
            kind: IntentKind::Planning,
            description: task.description.clone(),
        },
        context: build_context(worker_id, phase, memory, role, task),
    }
}

fn build_context(
    worker_id: AgentId,
    phase: WorkerPhase,
    memory: &TaskMemory,
    role: &RoleProfile,
    task: &Task,
) -> Context {
    let mut metadata = std::collections::HashMap::new();
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
        if let Some(llm_usage) = assignment
            .llm_usage
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            metadata.insert("assignment.llm_usage".to_string(), llm_usage.to_string());
        }
        if let Some(skill_autosuggest) = assignment.skill_autosuggest {
            metadata.insert(
                "assignment.skill_autosuggest".to_string(),
                skill_autosuggest.to_string(),
            );
        }
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
    extend_runtime_strategy_facts(&task.context.metadata, &mut facts);
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
            facts.extend(input.evidence.iter().filter_map(|item| {
                item.strip_prefix("search_evidence_packet:")
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|packet| Fact {
                        source: format!(
                            "worker.assignment.input.{}.evidence_packet",
                            input.node_id.0
                        ),
                        content: packet.to_string(),
                        reliability: 1.0,
                    })
            }));
        }
    }

    facts.extend(memory.progress.iter().map(|entry| Fact {
        source: "worker.runtime".to_string(),
        content: entry.clone(),
        reliability: 1.0,
    }));

    Context { facts, metadata }
}

fn extend_runtime_strategy_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let mode = runtime_response_mode(metadata);
    let answer_style = match mode {
        "quick" => {
            "回答模式：快速简答。优先给直接结论、关键理由和最多 3 \
             个下一步动作；减少铺垫、过程展开和长篇教学，除非用户明确要求详细推导。"
        }
        "execution" => {
            "回答模式：丰富执行。按用户要的交付物决定展开程度；用户要完整方案时给可执行动作、\
             风险边界和复盘字段；用户只要标题、话术、字段或短判断时仍直接交付，不强制岗位报告、\
             表格或完整方案。"
        }
        _ => {
            "回答模式：教学模式。默认面向电商学生和新手，先给可用结论，再用通俗语言解释概念、\
             判断过程、为什么这么做和下一步怎么练。"
        }
    };
    facts.push(Fact {
        source: "runtime.answer_style".to_string(),
        content: answer_style.to_string(),
        reliability: 1.0,
    });

    let explanation_depth = match runtime_learning_level(metadata) {
        "beginner" => {
            "讲解深度：低。尽量少用术语，遇到 \
             ROI、转化率、漏斗、客单价等专业词要用白话解释，并给简单例子。"
        }
        "professional" => {
            "讲解深度：高。保留关键指标、推导路径、约束假设、风险边界和可复核口径，\
             适合汇报和专业复盘。"
        }
        _ => "讲解深度：中。按电商岗位应用口径解释，术语不过度展开，但关键概念和动作口径要说清楚。",
    };
    facts.push(Fact {
        source: "runtime.explanation_depth".to_string(),
        content: explanation_depth.to_string(),
        reliability: 1.0,
    });
}

fn runtime_response_mode(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    if let Some(response_mode) = metadata
        .get("strategy.response_mode")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
    {
        return response_mode;
    }
    metadata
        .get("strategy.id")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
        .unwrap_or("learning")
}

fn runtime_learning_level(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    metadata
        .get("strategy.learning_level")
        .map(String::as_str)
        .filter(|value| matches!(*value, "beginner" | "higher_vocational" | "professional"))
        .unwrap_or("higher_vocational")
}

fn extend_context_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    extend_conversation_history_fact(metadata, facts);
    extend_named_context_fact(
        metadata,
        facts,
        "context.selected_summary",
        "commerce.selected_product_context",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "context.sources_json",
        "commerce.selected_product_context.sources",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "context.combined_summary",
        "request.combined_context",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContext.summary",
        "workspace.context_summary",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.summary",
        "workspace.context_pack_summary",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.items_json",
        "workspace.context_pack.items",
    );
}

fn extend_conversation_history_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let Some(value) = metadata
        .get("conversation.history_summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    facts.push(Fact {
        source: "conversation.history".to_string(),
        content: format!(
            "conversation.history：同一会话历史上下文如下。回答追问、补充、继续、\
             按前面数据等问题时，必须优先结合这段历史；不要要求用户重复已经在这里提供的信息。\\
             n{value}"
        ),
        reliability: 1.0,
    });
}

fn extend_named_context_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
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
        content: value.to_string(),
        reliability: 1.0,
    });
}

#[cfg(test)]
mod tests {
    use super::build_cognition_input;
    use crate::{
        agent::worker::{Task, TaskMemory, WorkerPhase},
        core::protocol::{
            AgentId, MessageContext, RoleWorkOutput, TaskGraphId, TaskId, TaskNodeId,
            WorkerAssignment, WorkerId, WorkerReport, WorkerReportStatus,
        },
        roles::{RoleId, RoleProfile},
    };

    #[test]
    fn assignment_input_search_evidence_packet_becomes_context_fact() {
        let packet = r#"{"intent":"competitor_price_or_benchmark","usable_claims":["competitor prices cluster at 12-19 CNY"],"gaps":[],"confidence":0.72}"#;
        let search_report = WorkerReport {
            graph_id: TaskGraphId("graph-search".to_string()),
            node_id: TaskNodeId("search".to_string()),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: "Search evidence prepared.".to_string(),
            role_output: Some(RoleWorkOutput {
                summary: "Search evidence prepared.".to_string(),
                evidence: vec![format!("search_evidence_packet:{packet}")],
                ..RoleWorkOutput::default()
            }),
            evidence: vec![format!("search_evidence_packet:{packet}")],
            risks: Vec::new(),
            open_questions: Vec::new(),
            status: WorkerReportStatus::Completed,
        };
        let task = Task {
            id: "task-final".to_string(),
            context: MessageContext::default(),
            description: "Give a pricing recommendation from search evidence.".to_string(),
            requester: AgentId("commander".to_string()),
            assignment: Some(WorkerAssignment {
                graph_id: TaskGraphId("graph-final".to_string()),
                node_id: TaskNodeId("ops_final".to_string()),
                task_id: TaskId("task-final".to_string()),
                worker_id: WorkerId("worker.ops".to_string()),
                role: "ops".to_string(),
                attempt: 1,
                root_task: "Give a pricing recommendation.".to_string(),
                objective: "Synthesize search evidence into an action plan.".to_string(),
                llm_usage: None,
                skill_autosuggest: Some(false),
                inputs: vec![search_report],
                rework_instruction: None,
            }),
        };
        let role = RoleProfile {
            id: RoleId("role.ops".to_string()),
            name: "Operations".to_string(),
            runtime_role: "ops".to_string(),
            priority: 1,
            domains: Vec::new(),
            keywords: Vec::new(),
            preferred_actions: Vec::new(),
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
        };
        let input = build_cognition_input(
            AgentId("worker.ops".to_string()),
            WorkerPhase::Thinking,
            &TaskMemory::default(),
            &role,
            &task,
        );

        assert!(
            input.context.facts.iter().any(|fact| {
                fact.source == "worker.assignment.input.search.evidence_packet"
                    && fact.content.contains("usable_claims")
                    && fact.content.contains("competitor_price_or_benchmark")
            }),
            "search evidence packet should be available as a downstream cognition fact"
        );
    }
}
