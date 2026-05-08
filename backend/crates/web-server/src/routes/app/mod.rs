use axum::{
    Router,
    middleware::from_fn,
    routing::{get, post},
};
use toolcraft_axum_kit::middleware::auth_mw::auth;
use toolcraft_jwt::VerifyJwt;

use crate::handlers::app::{
    act_alert, activate_package, add_conversation_message, alert_summary, approve_execution_queue,
    approve_tool_approval, archive_daily_briefing_record, cancel_long_task,
    consume_background_event, consume_background_events, create_alert, create_audit_log,
    create_background_event, create_campaign, create_competitor_change, create_conversation,
    create_execution_queue, create_long_task, create_product, create_product_competitor,
    create_product_knowledge, create_product_material, create_tool_approval, create_workspace,
    create_workspace_run, create_workspace_run_task, create_workspace_task,
    current_user_permissions, daily_briefing, deactivate_package, delete_campaign,
    delete_conversation, delete_product, delete_product_asset, delete_product_competitor,
    delete_product_knowledge, delete_product_material, delete_workspace, delete_workspace_task,
    disable_channel_connection, disable_platform_connection, dismiss_alert, dismiss_all_alerts,
    get_alert, get_campaign, get_channel_connection, get_conversation,
    get_conversation_file_access, get_daily_briefing_record, get_execution_queue, get_long_task,
    get_package, get_platform_connection, get_product, get_product_asset_access,
    get_product_competitor, get_product_material, get_tool_approval, get_workspace,
    get_workspace_run, get_workspace_task, list_alerts, list_audit_logs, list_background_events,
    list_campaigns, list_channel_catalog, list_channel_connections, list_competitor_changes,
    list_conversations, list_daily_briefing_records, list_execution_queue, list_long_tasks,
    list_package_catalog, list_package_history, list_package_locks, list_platform_catalog,
    list_platform_connections, list_product_assets, list_product_competitors,
    list_product_knowledge, list_product_material_versions, list_product_materials, list_products,
    list_store_metrics, list_tool_approvals, list_workspace_run_tasks, list_workspace_runs,
    list_workspace_tasks, list_workspaces, package_lock_drift, platform_summary,
    platform_sync_status, presign_conversation_file_upload, presign_product_asset_upload,
    product_dashboard, register_app_user, register_conversation_file, register_product_asset,
    reject_execution_queue, reject_tool_approval, rollback_package, save_channel_connection,
    save_package_locks, save_platform_connection, save_store_metric, snooze_alert, update_campaign,
    update_channel_connection, update_conversation, update_long_task, update_package,
    update_platform_connection, update_product, update_product_asset, update_product_competitor,
    update_product_knowledge, update_product_material, update_workspace, update_workspace_run,
    update_workspace_run_task, update_workspace_task, upsert_daily_briefing_record, upsert_package,
    validate_package,
};

