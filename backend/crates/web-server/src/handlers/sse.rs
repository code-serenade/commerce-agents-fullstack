use std::{convert::Infallible, time::Duration};

use agent::api::SessionEvent;
use axum::{
    Extension, Json,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures_util::Stream;
use serde_json::json;
use service::api::app::AppApi;
use toolcraft_axum_kit::{ApiError, CommonError, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::sse::AgentStreamRequest,
    error::{Error, from_biz_error},
    statics::{agent_manager::run_agent_stream, db_manager::get_app_ctx},
};

pub async fn stream_agent_session(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<AgentStreamRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    req.validate().map_err(Error::from)?;
    if req.input.trim().is_empty() {
        return Err(bad_request("input cannot be empty"));
    }

    let api = AppApi::new(get_app_ctx());
    api.get_current_user_permissions(auth_user.user_id.clone())
        .await
        .map_err(from_biz_error)?;

    let events = run_agent_stream(
        auth_user.user_id,
        req.workspace_id,
        req.agent_id,
        req.session_id,
        req.input,
    )
    .map_err(Into::<ApiError>::into)?;

    let stream = async_stream::stream! {
        let mut events = events;
        while let Some(event) = events.recv().await {
            match event {
                SessionEvent::LlmChunk { content, .. } => {
                    yield Ok(json_event("delta", json!({ "content": content })));
                }
                SessionEvent::Output { content, .. } => {
                    yield Ok(json_event("delta", json!({ "content": content })));
                }
                SessionEvent::Failed { reason, .. } => {
                    yield Ok(json_event("error", json!({ "message": reason })));
                    return;
                }
                SessionEvent::Finished { .. } => {
                    yield Ok(json_event("done", json!({})));
                    return;
                }
                _ => {}
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

fn json_event(event: &'static str, value: serde_json::Value) -> Event {
    Event::default().event(event).data(value.to_string())
}

fn bad_request(message: impl Into<String>) -> ApiError {
    (
        axum::http::StatusCode::BAD_REQUEST,
        CommonError {
            code: db_core::error::BIZ_INTERNAL_ERROR,
            message: message.into(),
        }
        .to_json(),
    )
}
