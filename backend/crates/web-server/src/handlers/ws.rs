use std::sync::Arc;

use axum::{
    Extension,
    extract::{
        Query,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use toolcraft_axum_kit::middleware::auth_mw::AuthUser;
use toolcraft_jwt::{AccessTokenVerifier, VerifyJwt};
use tracing::{debug, warn};

use crate::{
    dto::ws::{WsClientMessage, WsServerMessage},
    statics::ws_hub::get_ws_hub,
};

#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    token: Option<String>,
}

pub async fn websocket(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Query(query): Query<WsAuthQuery>,
    Extension(jwt): Extension<Arc<VerifyJwt>>,
) -> Result<impl IntoResponse, StatusCode> {
    let token = parse_bearer_token(&headers)
        .or(query.token)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = jwt
        .validate_access_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let auth_user = AuthUser {
        user_id: claims.sub,
        ext: claims.ext,
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, auth_user)))
}

async fn handle_socket(mut socket: WebSocket, auth_user: AuthUser) {
    let hub = get_ws_hub();
    let (connection_id, mut rx) = hub.register(auth_user.user_id.clone());

    debug!(
        user_id = %auth_user.user_id,
        connection_id = %connection_id,
        "websocket connected"
    );

    if send_json(
        &mut socket,
        WsServerMessage {
            kind: "connected",
            user_id: Some(auth_user.user_id.clone()),
            payload: auth_user.ext.clone(),
            message: None,
        },
    )
    .await
    .is_err()
    {
        hub.unregister(&auth_user.user_id, connection_id);
        return;
    }

    loop {
        tokio::select! {
            message = socket.recv() => {
                let Some(message) = message else {
                    break;
                };

                let message = match message {
                    Ok(message) => message,
                    Err(err) => {
                        warn!(
                            user_id = %auth_user.user_id,
                            connection_id = %connection_id,
                            error = %err,
                            "websocket receive error"
                        );
                        break;
                    }
                };

                if handle_client_message(&mut socket, message).await.is_err() {
                    break;
                }
            }
            server_message = rx.recv() => {
                let Some(server_message) = server_message else {
                    break;
                };

                if send_json(&mut socket, server_message).await.is_err() {
                    break;
                }
            }
        }
    }

    hub.unregister(&auth_user.user_id, connection_id);
    debug!(
        user_id = %auth_user.user_id,
        connection_id = %connection_id,
        "websocket disconnected"
    );
}

async fn handle_client_message(
    socket: &mut WebSocket,
    message: Message,
) -> Result<(), axum::Error> {
    match message {
        Message::Text(text) => handle_text_message(socket, text.as_str()).await,
        Message::Binary(_) => {
            send_json(
                socket,
                WsServerMessage {
                    kind: "error",
                    user_id: None,
                    payload: None,
                    message: Some("binary messages are not supported".to_string()),
                },
            )
            .await
        }
        Message::Close(_) => Err(axum::Error::new(WsClosed)),
        Message::Ping(_) | Message::Pong(_) => Ok(()),
    }
}

async fn handle_text_message(socket: &mut WebSocket, text: &str) -> Result<(), axum::Error> {
    let client_message = match serde_json::from_str::<WsClientMessage>(text) {
        Ok(message) => message,
        Err(_) => {
            return send_json(
                socket,
                WsServerMessage {
                    kind: "error",
                    user_id: None,
                    payload: None,
                    message: Some("message must be valid JSON".to_string()),
                },
            )
            .await;
        }
    };

    match client_message.kind.as_str() {
        "ping" => {
            send_json(
                socket,
                WsServerMessage {
                    kind: "pong",
                    user_id: None,
                    payload: Some(json!({})),
                    message: None,
                },
            )
            .await
        }
        _ => {
            send_json(
                socket,
                WsServerMessage {
                    kind: "error",
                    user_id: None,
                    payload: None,
                    message: Some("unsupported message type".to_string()),
                },
            )
            .await
        }
    }
}

async fn send_json(socket: &mut WebSocket, message: WsServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(&message).expect("websocket server message must serialize");
    socket.send(Message::Text(text.into())).await
}

#[derive(Debug)]
struct WsClosed;

impl std::fmt::Display for WsClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("websocket closed")
    }
}

impl std::error::Error for WsClosed {}

fn parse_bearer_token(headers: &HeaderMap) -> Option<String> {
    let authorization = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let mut parts = authorization.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?.trim();

    if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() && parts.next().is_none() {
        Some(token.to_string())
    } else {
        None
    }
}