pub fn app_routes() -> Router {
    Router::new().merge(
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
            .route("/products", get(list_products).post(create_product))
            .route("/products/dashboard", get(product_dashboard))
            .route("/campaigns", get(list_campaigns).post(create_campaign))
            .route(
                "/campaigns/{campaign_id}",
                get(get_campaign)
                    .patch(update_campaign)
                    .delete(delete_campaign),
            )
            .route(
                "/products/{product_id}",
                get(get_product)
                    .patch(update_product)
                    .delete(delete_product),
            )
            .route(
                "/products/{product_id}/knowledge",
                get(list_product_knowledge).post(create_product_knowledge),
            )
            .route(
                "/products/{product_id}/knowledge/{knowledge_id}",
                axum::routing::patch(update_product_knowledge).delete(delete_product_knowledge),
            )
            .route(
                "/products/{product_id}/assets/presign-upload",
                post(presign_product_asset_upload),
            )
            .route(
                "/products/{product_id}/assets",
                get(list_product_assets).post(register_product_asset),
            )
            .route(
                "/products/{product_id}/assets/{asset_id}",
                axum::routing::patch(update_product_asset).delete(delete_product_asset),
            )
            .route(
                "/products/{product_id}/assets/{asset_id}/access",
                get(get_product_asset_access),
            )
            .route(
                "/products/{product_id}/materials",
                get(list_product_materials).post(create_product_material),
            )
            .route(
                "/products/{product_id}/materials/{material_id}",
                get(get_product_material)
                    .patch(update_product_material)
                    .delete(delete_product_material),
            )
            .route(
                "/products/{product_id}/materials/{material_id}/versions",
                get(list_product_material_versions),
            )
            .route(
                "/products/{product_id}/competitors",
                get(list_product_competitors).post(create_product_competitor),
            )
            .route(
                "/products/{product_id}/competitors/changes",
                get(list_competitor_changes),
            )
            .route(
                "/products/{product_id}/competitors/{competitor_id}",
                get(get_product_competitor)
                    .patch(update_product_competitor)
                    .delete(delete_product_competitor),
            )
            .route(
                "/products/{product_id}/competitors/{competitor_id}/changes",
                post(create_competitor_change),
            )
            .route("/workspaces", get(list_workspaces).post(create_workspace))
            .route(
                "/workspaces/{workspace_id}",
                get(get_workspace)
                    .patch(update_workspace)
                    .delete(delete_workspace),
            )
            .route(
                "/workspaces/{workspace_id}/tasks",
                get(list_workspace_tasks).post(create_workspace_task),
            )
            .route(
                "/workspaces/{workspace_id}/tasks/{task_id}",
                get(get_workspace_task)
                    .patch(update_workspace_task)
                    .delete(delete_workspace_task),
            )
            .route(
                "/workspaces/{workspace_id}/runs",
                get(list_workspace_runs).post(create_workspace_run),
            )
            .route(
                "/workspaces/{workspace_id}/runs/{run_id}",
                get(get_workspace_run).patch(update_workspace_run),
            )
            .route(
                "/workspaces/{workspace_id}/runs/{run_id}/tasks",
                get(list_workspace_run_tasks).post(create_workspace_run_task),
            )
            .route(
                "/workspaces/{workspace_id}/runs/{run_id}/tasks/{run_task_id}",
                axum::routing::patch(update_workspace_run_task),
            )
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
            .route("/long-tasks", get(list_long_tasks).post(create_long_task))
            .route(
                "/long-tasks/{task_id}",
                get(get_long_task).patch(update_long_task),
            )
            .route("/long-tasks/{task_id}/cancel", post(cancel_long_task))
            .route("/briefing/daily", get(daily_briefing))
            .route(
                "/briefing/daily-records",
                get(list_daily_briefing_records).post(upsert_daily_briefing_record),
            )
            .route(
                "/briefing/daily-records/{briefing_id}",
                get(get_daily_briefing_record),
            )
            .route(
                "/briefing/daily-records/{briefing_id}/archive",
                post(archive_daily_briefing_record),
            )
            .route(
                "/briefing/notifications",
                get(list_background_events).post(create_background_event),
            )
            .route(
                "/briefing/notifications/consume-all",
                post(consume_background_events),
            )
            .route(
                "/briefing/notifications/{event_id}/consume",
                post(consume_background_event),
            )
            .route("/briefing/alerts", get(list_alerts).post(create_alert))
            .route("/briefing/alerts/summary", get(alert_summary))
            .route("/briefing/alerts/dismiss-all", post(dismiss_all_alerts))
            .route("/briefing/alerts/{alert_id}", get(get_alert))
            .route("/briefing/alerts/{alert_id}/snooze", post(snooze_alert))
            .route("/briefing/alerts/{alert_id}/dismiss", post(dismiss_alert))
            .route("/briefing/alerts/{alert_id}/act", post(act_alert))
            .route("/packages/catalog", get(list_package_catalog))
            .route("/packages", post(upsert_package))
            .route(
                "/packages/{package_type}/{package_id}",
                get(get_package).patch(update_package),
            )
            .route(
                "/packages/{package_type}/{package_id}/validate",
                get(validate_package),
            )
            .route(
                "/packages/{package_type}/{package_id}/activate",
                post(activate_package),
            )
            .route(
                "/packages/{package_type}/{package_id}/deactivate",
                post(deactivate_package),
            )
            .route(
                "/packages/{package_type}/{package_id}/rollback",
                post(rollback_package),
            )
            .route(
                "/packages/{package_type}/{package_id}/history",
                get(list_package_history),
            )
            .route(
                "/workspaces/{workspace_id}/runs/{run_id}/package-locks",
                get(list_package_locks).post(save_package_locks),
            )
            .route(
                "/workspaces/{workspace_id}/runs/{run_id}/package-locks/drift",
                get(package_lock_drift),
            )
            .route("/platform-connect/platforms", get(list_platform_catalog))
            .route(
                "/platform-connect/connections",
                get(list_platform_connections).post(save_platform_connection),
            )
            .route(
                "/platform-connect/connections/{connection_id}",
                get(get_platform_connection)
                    .patch(update_platform_connection)
                    .delete(disable_platform_connection),
            )
            .route("/platform-connect/channels", get(list_channel_catalog))
            .route(
                "/platform-connect/channels/connections",
                get(list_channel_connections).post(save_channel_connection),
            )
            .route(
                "/platform-connect/channels/connections/{connection_id}",
                get(get_channel_connection)
                    .patch(update_channel_connection)
                    .delete(disable_channel_connection),
            )
            .route(
                "/platform-connect/metrics",
                get(list_store_metrics).post(save_store_metric),
            )
            .route("/platform-connect/sync-status", get(platform_sync_status))
            .route("/platform-connect/summary", get(platform_summary))
            .route_layer(from_fn(auth::<VerifyJwt>)),
    )
}
