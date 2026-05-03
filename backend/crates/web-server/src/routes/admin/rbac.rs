use axum::{Router, middleware::from_fn, routing::get};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::{
    handlers::admin::{current_user_menus, current_user_permissions},
    routes::admin::{access::access_routes, account::account_routes},
};

pub fn admin_routes() -> Router {
    Router::new()
        .nest("/account", account_routes())
        .nest("/access", access_routes())
        .route("/me/permissions", get(current_user_permissions))
        .route("/me/menus", get(current_user_menus))
        .route_layer(from_fn(auth::<VerifyJwt>))
}
