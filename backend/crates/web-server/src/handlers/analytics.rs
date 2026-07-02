use axum::Json;
use db_core::error::BizError;
use error_code::app as app_error;
use service::api::analytics::AnalyticsApi;
use toolcraft_axum_kit::{ApiError, IntoCommonResponse, ResponseResult};
use validator::Validate;

use crate::{
    dto::analytics::{
        AnalyticsDeviceType, AnalyticsEventStatus, AnalyticsEventType, AnalyticsSourceKind,
        TrackAnalyticsEventRequest, TrackAnalyticsEventsRequest, TrackAnalyticsEventsResponse,
    },
    error::from_biz_error,
    statics::db_manager::get_app_ctx,
};

pub async fn track_anonymous_events(
    Json(req): Json<TrackAnalyticsEventsRequest>,
) -> ResponseResult<TrackAnalyticsEventsResponse> {
    req.validate().map_err(analytics_request_invalid)?;
    for event in &req.events {
        event.validate().map_err(analytics_request_invalid)?;
    }

    let api = AnalyticsApi::new(get_app_ctx());
    let resp = api
        .track_anonymous_events(
            req.events
                .into_iter()
                .map(map_track_event_to_service)
                .collect(),
        )
        .await
        .map_err(from_biz_error)?;

    Ok(map_track_response(resp).into_common_response().to_json())
}

pub(crate) fn analytics_request_invalid(err: validator::ValidationErrors) -> ApiError {
    from_biz_error(BizError::new(
        app_error::ANALYTICS_REQUEST_INVALID,
        format!("analytics request invalid: {err}"),
    ))
}

pub(crate) fn map_track_event_to_service(
    event: TrackAnalyticsEventRequest,
) -> service::dto::analytics::TrackAnalyticsEventInput {
    service::dto::analytics::TrackAnalyticsEventInput {
        event_id: event.event_id,
        event_type: map_event_type_to_service(event.event_type),
        occurred_at: event.occurred_at,
        anonymous_id: event.anonymous_id,
        session_id: event.session_id,
        route_path: event.route_path,
        page_title: event.page_title,
        referrer: event.referrer,
        source_kind: map_source_kind_to_service(event.source_kind),
        source_host: event.source_host,
        utm_source: event.utm_source,
        utm_medium: event.utm_medium,
        utm_campaign: event.utm_campaign,
        utm_content: event.utm_content,
        utm_term: event.utm_term,
        feature_key: event.feature_key,
        device_type: map_device_type_to_service(event.device_type),
        browser_name: event.browser_name,
        os_name: event.os_name,
        viewport_width: event.viewport_width,
        viewport_height: event.viewport_height,
        status: map_event_status_to_service(event.status),
        metadata: event.metadata,
    }
}

pub(crate) fn map_track_response(
    resp: service::dto::analytics::TrackAnalyticsEventsResponse,
) -> TrackAnalyticsEventsResponse {
    TrackAnalyticsEventsResponse {
        accepted: resp.accepted,
        duplicated: resp.duplicated,
        rejected: resp.rejected,
    }
}

pub(crate) fn map_overview_response(
    resp: service::dto::analytics::AnalyticsOverviewResponse,
) -> crate::dto::analytics::AnalyticsOverviewResponse {
    crate::dto::analytics::AnalyticsOverviewResponse {
        summary: crate::dto::analytics::AnalyticsSummary {
            pv: resp.summary.pv,
            uv: resp.summary.uv,
            sessions: resp.summary.sessions,
            active_accounts: resp.summary.active_accounts,
            agent_questions: resp.summary.agent_questions,
            token_used: resp.summary.token_used,
            avg_session_duration_sec: resp.summary.avg_session_duration_sec,
            bounce_rate: resp.summary.bounce_rate,
            frontend_errors: resp.summary.frontend_errors,
            api_errors: resp.summary.api_errors,
        },
        daily_traffic: resp
            .daily_traffic
            .into_iter()
            .map(|row| crate::dto::analytics::AnalyticsDailyTraffic {
                date: row.date,
                pv: row.pv,
                uv: row.uv,
                sessions: row.sessions,
                active_accounts: row.active_accounts,
                agent_questions: row.agent_questions,
                token_used: row.token_used,
                errors: row.errors,
            })
            .collect(),
    }
}

pub(crate) fn map_sources_response(
    resp: service::dto::analytics::AnalyticsSourcesResponse,
) -> crate::dto::analytics::AnalyticsSourcesResponse {
    crate::dto::analytics::AnalyticsSourcesResponse {
        source_kinds: resp
            .source_kinds
            .into_iter()
            .map(map_source_metric)
            .collect(),
        source_hosts: resp
            .source_hosts
            .into_iter()
            .map(map_source_metric)
            .collect(),
        campaigns: resp.campaigns.into_iter().map(map_source_metric).collect(),
    }
}

