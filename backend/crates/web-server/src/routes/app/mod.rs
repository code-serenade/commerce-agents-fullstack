mod account;
mod approval_audit;
mod briefing;
mod campaign;
mod package_governance;
mod platform_connect;
mod product;
mod workspace;

use axum::{Router, middleware::from_fn};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

pub fn app_routes() -> Router {
    Router::new().merge(
        Router::new()
            .merge(account::account_routes())
            .merge(product::product_routes())
            .merge(campaign::campaign_routes())
            .merge(workspace::workspace_routes())
            .merge(approval_audit::approval_audit_routes())
            .merge(briefing::briefing_routes())
            .merge(package_governance::package_governance_routes())
            .merge(platform_connect::platform_connect_routes())
            .route_layer(from_fn(auth::<VerifyJwt>)),
    )
}
