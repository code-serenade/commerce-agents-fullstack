pub mod business {
    pub mod accounting;
    pub mod coordination;
    pub mod creative;
    pub mod data;
    pub mod design;
    pub mod douyin_ads;
    pub mod engineering;
    pub mod learning;
    pub mod ops;
    pub mod search;
    pub mod service;
    pub mod web;
}

pub mod core {
    pub mod spec;
}

pub mod execution;

pub use core::spec::{
    Skill, SkillCategory, SkillContext, SkillError, SkillInputField, SkillOutcome, SkillPriority,
    SkillResult, SkillSpec, bool_param, f64_param, i64_param, object_params, optional_bool_param,
    optional_f64_param, optional_i64_param, optional_string_param, optional_string_vec_param,
    optional_u64_param, param, string_param, string_vec_param, u64_param,
};

pub use business::{
    accounting, coordination, creative, data, design, douyin_ads, engineering, learning, ops,
    search, service, web,
};
pub use execution::{
    LocalSkillExecutor, SkillAdapterKind, SkillExecutionMode, SkillExecutionResult,
    SkillExecutionStatus, SkillExecutor, SkillExecutorSet, SkillId, SkillRegistry, SkillRequest,
    SkillRiskLevel, SkillRuntimeError, adapter_kinds_for_spec, execution_mode_for_spec,
    risk_level_for_spec,
};
pub use search::SearchAdapterConfig;

/// Specs that have been ported into Rust but are intentionally not registered
/// into the runtime tool graph yet.
pub fn staged_specs() -> Vec<SkillSpec> {
    let mut specs = Vec::new();
    specs.extend(accounting::specs());
    specs.extend(coordination::specs());
    specs.extend(creative::specs());
    specs.extend(data::specs());
    specs.extend(design::specs());
    specs.extend(douyin_ads::specs());
    specs.extend(engineering::specs());
    specs.extend(learning::specs());
    specs.extend(ops::specs());
    specs.extend(search::specs());
    specs.extend(service::specs());
    specs.extend(web::specs());
    specs
}
