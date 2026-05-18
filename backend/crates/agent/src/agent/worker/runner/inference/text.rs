use crate::agent::worker::Task;

pub(in crate::agent::worker::runner) fn inference_text(task: &Task) -> String {
    let mut parts = vec![task.description.clone()];
    for key in [
        "context.summary",
        "context.source_titles",
        "rag.summary",
        "rag.source_titles",
    ] {
        if let Some(value) = task.context.metadata.get(key) {
            parts.push(value.clone());
        }
    }
    if let Some(assignment) = &task.assignment {
        parts.push(assignment.objective.clone());
        parts.extend(assignment.inputs.iter().map(|input| input.content.clone()));
        if let Some(instruction) = &assignment.rework_instruction {
            parts.push(instruction.clone());
        }
    }
    parts.join("\n")
}

pub(super) fn compact_task_text(value: &str) -> String {
    let trimmed = value.trim();
    const MAX_CHARS: usize = 80;
    if trimmed.chars().count() <= MAX_CHARS {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(MAX_CHARS).collect::<String>();
    output.push_str("...");
    output
}
