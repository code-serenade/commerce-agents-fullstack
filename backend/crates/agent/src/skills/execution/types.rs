use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::classification::{add_runtime_metadata, status_from_output};
use crate::skills::{SkillContext, SkillOutcome, SkillSpec};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SkillId(pub String);

impl SkillId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SkillId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SkillId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillRequest {
    pub request_id: String,
    pub skill_id: SkillId,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub context: SkillContext,
}

impl SkillRequest {
    pub fn new(request_id: impl Into<String>, skill_id: impl Into<SkillId>, input: Value) -> Self {
        Self {
            request_id: request_id.into(),
            skill_id: skill_id.into(),
            input,
            context: SkillContext::default(),
        }
    }

    pub fn with_context(mut self, context: SkillContext) -> Self {
        self.context = context;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillExecutionStatus {
    Success,
    Deferred,
    PendingApproval,
    NotFound,
    ValidationError,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillRiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillExecutionMode {
    Native,
    Deferred,
    Adapter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillAdapterKind {
    Search,
    Llm,
    Db,
    Platform,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    pub request_id: String,
    pub skill_id: SkillId,
    pub status: SkillExecutionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SkillExecutionResult {
    pub(super) fn from_outcome(
        request: &SkillRequest,
        spec: &SkillSpec,
        outcome: SkillOutcome,
        mode: SkillExecutionMode,
    ) -> Self {
        let mut metadata = outcome.metadata;
        add_runtime_metadata(&mut metadata, spec, mode);
        let status = status_from_output(&outcome.output).unwrap_or(SkillExecutionStatus::Success);
        Self {
            request_id: request.request_id.clone(),
            skill_id: request.skill_id.clone(),
            status,
            output: Some(outcome.output),
            summary: outcome.summary,
            metadata,
            error: None,
        }
    }

    pub(super) fn error(
        request: &SkillRequest,
        status: SkillExecutionStatus,
        message: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            skill_id: request.skill_id.clone(),
            status,
            output: None,
            summary: None,
            metadata: Map::new(),
            error: Some(message.into()),
        }
    }
}
