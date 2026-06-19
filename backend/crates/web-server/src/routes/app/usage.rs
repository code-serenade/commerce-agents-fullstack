use axum::{Router, routing::get};

use crate::handlers::app::get_current_team_usage_quota;

pub fn usage_routes() -> Router {
    Router::new().route("/usage/quota", get(get_current_team_usage_quota))
}
