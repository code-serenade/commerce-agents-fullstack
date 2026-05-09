use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::app::{
    act_alert, alert_summary, archive_daily_briefing_record, cancel_long_task,
    consume_background_event, consume_background_events, create_alert, create_background_event,
    create_long_task, daily_briefing, dismiss_alert, dismiss_all_alerts, get_alert,
    get_daily_briefing_record, get_long_task, list_alerts, list_background_events,
    list_daily_briefing_records, list_long_tasks, snooze_alert, update_long_task,
    upsert_daily_briefing_record,
};

pub fn briefing_routes() -> Router {
    Router::new()
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
}
