use axum::{Extension, Json};
use service::api::app::AppApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::app::{AppUserResponse, CurrentUserPermissionsResponse, RegisterAppUserRequest},
    error::{Error, from_biz_error},
    statics::db_manager::get_app_ctx,
};

fn map_role_response(role: service::dto::app::RoleResponse) -> crate::dto::app::RoleResponse {
    crate::dto::app::RoleResponse {
        id: role.id,
        name: role.name,
        code: role.code,
        created_at: role.created_at,
    }
}

fn map_app_user_response(app_user: service::dto::app::AppUserResponse) -> AppUserResponse {
    AppUserResponse {
        user_id: app_user.user_id,
        display_id: app_user.display_id,
        display_name: app_user.display_name,
        remark: app_user.remark,
        status: match app_user.status {
            service::dto::app::AppUserStatus::Enabled => crate::dto::app::AppUserStatus::Enabled,
            service::dto::app::AppUserStatus::Disabled => crate::dto::app::AppUserStatus::Disabled,
        },
        created_at: app_user.created_at,
        updated_at: app_user.updated_at,
        roles: app_user.roles.into_iter().map(map_role_response).collect(),
    }
}

pub async fn register_app_user(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<RegisterAppUserRequest>,
) -> ResponseResult<AppUserResponse> {
    req.validate().map_err(Error::from)?;
    let api = AppApi::new(get_app_ctx());
    let app_user = api
        .register_app_user(service::dto::app::RegisterAppUserRequest {
            user_id: auth_user.user_id,
            display_id: req.display_id,
            display_name: req.display_name,
            remark: req.remark,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(map_app_user_response(app_user)
        .into_common_response()
        .to_json())
}

pub async fn current_user_permissions(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<CurrentUserPermissionsResponse> {
    let api = AppApi::new(get_app_ctx());
    let resp = api
        .get_current_user_permissions(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(CurrentUserPermissionsResponse {
        user_id: resp.user_id,
        role_codes: resp.role_codes,
        permission_codes: resp.permission_codes,
    }
    .into_common_response()
    .to_json())
}
