use axum::{Router, routing::get};

use crate::handlers::app::{
    create_campaign, delete_campaign, get_campaign, list_campaigns, update_campaign,
};

pub fn campaign_routes() -> Router {
    Router::new()
        .route("/campaigns", get(list_campaigns).post(create_campaign))
        .route(
            "/campaigns/{campaign_id}",
            get(get_campaign)
                .patch(update_campaign)
                .delete(delete_campaign),
        )
}
