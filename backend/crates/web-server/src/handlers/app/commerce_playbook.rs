use axum::Extension;
use service::api::commerce_playbook::CommercePlaybookApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};

use crate::{
    dto::app::CommerceAgentPlaybookListResponse, error::from_biz_error,
    statics::db_manager::get_app_ctx,
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
