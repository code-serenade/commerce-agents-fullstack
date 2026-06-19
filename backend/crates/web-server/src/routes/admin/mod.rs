mod access;
mod account;
mod rbac;
mod usage;

use axum::Router;

pub fn admin_routes() -> Router {
    rbac::admin_routes()
}
