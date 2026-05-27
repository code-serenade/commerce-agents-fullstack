use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::handlers::app::{
    add_conversation_message, attach_conversation_file, bulk_delete_user_files,
    create_conversation, current_user_permissions, delete_conversation, delete_conversation_file,
    delete_user_file, get_conversation, get_conversation_file_access,
    get_conversation_file_extraction, list_conversations, list_user_files,
    presign_conversation_file_upload, preview_agent_context, register_app_user,
    register_conversation_file, trigger_conversation_file_extraction_job, update_conversation,
    update_conversation_file_extraction,
};

pub fn account_routes() -> Router {
    Router::new()
        .route("/register", post(register_app_user))
        .route("/me/permissions", get(current_user_permissions))
        .route(
            "/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/conversations/{conversation_id}",
            get(get_conversation)
                .patch(update_conversation)
                .delete(delete_conversation),
        )
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
        .route(
            "/conversation-files/{file_id}",
            delete(delete_conversation_file),
        )
        .route("/files", get(list_user_files))
        .route("/files/bulk-delete", post(bulk_delete_user_files))
        .route("/files/{file_id}", delete(delete_user_file))
        .route(
            "/files/{file_id}/attach-to-conversation",
            post(attach_conversation_file),
        )
        .route(
            "/conversation-files/{file_id}/extraction",
            get(get_conversation_file_extraction).put(update_conversation_file_extraction),
        )
        .route(
            "/conversation-files/{file_id}/extraction-jobs",
            post(trigger_conversation_file_extraction_job),
        )
        .route("/agent/context-preview", post(preview_agent_context))
}
