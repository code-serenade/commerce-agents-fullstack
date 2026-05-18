use axum::{Router, routing::get};

use crate::handlers::app::{get_external_search_status, query_external_search};

pub fn external_search_routes() -> Router {
    Router::new()
        .route("/external-search/status", get(get_external_search_status))
        .route("/external-search/search", get(query_external_search))
}
