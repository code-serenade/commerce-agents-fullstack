use axum::{Router, middleware::from_fn, routing::post};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::handlers::sse::stream_llm_chat;

pub fn sse_routes() -> Router {
    Router::new()
        .route("/llm/chat/stream", post(stream_llm_chat))
        .route_layer(from_fn(auth::<VerifyJwt>))
}
