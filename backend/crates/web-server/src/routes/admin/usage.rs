use axum::{
    Router,
    routing::{get, patch},
};

use crate::handlers::admin::usage::{get_team_usage, list_team_usages, update_team_quota};

pub fn usage_routes() -> Router {
    Router::new()
        .route("/accounts", get(list_team_usages))
        .route("/accounts/{usage_id}", get(get_team_usage))
        .route("/accounts/{usage_id}/quota", patch(update_team_quota))
        .route("/teams", get(list_team_usages))
        .route("/teams/{usage_id}", get(get_team_usage))
        .route("/teams/{usage_id}/quota", patch(update_team_quota))
}
