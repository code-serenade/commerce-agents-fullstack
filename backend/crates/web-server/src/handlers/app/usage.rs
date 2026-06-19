use axum::Extension;
use service::api::team_usage::TeamUsageApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};

use crate::{
    dto::team_usage::{CurrentTeamUsageQuotaResponse, TeamUsageStatus},
    error::from_biz_error,
    statics::db_manager::get_app_ctx,
};

pub async fn get_current_team_usage_quota(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<CurrentTeamUsageQuotaResponse> {
    let api = TeamUsageApi::new(get_app_ctx());
    let quota = api
        .get_current_team_usage_quota(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(map_current_team_usage_quota_response(quota)
        .into_common_response()
        .to_json())
}

fn map_current_team_usage_quota_response(
    quota: service::dto::team_usage::CurrentTeamUsageQuotaResponse,
) -> CurrentTeamUsageQuotaResponse {
    CurrentTeamUsageQuotaResponse {
        user_id: quota.user_id,
        display_id: quota.display_id,
        display_name: quota.display_name,
        school_name: quota.school_name,
        team_name: quota.team_name,
        question_limit: quota.question_limit,
        question_used: quota.question_used,
        question_remaining: quota.question_remaining,
        status: match quota.status {
            service::dto::team_usage::TeamUsageQuotaStatus::Enabled => TeamUsageStatus::Enabled,
            service::dto::team_usage::TeamUsageQuotaStatus::Disabled => TeamUsageStatus::Disabled,
        },
        policy_text: quota.policy_text,
    }
}
