use axum::{Extension, extract::Query};
use service::api::external_search::ExternalSearchApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::app::{ExternalSearchQuery, ExternalSearchResponse, ExternalSearchStatusResponse},
    error::{Error, from_biz_error},
    statics::{db_manager::get_app_ctx, external_search::get_external_search_config},
};

pub async fn get_external_search_status(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<ExternalSearchStatusResponse> {
    let api = ExternalSearchApi::new(get_app_ctx(), get_external_search_config());
    let response = api
        .status(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}

pub async fn query_external_search(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<ExternalSearchQuery>,
) -> ResponseResult<ExternalSearchResponse> {
    req.validate().map_err(Error::from)?;
    let api = ExternalSearchApi::new(get_app_ctx(), get_external_search_config());
    let response = api
        .query_external_search(service::dto::external_search::ExternalSearchQueryRequest {
            user_id: auth_user.user_id,
            q: req.q,
            topic: req.topic,
            days: req.days,
            max_results: req.max_results,
            bypass_cache: req.bypass_cache,
        })
        .await
        .map_err(from_biz_error)?;

    Ok(response.into_common_response().to_json())
}
