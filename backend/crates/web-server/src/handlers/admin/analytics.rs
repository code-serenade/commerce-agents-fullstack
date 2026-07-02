use axum::{Extension, extract::Query};
use service::api::analytics::AnalyticsApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::{
        analytics::{
            AnalyticsOverviewResponse, AnalyticsPagesResponse, AnalyticsRangeQuery,
            AnalyticsSourcesResponse, ListAnalyticsAccountsQuery, ListAnalyticsSchoolTeamsQuery,
        },
        common::PageResponse,
    },
    error::from_biz_error,
    handlers::analytics::{
        analytics_request_invalid, map_account_row, map_account_sort_by_to_service,
        map_granularity_to_service, map_overview_response, map_pages_response, map_school_team_row,
        map_school_team_sort_by_to_service, map_sort_order_to_service, map_sources_response,
    },
    statics::db_manager::get_app_ctx,
};

pub async fn get_analytics_overview(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> ResponseResult<AnalyticsOverviewResponse> {
    query.validate().map_err(analytics_request_invalid)?;
    let api = AnalyticsApi::new(get_app_ctx());
    let resp = api
        .get_overview(
            auth_user.user_id,
            query.from,
            query.to,
            query.granularity.map(map_granularity_to_service),
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_overview_response(resp).into_common_response().to_json())
}

pub async fn get_analytics_sources(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> ResponseResult<AnalyticsSourcesResponse> {
    query.validate().map_err(analytics_request_invalid)?;
    let api = AnalyticsApi::new(get_app_ctx());
    let resp = api
        .get_sources(auth_user.user_id, query.from, query.to)
        .await
        .map_err(from_biz_error)?;

    Ok(map_sources_response(resp).into_common_response().to_json())
}

pub async fn get_analytics_pages(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> ResponseResult<AnalyticsPagesResponse> {
    query.validate().map_err(analytics_request_invalid)?;
    let api = AnalyticsApi::new(get_app_ctx());
    let resp = api
        .get_pages(auth_user.user_id, query.from, query.to)
        .await
        .map_err(from_biz_error)?;

    Ok(map_pages_response(resp).into_common_response().to_json())
}

pub async fn list_analytics_accounts(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListAnalyticsAccountsQuery>,
) -> ResponseResult<PageResponse<crate::dto::analytics::AnalyticsAccountRow>> {
    query.validate().map_err(analytics_request_invalid)?;
    let api = AnalyticsApi::new(get_app_ctx());
    let rows = api
        .list_accounts(
            auth_user.user_id,
            service::dto::analytics::ListAnalyticsAccountsRequest {
                page: query.page,
                page_size: query.page_size,
                keyword: query.keyword,
                school_name: query.school_name,
                team_name: query.team_name,
                from: query.from,
                to: query.to,
                sort_by: query.sort_by.map(map_account_sort_by_to_service),
                sort_order: query.sort_order.map(map_sort_order_to_service),
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(PageResponse::from(rows.map(map_account_row))
        .into_common_response()
        .to_json())
}

pub async fn list_analytics_school_teams(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListAnalyticsSchoolTeamsQuery>,
) -> ResponseResult<PageResponse<crate::dto::analytics::AnalyticsSchoolTeamRow>> {
    query.validate().map_err(analytics_request_invalid)?;
    let api = AnalyticsApi::new(get_app_ctx());
    let rows = api
        .list_school_teams(
            auth_user.user_id,
            service::dto::analytics::ListAnalyticsSchoolTeamsRequest {
                page: query.page,
                page_size: query.page_size,
                school_name: query.school_name,
                team_name: query.team_name,
                from: query.from,
                to: query.to,
                sort_by: query.sort_by.map(map_school_team_sort_by_to_service),
                sort_order: query.sort_order.map(map_sort_order_to_service),
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(PageResponse::from(rows.map(map_school_team_row))
        .into_common_response()
        .to_json())
}
