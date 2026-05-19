use serde_json::{Value, json};

use crate::skills::{
    SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillSpec,
};

include!("specs.rs");
include!("executor.rs");
include!("core_ops.rs");
include!("optimization.rs");
include!("common.rs");
