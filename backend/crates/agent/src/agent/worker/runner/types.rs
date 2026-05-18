use crate::action::{Action, ActionResult};

pub type BoxedAction = Box<dyn Action<Result = ActionResult<String, String, ()>> + Send>;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ParsedSkillRequest {
    pub(super) skill_id: String,
    pub(super) input: serde_json::Value,
    pub(super) reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct ParsedWorkerRoleOutput {
    pub(super) content: String,
    pub(super) role_output: Option<crate::core::protocol::RoleWorkOutput>,
    pub(super) evidence: Vec<String>,
    pub(super) risks: Vec<String>,
    pub(super) open_questions: Vec<String>,
}
