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
        req.model_name,
        req.input,
    )
    .map_err(Into::<ApiError>::into)?;

    let stream = async_stream::stream! {
        let mut events = events;
        while let Some(event) = events.recv().await {
            let event_name = event.event_name();
            match event {
                SessionEvent::LlmChunk { content, .. } => {
                    yield Ok(json_event("delta", json!({ "content": content })));
                }
                SessionEvent::Output { content, .. } => {
                    yield Ok(json_event("delta", json!({ "content": content })));
                }
                SessionEvent::CommanderThinking { .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name
                    })));
                }
                SessionEvent::WorkerStarted { worker_id, task_id, .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "workerId": worker_id.0,
                        "taskId": task_id.0
                    })));
                }
                SessionEvent::WorkerFinished { worker_id, task_id, .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "workerId": worker_id.0,
                        "taskId": task_id.0
                    })));
                }
                SessionEvent::CollaborationStarted {
                    primary_worker_id,
                    support_worker_ids,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "primaryWorkerId": primary_worker_id.0,
                        "supportWorkerIds": support_worker_ids
                            .into_iter()
                            .map(|worker_id| worker_id.0)
                            .collect::<Vec<_>>()
                    })));
                }
                SessionEvent::CollaborationWorkerStarted { worker_id, task_id, .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "workerId": worker_id.0,
                        "taskId": task_id.0
                    })));
                }
                SessionEvent::CollaborationWorkerFinished {
                    worker_id,
                    task_id,
                    content,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "workerId": worker_id.0,
                        "taskId": task_id.0,
                        "content": content
                    })));
                }
                SessionEvent::CollaborationFinished { .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name
                    })));
                }
                SessionEvent::TaskGraphPlanned { graph, .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graph": graph
                    })));
                }
                SessionEvent::TaskNodeReady {
                    graph_id,
                    node_id,
                    worker_id,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "nodeId": node_id.0,
                        "workerId": worker_id.0
                    })));
                }
                SessionEvent::TaskNodeStarted {
                    graph_id,
                    node_id,
                    worker_id,
                    attempt,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "nodeId": node_id.0,
                        "workerId": worker_id.0,
                        "attempt": attempt
                    })));
                }
                SessionEvent::TaskNodeReported {
                    graph_id,
                    node_id,
                    worker_id,
                    report,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "nodeId": node_id.0,
                        "workerId": worker_id.0,
                        "report": report
                    })));
                }
                SessionEvent::TaskNodeEvaluated {
                    graph_id,
                    evaluation,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "evaluation": evaluation
                    })));
                }
                SessionEvent::TaskNodeReworkRequested {
                    graph_id,
                    node_id,
                    worker_id,
                    attempt,
                    instruction,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "nodeId": node_id.0,
                        "workerId": worker_id.0,
                        "attempt": attempt,
                        "instruction": instruction
                    })));
                }
                SessionEvent::TaskNodePassed {
                    graph_id,
                    node_id,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "nodeId": node_id.0
                    })));
                }
                SessionEvent::TaskNodeFailed {
                    graph_id,
                    node_id,
                    reason,
                    ..
                } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0,
                        "nodeId": node_id.0,
                        "reason": reason
                    })));
                }
                SessionEvent::TaskGraphFinished { graph_id, .. } => {
                    yield Ok(json_event("agent_event", json!({
                        "type": event_name,
                        "graphId": graph_id.0
                    })));
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
