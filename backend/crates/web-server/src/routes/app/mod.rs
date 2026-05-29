mod account;
mod approval_audit;
mod briefing;
mod campaign;
mod commerce_playbook;
mod package_governance;
mod platform_connect;
mod product;
mod workspace;

use axum::{Extension, Router};
use toolcraft_axum_kit::middleware::auth_mw::AuthUser;

pub fn app_routes() -> Router {
    Router::new().merge(
        Router::new()
            .merge(account::account_routes())
            .merge(product::product_routes())
            .merge(campaign::campaign_routes())
            .merge(commerce_playbook::commerce_playbook_routes())
            .merge(workspace::workspace_routes())
            .merge(approval_audit::approval_audit_routes())
            .merge(briefing::briefing_routes())
            .merge(package_governance::package_governance_routes())
            .merge(platform_connect::platform_connect_routes())
            .layer(Extension(local_test_auth_user())),
    )
}

fn local_test_auth_user() -> AuthUser {
    AuthUser {
        user_id: "70a18d84-3072-4e80-9d33-40ee2c43f57c".to_string(),
        ext: None,
    }
}
