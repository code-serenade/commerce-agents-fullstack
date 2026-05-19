use std::collections::BTreeMap;

use thiserror::Error;

use super::{
    classification::execution_mode_for_spec,
    types::{SkillExecutionMode, SkillId},
};
use crate::skills::{SkillCategory, SkillSpec, staged_specs};

#[derive(Debug, Error, PartialEq)]
pub enum SkillRuntimeError {
    #[error("duplicate skill id '{0}'")]
    DuplicateSkillId(String),
}

#[derive(Clone, Debug)]
pub struct SkillRegistry {
    specs: BTreeMap<SkillId, SkillSpec>,
}

impl SkillRegistry {
    pub fn staged() -> Result<Self, SkillRuntimeError> {
        Self::from_specs(staged_specs())
    }

    pub fn from_specs(specs: Vec<SkillSpec>) -> Result<Self, SkillRuntimeError> {
        let mut by_id = BTreeMap::new();
        for spec in specs {
            let id = SkillId::new(spec.id.clone());
            if by_id.insert(id.clone(), spec).is_some() {
                return Err(SkillRuntimeError::DuplicateSkillId(id.0));
            }
        }
        Ok(Self { specs: by_id })
    }

    pub fn get(&self, id: impl Into<SkillId>) -> Option<&SkillSpec> {
        self.specs.get(&id.into())
    }

    pub fn all(&self) -> Vec<&SkillSpec> {
        self.specs.values().collect()
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    pub fn by_category(&self, category: &SkillCategory) -> Vec<&SkillSpec> {
        self.specs
            .values()
            .filter(|spec| &spec.category == category)
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&SkillSpec> {
        self.specs
            .values()
            .filter(|spec| spec.tags.iter().any(|value| value == tag))
            .collect()
    }

    pub fn by_execution_mode(&self, mode: SkillExecutionMode) -> Vec<&SkillSpec> {
        self.specs
            .values()
            .filter(|spec| execution_mode_for_spec(spec) == mode)
            .collect()
    }
}
