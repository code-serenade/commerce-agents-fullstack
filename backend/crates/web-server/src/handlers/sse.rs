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
use service::{
    api::{agent_context::AgentContextApi, app::AppApi},
    dto::agent_context::{AgentContextSource, BuildAgentProductContextRequest},
};
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
    let (mut metadata, context_selection) = req.metadata.into_parts().map_err(bad_request)?;

    let api = AppApi::new(get_app_ctx());
    api.get_current_user_permissions(auth_user.user_id.clone())
        .await
        .map_err(from_biz_error)?;
    if !context_selection.is_empty() {
        let context_api = AgentContextApi::new(get_app_ctx());
        let context = context_api
            .build_product_context(BuildAgentProductContextRequest {
                user_id: auth_user.user_id.clone(),
                product_ids: context_selection.product_ids,
                context_refs: context_selection.context_refs,
            })
            .await
            .map_err(from_biz_error)?;
        merge_agent_context_metadata(&mut metadata, context)
            .map_err(|err| Error::Custom(format!("agent context metadata failed: {err}")))?;
    }

    let events = run_agent_stream(
        auth_user.user_id,
        req.workspace_id,
        req.agent_id,
        req.session_id,
        req.input,
        metadata,
    )
    .map_err(Into::<ApiError>::into)?;

    let stream = async_stream::stream! {
        let mut events = events;
        while let Some(event) = events.recv().await {
            let event_name = event.event_name();
            let event_value = serde_json::to_value(&event)
                .unwrap_or_else(|_| json!({ "type": event_name }));
            yield Ok(json_event(event_name, event_value));

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

fn merge_agent_context_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    context: service::dto::agent_context::AgentProductContextResponse,
) -> Result<(), serde_json::Error> {
    metadata.insert(
        "context.product_ids".to_string(),
        context.product_ids.join(","),
    );
    metadata.insert("context.refs".to_string(), context.context_refs.join(","));
    metadata.insert(
        "context.product_count".to_string(),
        context.product_count.to_string(),
    );
    metadata.insert(
        "context.knowledge_count".to_string(),
        context.knowledge_count.to_string(),
    );
    metadata.insert(
        "context.source_count".to_string(),
        context.sources.len().to_string(),
    );
    if !context.summary.trim().is_empty() {
        metadata.insert("context.summary".to_string(), context.summary);
    }
    if !context.sources.is_empty() {
        metadata.insert(
            "context.sources_json".to_string(),
            serde_json::to_string(&context.sources)?,
        );
        metadata.insert(
            "context.source_titles".to_string(),
            context
                .sources
                .iter()
                .map(format_context_source)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    Ok(())
}

fn format_context_source(source: &AgentContextSource) -> String {
    match &source.product_id {
        Some(product_id) => format!(
            "{}:{} product={} title={}",
            source.source_type, source.id, product_id, source.title
        ),
        None => format!(
            "{}:{} title={}",
            source.source_type, source.id, source.title
        ),
    }
}
