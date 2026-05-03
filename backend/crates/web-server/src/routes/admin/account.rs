use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::handlers::admin::{
    app::{
        app_user_metrics, delete_app_user, list_app_users, list_user_roles as list_app_user_roles,
        update_app_user, update_user_roles as update_app_user_roles,
    },
    create_admin_user, delete_admin_user, list_admin_users,
    list_user_roles as list_admin_user_roles, update_admin_user,
    update_user_roles as update_admin_user_roles,
};

pub fn account_routes() -> Router {
    Router::new()
        .route(
            "/admin-users",
            post(create_admin_user).get(list_admin_users),
        )
        .route(
            "/admin-users/{user_id}",
            patch(update_admin_user).delete(delete_admin_user),
        )
        .route(
            "/admin-users/{user_id}/roles",
            get(list_admin_user_roles).put(update_admin_user_roles),
        )
        .route("/app-users", get(list_app_users))
        .route("/app-users/metrics", get(app_user_metrics))
        .route(
            "/app-users/{user_id}",
            patch(update_app_user).delete(delete_app_user),
        )
        .route(
            "/app-users/{user_id}/roles",
            get(list_app_user_roles).put(update_app_user_roles),
        )
}
