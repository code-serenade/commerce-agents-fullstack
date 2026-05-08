use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    alerts::{Alert, AlertFilter, AlertService, AlertStatus, CreateAlert, UpdateAlert},
    app_users::{AppUserService, AppUserStatus},
    background_events::{
        BackgroundEvent, BackgroundEventFilter, BackgroundEventService, CreateBackgroundEvent,
        UpdateBackgroundEventConsumed,
    },
    conversations::ConversationService,
    daily_briefings::{
        DailyBriefing, DailyBriefingFilter, DailyBriefingService, UpdateDailyBriefing,
        UpsertDailyBriefing,
    },
    long_running_tasks::{
        CreateLongRunningTask, LongRunningTask, LongRunningTaskFilter, LongRunningTaskService,
        LongTaskStatus, UpdateLongRunningTask,
    },
    workspace_runs::{WorkspaceRun, WorkspaceRunService},
    workspaces::{WorkspaceService, WorkspaceStatus},
};
use serde_json::{Map, Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::briefing_ops::*;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct BriefingOpsApi {
    app_user_svc: AppUserService,
    workspace_svc: WorkspaceService,
    run_svc: WorkspaceRunService,
    conversation_svc: ConversationService,
    long_task_svc: LongRunningTaskService,
    briefing_svc: DailyBriefingService,
    event_svc: BackgroundEventService,
    alert_svc: AlertService,
}

impl BriefingOpsApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            run_svc: WorkspaceRunService::new(db.clone()),
            conversation_svc: ConversationService::new(db.clone()),
            long_task_svc: LongRunningTaskService::new(db.clone()),
            briefing_svc: DailyBriefingService::new(db.clone()),
            event_svc: BackgroundEventService::new(db.clone()),
            alert_svc: AlertService::new(db),
        }
    }

    pub async fn list_long_tasks(
        &self,
        req: ListLongTasksRequest,
    ) -> BizResult<LongTaskListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = self
            .ensure_optional_workspace_owner(user_id, req.workspace_id)
            .await?;
        let run_id = self
            .ensure_optional_run_owner(user_id, workspace_id, req.run_id)
            .await?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = LongRunningTaskFilter {
            user_id,
            workspace_id,
            run_id,
            status: parse_long_task_status(req.status)?,
            task_type: normalize_optional(req.task_type),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = LongRunningTaskFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.long_task_svc.count(count_filter).await?;
        let items = self
            .long_task_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_long_task)
            .collect();
        Ok(LongTaskListResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_long_task(
        &self,
        req: CreateLongTaskRequest,
    ) -> BizResult<LongTaskResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = self
            .ensure_optional_workspace_owner(user_id, req.workspace_id)
            .await?;
        let run_id = self
            .ensure_optional_run_owner(user_id, workspace_id, req.run_id)
            .await?;
        let status = parse_long_task_status(req.status)?.unwrap_or(LongTaskStatus::Pending);
        let now = OffsetDateTime::now_utc();
        let (started_at, completed_at) = default_task_timestamps(status, None, None, now);
        let task = self
            .long_task_svc
            .create(CreateLongRunningTask {
                id: Uuid::new_v4(),
                user_id,
                workspace_id,
                run_id,
                task_type: normalize_required(req.task_type, "task_type")?,
                status,
                title: normalize_required(req.title, "title")?,
                progress: clamp_progress(req.progress.unwrap_or_default()),
                total_steps: clamp_total_steps(req.total_steps.unwrap_or(100)),
                current_step: normalize_optional(req.current_step),
                role: normalize_optional(req.role),
                input_params: req.input_params.unwrap_or_else(|| json!({})),
                result: req.result.unwrap_or_else(|| json!({})),
                error_message: normalize_optional(req.error_message),
                started_at,
                completed_at,
            })
            .await?;
        Ok(map_long_task(task))
    }

    pub async fn get_long_task(&self, req: GetLongTaskRequest) -> BizResult<LongTaskResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let task_id = parse_uuid(&req.task_id, "task_id")?;
        let task = self.ensure_long_task_owner(user_id, task_id).await?;
        Ok(map_long_task(task))
    }

    pub async fn update_long_task(
        &self,
        req: UpdateLongTaskRequest,
    ) -> BizResult<LongTaskResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let task_id = parse_uuid(&req.task_id, "task_id")?;
        let current = self.ensure_long_task_owner(user_id, task_id).await?;
        let status = parse_long_task_status(req.status)?.unwrap_or(current.status);
        let now = OffsetDateTime::now_utc();
        let (started_at, completed_at) = default_task_timestamps(
            status,
            req.started_at.or(current.started_at),
            req.completed_at.or(current.completed_at),
            now,
        );
        let task = self
            .long_task_svc
            .update(UpdateLongRunningTask {
                id: current.id,
                status,
                title: req
                    .title
                    .map(|value| normalize_required(value, "title"))
                    .transpose()?
                    .unwrap_or(current.title),
                progress: req.progress.map(clamp_progress).unwrap_or(current.progress),
                total_steps: req
                    .total_steps
                    .map(clamp_total_steps)
                    .unwrap_or(current.total_steps),
                current_step: req
                    .current_step
                    .map(normalize_string)
                    .unwrap_or(current.current_step),
                role: req.role.map(normalize_string).unwrap_or(current.role),
                result: req.result.unwrap_or(current.result),
                error_message: req
                    .error_message
                    .map(normalize_string)
                    .unwrap_or(current.error_message),
                started_at,
                completed_at,
            })
            .await?;
        Ok(map_long_task(task))
    }

    pub async fn cancel_long_task(
        &self,
        req: CancelLongTaskRequest,
    ) -> BizResult<LongTaskResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let task_id = parse_uuid(&req.task_id, "task_id")?;
        let current = self.ensure_long_task_owner(user_id, task_id).await?;
        let now = OffsetDateTime::now_utc();
        let task = self
            .long_task_svc
            .update(UpdateLongRunningTask {
                id: current.id,
                status: LongTaskStatus::Cancelled,
                title: current.title,
                progress: current.progress,
                total_steps: current.total_steps,
                current_step: current.current_step,
                role: current.role,
                result: current.result,
                error_message: normalize_optional(req.error_message)
                    .or(current.error_message)
                    .or_else(|| Some("cancelled by user".to_string())),
                started_at: current.started_at,
                completed_at: Some(now),
            })
            .await?;
        Ok(map_long_task(task))
    }

    pub async fn daily_briefing(
        &self,
        req: GetDailyBriefingRequest,
    ) -> BizResult<DailyBriefingSummaryResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let date = normalize_optional(req.date).unwrap_or_else(today_string);
        validate_date_key(&date)?;
        let record = self
            .briefing_svc
            .get_by_user_and_date(user_id, &date)
            .await?
            .filter(|record| !record.archived);
        if let Some(record) = record {
            return Ok(DailyBriefingSummaryResponse {
                date,
                generated: true,
                title: record.title.clone(),
                summary: record.summary.clone(),
                sections: record.sections.clone(),
                product_summary: record.product_summary.clone(),
                alert_summary: record.alert_summary.clone(),
                task_summary: record.task_summary.clone(),
                source: record.source.clone(),
                briefing: Some(map_briefing(record)),
            });
        }

        let alert_summary = self.derived_alert_summary(user_id).await?;
        let task_summary = self.derived_task_summary(user_id).await?;
        Ok(DailyBriefingSummaryResponse {
            date: date.clone(),
            generated: false,
            title: format!("{date} daily briefing"),
            summary: None,
            sections: json!([]),
            product_summary: json!({
                "source": "derived",
                "available": false,
                "reason": "product-level LLM briefing is deferred in this phase"
            }),
            alert_summary,
            task_summary,
            source: "derived".to_string(),
            briefing: None,
        })
    }

    pub async fn list_daily_briefing_records(
        &self,
        req: ListDailyBriefingRecordsRequest,
    ) -> BizResult<DailyBriefingListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        if let Some(date) = req.date_from.as_deref() {
            validate_date_key(date)?;
        }
        if let Some(date) = req.date_to.as_deref() {
            validate_date_key(date)?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = DailyBriefingFilter {
            user_id,
            archived: req.archived,
            status: normalize_optional(req.status),
            date_from: req.date_from.and_then(normalize_string),
            date_to: req.date_to.and_then(normalize_string),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = DailyBriefingFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.briefing_svc.count(count_filter).await?;
        let records = self
            .briefing_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_briefing)
            .collect();
        Ok(DailyBriefingListResponse {
            records,
            total,
            page,
            page_size,
        })
    }

    pub async fn upsert_daily_briefing_record(
        &self,
        req: UpsertDailyBriefingRecordRequest,
    ) -> BizResult<DailyBriefingResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let briefing_date = normalize_required(req.briefing_date, "briefing_date")?;
        validate_date_key(&briefing_date)?;
        let record = self
            .briefing_svc
            .upsert(UpsertDailyBriefing {
                id: Uuid::new_v4(),
                user_id,
                briefing_date,
                title: normalize_required(req.title, "title")?,
                summary: normalize_optional(req.summary),
                sections: req.sections.unwrap_or_else(|| json!([])),
                product_summary: req.product_summary.unwrap_or_else(|| json!({})),
                alert_summary: req.alert_summary.unwrap_or_else(|| json!({})),
                task_summary: req.task_summary.unwrap_or_else(|| json!({})),
                source: normalize_optional(req.source).unwrap_or_else(|| "manual".to_string()),
                status: normalize_optional(req.status).unwrap_or_else(|| "ready".to_string()),
                archived: req.archived.unwrap_or(false),
            })
            .await?;
        Ok(map_briefing(record))
    }

    pub async fn get_daily_briefing_record(
        &self,
        req: GetDailyBriefingRecordRequest,
    ) -> BizResult<DailyBriefingResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let briefing_id = parse_uuid(&req.briefing_id, "briefing_id")?;
        let record = self.ensure_briefing_owner(user_id, briefing_id).await?;
        Ok(map_briefing(record))
    }

    pub async fn archive_daily_briefing_record(
        &self,
        req: ArchiveDailyBriefingRecordRequest,
    ) -> BizResult<DailyBriefingResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let briefing_id = parse_uuid(&req.briefing_id, "briefing_id")?;
        let current = self.ensure_briefing_owner(user_id, briefing_id).await?;
        let record = self
            .briefing_svc
            .update(UpdateDailyBriefing {
                id: current.id,
                title: current.title,
                summary: current.summary,
                sections: current.sections,
                product_summary: current.product_summary,
                alert_summary: current.alert_summary,
                task_summary: current.task_summary,
                source: current.source,
                status: current.status,
                archived: true,
            })
            .await?;
        Ok(map_briefing(record))
    }

    pub async fn list_background_events(
        &self,
        req: ListBackgroundEventsRequest,
    ) -> BizResult<BackgroundEventListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = self
            .ensure_optional_conversation_owner(user_id, req.conversation_id)
            .await?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = BackgroundEventFilter {
            user_id,
            conversation_id,
            event_type: normalize_optional(req.event_type),
            consumed: req.consumed,
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = BackgroundEventFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.event_svc.count(count_filter).await?;
        let events = self
            .event_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_background_event)
            .collect();
        Ok(BackgroundEventListResponse {
            events,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_background_event(
        &self,
        req: CreateBackgroundEventRequest,
    ) -> BizResult<BackgroundEventResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = self
            .ensure_optional_conversation_owner(user_id, req.conversation_id)
            .await?;
        let consumed = req.consumed.unwrap_or(false);
        let event = self
            .event_svc
            .create(CreateBackgroundEvent {
                id: Uuid::new_v4(),
                user_id,
                conversation_id,
                event_type: normalize_required(req.event_type, "event_type")?,
                title: normalize_optional(req.title),
                data: req.data.unwrap_or_else(|| json!({})),
                consumed,
                consumed_at: consumed.then(OffsetDateTime::now_utc),
            })
            .await?;
        Ok(map_background_event(event))
    }

    pub async fn consume_background_event(
        &self,
        req: ConsumeBackgroundEventRequest,
    ) -> BizResult<BackgroundEventResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let event_id = parse_uuid(&req.event_id, "event_id")?;
        let current = self
            .ensure_background_event_owner(user_id, event_id)
            .await?;
        let event = self
            .event_svc
            .update_consumed(UpdateBackgroundEventConsumed {
                id: current.id,
                consumed: true,
                consumed_at: Some(OffsetDateTime::now_utc()),
            })
            .await?;
        Ok(map_background_event(event))
    }

    pub async fn consume_background_events(
        &self,
        req: ConsumeBackgroundEventsRequest,
    ) -> BizResult<BatchUpdateResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = self
            .ensure_optional_conversation_owner(user_id, req.conversation_id)
            .await?;
        let events = self
            .event_svc
            .list(BackgroundEventFilter {
                user_id,
                conversation_id,
                event_type: normalize_optional(req.event_type),
                consumed: Some(false),
                limit: None,
                offset: None,
            })
            .await?;
        let now = OffsetDateTime::now_utc();
        let mut updated = 0;
        for event in events {
            self.event_svc
                .update_consumed(UpdateBackgroundEventConsumed {
                    id: event.id,
                    consumed: true,
                    consumed_at: Some(now),
                })
                .await?;
            updated += 1;
        }
        Ok(BatchUpdateResponse { updated })
    }

    pub async fn list_alerts(&self, req: ListAlertsRequest) -> BizResult<AlertListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = self
            .ensure_optional_workspace_owner(user_id, req.workspace_id)
            .await?;
        let run_id = self
            .ensure_optional_run_owner(user_id, workspace_id, req.run_id)
            .await?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = AlertFilter {
            user_id,
            workspace_id,
            run_id,
            alert_type: normalize_optional(req.alert_type),
            severity: normalize_optional(req.severity).map(normalize_severity),
            status: parse_alert_status(req.status)?,
            include_snoozed: req.include_snoozed.unwrap_or(false),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = AlertFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.alert_svc.count(count_filter).await?;
        let alerts = self
            .alert_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_alert)
            .collect();
        Ok(AlertListResponse {
            alerts,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_alert(&self, req: CreateAlertRequest) -> BizResult<AlertResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = self
            .ensure_optional_workspace_owner(user_id, req.workspace_id)
            .await?;
        let run_id = self
            .ensure_optional_run_owner(user_id, workspace_id, req.run_id)
            .await?;
        let dedup_key = normalize_optional(req.dedup_key);
        if let Some(dedup_key) = dedup_key.as_deref()
            && let Some(existing) = self
                .alert_svc
                .get_active_by_dedup(user_id, dedup_key)
                .await?
        {
            return Ok(map_alert(existing));
        }
        let now = OffsetDateTime::now_utc();
        let status = if req.snoozed_until.is_some_and(|time| time > now) {
            AlertStatus::Snoozed
        } else {
            AlertStatus::Active
        };
        let alert = self
            .alert_svc
            .create(CreateAlert {
                id: Uuid::new_v4(),
                user_id,
                workspace_id,
                run_id,
                alert_type: normalize_required(req.alert_type, "alert_type")?,
                severity: normalize_severity(
                    normalize_optional(req.severity).unwrap_or_else(|| "info".to_string()),
                ),
                message: normalize_required(req.message, "message")?,
                role: normalize_optional(req.role),
                status,
                dedup_key,
                snoozed_until: req.snoozed_until,
                payload: req.payload.unwrap_or_else(|| json!({})),
                acted_at: None,
                dismissed_at: None,
            })
            .await?;
        Ok(map_alert(alert))
    }

    pub async fn get_alert(&self, req: GetAlertRequest) -> BizResult<AlertResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let alert_id = parse_uuid(&req.alert_id, "alert_id")?;
        let alert = self.ensure_alert_owner(user_id, alert_id).await?;
        Ok(map_alert(alert))
    }

    pub async fn alert_summary(&self, user_id: String) -> BizResult<AlertSummaryResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let active = self
            .alert_svc
            .count(AlertFilter {
                user_id,
                status: Some(AlertStatus::Active),
                include_snoozed: false,
                ..Default::default()
            })
            .await?;
        let by_severity =
            grouped_counts_to_json(self.alert_svc.count_group_by_severity(user_id).await?);
        let by_type = grouped_counts_to_json(self.alert_svc.count_group_by_type(user_id).await?);
        Ok(AlertSummaryResponse {
            active,
            by_severity,
            by_type,
        })
    }

    pub async fn snooze_alert(&self, req: SnoozeAlertRequest) -> BizResult<AlertResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let alert_id = parse_uuid(&req.alert_id, "alert_id")?;
        let current = self.ensure_alert_owner(user_id, alert_id).await?;
        let alert = self
            .alert_svc
            .update(UpdateAlert {
                id: current.id,
                status: AlertStatus::Snoozed,
                snoozed_until: Some(req.snoozed_until),
                acted_at: current.acted_at,
                dismissed_at: current.dismissed_at,
                payload: current.payload,
            })
            .await?;
        Ok(map_alert(alert))
    }

    pub async fn dismiss_alert(&self, req: DismissAlertRequest) -> BizResult<AlertResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let alert_id = parse_uuid(&req.alert_id, "alert_id")?;
        let current = self.ensure_alert_owner(user_id, alert_id).await?;
        let alert = self
            .alert_svc
            .update(UpdateAlert {
                id: current.id,
                status: AlertStatus::Dismissed,
                snoozed_until: current.snoozed_until,
                acted_at: current.acted_at,
                dismissed_at: Some(OffsetDateTime::now_utc()),
                payload: merge_payload(current.payload, "dismissReason", req.reason),
            })
            .await?;
        Ok(map_alert(alert))
    }

    pub async fn act_alert(&self, req: ActAlertRequest) -> BizResult<AlertResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let alert_id = parse_uuid(&req.alert_id, "alert_id")?;
        let current = self.ensure_alert_owner(user_id, alert_id).await?;
        let alert = self
            .alert_svc
            .update(UpdateAlert {
                id: current.id,
                status: AlertStatus::Acted,
                snoozed_until: current.snoozed_until,
                acted_at: Some(OffsetDateTime::now_utc()),
                dismissed_at: current.dismissed_at,
                payload: merge_payload_value(
                    current.payload,
                    "actionResult",
                    req.result.unwrap_or_else(|| json!({})),
                ),
            })
            .await?;
        Ok(map_alert(alert))
    }

    pub async fn dismiss_all_alerts(
        &self,
        req: DismissAllAlertsRequest,
    ) -> BizResult<BatchUpdateResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = self
            .ensure_optional_workspace_owner(user_id, req.workspace_id)
            .await?;
        let run_id = self
            .ensure_optional_run_owner(user_id, workspace_id, req.run_id)
            .await?;
        let alerts = self
            .alert_svc
            .list(AlertFilter {
                user_id,
                workspace_id,
                run_id,
                alert_type: normalize_optional(req.alert_type),
                severity: normalize_optional(req.severity).map(normalize_severity),
                status: Some(AlertStatus::Active),
                include_snoozed: true,
                limit: None,
                offset: None,
            })
            .await?;
        let now = OffsetDateTime::now_utc();
        let mut updated = 0;
        for alert in alerts {
            self.alert_svc
                .update(UpdateAlert {
                    id: alert.id,
                    status: AlertStatus::Dismissed,
                    snoozed_until: alert.snoozed_until,
                    acted_at: alert.acted_at,
                    dismissed_at: Some(now),
                    payload: alert.payload,
                })
                .await?;
            updated += 1;
        }
        Ok(BatchUpdateResponse { updated })
    }

    async fn derived_alert_summary(&self, user_id: Uuid) -> BizResult<Value> {
        let active = self
            .alert_svc
            .count(AlertFilter {
                user_id,
                status: Some(AlertStatus::Active),
                include_snoozed: false,
                ..Default::default()
            })
            .await?;
        Ok(json!({
            "source": "derived",
            "active": active,
            "bySeverity": grouped_counts_to_json(self.alert_svc.count_group_by_severity(user_id).await?),
            "byType": grouped_counts_to_json(self.alert_svc.count_group_by_type(user_id).await?)
        }))
    }

    async fn derived_task_summary(&self, user_id: Uuid) -> BizResult<Value> {
        let mut status_counts = Map::new();
        for status in [
            LongTaskStatus::Pending,
            LongTaskStatus::Running,
            LongTaskStatus::Completed,
            LongTaskStatus::Failed,
            LongTaskStatus::Cancelled,
        ] {
            let count = self
                .long_task_svc
                .count(LongRunningTaskFilter {
                    user_id,
                    status: Some(status),
                    ..Default::default()
                })
                .await?;
            status_counts.insert(status.as_str().to_string(), json!(count));
        }
        Ok(json!({
            "source": "derived",
            "byStatus": status_counts
        }))
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<()> {
        let user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user is disabled".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_optional_workspace_owner(
        &self,
        user_id: Uuid,
        workspace_id: Option<String>,
    ) -> BizResult<Option<Uuid>> {
        let workspace_id = parse_optional_uuid(workspace_id, "workspace_id")?;
        if let Some(workspace_id) = workspace_id {
            let workspace = self
                .workspace_svc
                .get_by_id(workspace_id)
                .await?
                .ok_or_else(|| {
                    BizError::new(
                        app_error::WORKSPACE_NOT_FOUND,
                        "workspace not found".to_string(),
                    )
                })?;
            if workspace.user_id != user_id {
                return Err(BizError::new(
                    app_error::WORKSPACE_FORBIDDEN,
                    "workspace does not belong to current user".to_string(),
                ));
            }
            if workspace.status == WorkspaceStatus::Archived {
                return Err(BizError::new(
                    app_error::WORKSPACE_NOT_FOUND,
                    "workspace not found".to_string(),
                ));
            }
        }
        Ok(workspace_id)
    }

    async fn ensure_optional_run_owner(
        &self,
        user_id: Uuid,
        workspace_id: Option<Uuid>,
        run_id: Option<String>,
    ) -> BizResult<Option<Uuid>> {
        let run_id = parse_optional_uuid(run_id, "run_id")?;
        if let Some(run_id) = run_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::BRIEFING_OPS_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        Ok(run_id)
    }

    async fn ensure_run_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        run_id: Uuid,
    ) -> BizResult<WorkspaceRun> {
        let run = self.run_svc.get_by_id(run_id).await?.ok_or_else(|| {
            BizError::new(
                app_error::WORKSPACE_RUN_NOT_FOUND,
                "workspace run not found".to_string(),
            )
        })?;
        if run.user_id != user_id || run.workspace_id != workspace_id {
            return Err(BizError::new(
                app_error::WORKSPACE_FORBIDDEN,
                "workspace run does not belong to current workspace".to_string(),
            ));
        }
        Ok(run)
    }

    async fn ensure_optional_conversation_owner(
        &self,
        user_id: Uuid,
        conversation_id: Option<String>,
    ) -> BizResult<Option<Uuid>> {
        let conversation_id = parse_optional_uuid(conversation_id, "conversation_id")?;
        if let Some(conversation_id) = conversation_id {
            let conversation = self
                .conversation_svc
                .get_by_id(conversation_id)
                .await?
                .ok_or_else(|| {
                    BizError::new(
                        app_error::CONVERSATION_NOT_FOUND,
                        "conversation not found".to_string(),
                    )
                })?;
            if conversation.user_id != user_id {
                return Err(BizError::new(
                    app_error::CONVERSATION_FORBIDDEN,
                    "conversation does not belong to current user".to_string(),
                ));
            }
        }
        Ok(conversation_id)
    }

    async fn ensure_long_task_owner(
        &self,
        user_id: Uuid,
        task_id: Uuid,
    ) -> BizResult<LongRunningTask> {
        let task = self
            .long_task_svc
            .get_by_id(task_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::LONG_TASK_NOT_FOUND,
                    "long running task not found".to_string(),
                )
            })?;
        if task.user_id != user_id {
            return Err(BizError::new(
                app_error::LONG_TASK_FORBIDDEN,
                "long running task does not belong to current user".to_string(),
            ));
        }
        Ok(task)
    }

    async fn ensure_briefing_owner(
        &self,
        user_id: Uuid,
        briefing_id: Uuid,
    ) -> BizResult<DailyBriefing> {
        let record = self
            .briefing_svc
            .get_by_id(briefing_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::BRIEFING_NOT_FOUND,
                    "daily briefing not found".to_string(),
                )
            })?;
        if record.user_id != user_id {
            return Err(BizError::new(
                app_error::BRIEFING_FORBIDDEN,
                "daily briefing does not belong to current user".to_string(),
            ));
        }
        Ok(record)
    }

    async fn ensure_background_event_owner(
        &self,
        user_id: Uuid,
        event_id: Uuid,
    ) -> BizResult<BackgroundEvent> {
        let event = self.event_svc.get_by_id(event_id).await?.ok_or_else(|| {
            BizError::new(
                app_error::BACKGROUND_EVENT_NOT_FOUND,
                "background event not found".to_string(),
            )
        })?;
        if event.user_id != user_id {
            return Err(BizError::new(
                app_error::BACKGROUND_EVENT_FORBIDDEN,
                "background event does not belong to current user".to_string(),
            ));
        }
        Ok(event)
    }

    async fn ensure_alert_owner(&self, user_id: Uuid, alert_id: Uuid) -> BizResult<Alert> {
        let alert = self.alert_svc.get_by_id(alert_id).await?.ok_or_else(|| {
            BizError::new(app_error::ALERT_NOT_FOUND, "alert not found".to_string())
        })?;
        if alert.user_id != user_id {
            return Err(BizError::new(
                app_error::ALERT_FORBIDDEN,
                "alert does not belong to current user".to_string(),
            ));
        }
        Ok(alert)
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::BRIEFING_OPS_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_uuid(value: Option<String>, label: &str) -> BizResult<Option<Uuid>> {
    value
        .and_then(normalize_string)
        .map(|value| parse_uuid(&value, label))
        .transpose()
}

fn parse_long_task_status(value: Option<String>) -> BizResult<Option<LongTaskStatus>> {
    parse_status(value, |value| match value {
        "pending" => Ok(LongTaskStatus::Pending),
        "running" => Ok(LongTaskStatus::Running),
        "completed" => Ok(LongTaskStatus::Completed),
        "failed" => Ok(LongTaskStatus::Failed),
        "cancelled" => Ok(LongTaskStatus::Cancelled),
        _ => Err(()),
    })
}

fn parse_alert_status(value: Option<String>) -> BizResult<Option<AlertStatus>> {
    parse_status(value, |value| match value {
        "active" => Ok(AlertStatus::Active),
        "snoozed" => Ok(AlertStatus::Snoozed),
        "dismissed" => Ok(AlertStatus::Dismissed),
        "acted" => Ok(AlertStatus::Acted),
        _ => Err(()),
    })
}

fn parse_status<T>(
    value: Option<String>,
    parser: impl Fn(&str) -> Result<T, ()>,
) -> BizResult<Option<T>> {
    value
        .and_then(normalize_string)
        .map(|value| {
            parser(value.as_str()).map_err(|_| {
                BizError::new(
                    app_error::BRIEFING_OPS_REQUEST_INVALID,
                    format!("invalid status: {value}"),
                )
            })
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::BRIEFING_OPS_REQUEST_INVALID,
            format!("{label} cannot be empty"),
        ));
    }
    Ok(value)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(normalize_string)
}

