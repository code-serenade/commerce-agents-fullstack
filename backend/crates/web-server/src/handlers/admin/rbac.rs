use axum::{Extension, Json, extract::Path};
use db_core::error::BizError;
use error_code::admin as admin_error;
use service::api::admin::AdminApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    clients::auth_client,
    dto::admin::*,
    error::{Error, from_biz_error},
    statics::db_manager::get_admin_ctx,
};

fn map_role_response(role: service::dto::admin::RoleResponse) -> RoleResponse {
    RoleResponse {
        id: role.id,
        name: role.name,
        code: role.code,
        created_at: role.created_at,
    }
}

fn map_admin_user_response(
    admin_user: service::dto::admin::AdminUserResponse,
) -> AdminUserResponse {
    AdminUserResponse {
        user_id: admin_user.user_id,
        display_id: admin_user.display_id,
        display_name: admin_user.display_name,
        remark: admin_user.remark,
        status: match admin_user.status {
            service::dto::admin::AdminUserStatus::Enabled => AdminUserStatus::Enabled,
            service::dto::admin::AdminUserStatus::Disabled => AdminUserStatus::Disabled,
        },
        created_at: admin_user.created_at,
        updated_at: admin_user.updated_at,
        roles: admin_user
            .roles
            .into_iter()
            .map(map_role_response)
            .collect(),
    }
}

fn map_permission_kind(kind: service::dto::admin::PermissionKind) -> PermissionKind {
    match kind {
        service::dto::admin::PermissionKind::Group => PermissionKind::Group,
        service::dto::admin::PermissionKind::Action => PermissionKind::Action,
    }
}

fn map_permission_tree(node: service::dto::admin::PermissionTreeNode) -> PermissionTreeNode {
    PermissionTreeNode {
        id: node.id,
        name: node.name,
        kind: map_permission_kind(node.kind),
        created_at: node.created_at,
        updated_at: node.updated_at,
        children: node.children.into_iter().map(map_permission_tree).collect(),
    }
}

fn map_role_permission_tree(
    node: service::dto::admin::RolePermissionTreeNode,
) -> RolePermissionTreeNode {
    RolePermissionTreeNode {
        id: node.id,
        name: node.name,
        kind: map_permission_kind(node.kind),
        created_at: node.created_at,
        updated_at: node.updated_at,
        checked: node.checked,
        children: node
            .children
            .into_iter()
            .map(map_role_permission_tree)
            .collect(),
    }
}

