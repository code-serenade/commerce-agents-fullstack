use agent::api::RoleCatalog;
use axum::Extension;
use service::api::commerce_playbook::CommercePlaybookApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};

use crate::{
    dto::app::{
        AgentRuntimeChoiceResponse, AgentRuntimeModelOptionResponse, AgentRuntimeOptionsResponse,
        AgentRuntimeRoleOptionResponse, CommerceAgentPlaybookListResponse,
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
