use axum::{Router, routing::get};

use crate::handlers::admin::analytics::{
    get_analytics_overview, get_analytics_pages, get_analytics_sources, list_analytics_accounts,
    list_analytics_school_teams,
};

pub fn analytics_routes() -> Router {
    Router::new()
        .route("/overview", get(get_analytics_overview))
        .route("/sources", get(get_analytics_sources))
        .route("/pages", get(get_analytics_pages))
        .route("/accounts", get(list_analytics_accounts))
        .route("/school-teams", get(list_analytics_school_teams))
}
