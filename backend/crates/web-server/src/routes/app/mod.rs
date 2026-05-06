use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post},
};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::handlers::app::{
    add_conversation_message, agent_llm_config, create_conversation, current_user_permissions,
    get_conversation, get_conversation_file_access, list_conversations,
    presign_conversation_file_upload, register_app_user, register_conversation_file,
};

pub fn app_routes() -> Router {
    Router::new().merge(
        Router::new()
            .route("/register", post(register_app_user))
            .route("/agent/llm-config", get(agent_llm_config))
            .route("/me/permissions", get(current_user_permissions))
            .route(
                "/conversations",
                get(list_conversations).post(create_conversation),
            )
            .route("/conversations/{conversation_id}", get(get_conversation))
            .route(
                "/conversations/{conversation_id}/messages",
                post(add_conversation_message),
            )
            .route(
                "/conversations/{conversation_id}/files/presign-upload",
                post(presign_conversation_file_upload),
            )
            .route(
                "/conversations/{conversation_id}/files",
                post(register_conversation_file),
            )
            .route(
                "/conversation-files/{file_id}/access",
                get(get_conversation_file_access),
            )
            .route_layer(from_fn(auth::<VerifyJwt>)),
    )
}
