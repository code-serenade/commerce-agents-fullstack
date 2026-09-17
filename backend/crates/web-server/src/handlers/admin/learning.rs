use axum::{
    Extension,
    extract::{Path, Query},
};
use db_core::error::BizError;
use error_code::app as app_error;
use service::{api::learning_admin::LearningAdminApi, dto::learning_progress as learning_dto};
use toolcraft_axum_kit::{
    ApiError, IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser,
};
use validator::Validate;

use crate::{
    dto::{
        common::PageResponse,
        learning::{LearningSortOrder, LearningStudentSortBy, ListLearningStudentsQuery},
    },
    error::from_biz_error,
    statics::db_manager::get_app_ctx,
};

pub async fn list_learning_students(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListLearningStudentsQuery>,
) -> ResponseResult<PageResponse<learning_dto::LearningAdminStudentRow>> {
    query.validate().map_err(learning_request_invalid)?;
    let api = LearningAdminApi::new(get_app_ctx());
    let rows = api
        .list_students(
            auth_user.user_id,
            learning_dto::ListLearningAdminStudentsRequest {
                page: query.page,
                page_size: query.page_size,
                keyword: query.keyword,
                sort_by: query.sort_by.map(map_sort_by),
                sort_order: query.sort_order.map(map_sort_order),
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(PageResponse::from(rows).into_common_response().to_json())
}

pub async fn get_learning_student_detail(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> ResponseResult<learning_dto::LearningAdminStudentDetail> {
    let api = LearningAdminApi::new(get_app_ctx());
    let detail = api
        .get_student_detail(auth_user.user_id, user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(detail.into_common_response().to_json())
}

pub async fn get_learning_student_metrics(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<learning_dto::LearningAdminMetrics> {
    let api = LearningAdminApi::new(get_app_ctx());
    let metrics = api
        .get_metrics(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(metrics.into_common_response().to_json())
}

fn learning_request_invalid(err: validator::ValidationErrors) -> ApiError {
    from_biz_error(BizError::new(
        app_error::LEARNING_REQUEST_INVALID,
        format!("learning request invalid: {err}"),
    ))
}

fn map_sort_by(value: LearningStudentSortBy) -> learning_dto::LearningAdminSortBy {
    match value {
        LearningStudentSortBy::CoverageRate => learning_dto::LearningAdminSortBy::CoverageRate,
        LearningStudentSortBy::ObservedKnowledgeCount => {
            learning_dto::LearningAdminSortBy::ObservedKnowledgeCount
        }
        LearningStudentSortBy::LastAnalyzedAt => learning_dto::LearningAdminSortBy::LastAnalyzedAt,
        LearningStudentSortBy::UpdatedAt => learning_dto::LearningAdminSortBy::UpdatedAt,
    }
}

fn map_sort_order(value: LearningSortOrder) -> learning_dto::LearningSortOrder {
    match value {
        LearningSortOrder::Asc => learning_dto::LearningSortOrder::Asc,
        LearningSortOrder::Desc => learning_dto::LearningSortOrder::Desc,
    }
}
