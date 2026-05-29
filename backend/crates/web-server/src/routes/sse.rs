use axum::{Extension, Router, routing::post};
use toolcraft_axum_kit::middleware::auth_mw::AuthUser;

use crate::handlers::sse::stream_agent_session;

pub fn sse_routes() -> Router {
    Router::new()
        .route("/agent/session/stream", post(stream_agent_session))
        .layer(Extension(local_test_auth_user()))
}

fn local_test_auth_user() -> AuthUser {
    AuthUser {
        user_id: "70a18d84-3072-4e80-9d33-40ee2c43f57c".to_string(),
        ext: None,
    }
}
