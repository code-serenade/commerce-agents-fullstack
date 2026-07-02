use axum::{Router, routing::post};

use crate::handlers::analytics::track_anonymous_events;

pub fn analytics_routes() -> Router {
    Router::new().route("/events", post(track_anonymous_events))
}
