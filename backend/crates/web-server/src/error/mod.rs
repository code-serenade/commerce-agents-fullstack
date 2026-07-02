use axum::http::StatusCode;
use db_core::error::{BIZ_INTERNAL_ERROR, BizError};
use error_code::{admin as admin_error, agent as agent_error, app as app_error};
use thiserror::Error;
use toolcraft_axum_kit::{ApiError, CommonError};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum Error {
    #[error("config error: {0}")]
    #[allow(clippy::enum_variant_names)]
    Config(#[from] toolcraft_config::error::Error),

    #[error("validation error: {0}")]
    #[allow(clippy::enum_variant_names)]
    Validation(#[from] validator::ValidationErrors),

    #[error(transparent)]
    Jwt(#[from] toolcraft_jwt::error::Error), // catch-all for other errors

    #[error("custom error: {0}")]
    Custom(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            CommonError {
                code: BIZ_INTERNAL_ERROR,
                message: err.to_string(),
            }
            .to_json(),
        )
    }
}

pub fn from_biz_error(err: BizError) -> ApiError {
    let status = match err.code() {
        app_error::APP_USER_NOT_FOUND => StatusCode::NOT_FOUND,
        app_error::APP_USER_DISABLED => StatusCode::FORBIDDEN,
        app_error::CONVERSATION_NOT_FOUND | app_error::CONVERSATION_FILE_NOT_FOUND => {
            StatusCode::NOT_FOUND
        }
        app_error::CONVERSATION_FORBIDDEN | app_error::CONVERSATION_FILE_FORBIDDEN => {
            StatusCode::FORBIDDEN
        }
        app_error::PRODUCT_NOT_FOUND => StatusCode::NOT_FOUND,
        app_error::PRODUCT_FORBIDDEN => StatusCode::FORBIDDEN,
        app_error::PRODUCT_KNOWLEDGE_NOT_FOUND => StatusCode::NOT_FOUND,
        app_error::PRODUCT_KNOWLEDGE_FORBIDDEN => StatusCode::FORBIDDEN,
        app_error::PRODUCT_ASSET_NOT_FOUND | app_error::PRODUCT_MATERIAL_NOT_FOUND => {
            StatusCode::NOT_FOUND
        }
        app_error::PRODUCT_ASSET_FORBIDDEN | app_error::PRODUCT_MATERIAL_FORBIDDEN => {
            StatusCode::FORBIDDEN
        }
        app_error::PRODUCT_COMPETITOR_NOT_FOUND => StatusCode::NOT_FOUND,
        app_error::PRODUCT_COMPETITOR_FORBIDDEN => StatusCode::FORBIDDEN,
        app_error::CAMPAIGN_NOT_FOUND => StatusCode::NOT_FOUND,
        app_error::CAMPAIGN_FORBIDDEN => StatusCode::FORBIDDEN,
        app_error::WORKSPACE_NOT_FOUND
        | app_error::WORKSPACE_TASK_NOT_FOUND
        | app_error::WORKSPACE_RUN_NOT_FOUND
        | app_error::WORKSPACE_RUN_TASK_NOT_FOUND
        | app_error::WORKSPACE_ARTIFACT_NOT_FOUND
        | app_error::AGENT_QUALITY_CHECK_NOT_FOUND
        | app_error::AGENT_LEARNING_NOT_FOUND
        | app_error::AGENT_TRUST_NOT_FOUND
        | app_error::EXECUTION_QUEUE_NOT_FOUND
        | app_error::PLATFORM_CONNECTION_NOT_FOUND
        | app_error::CHANNEL_CONNECTION_NOT_FOUND
        | app_error::TOOL_APPROVAL_NOT_FOUND
        | app_error::AUDIT_LOG_NOT_FOUND
        | app_error::LONG_TASK_NOT_FOUND
        | app_error::BRIEFING_NOT_FOUND
        | app_error::BACKGROUND_EVENT_NOT_FOUND
        | app_error::ALERT_NOT_FOUND
        | app_error::PACKAGE_NOT_FOUND
        | app_error::PACKAGE_LOCK_NOT_FOUND => StatusCode::NOT_FOUND,
        app_error::WORKSPACE_FORBIDDEN
        | app_error::WORKSPACE_TASK_FORBIDDEN
        | app_error::WORKSPACE_ARTIFACT_FORBIDDEN
        | app_error::AGENT_QUALITY_FORBIDDEN
        | app_error::EXECUTION_QUEUE_FORBIDDEN
        | app_error::PLATFORM_CONNECTION_FORBIDDEN
        | app_error::CHANNEL_CONNECTION_FORBIDDEN
        | app_error::TOOL_APPROVAL_FORBIDDEN
        | app_error::LONG_TASK_FORBIDDEN
        | app_error::BRIEFING_FORBIDDEN
        | app_error::BACKGROUND_EVENT_FORBIDDEN
        | app_error::ALERT_FORBIDDEN
        | app_error::PACKAGE_FORBIDDEN
        | app_error::PACKAGE_LOCK_FORBIDDEN => StatusCode::FORBIDDEN,
        app_error::CONVERSATION_REQUEST_INVALID | app_error::PRODUCT_REQUEST_INVALID => {
            StatusCode::BAD_REQUEST
        }
        app_error::PRODUCT_KNOWLEDGE_REQUEST_INVALID
        | app_error::PRODUCT_ASSET_REQUEST_INVALID
        | app_error::PRODUCT_MATERIAL_REQUEST_INVALID
        | app_error::PRODUCT_COMPETITOR_REQUEST_INVALID
        | app_error::CAMPAIGN_REQUEST_INVALID
        | app_error::WORKSPACE_REQUEST_INVALID
        | app_error::WORKSPACE_ARTIFACT_REQUEST_INVALID
        | app_error::AGENT_QUALITY_REQUEST_INVALID
        | app_error::EXECUTION_QUEUE_REQUEST_INVALID
        | app_error::PLATFORM_CONNECT_REQUEST_INVALID
        | app_error::EXTERNAL_SEARCH_REQUEST_INVALID
        | app_error::APPROVAL_AUDIT_REQUEST_INVALID
        | app_error::BRIEFING_OPS_REQUEST_INVALID
        | app_error::PACKAGE_REQUEST_INVALID
        | app_error::TEAM_USAGE_REQUEST_INVALID
        | app_error::ANALYTICS_REQUEST_INVALID => StatusCode::BAD_REQUEST,
        app_error::TEAM_USAGE_QUOTA_EXCEEDED => StatusCode::TOO_MANY_REQUESTS,
        app_error::TEAM_USAGE_DISABLED => StatusCode::FORBIDDEN,
        app_error::PACKAGE_VALIDATION_FAILED => StatusCode::CONFLICT,
        agent_error::AGENT_PROMPT_ERROR | agent_error::AGENT_SETTINGS_ERROR => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        agent_error::AGENT_LLM_ERROR | agent_error::AGENT_EXTERNAL_ERROR => StatusCode::BAD_GATEWAY,
        agent_error::AGENT_COGNITION_ERROR
        | agent_error::AGENT_PERCEPTION_ERROR
        | agent_error::AGENT_INTERNAL_ERROR => StatusCode::INTERNAL_SERVER_ERROR,
        admin_error::ADMIN_USER_NOT_FOUND
        | admin_error::ADMIN_USER_DISABLED
        | admin_error::ADMIN_PERMISSION_DENIED
        | admin_error::ADMIN_ROLE_RESERVED => StatusCode::FORBIDDEN,
        admin_error::ADMIN_AUTH_USER_NOT_FOUND | admin_error::ADMIN_USAGE_ACCOUNT_NOT_FOUND => {
            StatusCode::NOT_FOUND
        }
        admin_error::ADMIN_USAGE_REQUEST_INVALID => StatusCode::BAD_REQUEST,
        _ => StatusCode::BAD_REQUEST,
    };

    (
        status,
        CommonError {
            code: err.code(),
            message: err.message().to_string(),
        }
        .to_json(),
    )
}
