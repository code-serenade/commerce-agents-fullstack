use axum::{Router, routing::get};

use crate::handlers::app::{
    disable_channel_connection, disable_platform_connection, get_channel_connection,
    get_platform_connection, list_channel_catalog, list_channel_connections, list_platform_catalog,
    list_platform_connections, list_store_metrics, platform_summary, platform_sync_status,
    save_channel_connection, save_platform_connection, save_store_metric,
    update_channel_connection, update_platform_connection,
};

pub fn platform_connect_routes() -> Router {
    Router::new()
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
}
