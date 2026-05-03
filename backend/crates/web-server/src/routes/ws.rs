use axum::{Router, routing::get};

use crate::handlers::ws::websocket;

pub fn ws_routes() -> Router {
    Router::new().route("/", get(websocket))
}