fn normalize_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn normalize_severity(value: String) -> String {
    let value = value.trim().to_lowercase();
    if value.is_empty() {
        "info".to_string()
    } else {
        value
    }
}

fn validate_date_key(value: &str) -> BizResult<()> {
    let valid = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(BizError::new(
            app_error::BRIEFING_OPS_REQUEST_INVALID,
            "date must use YYYY-MM-DD format".to_string(),
        ))
    }
}

fn today_string() -> String {
    OffsetDateTime::now_utc().date().to_string()
}

fn clamp_progress(progress: i32) -> i32 {
    progress.clamp(0, 100)
}

fn clamp_total_steps(total_steps: i32) -> i32 {
    total_steps.max(1)
}

fn default_task_timestamps(
    status: LongTaskStatus,
    started_at: Option<OffsetDateTime>,
    completed_at: Option<OffsetDateTime>,
    now: OffsetDateTime,
) -> (Option<OffsetDateTime>, Option<OffsetDateTime>) {
    let started_at = match status {
        LongTaskStatus::Running | LongTaskStatus::Completed | LongTaskStatus::Failed => {
            started_at.or(Some(now))
        }
        LongTaskStatus::Pending | LongTaskStatus::Cancelled => started_at,
    };
    let completed_at = match status {
        LongTaskStatus::Completed | LongTaskStatus::Failed | LongTaskStatus::Cancelled => {
            completed_at.or(Some(now))
        }
        LongTaskStatus::Pending | LongTaskStatus::Running => completed_at,
    };
    (started_at, completed_at)
}