pub async fn create_admin_user(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAdminUserRequest>,
) -> ResponseResult<AdminUserResponse> {
    req.validate().map_err(Error::from)?;
    let target_user = auth_client::get_user_by_identifier(&req.identifier)
        .await?
        .ok_or_else(|| {
            from_biz_error(BizError::new(
                admin_error::ADMIN_AUTH_USER_NOT_FOUND,
                format!("auth user not found: {}", req.identifier),
            ))
        })?;
    let api = AdminApi::new(get_admin_ctx());
    let admin_user = api
        .create_admin_user(
            auth_user.user_id,
            service::dto::admin::CreateAdminUserRequest {
                user_id: target_user.id,
                display_id: target_user
                    .display_user_id
                    .unwrap_or_else(|| target_user.username.clone()),
                display_name: target_user
                    .display_name
                    .unwrap_or_else(|| target_user.username.clone()),
                remark: req.remark,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_admin_user_response(admin_user)
        .into_common_response()
        .to_json())
}

pub async fn list_admin_users(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<AdminUserResponse>> {
    let api = AdminApi::new(get_admin_ctx());
    let admin_users = api
        .list_admin_users(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(admin_users
        .into_iter()
        .map(map_admin_user_response)
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn update_admin_user(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateAdminUserRequest>,
) -> ResponseResult<AdminUserResponse> {
    req.validate().map_err(Error::from)?;
    let api = AdminApi::new(get_admin_ctx());
    let admin_user = api
        .update_admin_user(
            auth_user.user_id,
            service::dto::admin::UpdateAdminUserRequest {
                user_id,
                remark: req.remark,
                status: match req.status {
                    AdminUserStatus::Enabled => service::dto::admin::AdminUserStatus::Enabled,
                    AdminUserStatus::Disabled => service::dto::admin::AdminUserStatus::Disabled,
                },
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_admin_user_response(admin_user)
        .into_common_response()
        .to_json())
}

pub async fn delete_admin_user(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> ResponseResult<()> {
    let api = AdminApi::new(get_admin_ctx());
    api.delete_admin_user(auth_user.user_id, user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn create_role(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateRoleRequest>,
) -> ResponseResult<RoleResponse> {
    req.validate().map_err(Error::from)?;
    let api = AdminApi::new(get_admin_ctx());
    let role = api
        .create_role(
            auth_user.user_id,
            service::dto::admin::CreateRoleRequest {
                name: req.name,
                code: req.code,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(RoleResponse {
        id: role.id,
        name: role.name,
        code: role.code,
        created_at: role.created_at,
    }
    .into_common_response()
    .to_json())
}

pub async fn list_roles(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<RoleResponse>> {
    let api = AdminApi::new(get_admin_ctx());
    let roles = api
        .list_roles(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(roles
        .into_iter()
        .map(|role| RoleResponse {
            id: role.id,
            name: role.name,
            code: role.code,
            created_at: role.created_at,
        })
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn delete_role(
    Extension(auth_user): Extension<AuthUser>,
    Path(role_id): Path<i64>,
) -> ResponseResult<()> {
    let api = AdminApi::new(get_admin_ctx());
    api.delete_role(auth_user.user_id, role_id)
        .await
        .map_err(from_biz_error)?;

    Ok(().into_common_response().to_json())
}

pub async fn create_menu(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateMenuRequest>,
) -> ResponseResult<MenuResponse> {
    req.validate().map_err(Error::from)?;
    let api = AdminApi::new(get_admin_ctx());
    let menu = api
        .create_menu(
            auth_user.user_id,
            service::dto::admin::CreateMenuRequest {
                name: req.name,
                parent_id: req.parent_id,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(MenuResponse {
        id: menu.id,
        name: menu.name,
        parent_id: menu.parent_id,
        created_at: menu.created_at,
        updated_at: menu.updated_at,
    }
    .into_common_response()
    .to_json())
}

pub async fn list_menus(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<MenuResponse>> {
    let api = AdminApi::new(get_admin_ctx());
    let menus = api
        .list_menus(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(menus
        .into_iter()
        .map(|menu| MenuResponse {
            id: menu.id,
            name: menu.name,
            parent_id: menu.parent_id,
            created_at: menu.created_at,
            updated_at: menu.updated_at,
        })
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn list_user_roles(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> ResponseResult<Vec<UserRoleOptionResponse>> {
    let api = AdminApi::new(get_admin_ctx());
    let roles = api
        .list_user_roles(auth_user.user_id, user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(roles
        .into_iter()
        .map(|role| UserRoleOptionResponse {
            id: role.id,
            name: role.name,
            code: role.code,
            created_at: role.created_at,
            checked: role.checked,
        })
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn update_user_roles(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRolesRequest>,
) -> ResponseResult<Vec<UserRoleOptionResponse>> {
    req.validate().map_err(Error::from)?;
    let api = AdminApi::new(get_admin_ctx());
    let roles = api
        .update_user_roles(
            auth_user.user_id,
            service::dto::admin::UpdateUserRolesRequest {
                user_id,
                role_ids: req.role_ids,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(roles
        .into_iter()
        .map(|role| UserRoleOptionResponse {
            id: role.id,
            name: role.name,
            code: role.code,
            created_at: role.created_at,
            checked: role.checked,
        })
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn list_permissions(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<PermissionTreeNode>> {
    let api = AdminApi::new(get_admin_ctx());
    let tree = api
        .list_permissions(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(tree
        .into_iter()
        .map(map_permission_tree)
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn list_role_permissions(
    Extension(auth_user): Extension<AuthUser>,
    Path(role_id): Path<i64>,
) -> ResponseResult<Vec<RolePermissionTreeNode>> {
    let api = AdminApi::new(get_admin_ctx());
    let tree = api
        .list_role_permissions(auth_user.user_id, role_id)
        .await
        .map_err(from_biz_error)?;

    Ok(tree
        .into_iter()
        .map(map_role_permission_tree)
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn update_role_permissions(
    Extension(auth_user): Extension<AuthUser>,
    Path(role_id): Path<i64>,
    Json(req): Json<UpdateRolePermissionsRequest>,
) -> ResponseResult<Vec<RolePermissionTreeNode>> {
    req.validate().map_err(Error::from)?;
    let api = AdminApi::new(get_admin_ctx());
    let tree = api
        .update_role_permissions(
            auth_user.user_id,
            service::dto::admin::UpdateRolePermissionsRequest {
                role_id,
                permission_ids: req.permission_ids,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(tree
        .into_iter()
        .map(map_role_permission_tree)
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

pub async fn current_user_permissions(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<CurrentUserPermissionsResponse> {
    let api = AdminApi::new(get_admin_ctx());
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

pub async fn current_user_menus(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<Vec<MenuTreeNode>> {
    let api = AdminApi::new(get_admin_ctx());
    let menus = api
        .get_current_user_menus(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(menus
        .into_iter()
        .map(map_menu_tree)
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

fn map_menu_tree(node: service::dto::admin::MenuTreeNode) -> MenuTreeNode {
    MenuTreeNode {
        id: node.id,
        name: node.name,
        parent_id: node.parent_id,
        created_at: node.created_at,
        updated_at: node.updated_at,
        children: node.children.into_iter().map(map_menu_tree).collect(),
    }
}
