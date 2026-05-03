mod admin;
mod app;
mod sse;
mod ws;

use std::sync::Arc;

use admin::admin_routes;
use app::app_routes;
use axum::{Extension, Router};
use sse::sse_routes;
use toolcraft_axum_kit::middleware::cors::create_cors;
use toolcraft_jwt::VerifyJwt;
use ws::ws_routes;

pub fn create_routes(jwt: Arc<VerifyJwt>) -> Router {
    let cors = create_cors();
    let api_routes = Router::new()
        .nest("/admin", admin_routes())
        .nest("/app", app_routes())
        .nest("/sse", sse_routes())
        .nest("/ws", ws_routes());

    Router::new()
        .nest("/api", api_routes)
        .layer(Extension(jwt))
        .layer(cors)
}
