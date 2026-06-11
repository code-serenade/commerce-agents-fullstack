use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::app::{
    create_campaign, create_campaign_draft, create_campaign_form_draft, delete_campaign,
    get_campaign, import_campaign_template, list_campaign_templates, list_campaigns,
    update_campaign,
};

pub fn campaign_routes() -> Router {
    Router::new()
        .route("/campaigns", get(list_campaigns).post(create_campaign))
        .route("/campaigns/templates", get(list_campaign_templates))
        .route(
            "/campaigns/templates/import",
            post(import_campaign_template),
        )
        .route("/campaigns/form-draft", post(create_campaign_form_draft))
        .route("/campaigns/draft", post(create_campaign_draft))
        .route(
            "/campaigns/{campaign_id}",
            get(get_campaign)
                .patch(update_campaign)
                .delete(delete_campaign),
        )
}
