use std::collections::{HashMap, HashSet};

use db_core::{
    DbContext, PaginatedResponse, PaginationParams,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    agent_usage_events::{AgentUsageEventService, AgentUsageEventStats},
    analytics_events::{AnalyticsEvent, AnalyticsEventService, CreateAnalyticsEvent},
    analytics_sessions::{
        AnalyticsSession, AnalyticsSessionService, CreateAnalyticsSession, UpdateAnalyticsSession,
    },
    app_users::{AppUser, AppUserService},
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    api::admin::AdminApi,
    dto::analytics::{
        AnalyticsAccountRow, AnalyticsAccountSortBy, AnalyticsActorKind, AnalyticsDailyTraffic,
        AnalyticsGranularity, AnalyticsOverviewResponse, AnalyticsPageMetric,
        AnalyticsPagesResponse, AnalyticsSchoolTeamRow, AnalyticsSchoolTeamSortBy,
        AnalyticsSortOrder, AnalyticsSourceMetric, AnalyticsSourcesResponse, AnalyticsSummary,
        AnalyticsTopPage, ListAnalyticsAccountsRequest, ListAnalyticsSchoolTeamsRequest,
        TrackAnalyticsEventInput, TrackAnalyticsEventsRequest, TrackAnalyticsEventsResponse,
    },
};

const PERM_ANALYTICS_OVERVIEW: &str = "analytics:overview";
const PERM_ANALYTICS_ACCOUNTS: &str = "analytics:accounts";
const PERM_ANALYTICS_SCHOOL_TEAMS: &str = "analytics:school_teams";
const DEFAULT_UNKNOWN_SCHOOL: &str = "未填写学校";
const DEFAULT_UNKNOWN_TEAM: &str = "未填写队伍";
const DEFAULT_ROUTE_PATH: &str = "/";
const MAX_EVENTS_PER_BATCH: usize = 20;

