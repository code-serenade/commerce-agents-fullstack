use axum::{
    Router,
    routing::{get, patch},
};

use crate::handlers::admin::usage::{
    get_team_usage, list_team_members, list_team_usages, update_team_quota,
};

pub fn usage_routes() -> Router {
    Router::new()
        .route("/teams", get(list_team_usages))
        .route("/teams/{team_id}", get(get_team_usage))
        .route("/teams/{team_id}/quota", patch(update_team_quota))
        .route("/teams/{team_id}/members", get(list_team_members))
}