pub(crate) fn map_pages_response(
    resp: service::dto::analytics::AnalyticsPagesResponse,
) -> crate::dto::analytics::AnalyticsPagesResponse {
    crate::dto::analytics::AnalyticsPagesResponse {
        items: resp
            .items
            .into_iter()
            .map(|row| crate::dto::analytics::AnalyticsPageMetric {
                route_path: row.route_path,
                pv: row.pv,
                uv: row.uv,
                sessions: row.sessions,
                avg_session_duration_sec: row.avg_session_duration_sec,
                entry_count: row.entry_count,
                exit_count: row.exit_count,
                errors: row.errors,
            })
            .collect(),
    }
}

pub(crate) fn map_account_row(
    row: service::dto::analytics::AnalyticsAccountRow,
) -> crate::dto::analytics::AnalyticsAccountRow {
    crate::dto::analytics::AnalyticsAccountRow {
        user_id: row.user_id,
        display_id: row.display_id,
        display_name: row.display_name,
        school_name: row.school_name,
        team_name: row.team_name,
        pv: row.pv,
        sessions: row.sessions,
        active_days: row.active_days,
        agent_questions: row.agent_questions,
        token_used: row.token_used,
        last_seen_at: row.last_seen_at,
        top_pages: row
            .top_pages
            .into_iter()
            .map(|page| crate::dto::analytics::AnalyticsTopPage {
                route_path: page.route_path,
                pv: page.pv,
            })
            .collect(),
    }
}

pub(crate) fn map_school_team_row(
    row: service::dto::analytics::AnalyticsSchoolTeamRow,
) -> crate::dto::analytics::AnalyticsSchoolTeamRow {
    crate::dto::analytics::AnalyticsSchoolTeamRow {
        school_name: row.school_name,
        team_name: row.team_name,
        account_count: row.account_count,
        active_accounts: row.active_accounts,
        pv: row.pv,
        uv: row.uv,
        sessions: row.sessions,
        agent_questions: row.agent_questions,
        token_used: row.token_used,
        last_seen_at: row.last_seen_at,
    }
}

pub(crate) fn map_granularity_to_service(
    value: crate::dto::analytics::AnalyticsGranularity,
) -> service::dto::analytics::AnalyticsGranularity {
    match value {
        crate::dto::analytics::AnalyticsGranularity::Day => {
            service::dto::analytics::AnalyticsGranularity::Day
        }
        crate::dto::analytics::AnalyticsGranularity::Week => {
            service::dto::analytics::AnalyticsGranularity::Week
        }
    }
}

pub(crate) fn map_account_sort_by_to_service(
    value: crate::dto::analytics::AnalyticsAccountSortBy,
) -> service::dto::analytics::AnalyticsAccountSortBy {
    match value {
        crate::dto::analytics::AnalyticsAccountSortBy::LastSeenAt => {
            service::dto::analytics::AnalyticsAccountSortBy::LastSeenAt
        }
        crate::dto::analytics::AnalyticsAccountSortBy::Pv => {
            service::dto::analytics::AnalyticsAccountSortBy::Pv
        }
        crate::dto::analytics::AnalyticsAccountSortBy::Sessions => {
            service::dto::analytics::AnalyticsAccountSortBy::Sessions
        }
        crate::dto::analytics::AnalyticsAccountSortBy::AgentQuestions => {
            service::dto::analytics::AnalyticsAccountSortBy::AgentQuestions
        }
        crate::dto::analytics::AnalyticsAccountSortBy::ActiveDays => {
            service::dto::analytics::AnalyticsAccountSortBy::ActiveDays
        }
    }
}

pub(crate) fn map_school_team_sort_by_to_service(
    value: crate::dto::analytics::AnalyticsSchoolTeamSortBy,
) -> service::dto::analytics::AnalyticsSchoolTeamSortBy {
    match value {
        crate::dto::analytics::AnalyticsSchoolTeamSortBy::ActiveAccounts => {
            service::dto::analytics::AnalyticsSchoolTeamSortBy::ActiveAccounts
        }
        crate::dto::analytics::AnalyticsSchoolTeamSortBy::Pv => {
            service::dto::analytics::AnalyticsSchoolTeamSortBy::Pv
        }
        crate::dto::analytics::AnalyticsSchoolTeamSortBy::Sessions => {
            service::dto::analytics::AnalyticsSchoolTeamSortBy::Sessions
        }
        crate::dto::analytics::AnalyticsSchoolTeamSortBy::AgentQuestions => {
            service::dto::analytics::AnalyticsSchoolTeamSortBy::AgentQuestions
        }
        crate::dto::analytics::AnalyticsSchoolTeamSortBy::LastSeenAt => {
            service::dto::analytics::AnalyticsSchoolTeamSortBy::LastSeenAt
        }
    }
}