pub struct AnalyticsApi {
    db: DbContext,
    admin_api: AdminApi,
    event_svc: AnalyticsEventService,
    session_svc: AnalyticsSessionService,
    app_user_svc: AppUserService,
    agent_usage_svc: AgentUsageEventService,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActorKey(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TeamKey {
    school_name: String,
    team_name: String,
}

#[derive(Debug, Clone)]
struct TrackAnalyticsEventContext {
    actor_kind: AnalyticsActorKind,
    user_id: Option<Uuid>,
    school_name: Option<String>,
    team_name: Option<String>,
}

impl AnalyticsApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            admin_api: AdminApi::new(db.clone()),
            event_svc: AnalyticsEventService::new(db.clone()),
            session_svc: AnalyticsSessionService::new(db.clone()),
            app_user_svc: AppUserService::new(db.clone()),
            agent_usage_svc: AgentUsageEventService::new(db),
        }
    }

    pub async fn track_anonymous_events(
        &self,
        events: Vec<TrackAnalyticsEventInput>,
    ) -> BizResult<TrackAnalyticsEventsResponse> {
        self.track_events(TrackAnalyticsEventsRequest {
            actor_kind: AnalyticsActorKind::Anonymous,
            user_id: None,
            school_name: None,
            team_name: None,
            events,
        })
        .await
    }

    pub async fn track_app_user_events(
        &self,
        current_user_id: String,
        events: Vec<TrackAnalyticsEventInput>,
    ) -> BizResult<TrackAnalyticsEventsResponse> {
        let user_id = Uuid::parse_str(&current_user_id).map_err(|_| {
            BizError::new(
                app_error::ANALYTICS_REQUEST_INVALID,
                "invalid current user id".to_owned(),
            )
        })?;
        let app_user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_owned(),
                )
            })?;
        let team = team_key_from_app_user(&app_user);

        self.track_events(TrackAnalyticsEventsRequest {
            actor_kind: AnalyticsActorKind::AppUser,
            user_id: Some(user_id),
            school_name: Some(team.school_name),
            team_name: Some(team.team_name),
            events,
        })
        .await
    }

    async fn track_events(
        &self,
        req: TrackAnalyticsEventsRequest,
    ) -> BizResult<TrackAnalyticsEventsResponse> {
        if req.events.len() > MAX_EVENTS_PER_BATCH {
            return Err(BizError::new(
                app_error::ANALYTICS_REQUEST_INVALID,
                format!("events length must be <= {MAX_EVENTS_PER_BATCH}"),
            ));
        }

        let mut accepted = 0;
        let mut duplicated = 0;
        let mut rejected = 0;
        let context = TrackAnalyticsEventContext {
            actor_kind: req.actor_kind,
            user_id: req.user_id,
            school_name: req.school_name,
            team_name: req.team_name,
        };

        for event in req.events {
            if !is_valid_event(&event) {
                rejected += 1;
                continue;
            }

            if self.track_one_event(&context, event).await? {
                accepted += 1;
            } else {
                duplicated += 1;
            }
        }

        Ok(TrackAnalyticsEventsResponse {
            accepted,
            duplicated,
            rejected,
        })
    }

    async fn track_one_event(
        &self,
        context: &TrackAnalyticsEventContext,
        event: TrackAnalyticsEventInput,
    ) -> BizResult<bool> {
        let tx = self.db.begin().await?;
        let result: BizResult<bool> = async {
            let api = AnalyticsApi::new(tx.clone());
            api.track_one_event_in_current_context(context, event).await
        }
        .await;
        commit_or_rollback(tx, result).await
    }

    async fn track_one_event_in_current_context(
        &self,
        context: &TrackAnalyticsEventContext,
        event: TrackAnalyticsEventInput,
    ) -> BizResult<bool> {
        let persisted = self
            .event_svc
            .create_if_event_id_absent(CreateAnalyticsEvent {
                id: Uuid::new_v4(),
                event_id: trim_to_len(Some(event.event_id), 128).unwrap_or_default(),
                event_type: event.event_type.as_str().to_owned(),
                occurred_at: event.occurred_at,
                actor_kind: context.actor_kind.as_str().to_owned(),
                user_id: context.user_id,
                anonymous_id: trim_to_len(event.anonymous_id, 128),
                session_id: trim_to_len(Some(event.session_id), 128).unwrap_or_default(),
                school_name: trim_to_len(context.school_name.clone(), 128),
                team_name: trim_to_len(context.team_name.clone(), 128),
                route_path: trim_to_len(event.route_path, 256),
                page_title: trim_to_len(event.page_title, 256),
                referrer: trim_to_len(event.referrer, 512),
                source_kind: event.source_kind.as_str().to_owned(),
                source_host: trim_to_len(event.source_host, 255),
                utm_source: trim_to_len(event.utm_source, 128),
                utm_medium: trim_to_len(event.utm_medium, 128),
                utm_campaign: trim_to_len(event.utm_campaign, 128),
                utm_content: trim_to_len(event.utm_content, 128),
                utm_term: trim_to_len(event.utm_term, 128),
                feature_key: trim_to_len(event.feature_key, 128),
                device_type: event.device_type.as_str().to_owned(),
                browser_name: trim_to_len(event.browser_name, 64),
                os_name: trim_to_len(event.os_name, 64),
                viewport_width: event.viewport_width,
                viewport_height: event.viewport_height,
                status: event.status.as_str().to_owned(),
                metadata: event.metadata,
            })
            .await?;

        let Some(persisted) = persisted else {
            return Ok(false);
        };

        self.upsert_session(&persisted).await?;
        Ok(true)
    }

    pub async fn get_overview(
        &self,
        current_admin_user_id: String,
        from: Option<OffsetDateTime>,
        to: Option<OffsetDateTime>,
        granularity: Option<AnalyticsGranularity>,
    ) -> BizResult<AnalyticsOverviewResponse> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_ANALYTICS_OVERVIEW)
            .await?;
        let (from, to) = normalize_range(from, to);
        let dataset = self.load_dataset(from, to).await?;
        let summary = build_summary(&dataset);
        let daily_traffic = match granularity.unwrap_or(AnalyticsGranularity::Day) {
            AnalyticsGranularity::Day => build_daily_traffic(&dataset),
            AnalyticsGranularity::Week => build_daily_traffic(&dataset),
        };

        Ok(AnalyticsOverviewResponse {
            summary,
            daily_traffic,
        })
    }

    pub async fn get_sources(
        &self,
        current_admin_user_id: String,
        from: Option<OffsetDateTime>,
        to: Option<OffsetDateTime>,
    ) -> BizResult<AnalyticsSourcesResponse> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_ANALYTICS_OVERVIEW)
            .await?;
        let (from, to) = normalize_range(from, to);
        let dataset = self.load_dataset(from, to).await?;

        Ok(AnalyticsSourcesResponse {
            source_kinds: top_source_metrics(&dataset, SourceMetricKind::SourceKind, 20),
            source_hosts: top_source_metrics(&dataset, SourceMetricKind::SourceHost, 20),
            campaigns: top_source_metrics(&dataset, SourceMetricKind::Campaign, 20),
        })
    }

    pub async fn get_pages(
        &self,
        current_admin_user_id: String,
        from: Option<OffsetDateTime>,
        to: Option<OffsetDateTime>,
    ) -> BizResult<AnalyticsPagesResponse> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_ANALYTICS_OVERVIEW)
            .await?;
        let (from, to) = normalize_range(from, to);
        let dataset = self.load_dataset(from, to).await?;

        Ok(AnalyticsPagesResponse {
            items: build_page_metrics(&dataset),
        })
    }

    pub async fn list_accounts(
        &self,
        current_admin_user_id: String,
        req: ListAnalyticsAccountsRequest,
    ) -> BizResult<PaginatedResponse<AnalyticsAccountRow>> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_ANALYTICS_ACCOUNTS)
            .await?;
        let (from, to) = normalize_range(req.from, req.to);
        let dataset = self.load_dataset(from, to).await?;
        let users = self.app_user_svc.list_all().await?;
        let mut rows = build_account_rows(&dataset, users);

        if let Some(keyword) = normalize_optional(req.keyword) {
            let keyword = keyword.to_ascii_lowercase();
            rows.retain(|row| {
                row.user_id.to_ascii_lowercase().contains(&keyword)
                    || row.display_id.to_ascii_lowercase().contains(&keyword)
                    || row.display_name.to_ascii_lowercase().contains(&keyword)
                    || row.school_name.to_ascii_lowercase().contains(&keyword)
                    || row.team_name.to_ascii_lowercase().contains(&keyword)
            });
        }
        if let Some(school_name) = normalize_optional(req.school_name) {
            rows.retain(|row| row.school_name == school_name);
        }
        if let Some(team_name) = normalize_optional(req.team_name) {
            rows.retain(|row| row.team_name == team_name);
        }

        sort_account_rows(&mut rows, req.sort_by, req.sort_order);
        Ok(paginate_rows(
            rows,
            PaginationParams::new(req.page, req.page_size),
        ))
    }

    pub async fn list_school_teams(
        &self,
        current_admin_user_id: String,
        req: ListAnalyticsSchoolTeamsRequest,
    ) -> BizResult<PaginatedResponse<AnalyticsSchoolTeamRow>> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_ANALYTICS_SCHOOL_TEAMS)
            .await?;
        let (from, to) = normalize_range(req.from, req.to);
        let dataset = self.load_dataset(from, to).await?;
        let users = self.app_user_svc.list_all().await?;
        let mut rows = build_school_team_rows(&dataset, users);

        if let Some(school_name) = normalize_optional(req.school_name) {
            rows.retain(|row| row.school_name == school_name);
        }
        if let Some(team_name) = normalize_optional(req.team_name) {
            rows.retain(|row| row.team_name == team_name);
        }

        sort_school_team_rows(&mut rows, req.sort_by, req.sort_order);
        Ok(paginate_rows(
            rows,
            PaginationParams::new(req.page, req.page_size),
        ))
    }

    async fn upsert_session(&self, event: &AnalyticsEvent) -> BizResult<()> {
        let is_page_view = event.event_type == "page_view";
        match self
            .session_svc
            .get_by_session_id(&event.session_id)
            .await?
        {
            Some(session) => {
                self.update_existing_session(session, event, is_page_view)
                    .await?;
            }
            None => {
                if self
                    .session_svc
                    .create_if_session_id_absent(CreateAnalyticsSession {
                        id: Uuid::new_v4(),
                        session_id: event.session_id.clone(),
                        actor_kind: event.actor_kind.clone(),
                        user_id: event.user_id,
                        anonymous_id: event.anonymous_id.clone(),
                        school_name: event.school_name.clone(),
                        team_name: event.team_name.clone(),
                        first_seen_at: event.occurred_at,
                        last_seen_at: event.occurred_at,
                        entry_path: event.route_path.clone(),
                        exit_path: event.route_path.clone(),
                        source_kind: event.source_kind.clone(),
                        source_host: event.source_host.clone(),
                        page_view_count: i32::from(is_page_view),
                        event_count: 1,
                    })
                    .await?
                    .is_none()
                    && let Some(session) = self
                        .session_svc
                        .get_by_session_id(&event.session_id)
                        .await?
                {
                    self.update_existing_session(session, event, is_page_view)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn update_existing_session(
        &self,
        session: AnalyticsSession,
        event: &AnalyticsEvent,
        is_page_view: bool,
    ) -> BizResult<()> {
        self.session_svc
            .update(UpdateAnalyticsSession {
                id: session.id,
                actor_kind: event.actor_kind.clone(),
                user_id: event.user_id.or(session.user_id),
                anonymous_id: event.anonymous_id.clone().or(session.anonymous_id),
                school_name: event.school_name.clone().or(session.school_name),
                team_name: event.team_name.clone().or(session.team_name),
                last_seen_at: event.occurred_at.max(session.last_seen_at),
                exit_path: event.route_path.clone().or(session.exit_path),
                source_kind: event.source_kind.clone(),
                source_host: event.source_host.clone().or(session.source_host),
                page_view_count: session.page_view_count + i32::from(is_page_view),
                event_count: session.event_count + 1,
            })
            .await?;
        Ok(())
    }

    async fn load_dataset(&self, from: OffsetDateTime, to: OffsetDateTime) -> BizResult<Dataset> {
        Ok(Dataset {
            events: self.event_svc.list_between(from, to).await?,
            sessions: self.session_svc.list_between(from, to).await?,
            agent_stats: self.agent_usage_svc.stats_between(from, to).await?,
        })
    }
}

#[derive(Debug, Clone)]
struct Dataset {
    events: Vec<AnalyticsEvent>,
    sessions: Vec<AnalyticsSession>,
    agent_stats: Vec<AgentUsageEventStats>,
}

#[derive(Debug, Clone, Copy)]
enum SourceMetricKind {
    SourceKind,
    SourceHost,
    Campaign,
}

fn is_valid_event(event: &TrackAnalyticsEventInput) -> bool {
    !event.event_id.trim().is_empty() && !event.session_id.trim().is_empty()
}

fn build_summary(dataset: &Dataset) -> AnalyticsSummary {
    let pv = count_events(&dataset.events, "page_view");
    let uv = count_unique_actors(&dataset.events);
    let sessions = dataset.sessions.len() as i64;
    let active_accounts = dataset
        .events
        .iter()
        .filter_map(|event| event.user_id)
        .collect::<HashSet<_>>()
        .len() as i64;
    let agent_questions = dataset
        .agent_stats
        .iter()
        .map(|stat| stat.question_used)
        .sum();
    let token_used = dataset.agent_stats.iter().map(|stat| stat.token_used).sum();
    let avg_session_duration_sec = average_session_duration_sec(&dataset.sessions);
    let bounce_rate = bounce_rate(&dataset.sessions);

    AnalyticsSummary {
        pv,
        uv,
        sessions,
        active_accounts,
        agent_questions,
        token_used,
        avg_session_duration_sec,
        bounce_rate,
        frontend_errors: count_events(&dataset.events, "frontend_error"),
        api_errors: count_events(&dataset.events, "api_error"),
    }
}

fn build_daily_traffic(dataset: &Dataset) -> Vec<AnalyticsDailyTraffic> {
    let mut rows = HashMap::<time::Date, AnalyticsDailyTraffic>::new();
    let mut uv_by_date = HashMap::<time::Date, HashSet<ActorKey>>::new();
    let mut accounts_by_date = HashMap::<time::Date, HashSet<Uuid>>::new();

    for event in &dataset.events {
        let date = event.received_at.date();
        let row = rows.entry(date).or_insert_with(|| empty_daily(date));
        if event.event_type == "page_view" {
            row.pv += 1;
        }
        if matches!(event.event_type.as_str(), "frontend_error" | "api_error") {
            row.errors += 1;
        }
        uv_by_date
            .entry(date)
            .or_default()
            .insert(actor_key_for_event(event));
        if let Some(user_id) = event.user_id {
            accounts_by_date.entry(date).or_default().insert(user_id);
        }
    }

    for session in &dataset.sessions {
        let date = session.first_seen_at.date();
        rows.entry(date)
            .or_insert_with(|| empty_daily(date))
            .sessions += 1;
    }

    for stat in &dataset.agent_stats {
        if let Some(last_used_at) = stat.last_used_at {
            let date = last_used_at.date();
            let row = rows.entry(date).or_insert_with(|| empty_daily(date));
            row.agent_questions += stat.question_used;
            row.token_used += stat.token_used;
        }
    }

    for (date, actors) in uv_by_date {
        rows.entry(date).or_insert_with(|| empty_daily(date)).uv = actors.len() as i64;
    }
    for (date, accounts) in accounts_by_date {
        rows.entry(date)
            .or_insert_with(|| empty_daily(date))
            .active_accounts = accounts.len() as i64;
    }

    let mut rows = rows.into_values().collect::<Vec<_>>();
    rows.sort_by_key(|row| row.date);
    rows
}

fn top_source_metrics(
    dataset: &Dataset,
    metric_kind: SourceMetricKind,
    limit: usize,
) -> Vec<AnalyticsSourceMetric> {
    let mut grouped = HashMap::<String, Vec<&AnalyticsEvent>>::new();
    for event in &dataset.events {
        let key = match metric_kind {
            SourceMetricKind::SourceKind => event.source_kind.clone(),
            SourceMetricKind::SourceHost => event
                .source_host
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
            SourceMetricKind::Campaign => event
                .utm_campaign
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
        };
        grouped.entry(key).or_default().push(event);
    }

    let agent_questions_by_source = agent_questions_by_source(dataset, metric_kind);
    let mut metrics = grouped
        .into_iter()
        .map(|(key, events)| AnalyticsSourceMetric {
            pv: events
                .iter()
                .filter(|event| event.event_type == "page_view")
                .count() as i64,
            uv: events
                .iter()
                .map(|event| actor_key_for_event(event))
                .collect::<HashSet<_>>()
                .len() as i64,
            sessions: events
                .iter()
                .map(|event| event.session_id.clone())
                .collect::<HashSet<_>>()
                .len() as i64,
            agent_questions: *agent_questions_by_source.get(&key).unwrap_or(&0),
            key,
        })
        .collect::<Vec<_>>();
    metrics.sort_by_key(|metric| metric.pv);
    metrics.reverse();
    metrics.truncate(limit);
    metrics
}

fn build_page_metrics(dataset: &Dataset) -> Vec<AnalyticsPageMetric> {
    let mut events_by_route = HashMap::<String, Vec<&AnalyticsEvent>>::new();
    for event in &dataset.events {
        let route = event
            .route_path
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ROUTE_PATH.to_owned());
        events_by_route.entry(route).or_default().push(event);
    }

    let mut rows = events_by_route
        .into_iter()
        .map(|(route_path, events)| {
            let route_sessions = dataset
                .sessions
                .iter()
                .filter(|session| {
                    session.entry_path.as_deref() == Some(route_path.as_str())
                        || session.exit_path.as_deref() == Some(route_path.as_str())
                        || events
                            .iter()
                            .any(|event| event.session_id == session.session_id)
                })
                .collect::<Vec<_>>();
            AnalyticsPageMetric {
                route_path: route_path.clone(),
                pv: events
                    .iter()
                    .filter(|event| event.event_type == "page_view")
                    .count() as i64,
                uv: events
                    .iter()
                    .map(|event| actor_key_for_event(event))
                    .collect::<HashSet<_>>()
                    .len() as i64,
                sessions: events
                    .iter()
                    .map(|event| event.session_id.clone())
                    .collect::<HashSet<_>>()
                    .len() as i64,
                avg_session_duration_sec: average_session_duration_sec_refs(&route_sessions),
                entry_count: dataset
                    .sessions
                    .iter()
                    .filter(|session| session.entry_path.as_deref() == Some(route_path.as_str()))
                    .count() as i64,
                exit_count: dataset
                    .sessions
                    .iter()
                    .filter(|session| session.exit_path.as_deref() == Some(route_path.as_str()))
                    .count() as i64,
                errors: events
                    .iter()
                    .filter(|event| {
                        matches!(event.event_type.as_str(), "frontend_error" | "api_error")
                    })
                    .count() as i64,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.pv);
    rows.reverse();
    rows
}

fn build_account_rows(dataset: &Dataset, users: Vec<AppUser>) -> Vec<AnalyticsAccountRow> {
    let users_by_id = users
        .into_iter()
        .map(|user| (user.user_id, user))
        .collect::<HashMap<_, _>>();
    let mut user_ids = users_by_id.keys().copied().collect::<HashSet<_>>();
    user_ids.extend(dataset.events.iter().filter_map(|event| event.user_id));
    user_ids.extend(dataset.agent_stats.iter().map(|stat| stat.user_id));

    let agent_by_user = dataset
        .agent_stats
        .iter()
        .map(|stat| (stat.user_id, stat))
        .collect::<HashMap<_, _>>();
    let mut rows = Vec::new();
    for user_id in user_ids {
        let events = dataset
            .events
            .iter()
            .filter(|event| event.user_id == Some(user_id))
            .collect::<Vec<_>>();
        let sessions = dataset
            .sessions
            .iter()
            .filter(|session| session.user_id == Some(user_id))
            .collect::<Vec<_>>();
        let user = users_by_id.get(&user_id);
        let agent = agent_by_user.get(&user_id);
        let team = user
            .map(team_key_from_app_user)
            .or_else(|| {
                agent.map(|stat| normalize_team_key(Some(&stat.school_name), Some(&stat.team_name)))
            })
            .unwrap_or_else(|| normalize_team_key(None, None));
        rows.push(AnalyticsAccountRow {
            user_id: user_id.to_string(),
            display_id: user
                .map(|user| user.display_id.clone())
                .unwrap_or_else(|| user_id.to_string()),
            display_name: user
                .map(|user| user.display_name.clone())
                .unwrap_or_else(|| user_id.to_string()),
            school_name: team.school_name,
            team_name: team.team_name,
            pv: events
                .iter()
                .filter(|event| event.event_type == "page_view")
                .count() as i64,
            sessions: sessions.len() as i64,
            active_days: events
                .iter()
                .map(|event| event.received_at.date())
                .collect::<HashSet<_>>()
                .len() as i64,
            agent_questions: agent.map(|stat| stat.question_used).unwrap_or(0),
            token_used: agent.map(|stat| stat.token_used).unwrap_or(0),
            last_seen_at: latest_time(
                events
                    .iter()
                    .map(|event| event.received_at)
                    .chain(agent.and_then(|stat| stat.last_used_at).into_iter()),
            ),
            top_pages: top_pages(&events),
        });
    }

    rows
}

fn build_school_team_rows(dataset: &Dataset, users: Vec<AppUser>) -> Vec<AnalyticsSchoolTeamRow> {
    let mut account_count = HashMap::<TeamKey, i64>::new();
    for user in users {
        *account_count
            .entry(team_key_from_app_user(&user))
            .or_default() += 1;
    }

    let mut team_keys = account_count.keys().cloned().collect::<HashSet<_>>();
    team_keys.extend(dataset.events.iter().map(team_key_from_event));
    team_keys.extend(
        dataset
            .agent_stats
            .iter()
            .map(|stat| normalize_team_key(Some(&stat.school_name), Some(&stat.team_name))),
    );

    let mut rows = Vec::new();
    for team in team_keys {
        let events = dataset
            .events
            .iter()
            .filter(|event| team_key_from_event(event) == team)
            .collect::<Vec<_>>();
        let sessions = dataset
            .sessions
            .iter()
            .filter(|session| team_key_from_session(session) == team)
            .collect::<Vec<_>>();
        let agent_stats = dataset
            .agent_stats
            .iter()
            .filter(|stat| {
                normalize_team_key(Some(&stat.school_name), Some(&stat.team_name)) == team
            })
            .collect::<Vec<_>>();
        rows.push(AnalyticsSchoolTeamRow {
            school_name: team.school_name.clone(),
            team_name: team.team_name.clone(),
            account_count: *account_count.get(&team).unwrap_or(&0),
            active_accounts: events
                .iter()
                .filter_map(|event| event.user_id)
                .collect::<HashSet<_>>()
                .len() as i64,
            pv: events
                .iter()
                .filter(|event| event.event_type == "page_view")
                .count() as i64,
            uv: events
                .iter()
                .map(|event| actor_key_for_event(event))
                .collect::<HashSet<_>>()
                .len() as i64,
            sessions: sessions.len() as i64,
            agent_questions: agent_stats.iter().map(|stat| stat.question_used).sum(),
            token_used: agent_stats.iter().map(|stat| stat.token_used).sum(),
            last_seen_at: latest_time(
                events
                    .iter()
                    .map(|event| event.received_at)
                    .chain(agent_stats.iter().filter_map(|stat| stat.last_used_at)),
            ),
        });
    }

    rows
}

fn top_pages(events: &[&AnalyticsEvent]) -> Vec<AnalyticsTopPage> {
    let mut counts = HashMap::<String, i64>::new();
    for event in events {
        if event.event_type != "page_view" {
            continue;
        }
        let route = event
            .route_path
            .clone()
            .unwrap_or_else(|| DEFAULT_ROUTE_PATH.to_owned());
        *counts.entry(route).or_default() += 1;
    }
    let mut rows = counts
        .into_iter()
        .map(|(route_path, pv)| AnalyticsTopPage { route_path, pv })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.pv);
    rows.reverse();
    rows.truncate(3);
    rows
}

fn agent_questions_by_source(
    dataset: &Dataset,
    metric_kind: SourceMetricKind,
) -> HashMap<String, i64> {
    let mut source_by_session = HashMap::<String, String>::new();
    for event in &dataset.events {
        let key = match metric_kind {
            SourceMetricKind::SourceKind => event.source_kind.clone(),
            SourceMetricKind::SourceHost => event
                .source_host
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
            SourceMetricKind::Campaign => event
                .utm_campaign
                .clone()
                .unwrap_or_else(|| "unknown".to_owned()),
        };
        source_by_session
            .entry(event.session_id.clone())
            .or_insert(key);
    }

    dataset
        .events
        .iter()
        .filter(|event| event.event_type == "agent_question_submit")
        .fold(HashMap::<String, i64>::new(), |mut acc, event| {
            let key = source_by_session
                .get(&event.session_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_owned());
            *acc.entry(key).or_default() += 1;
            acc
        })
}

fn count_events(events: &[AnalyticsEvent], event_type: &str) -> i64 {
    events
        .iter()
        .filter(|event| event.event_type == event_type)
        .count() as i64
}

fn count_unique_actors(events: &[AnalyticsEvent]) -> i64 {
    events
        .iter()
        .map(actor_key_for_event)
        .collect::<HashSet<_>>()
        .len() as i64
}

fn actor_key_for_event(event: &AnalyticsEvent) -> ActorKey {
    if let Some(user_id) = event.user_id {
        return ActorKey(format!("user:{user_id}"));
    }
    if let Some(anonymous_id) = &event.anonymous_id {
        return ActorKey(format!("anon:{anonymous_id}"));
    }
    ActorKey(format!("session:{}", event.session_id))
}

fn team_key_from_app_user(app_user: &AppUser) -> TeamKey {
    normalize_team_key(
        app_user.school_name.as_deref(),
        app_user.team_name.as_deref(),
    )
}

fn team_key_from_event(event: &AnalyticsEvent) -> TeamKey {
    normalize_team_key(event.school_name.as_deref(), event.team_name.as_deref())
}

fn team_key_from_session(session: &AnalyticsSession) -> TeamKey {
    normalize_team_key(session.school_name.as_deref(), session.team_name.as_deref())
}

fn normalize_team_key(school_name: Option<&str>, team_name: Option<&str>) -> TeamKey {
    TeamKey {
        school_name: school_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_UNKNOWN_SCHOOL)
            .to_owned(),
        team_name: team_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_UNKNOWN_TEAM)
            .to_owned(),
    }
}

fn normalize_range(
    from: Option<OffsetDateTime>,
    to: Option<OffsetDateTime>,
) -> (OffsetDateTime, OffsetDateTime) {
    let to = to.unwrap_or_else(OffsetDateTime::now_utc);
    let from = from.unwrap_or_else(|| to - Duration::days(30));
    if from > to { (to, from) } else { (from, to) }
}

fn average_session_duration_sec(sessions: &[AnalyticsSession]) -> i64 {
    if sessions.is_empty() {
        return 0;
    }
    let total = sessions
        .iter()
        .map(|session| {
            (session.last_seen_at - session.first_seen_at)
                .whole_seconds()
                .max(0)
        })
        .sum::<i64>();
    total / sessions.len() as i64
}

fn average_session_duration_sec_refs(sessions: &[&AnalyticsSession]) -> i64 {
    if sessions.is_empty() {
        return 0;
    }
    let total = sessions
        .iter()
        .map(|session| {
            (session.last_seen_at - session.first_seen_at)
                .whole_seconds()
                .max(0)
        })
        .sum::<i64>();
    total / sessions.len() as i64
}

fn bounce_rate(sessions: &[AnalyticsSession]) -> f64 {
    if sessions.is_empty() {
        return 0.0;
    }
    let bounced = sessions
        .iter()
        .filter(|session| session.page_view_count <= 1)
        .count() as f64;
    bounced / sessions.len() as f64
}

fn empty_daily(date: time::Date) -> AnalyticsDailyTraffic {
    AnalyticsDailyTraffic {
        date,
        pv: 0,
        uv: 0,
        sessions: 0,
        active_accounts: 0,
        agent_questions: 0,
        token_used: 0,
        errors: 0,
    }
}

fn sort_account_rows(
    rows: &mut [AnalyticsAccountRow],
    sort_by: Option<AnalyticsAccountSortBy>,
    sort_order: Option<AnalyticsSortOrder>,
) {
    match sort_by.unwrap_or(AnalyticsAccountSortBy::LastSeenAt) {
        AnalyticsAccountSortBy::LastSeenAt => rows.sort_by_key(|row| row.last_seen_at),
        AnalyticsAccountSortBy::Pv => rows.sort_by_key(|row| row.pv),
        AnalyticsAccountSortBy::Sessions => rows.sort_by_key(|row| row.sessions),
        AnalyticsAccountSortBy::AgentQuestions => rows.sort_by_key(|row| row.agent_questions),
        AnalyticsAccountSortBy::ActiveDays => rows.sort_by_key(|row| row.active_days),
    }
    if !matches!(sort_order, Some(AnalyticsSortOrder::Asc)) {
        rows.reverse();
    }
}

fn sort_school_team_rows(
    rows: &mut [AnalyticsSchoolTeamRow],
    sort_by: Option<AnalyticsSchoolTeamSortBy>,
    sort_order: Option<AnalyticsSortOrder>,
) {
    match sort_by.unwrap_or(AnalyticsSchoolTeamSortBy::LastSeenAt) {
        AnalyticsSchoolTeamSortBy::LastSeenAt => rows.sort_by_key(|row| row.last_seen_at),
        AnalyticsSchoolTeamSortBy::ActiveAccounts => rows.sort_by_key(|row| row.active_accounts),
        AnalyticsSchoolTeamSortBy::Pv => rows.sort_by_key(|row| row.pv),
        AnalyticsSchoolTeamSortBy::Sessions => rows.sort_by_key(|row| row.sessions),
        AnalyticsSchoolTeamSortBy::AgentQuestions => rows.sort_by_key(|row| row.agent_questions),
    }
    if !matches!(sort_order, Some(AnalyticsSortOrder::Asc)) {
        rows.reverse();
    }
}

fn latest_time(values: impl Iterator<Item = OffsetDateTime>) -> Option<OffsetDateTime> {
    values.max()
}

fn paginate_rows<T>(rows: Vec<T>, pagination: PaginationParams) -> PaginatedResponse<T> {
    let page = pagination.validate();
    let limit = page.limit();
    let total = rows.len() as u64;
    let total_pages = if total == 0 { 0 } else { total.div_ceil(limit) };
    let offset = ((page.page - 1) * limit) as usize;
    let items = rows
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .collect::<Vec<_>>();

    PaginatedResponse {
        items,
        page: page.page,
        page_size: limit,
        total,
        total_pages,
        has_next: page.page < total_pages,
        has_prev: page.page > 1 && total_pages > 0,
    }
}

fn trim_to_len(value: Option<String>, max_len: usize) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.chars().take(max_len).collect())
        }
    })
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

async fn commit_or_rollback<T>(tx: DbContext, result: BizResult<T>) -> BizResult<T> {
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}
