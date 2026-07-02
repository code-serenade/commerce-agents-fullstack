use axum::{Extension, Json};
use service::api::analytics::AnalyticsApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::analytics::{TrackAnalyticsEventsRequest, TrackAnalyticsEventsResponse},
    error::from_biz_error,
    handlers::analytics::{
        analytics_request_invalid, map_track_event_to_service, map_track_response,
    },
    statics::db_manager::get_app_ctx,
};

pub async fn track_app_user_events(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<TrackAnalyticsEventsRequest>,
) -> ResponseResult<TrackAnalyticsEventsResponse> {
    req.validate().map_err(analytics_request_invalid)?;
    for event in &req.events {
        event.validate().map_err(analytics_request_invalid)?;
    }

    let api = AnalyticsApi::new(get_app_ctx());
    let resp = api
        .track_app_user_events(
            auth_user.user_id,
            req.events
                .into_iter()
                .map(map_track_event_to_service)
                .collect(),
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_track_response(resp).into_common_response().to_json())
}
