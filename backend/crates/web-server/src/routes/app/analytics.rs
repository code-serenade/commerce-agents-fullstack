use axum::{Router, routing::post};

use crate::handlers::app::track_app_user_events;

pub fn analytics_routes() -> Router {
    Router::new().route("/analytics/events", post(track_app_user_events))
}
