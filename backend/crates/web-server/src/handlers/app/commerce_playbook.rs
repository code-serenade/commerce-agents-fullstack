use agent::api::{
    RoleCatalog, RoleSkillCatalog, SkillAdapterKind, SkillCategory, SkillExecutionMode,
    SkillInputField, SkillPriority, SkillRegistry, SkillRiskLevel, SkillSpec,
    adapter_kinds_for_spec, execution_mode_for_spec, risk_level_for_spec,
};
use axum::Extension;
use service::api::commerce_playbook::CommercePlaybookApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};

use crate::{
    dto::app::{
        AgentCapabilityCatalogResponse, AgentCapabilityCatalogSummaryResponse,
        AgentCapabilityRoleResponse, AgentCapabilitySkillInputResponse,
        AgentCapabilitySkillResponse, AgentRuntimeChoiceResponse, AgentRuntimeModelOptionResponse,
        AgentRuntimeOptionsResponse, AgentRuntimeRoleOptionResponse,
        CommerceAgentPlaybookListResponse,
    },
    error::from_biz_error,
    statics::{
        agent_manager::{list_agent_runtime_models, select_agent_runtime_model},
        db_manager::get_app_ctx,
    },
};

pub async fn list_commerce_agent_playbooks(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<CommerceAgentPlaybookListResponse> {
    let api = CommercePlaybookApi::new(get_app_ctx());
    let response = api
        .list_agent_playbooks(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn get_agent_runtime_options(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<AgentRuntimeOptionsResponse> {
    let api = CommercePlaybookApi::new(get_app_ctx());
    api.ensure_current_user_enabled(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    let models = list_agent_runtime_models()?;
    let default_model = select_agent_runtime_model(None)?;
    let roles = RoleCatalog::configured()
        .roles()
        .iter()
        .map(|role| {
            let worker = role.to_worker_profile();
            AgentRuntimeRoleOptionResponse {
                id: role.id.0.clone(),
                name: role.name.clone(),
                runtime_role: role.runtime_role.clone(),
                capabilities: worker.capabilities,
                is_default: role.runtime_role == "chat",
            }
        })
        .collect::<Vec<_>>();

    Ok(AgentRuntimeOptionsResponse {
        default_model_name: Some(default_model.selected_name),
        models: models
            .into_iter()
            .map(|model| AgentRuntimeModelOptionResponse {
                name: model.name,
                model: model.model,
                is_default: model.is_default,
            })
            .collect(),
        strategies: runtime_choices(&[
            (
                "execution",
                "快速回答",
                "直接给出可执行答案和下一步动作。",
                false,
            ),
            (
                "learning",
                "边讲边教",
                "解释判断过程，同时保留电商业务动作。",
                false,
            ),
            (
                "collab",
                "Agent 集群",
                "调度多个电商虚拟员工分工处理并汇总。",
                true,
            ),
        ]),
        response_modes: runtime_choices(&[
            ("execution", "执行导向", "先输出业务答案和动作清单。", true),
            ("learning", "教学导向", "补充推理路径和学习说明。", false),
        ]),
        learning_levels: runtime_choices(&[
            ("beginner", "入门", "减少术语，强调操作顺序。", false),
            (
                "higher_vocational",
                "实操",
                "按电商岗位实操标准解释。",
                true,
            ),
            (
                "professional",
                "专业",
                "保留更完整的策略、指标和风险推导。",
                false,
            ),
        ]),
        collaboration_modes: runtime_choices(&[
            ("auto", "自动调度", "后端按任务自动选择岗位。", false),
            ("single", "指定单岗", "用户指定一个主岗位回答。", false),
            ("manual", "指定多岗", "用户指定多个岗位协同执行。", true),
        ]),
        roles,
    }
    .into_common_response()
    .to_json())
}

pub async fn get_agent_capability_catalog(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<AgentCapabilityCatalogResponse> {
    let api = CommercePlaybookApi::new(get_app_ctx());
    api.ensure_current_user_enabled(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    let role_catalog = RoleCatalog::configured();
    let skill_catalog = RoleSkillCatalog::staged();
    let registry = SkillRegistry::staged().map_err(|error| {
        crate::error::Error::Custom(format!("failed to load skill registry: {error}"))
    })?;
    let roles = role_catalog
        .roles()
        .iter()
        .map(|role| {
            let skills = skill_catalog
                .recommended_for_role(role)
                .into_iter()
                .map(map_skill)
                .collect::<Vec<_>>();
            AgentCapabilityRoleResponse {
                id: role.id.0.clone(),
                name: role.name.clone(),
                runtime_role: role.runtime_role.clone(),
                priority: role.priority,
                domains: role.domains.clone(),
                keywords: role.keywords.clone(),
                preferred_actions: role
                    .preferred_actions
                    .iter()
                    .map(|action| action.as_str().to_string())
                    .collect(),
                required_capabilities: role.required_capabilities.clone(),
                optional_capabilities: role.optional_capabilities.clone(),
                skill_count: skills.len(),
                skills,
            }
        })
        .collect::<Vec<_>>();

    Ok(AgentCapabilityCatalogResponse {
        source: "agent_runtime".to_string(),
        version: "post_8_137".to_string(),
        summary: capability_summary(&registry),
        roles,
    }
    .into_common_response()
    .to_json())
}

fn runtime_choices(items: &[(&str, &str, &str, bool)]) -> Vec<AgentRuntimeChoiceResponse> {
    items
        .iter()
        .map(
            |(id, label, description, is_default)| AgentRuntimeChoiceResponse {
                id: (*id).to_string(),
                label: (*label).to_string(),
                description: (*description).to_string(),
                is_default: *is_default,
            },
        )
        .collect()
}

fn capability_summary(registry: &SkillRegistry) -> AgentCapabilityCatalogSummaryResponse {
    let mut native_skill_count = 0;
    let mut adapter_skill_count = 0;
    let mut approval_skill_count = 0;
    for spec in registry.all() {
        match execution_mode_for_spec(spec) {
            SkillExecutionMode::Native => native_skill_count += 1,
            SkillExecutionMode::Adapter | SkillExecutionMode::Deferred => {
                adapter_skill_count += 1;
            }
        }
        if risk_level_for_spec(spec) == SkillRiskLevel::High {
            approval_skill_count += 1;
        }
    }

    AgentCapabilityCatalogSummaryResponse {
        role_count: RoleCatalog::configured().roles().len(),
        skill_count: registry.len(),
        native_skill_count,
        adapter_skill_count,
        approval_skill_count,
    }
}

fn map_skill(spec: &SkillSpec) -> AgentCapabilitySkillResponse {
    let execution_mode = execution_mode_for_spec(spec);
    let risk_level = risk_level_for_spec(spec);
    AgentCapabilitySkillResponse {
        id: spec.id.clone(),
        name: spec.name.clone(),
        description: spec.description.clone(),
        category: category_label(&spec.category),
        priority: priority_label(spec.priority),
        execution_mode: execution_mode_label(execution_mode),
        risk_level: risk_level_label(risk_level),
        approval_required: risk_level == SkillRiskLevel::High,
        adapter_requirements: adapter_kinds_for_spec(spec)
            .into_iter()
            .map(adapter_kind_label)
            .collect(),
        tags: spec.tags.clone(),
        inputs: spec.inputs.iter().map(map_skill_input).collect(),
    }
}

fn map_skill_input(input: &SkillInputField) -> AgentCapabilitySkillInputResponse {
    AgentCapabilitySkillInputResponse {
        name: input.name.clone(),
        description: input.description.clone(),
        required: input.required,
        schema: input.schema.clone(),
        default: input.default.clone(),
    }
}

fn category_label(category: &SkillCategory) -> String {
    match category {
        SkillCategory::General => "general",
        SkillCategory::Planning => "planning",
        SkillCategory::Research => "research",
        SkillCategory::Coding => "coding",
        SkillCategory::Filesystem => "filesystem",
        SkillCategory::Shell => "shell",
        SkillCategory::Browser => "browser",
        SkillCategory::Memory => "memory",
        SkillCategory::Communication => "communication",
        SkillCategory::Data => "data",
        SkillCategory::Custom(value) => value.as_str(),
    }
    .to_string()
}

fn priority_label(priority: SkillPriority) -> String {
    match priority {
        SkillPriority::Low => "low",
        SkillPriority::Normal => "normal",
        SkillPriority::High => "high",
        SkillPriority::Critical => "critical",
    }
    .to_string()
}

fn execution_mode_label(mode: SkillExecutionMode) -> String {
    match mode {
        SkillExecutionMode::Native => "native",
        SkillExecutionMode::Deferred => "deferred",
        SkillExecutionMode::Adapter => "adapter",
    }
    .to_string()
}

fn risk_level_label(level: SkillRiskLevel) -> String {
    match level {
        SkillRiskLevel::Low => "low",
        SkillRiskLevel::Medium => "medium",
        SkillRiskLevel::High => "high",
    }
    .to_string()
}

fn adapter_kind_label(kind: SkillAdapterKind) -> String {
    match kind {
        SkillAdapterKind::Search => "search",
        SkillAdapterKind::Llm => "llm",
        SkillAdapterKind::Db => "db",
        SkillAdapterKind::Platform => "platform",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_capability_catalog_contains_real_roles_and_skills() {
        let role_catalog = RoleCatalog::configured();
        let skill_catalog = RoleSkillCatalog::staged();
        let registry = SkillRegistry::staged().expect("skill registry should load");

        let summary = capability_summary(&registry);
        assert!(summary.role_count >= 10);
        assert!(summary.skill_count >= 100);
        assert!(summary.native_skill_count > summary.adapter_skill_count);
        assert!(summary.approval_skill_count >= 3);

        let data_role = role_catalog
            .roles()
            .iter()
            .find(|role| role.runtime_role == "data")
            .expect("data role should exist");
        let data_skills = skill_catalog
            .recommended_for_role(data_role)
            .into_iter()
            .map(map_skill)
            .collect::<Vec<_>>();

        assert!(
            data_skills
                .iter()
                .any(|skill| skill.id == "data_comprehensive_diagnosis")
        );
        assert!(
            data_skills
                .iter()
                .any(|skill| skill.id == "learning_teaching_explain")
        );
        assert!(
            data_skills
                .iter()
                .all(|skill| !skill.name.trim().is_empty())
        );
    }
}
