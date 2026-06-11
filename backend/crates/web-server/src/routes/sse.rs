use axum::{Router, middleware::from_fn, routing::post};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::handlers::sse::{stream_agent_session, stream_form_draft};

pub fn sse_routes() -> Router {
    Router::new()
        .route("/agent/session/stream", post(stream_agent_session))
        .route("/form-draft/stream", post(stream_form_draft))
        .route_layer(from_fn(auth::<VerifyJwt>))
}
