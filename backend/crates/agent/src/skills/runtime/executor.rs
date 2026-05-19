use async_trait::async_trait;
use serde_json::Value;

use super::{
    classification::execution_mode_for_spec,
    registry::{SkillRegistry, SkillRuntimeError},
    router::run_known_skill,
    types::{SkillExecutionResult, SkillExecutionStatus, SkillRequest},
};
use crate::skills::{SearchAdapterConfig, SkillError, SkillSpec};

#[async_trait]
pub trait SkillExecutor: Send + Sync {
    async fn execute(&self, request: SkillRequest) -> SkillExecutionResult;
}

#[derive(Clone, Debug)]
pub struct LocalSkillExecutor {
    registry: SkillRegistry,
    search: SearchAdapterConfig,
}

impl LocalSkillExecutor {
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            registry,
            search: SearchAdapterConfig::default(),
        }
    }

    pub fn new_with_search(registry: SkillRegistry, search: SearchAdapterConfig) -> Self {
        Self { registry, search }
    }

    pub fn staged() -> Result<Self, SkillRuntimeError> {
        Ok(Self::new(SkillRegistry::staged()?))
    }

    pub fn staged_with_search(search: SearchAdapterConfig) -> Result<Self, SkillRuntimeError> {
        Ok(Self::new_with_search(SkillRegistry::staged()?, search))
    }

    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }
}

#[async_trait]
impl SkillExecutor for LocalSkillExecutor {
    async fn execute(&self, request: SkillRequest) -> SkillExecutionResult {
        if let Err(message) = validate_request_envelope(&request) {
            return SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::ValidationError,
                message,
            );
        }

        let Some(spec) = self.registry.get(request.skill_id.clone()) else {
            return SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::NotFound,
                "skill id not found",
            );
        };

        if let Err(error) = validate_request(spec, &request.input) {
            return SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::ValidationError,
                error.to_string(),
            );
        }

        let mode = execution_mode_for_spec(spec);
        let outcome = run_known_skill(
            spec.id.as_str(),
            request.input.clone(),
            request.context.clone(),
            &self.search,
        )
        .await;
        match outcome {
            Ok(outcome) => SkillExecutionResult::from_outcome(&request, spec, outcome, mode),
            Err(error) => SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::Failed,
                error.to_string(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillExecutorSet {
    local: LocalSkillExecutor,
}

impl SkillExecutorSet {
    pub fn staged() -> Result<Self, SkillRuntimeError> {
        Ok(Self {
            local: LocalSkillExecutor::staged()?,
        })
    }

    pub fn staged_with_search(search: SearchAdapterConfig) -> Result<Self, SkillRuntimeError> {
        Ok(Self {
            local: LocalSkillExecutor::staged_with_search(search)?,
        })
    }

    pub fn registry(&self) -> &SkillRegistry {
        self.local.registry()
    }
}

#[async_trait]
impl SkillExecutor for SkillExecutorSet {
    async fn execute(&self, request: SkillRequest) -> SkillExecutionResult {
        self.local.execute(request).await
    }
}

fn validate_request_envelope(request: &SkillRequest) -> Result<(), &'static str> {
    if request.request_id.trim().is_empty() {
        return Err("request_id is required");
    }
    if request.skill_id.as_str().trim().is_empty() {
        return Err("skill_id is required");
    }
    Ok(())
}

fn validate_request(spec: &SkillSpec, input: &Value) -> Result<(), SkillError> {
    let object = input
        .as_object()
        .ok_or_else(|| SkillError::invalid_parameter("$", "object"))?;
    for field in &spec.inputs {
        if field.required && !object.contains_key(&field.name) {
            return Err(SkillError::missing_parameter(field.name.clone()));
        }
    }
    Ok(())
}
