use crate::{
    dto::app::{
        LearningEvidenceQuery, LearningEvidenceRange, LearningProbeHistoryQuery,
        NextLearningProbeQuery, SubmitLearningProbeChoiceRequest, UpdateLearningSettingsRequest,
    },
    error::from_biz_error,
    statics::{db_manager::get_app_ctx, learning_runtime::get_question_delivery_mode},
};
use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use service::{
    api::learning_progress::LearningProgressApi,
    dto::learning_progress::{
        LearningEvidenceEntry, LearningProbeHistoryEntry, LearningProbeResponse,
        LearningProbeResultResponse, LearningProbeStatusResponse, LearningProfileResponse,
    },
};
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult, middleware::auth_mw::AuthUser};

pub async fn get_learning_evidence(
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<LearningEvidenceQuery>,
) -> ResponseResult<Vec<LearningEvidenceEntry>> {
    let range = query.range.unwrap_or(LearningEvidenceRange::Recent);
    let entries = LearningProgressApi::new(get_app_ctx())
        .list_evidence(
            auth_user.user_id,
            match range {
                LearningEvidenceRange::Recent => {
                    service::dto::learning_progress::LearningEvidenceRange::Recent
                }
                LearningEvidenceRange::Days30 => {
                    service::dto::learning_progress::LearningEvidenceRange::Days30
                }
                LearningEvidenceRange::Days90 => {
                    service::dto::learning_progress::LearningEvidenceRange::Days90
                }
            },
            query
                .limit
                .unwrap_or(if matches!(range, LearningEvidenceRange::Recent) {
                    10
                } else {
                    50
                })
                .min(50),
        )
        .await
        .map_err(from_biz_error)?;
    Ok(entries.into_common_response().to_json())
}

pub async fn get_learning_profile(
    Extension(auth_user): Extension<AuthUser>,
) -> ResponseResult<LearningProfileResponse> {
    let response = LearningProgressApi::new(get_app_ctx())
        .get_profile(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn update_learning_settings(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateLearningSettingsRequest>,
) -> ResponseResult<LearningProfileResponse> {
    let response = LearningProgressApi::new(get_app_ctx())
        .update_settings(
            auth_user.user_id,
            service::dto::learning_progress::UpdateLearningSettingsRequest {
                enabled: req.enabled,
            },
        )
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_next_learning_probe(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<NextLearningProbeQuery>,
) -> ResponseResult<Option<LearningProbeResponse>> {
    let response = LearningProgressApi::new_with_question_delivery_mode(
        get_app_ctx(),
        get_question_delivery_mode(),
    )
    .get_next_probe(
        auth_user.user_id,
        req.conversation_id,
        req.assistant_message_id,
        req.continue_from_probe_id,
    )
    .await
    .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_learning_probe_history(
    Extension(auth_user): Extension<AuthUser>,
    Query(req): Query<LearningProbeHistoryQuery>,
) -> ResponseResult<Vec<LearningProbeHistoryEntry>> {
    let response = LearningProgressApi::new_with_question_delivery_mode(
        get_app_ctx(),
        get_question_delivery_mode(),
    )
    .list_probe_history(
        auth_user.user_id,
        req.conversation_id,
        req.assistant_message_id,
    )
    .await
    .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn skip_learning_probe(
    Extension(auth_user): Extension<AuthUser>,
    Path(probe_id): Path<String>,
) -> ResponseResult<LearningProbeStatusResponse> {
    let response = LearningProgressApi::new(get_app_ctx())
        .skip_probe(auth_user.user_id, probe_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn engage_learning_probe(
    Extension(auth_user): Extension<AuthUser>,
    Path(probe_id): Path<String>,
) -> ResponseResult<LearningProbeStatusResponse> {
    let response = LearningProgressApi::new(get_app_ctx())
        .engage_probe(auth_user.user_id, probe_id)
        .await
        .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn submit_learning_probe_choice(
    Extension(auth_user): Extension<AuthUser>,
    Path(probe_id): Path<String>,
    Json(req): Json<SubmitLearningProbeChoiceRequest>,
) -> ResponseResult<LearningProbeResultResponse> {
    let response = LearningProgressApi::new_with_question_delivery_mode(
        get_app_ctx(),
        get_question_delivery_mode(),
    )
    .submit_choice(auth_user.user_id, probe_id, req.choice_id)
    .await
    .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}

pub async fn get_learning_probe_result(
    Extension(auth_user): Extension<AuthUser>,
    Path(probe_id): Path<String>,
) -> ResponseResult<LearningProbeResultResponse> {
    let response = LearningProgressApi::new_with_question_delivery_mode(
        get_app_ctx(),
        get_question_delivery_mode(),
    )
    .get_probe_result(auth_user.user_id, probe_id)
    .await
    .map_err(from_biz_error)?;
    Ok(response.into_common_response().to_json())
}
