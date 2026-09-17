use crate::handlers::app::{
    engage_learning_probe, get_learning_evidence, get_learning_probe_history,
    get_learning_probe_result, get_learning_profile, get_next_learning_probe, skip_learning_probe,
    submit_learning_probe_choice, update_learning_settings,
};
use axum::{
    Router,
    routing::{get, patch, post},
};

pub fn learning_routes() -> Router {
    Router::new()
        .route("/learning/profile", get(get_learning_profile))
        .route("/learning/evidence", get(get_learning_evidence))
        .route("/learning/settings", patch(update_learning_settings))
        .route("/learning/probes/next", get(get_next_learning_probe))
        .route("/learning/probes/history", get(get_learning_probe_history))
        .route(
            "/learning/probes/{probe_id}/engage",
            post(engage_learning_probe),
        )
        .route(
            "/learning/probes/{probe_id}/result",
            get(get_learning_probe_result),
        )
        .route(
            "/learning/probes/{probe_id}/answer-choice",
            post(submit_learning_probe_choice),
        )
        .route(
            "/learning/probes/{probe_id}/skip",
            post(skip_learning_probe),
        )
}
