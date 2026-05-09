use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::app::{
    activate_package, deactivate_package, get_package, list_package_catalog, list_package_history,
    list_package_locks, package_lock_drift, rollback_package, save_package_locks, update_package,
    upsert_package, validate_package,
};

pub fn package_governance_routes() -> Router {
    Router::new()
        .route("/packages/catalog", get(list_package_catalog))
        .route("/packages", post(upsert_package))
        .route(
            "/packages/{package_type}/{package_id}",
            get(get_package).patch(update_package),
        )
        .route(
            "/packages/{package_type}/{package_id}/validate",
            get(validate_package),
        )
        .route(
            "/packages/{package_type}/{package_id}/activate",
            post(activate_package),
        )
        .route(
            "/packages/{package_type}/{package_id}/deactivate",
            post(deactivate_package),
        )
        .route(
            "/packages/{package_type}/{package_id}/rollback",
            post(rollback_package),
        )
        .route(
            "/packages/{package_type}/{package_id}/history",
            get(list_package_history),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/package-locks",
            get(list_package_locks).post(save_package_locks),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/package-locks/drift",
            get(package_lock_drift),
        )
}