fn pagination(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    (page, page_size, offset)
}

fn grouped_counts_to_json(items: Vec<(String, i64)>) -> Value {
    let mut map = Map::new();
    for (key, count) in items {
        map.insert(key, json!(count.max(0) as u64));
    }
    Value::Object(map)
}

fn merge_payload(payload: Value, key: &str, value: Option<String>) -> Value {
    match value.and_then(normalize_string) {
        Some(value) => merge_payload_value(payload, key, json!(value)),
        None => payload,
    }
}

fn merge_payload_value(payload: Value, key: &str, value: Value) -> Value {
    let mut map = match payload {
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("previousPayload".to_string(), other);
            map
        }
    };
    map.insert(key.to_string(), value);
    Value::Object(map)
}

fn map_long_task(item: LongRunningTask) -> LongTaskResponse {
    LongTaskResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        workspace_id: item.workspace_id.map(|id| id.to_string()),
        run_id: item.run_id.map(|id| id.to_string()),
        task_type: item.task_type,
        status: item.status.as_str().to_string(),
        title: item.title,
        progress: item.progress,
        total_steps: item.total_steps,
        current_step: item.current_step,
        role: item.role,
        input_params: item.input_params,
        result: item.result,
        error_message: item.error_message,
        started_at: item.started_at,
        completed_at: item.completed_at,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_briefing(item: DailyBriefing) -> DailyBriefingResponse {
    DailyBriefingResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        briefing_date: item.briefing_date,
        title: item.title,
        summary: item.summary,
        sections: item.sections,
        product_summary: item.product_summary,
        alert_summary: item.alert_summary,
        task_summary: item.task_summary,
        source: item.source,
        status: item.status,
        archived: item.archived,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_background_event(item: BackgroundEvent) -> BackgroundEventResponse {
    BackgroundEventResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        conversation_id: item.conversation_id.map(|id| id.to_string()),
        event_type: item.event_type,
        title: item.title,
        data: item.data,
        consumed: item.consumed,
        consumed_at: item.consumed_at,
        created_at: item.created_at,
    }
}

fn map_alert(item: Alert) -> AlertResponse {
    AlertResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        workspace_id: item.workspace_id.map(|id| id.to_string()),
        run_id: item.run_id.map(|id| id.to_string()),
        alert_type: item.alert_type,
        severity: item.severity,
        message: item.message,
        role: item.role,
        status: item.status.as_str().to_string(),
        dedup_key: item.dedup_key,
        snoozed_until: item.snoozed_until,
        payload: item.payload,
        acted_at: item.acted_at,
        dismissed_at: item.dismissed_at,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}
