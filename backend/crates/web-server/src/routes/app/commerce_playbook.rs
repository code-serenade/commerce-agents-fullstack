use axum::{Router, routing::get};

use crate::handlers::app::list_commerce_agent_playbooks;

pub fn commerce_playbook_routes() -> Router {
    Router::new().route("/agent/playbooks", get(list_commerce_agent_playbooks))
}
