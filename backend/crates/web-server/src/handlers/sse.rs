use std::{convert::Infallible, time::Duration};

use axum::{
    Extension, Json,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures_util::{Stream, StreamExt};
use model_gateway_rs::{llm::Llm, model::llm::LlmInput};
use serde::Deserialize;
use serde_json::{Value, json};
use service::api::app::AppApi;
use toolcraft_axum_kit::{ApiError, CommonError, middleware::auth_mw::AuthUser};
use validator::Validate;

use crate::{
    dto::sse::LlmChatStreamRequest,
    error::{Error, from_biz_error},
    statics::{db_manager::get_app_ctx, llm_client::get_llm_client},
};

pub async fn stream_llm_chat(
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<LlmChatStreamRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    req.validate().map_err(Error::from)?;
    if req.messages.is_empty() {
        return Err(bad_request("messages cannot be empty"));
    }

    let api = AppApi::new(get_app_ctx());
    api.get_current_user_permissions(auth_user.user_id)
        .await
        .map_err(from_biz_error)?;

    let llm = get_llm_client(req.llm_name.as_deref()).map_err(Into::<ApiError>::into)?;
    let provider_stream = llm
        .chat_stream(LlmInput {
            messages: req.messages,
        })
        .await
        .map_err(|err| Error::Custom(format!("LLM chat_stream failed: {err}")))
        .map_err(Into::<ApiError>::into)?;

    let stream = async_stream::stream! {
        let mut parser = OpenAiSseParser::default();
        let mut provider_stream = provider_stream;

        while let Some(chunk) = provider_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for event in parser.push(&text) {
                        yield Ok(event);
                    }
                }
                Err(err) => {
                    yield Ok(error_event(format!("LLM stream failed: {err}")));
                    return;
                }
            }
        }

        for event in parser.finish() {
            yield Ok(event);
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
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

fn json_event(event: &'static str, value: Value) -> Event {
    Event::default().event(event).data(value.to_string())
}

fn error_event(message: impl Into<String>) -> Event {
    json_event("error", json!({ "message": message.into() }))
}

#[derive(Default)]
struct OpenAiSseParser {
    buffer: String,
    done_sent: bool,
}

impl OpenAiSseParser {
    fn push(&mut self, text: &str) -> Vec<Event> {
        self.buffer.push_str(text);
        self.buffer = self.buffer.replace("\r\n", "\n");
        let mut events = Vec::new();

        while let Some(index) = self.buffer.find("\n\n") {
            let frame = self.buffer[.. index].to_owned();
            self.buffer.drain(.. index + 2);
            events.extend(self.parse_frame(&frame));
        }

        events
    }

    fn finish(&mut self) -> Vec<Event> {
        let frame = self.buffer.trim().to_owned();
        self.buffer.clear();
        if frame.is_empty() {
            self.done_event()
        } else {
            let mut events = self.parse_frame(&frame);
            events.extend(self.done_event());
            events
        }
    }

    fn parse_frame(&mut self, frame: &str) -> Vec<Event> {
        let data = frame
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect::<Vec<_>>()
            .join("\n");

        if data.is_empty() {
            return Vec::new();
        }

        if data == "[DONE]" {
            return self.done_event();
        }

        let chunk = match serde_json::from_str::<OpenAiStreamChunk>(&data) {
            Ok(chunk) => chunk,
            Err(err) => {
                return vec![error_event(format!(
                    "decode LLM stream chunk failed: {err}"
                ))];
            }
        };

        let mut events = Vec::new();
        for choice in chunk.choices {
            if let Some(reasoning) =
                non_empty(choice.delta.reasoning.or(choice.delta.reasoning_content))
            {
                events.push(json_event("reasoning", json!({ "content": reasoning })));
            }

            if let Some(content) = non_empty(choice.delta.content) {
                events.push(json_event("delta", json!({ "content": content })));
            }

            if choice.finish_reason.is_some_and(|value| !value.is_null()) {
                events.extend(self.done_event());
            }
        }

        events
    }

    fn done_event(&mut self) -> Vec<Event> {
        if self.done_sent {
            Vec::new()
        } else {
            self.done_sent = true;
            vec![json_event("done", json!({}))]
        }
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.is_empty())
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
    reasoning: Option<String>,
    reasoning_content: Option<String>,
}
