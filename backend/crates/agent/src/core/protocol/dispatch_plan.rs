use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchPlannedStep {
    pub id: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchContextBudget {
    pub memory_record_count: usize,
    pub history_summary_available: bool,
    pub recent_history_message_count: usize,
    pub history_total_message_count: usize,
    pub history_truncated: bool,
    pub history_older_summary_available: bool,
    pub explicit_product_count: usize,
    pub explicit_context_ref_count: usize,
    pub workspace_context_ref_count: usize,
    pub workspace_pack_item_count: usize,
    pub approximate_input_chars: usize,
}
