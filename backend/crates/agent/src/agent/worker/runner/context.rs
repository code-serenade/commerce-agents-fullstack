use crate::{
    agent::worker::{Task, TaskMemory, WorkerPhase},
    cognition::{CognitionInput, Context, Fact, Intent, IntentKind},
    core::protocol::AgentId,
    roles::{RoleProfile, RolePromptBuilder, RolePromptInput},
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
    facts.extend(memory.progress.iter().map(|entry| Fact {
        source: "worker.runtime".to_string(),
        content: entry.clone(),
        reliability: 1.0,
    }));

    Context { facts, metadata }
}

fn extend_context_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    extend_named_context_fact(
        metadata,
        facts,
        "conversation.history_summary",
        "conversation.history",
    );
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