pub(crate) fn map_sort_order_to_service(
    value: crate::dto::analytics::AnalyticsSortOrder,
) -> service::dto::analytics::AnalyticsSortOrder {
    match value {
        crate::dto::analytics::AnalyticsSortOrder::Asc => {
            service::dto::analytics::AnalyticsSortOrder::Asc
        }
        crate::dto::analytics::AnalyticsSortOrder::Desc => {
            service::dto::analytics::AnalyticsSortOrder::Desc
        }
    }
}

fn map_source_metric(
    row: service::dto::analytics::AnalyticsSourceMetric,
) -> crate::dto::analytics::AnalyticsSourceMetric {
    crate::dto::analytics::AnalyticsSourceMetric {
        key: row.key,
        pv: row.pv,
        uv: row.uv,
        sessions: row.sessions,
        agent_questions: row.agent_questions,
    }
}

fn map_event_type_to_service(
    value: AnalyticsEventType,
) -> service::dto::analytics::AnalyticsEventType {
    match value {
        AnalyticsEventType::PageView => service::dto::analytics::AnalyticsEventType::PageView,
        AnalyticsEventType::SessionStart => {
            service::dto::analytics::AnalyticsEventType::SessionStart
        }
        AnalyticsEventType::SessionEnd => service::dto::analytics::AnalyticsEventType::SessionEnd,
        AnalyticsEventType::FeatureClick => {
            service::dto::analytics::AnalyticsEventType::FeatureClick
        }
        AnalyticsEventType::AgentQuestionSubmit => {
            service::dto::analytics::AnalyticsEventType::AgentQuestionSubmit
        }
        AnalyticsEventType::AgentAnswerView => {
            service::dto::analytics::AnalyticsEventType::AgentAnswerView
        }
        AnalyticsEventType::ProcessPanelOpen => {
            service::dto::analytics::AnalyticsEventType::ProcessPanelOpen
        }
        AnalyticsEventType::ReportView => service::dto::analytics::AnalyticsEventType::ReportView,
        AnalyticsEventType::ReportCopy => service::dto::analytics::AnalyticsEventType::ReportCopy,
        AnalyticsEventType::ReportDownload => {
            service::dto::analytics::AnalyticsEventType::ReportDownload
        }
        AnalyticsEventType::FileUpload => service::dto::analytics::AnalyticsEventType::FileUpload,
        AnalyticsEventType::LoginClick => service::dto::analytics::AnalyticsEventType::LoginClick,
        AnalyticsEventType::LoginSuccess => {
            service::dto::analytics::AnalyticsEventType::LoginSuccess
        }
        AnalyticsEventType::RegisterSuccess => {
            service::dto::analytics::AnalyticsEventType::RegisterSuccess
        }
        AnalyticsEventType::FrontendError => {
            service::dto::analytics::AnalyticsEventType::FrontendError
        }
        AnalyticsEventType::ApiError => service::dto::analytics::AnalyticsEventType::ApiError,
    }
}

fn map_source_kind_to_service(
    value: AnalyticsSourceKind,
) -> service::dto::analytics::AnalyticsSourceKind {
    match value {
        AnalyticsSourceKind::Direct => service::dto::analytics::AnalyticsSourceKind::Direct,
        AnalyticsSourceKind::Search => service::dto::analytics::AnalyticsSourceKind::Search,
        AnalyticsSourceKind::Referral => service::dto::analytics::AnalyticsSourceKind::Referral,
        AnalyticsSourceKind::Social => service::dto::analytics::AnalyticsSourceKind::Social,
        AnalyticsSourceKind::Ad => service::dto::analytics::AnalyticsSourceKind::Ad,
        AnalyticsSourceKind::Internal => service::dto::analytics::AnalyticsSourceKind::Internal,
        AnalyticsSourceKind::Unknown => service::dto::analytics::AnalyticsSourceKind::Unknown,
    }
}

fn map_device_type_to_service(
    value: AnalyticsDeviceType,
) -> service::dto::analytics::AnalyticsDeviceType {
    match value {
        AnalyticsDeviceType::Desktop => service::dto::analytics::AnalyticsDeviceType::Desktop,
        AnalyticsDeviceType::Mobile => service::dto::analytics::AnalyticsDeviceType::Mobile,
        AnalyticsDeviceType::Tablet => service::dto::analytics::AnalyticsDeviceType::Tablet,
        AnalyticsDeviceType::Bot => service::dto::analytics::AnalyticsDeviceType::Bot,
        AnalyticsDeviceType::Unknown => service::dto::analytics::AnalyticsDeviceType::Unknown,
    }
}

fn map_event_status_to_service(
    value: AnalyticsEventStatus,
) -> service::dto::analytics::AnalyticsEventStatus {
    match value {
        AnalyticsEventStatus::Ok => service::dto::analytics::AnalyticsEventStatus::Ok,
        AnalyticsEventStatus::Failed => service::dto::analytics::AnalyticsEventStatus::Failed,
    }
}
