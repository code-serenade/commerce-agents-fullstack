use axum::{Router, middleware::from_fn, routing::post};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::handlers::sse::stream_agent_session;

pub fn sse_routes() -> Router {
    Router::new()
        .route("/agent/session/stream", post(stream_agent_session))
        .route_layer(from_fn(auth::<VerifyJwt>))
}
