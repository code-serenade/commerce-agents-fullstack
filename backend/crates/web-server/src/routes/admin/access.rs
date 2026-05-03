use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::handlers::admin::{
    app::{
        create_role as create_app_role, delete_role as delete_app_role,
        list_permissions as list_app_permissions,
        list_role_permissions as list_app_role_permissions, list_roles as list_app_roles,
        update_role_permissions as update_app_role_permissions,
    },
    create_menu, create_role as create_admin_role, delete_role as delete_admin_role, list_menus,
    list_permissions as list_admin_permissions,
    list_role_permissions as list_admin_role_permissions, list_roles as list_admin_roles,
    update_role_permissions as update_admin_role_permissions,
};

pub fn access_routes() -> Router {
    Router::new()
        .route("/roles", post(create_admin_role).get(list_admin_roles))
        .route("/roles/{role_id}", delete(delete_admin_role))
        .route(
            "/roles/{role_id}/permissions",
            get(list_admin_role_permissions).put(update_admin_role_permissions),
        )
        .route("/permissions", get(list_admin_permissions))
        .route("/menus", post(create_menu).get(list_menus))
        .route("/app-roles", post(create_app_role).get(list_app_roles))
        .route("/app-roles/{role_id}", delete(delete_app_role))
        .route(
            "/app-roles/{role_id}/permissions",
            get(list_app_role_permissions).put(update_app_role_permissions),
        )
        .route("/app-permissions", get(list_app_permissions))
}
