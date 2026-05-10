use axum::{Router, routing::get};

use crate::handlers::app::{get_agent_runtime_options, list_commerce_agent_playbooks};

pub fn commerce_playbook_routes() -> Router {
    Router::new()
        .route("/agent/playbooks", get(list_commerce_agent_playbooks))
        .route("/agent/runtime-options", get(get_agent_runtime_options))
}
