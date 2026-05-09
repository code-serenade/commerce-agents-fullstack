use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::app::{
    approve_execution_queue, approve_tool_approval, create_audit_log, create_execution_queue,
    create_tool_approval, get_execution_queue, get_tool_approval, list_audit_logs,
    list_execution_queue, list_tool_approvals, reject_execution_queue, reject_tool_approval,
};

pub fn approval_audit_routes() -> Router {
    Router::new()
        .route(
            "/execution-queue",
            get(list_execution_queue).post(create_execution_queue),
        )
        .route("/execution-queue/{execution_id}", get(get_execution_queue))
        .route(
            "/execution-queue/{execution_id}/approve",
            post(approve_execution_queue),
        )
        .route(
            "/execution-queue/{execution_id}/reject",
            post(reject_execution_queue),
        )
        .route(
            "/approvals/tool-requests",
            get(list_tool_approvals).post(create_tool_approval),
        )
        .route(
            "/approvals/tool-requests/{approval_id}",
            get(get_tool_approval),
        )
        .route(
            "/approvals/tool-requests/{approval_id}/approve",
            post(approve_tool_approval),
        )
        .route(
            "/approvals/tool-requests/{approval_id}/reject",
            post(reject_tool_approval),
        )
        .route(
            "/approvals/audit-logs",
            get(list_audit_logs).post(create_audit_log),
        )
}
