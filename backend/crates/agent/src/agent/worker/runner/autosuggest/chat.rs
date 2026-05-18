fn autosuggest_chat(text: &str, _task: &Task) -> Option<ParsedSkillRequest> {
    if has_chat_save_memory_intent(text) {
        return Some(autosuggest_request(
            chat_save_memory_skill_id(),
            serde_json::json!({
                "key": infer_memory_key(text),
                "content": compact_task_text(text),
                "memory_type": infer_memory_type(text),
            }),
            "根据记忆保存意图自动补偿协调保存记忆 skill",
        ));
    }
    if has_chat_status_query_intent(text) {
        return Some(autosuggest_request(
            chat_status_query_skill_id(),
            serde_json::json!({
                "query_type": infer_status_query_type_from_text(text),
            }),
            "根据状态/进度/会话可见性意图自动补偿协调状态查询 skill",
        ));
    }
    None
}
