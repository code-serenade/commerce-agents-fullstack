use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use service::api::team_usage::TeamUsageApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::{common::PageResponse, team_usage::*},
    error::{Error, from_biz_error},
    statics::db_manager::get_app_ctx,
};

pub async fn list_team_usages(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListTeamUsagesQuery>,
) -> ResponseResult<PageResponse<TeamUsageResponse>> {
    query.validate().map_err(Error::from)?;
    let api = TeamUsageApi::new(get_app_ctx());
    let rows = api
        .list_team_usages(
            auth_user.user_id,
            service::dto::team_usage::ListTeamUsagesRequest {
                page: query.page,
                page_size: query.page_size,
                keyword: query.keyword,
                status: query.status.map(map_status_to_service),
                sort_by: query.sort_by.map(map_sort_by_to_service),
                sort_order: query.sort_order.map(map_sort_order_to_service),
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(PageResponse::from(rows.map(map_team_usage_response))
        .into_common_response()
        .to_json())
}

pub async fn get_team_usage(
    Extension(auth_user): Extension<AuthUser>,
    Path(team_id): Path<String>,
) -> ResponseResult<TeamUsageResponse> {
    let api = TeamUsageApi::new(get_app_ctx());
    let row = api
        .get_team_usage(auth_user.user_id, team_id)
        .await
        .map_err(from_biz_error)?;

    Ok(map_team_usage_response(row)
        .into_common_response()
        .to_json())
}

pub async fn update_team_quota(
    Extension(auth_user): Extension<AuthUser>,
    Path(team_id): Path<String>,
    Json(req): Json<UpdateTeamUsageQuotaRequest>,
) -> ResponseResult<TeamUsageResponse> {
    req.validate().map_err(Error::from)?;
    let api = TeamUsageApi::new(get_app_ctx());
    let row = api
        .update_team_quota(
            auth_user.user_id,
            service::dto::team_usage::UpdateTeamUsageQuotaRequest {
                team_id,
                question_limit: req.question_limit,
                status: map_status_to_service(req.status),
                remark: req.remark,
            },
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_team_usage_response(row)
        .into_common_response()
        .to_json())
}

pub async fn list_team_members(
    Extension(auth_user): Extension<AuthUser>,
    Path(team_id): Path<String>,
) -> ResponseResult<Vec<TeamUsageMemberResponse>> {
    let api = TeamUsageApi::new(get_app_ctx());
    let members = api
        .list_team_members(auth_user.user_id, team_id)
        .await
        .map_err(from_biz_error)?;

    Ok(members
        .into_iter()
        .map(map_team_usage_member_response)
        .collect::<Vec<_>>()
        .into_common_response()
        .to_json())
}

fn map_team_usage_response(row: service::dto::team_usage::TeamUsageResponse) -> TeamUsageResponse {
    TeamUsageResponse {
        id: row.id,
        school_name: row.school_name,
        team_name: row.team_name,
        member_count: row.member_count,
        question_limit: row.question_limit,
        question_used: row.question_used,
        question_remaining: row.question_remaining,
        token_used: row.token_used,
        status: map_status_from_service(row.status),
        remark: row.remark,
        last_used_at: row.last_used_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn map_team_usage_member_response(
    member: service::dto::team_usage::TeamUsageMemberResponse,
) -> TeamUsageMemberResponse {
    TeamUsageMemberResponse {
        user_id: member.user_id,
        display_id: member.display_id,
        display_name: member.display_name,
        school_name: member.school_name,
        team_name: member.team_name,
        status: match member.status {
            service::dto::team_usage::TeamUsageMemberStatus::Enabled => AppUserStatus::Enabled,
            service::dto::team_usage::TeamUsageMemberStatus::Disabled => AppUserStatus::Disabled,
        },
        question_used: member.question_used,
        token_used: member.token_used,
        last_used_at: member.last_used_at,
        created_at: member.created_at,
        updated_at: member.updated_at,
    }
}

fn map_status_to_service(
    status: TeamUsageStatus,
) -> service::dto::team_usage::TeamUsageQuotaStatus {
    match status {
        TeamUsageStatus::Enabled => service::dto::team_usage::TeamUsageQuotaStatus::Enabled,
        TeamUsageStatus::Disabled => service::dto::team_usage::TeamUsageQuotaStatus::Disabled,
    }
}

fn map_status_from_service(
    status: service::dto::team_usage::TeamUsageQuotaStatus,
) -> TeamUsageStatus {
    match status {
        service::dto::team_usage::TeamUsageQuotaStatus::Enabled => TeamUsageStatus::Enabled,
        service::dto::team_usage::TeamUsageQuotaStatus::Disabled => TeamUsageStatus::Disabled,
    }
}

fn map_sort_by_to_service(sort_by: TeamUsageSortBy) -> service::dto::team_usage::TeamUsageSortBy {
    match sort_by {
        TeamUsageSortBy::UpdatedAt => service::dto::team_usage::TeamUsageSortBy::UpdatedAt,
        TeamUsageSortBy::QuestionUsed => service::dto::team_usage::TeamUsageSortBy::QuestionUsed,
        TeamUsageSortBy::QuestionRemaining => {
            service::dto::team_usage::TeamUsageSortBy::QuestionRemaining
        }
        TeamUsageSortBy::MemberCount => service::dto::team_usage::TeamUsageSortBy::MemberCount,
        TeamUsageSortBy::LastUsedAt => service::dto::team_usage::TeamUsageSortBy::LastUsedAt,
    }
}

fn map_sort_order_to_service(sort_order: SortOrder) -> service::dto::team_usage::SortOrder {
    match sort_order {
        SortOrder::Asc => service::dto::team_usage::SortOrder::Asc,
        SortOrder::Desc => service::dto::team_usage::SortOrder::Desc,
    }
}
