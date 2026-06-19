use std::collections::{HashMap, HashSet};

use db_core::{
    DbContext, PaginatedResponse, PaginationParams,
    error::{BizError, BizResult},
};
use error_code::{admin as admin_error, app as app_error};
use repo::table::{
    agent_usage_events::{AgentUsageEventService, CreateAgentUsageEvent, TeamUsageEventStats},
    app_users::{AppUser, AppUserService, AppUserStatus},
    team_usage_quotas::{
        CreateTeamUsageQuota, TeamUsageQuota, TeamUsageQuotaService, TeamUsageQuotaStatus,
        UpdateTeamUsageQuota,
    },
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::admin::AdminApi,
    dto::team_usage::{
        AgentQuestionPreflight, AgentQuestionPreflightRequest, ListTeamUsagesRequest,
        RecordAgentQuestionStartedRequest, SortOrder, TeamUsageMemberResponse,
        TeamUsageMemberStatus, TeamUsageResponse, TeamUsageSortBy, UpdateTeamUsageQuotaRequest,
    },
};

const PERM_TEAM_USAGE: &str = "usage:team_usage";
const DEFAULT_QUESTION_LIMIT: i32 = 50;
const DEFAULT_UNKNOWN_SCHOOL: &str = "未填写学校";
const DEFAULT_UNKNOWN_TEAM: &str = "未填写队伍";

pub struct TeamUsageApi {
    admin_api: AdminApi,
    app_user_svc: AppUserService,
    quota_svc: TeamUsageQuotaService,
    usage_event_svc: AgentUsageEventService,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TeamKey {
    school_name: String,
    team_name: String,
}

impl TeamUsageApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            admin_api: AdminApi::new(db.clone()),
            app_user_svc: AppUserService::new(db.clone()),
            quota_svc: TeamUsageQuotaService::new(db.clone()),
            usage_event_svc: AgentUsageEventService::new(db),
        }
    }

    pub async fn preflight_agent_question(
        &self,
        req: AgentQuestionPreflightRequest,
    ) -> BizResult<AgentQuestionPreflight> {
        let user_id = parse_uuid(
            &req.user_id,
            "user_id",
            app_error::TEAM_USAGE_REQUEST_INVALID,
        )?;
        let app_user = self.ensure_app_user_enabled(user_id).await?;
        let team_key = team_key_from_app_user(&app_user);
        let quota = self.ensure_quota(&team_key).await?;

        if quota.status == TeamUsageQuotaStatus::Disabled {
            return Err(BizError::new(
                app_error::TEAM_USAGE_DISABLED,
                "team usage is disabled".to_string(),
            ));
        }

        let stats = self
            .usage_event_svc
            .team_stats_by_team(&team_key.school_name, &team_key.team_name)
            .await?;
        if stats.question_used >= i64::from(quota.question_limit) {
            return Err(BizError::new(
                app_error::TEAM_USAGE_QUOTA_EXCEEDED,
                "team question quota exceeded".to_string(),
            ));
        }

        Ok(AgentQuestionPreflight {
            user_id,
            school_name: team_key.school_name,
            team_name: team_key.team_name,
            question_limit: quota.question_limit,
            question_used: stats.question_used,
            session_id: req.session_id,
            agent_id: req.agent_id,
            conversation_id: req
                .conversation_id
                .as_deref()
                .map(|value| {
                    parse_uuid(
                        value,
                        "conversation_id",
                        app_error::TEAM_USAGE_REQUEST_INVALID,
                    )
                })
                .transpose()?,
        })
    }

    pub async fn record_agent_question_started(
        &self,
        req: RecordAgentQuestionStartedRequest,
    ) -> BizResult<()> {
        let workspace_id = req
            .workspace_id
            .as_deref()
            .map(|value| parse_uuid(value, "workspace_id", app_error::TEAM_USAGE_REQUEST_INVALID))
            .transpose()?;
        let run_id = req
            .run_id
            .as_deref()
            .map(|value| parse_uuid(value, "run_id", app_error::TEAM_USAGE_REQUEST_INVALID))
            .transpose()?;

        self.usage_event_svc
            .create(CreateAgentUsageEvent {
                id: Uuid::new_v4(),
                user_id: req.preflight.user_id,
                school_name: req.preflight.school_name,
                team_name: req.preflight.team_name,
                conversation_id: req.preflight.conversation_id,
                workspace_id,
                run_id,
                session_id: req.preflight.session_id,
                agent_id: req.preflight.agent_id,
                question_count: 1,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
                status: "started".to_string(),
                metadata: json!({}),
            })
            .await?;

        Ok(())
    }

    pub async fn list_team_usages(
        &self,
        current_admin_user_id: String,
        req: ListTeamUsagesRequest,
    ) -> BizResult<PaginatedResponse<TeamUsageResponse>> {
        self.ensure_admin_permission(current_admin_user_id).await?;
        let mut rows = self.build_team_usage_rows().await?;

        if let Some(keyword) = normalize_optional(req.keyword) {
            let keyword = keyword.to_ascii_lowercase();
            rows.retain(|row| {
                row.school_name.to_ascii_lowercase().contains(&keyword)
                    || row.team_name.to_ascii_lowercase().contains(&keyword)
                    || row
                        .remark
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&keyword)
            });
        }
        if let Some(status) = req.status {
            rows.retain(|row| row.status == status);
        }

        sort_team_usage_rows(&mut rows, req.sort_by, req.sort_order);

        Ok(paginate_rows(
            rows,
            PaginationParams::new(req.page, req.page_size),
        ))
    }

    pub async fn get_team_usage(
        &self,
        current_admin_user_id: String,
        team_id: String,
    ) -> BizResult<TeamUsageResponse> {
        self.ensure_admin_permission(current_admin_user_id).await?;
        let quota = self.ensure_quota_exists(&team_id).await?;
        self.map_quota_to_response(quota).await
    }

    pub async fn update_team_quota(
        &self,
        current_admin_user_id: String,
        req: UpdateTeamUsageQuotaRequest,
    ) -> BizResult<TeamUsageResponse> {
        self.ensure_admin_permission(current_admin_user_id).await?;
        if req.question_limit < 0 || req.question_limit > 1_000_000 {
            return Err(BizError::new(
                admin_error::ADMIN_USAGE_REQUEST_INVALID,
                "question_limit must be between 0 and 1000000".to_string(),
            ));
        }
        if let Some(token_limit) = req.token_limit
            && !(0..=1_000_000_000).contains(&token_limit)
        {
            return Err(BizError::new(
                admin_error::ADMIN_USAGE_REQUEST_INVALID,
                "token_limit must be between 0 and 1000000000".to_string(),
            ));
        }

        let quota = self.ensure_quota_exists(&req.team_id).await?;
        let updated = self
            .quota_svc
            .update(UpdateTeamUsageQuota {
                id: quota.id,
                question_limit: req.question_limit,
                token_limit: req.token_limit,
                status: req.status,
                remark: normalize_optional(req.remark),
            })
            .await?;

        self.map_quota_to_response(updated).await
    }

    pub async fn list_team_members(
        &self,
        current_admin_user_id: String,
        team_id: String,
    ) -> BizResult<Vec<TeamUsageMemberResponse>> {
        self.ensure_admin_permission(current_admin_user_id).await?;
        let quota = self.ensure_quota_exists(&team_id).await?;
        let team_key = TeamKey {
            school_name: quota.school_name,
            team_name: quota.team_name,
        };
        let members = self.list_members_by_normalized_team(&team_key).await?;

        Ok(members.into_iter().map(map_member_response).collect())
    }

    async fn build_team_usage_rows(&self) -> BizResult<Vec<TeamUsageResponse>> {
        let app_users = self.app_user_svc.list_all().await?;
        let quotas = self.quota_svc.list_all().await?;
        let stats = self.usage_event_svc.team_stats().await?;
        let member_counts = self.app_user_svc.count_members_by_team().await?;

        let mut team_keys = HashSet::<TeamKey>::new();
        for app_user in &app_users {
            team_keys.insert(team_key_from_app_user(app_user));
        }
        for quota in &quotas {
            team_keys.insert(TeamKey {
                school_name: quota.school_name.clone(),
                team_name: quota.team_name.clone(),
            });
        }
        for stat in &stats {
            team_keys.insert(TeamKey {
                school_name: stat.school_name.clone(),
                team_name: stat.team_name.clone(),
            });
        }

        let mut quotas_by_team = quotas
            .into_iter()
            .map(|quota| {
                (
                    TeamKey {
                        school_name: quota.school_name.clone(),
                        team_name: quota.team_name.clone(),
                    },
                    quota,
                )
            })
            .collect::<HashMap<_, _>>();
        let stats_by_team = stats
            .into_iter()
            .map(|stat| {
                (
                    TeamKey {
                        school_name: stat.school_name.clone(),
                        team_name: stat.team_name.clone(),
                    },
                    stat,
                )
            })
            .collect::<HashMap<_, _>>();
        let member_counts_by_team = member_counts
            .into_iter()
            .map(|(school_name, team_name, count)| {
                (
                    normalize_team_key(school_name.as_deref(), team_name.as_deref()),
                    count,
                )
            })
            .collect::<HashMap<_, _>>();

        let mut rows = Vec::new();
        for team_key in team_keys {
            let quota = match quotas_by_team.remove(&team_key) {
                Some(quota) => quota,
                None => self.ensure_quota(&team_key).await?,
            };
            let stats = stats_by_team
                .get(&team_key)
                .cloned()
                .unwrap_or_else(|| empty_stats(&team_key));
            rows.push(map_team_usage_response(
                quota,
                member_counts_by_team
                    .get(&team_key)
                    .copied()
                    .unwrap_or_default()
                    .max(0) as u64,
                stats,
            ));
        }

        Ok(rows)
    }

    async fn map_quota_to_response(&self, quota: TeamUsageQuota) -> BizResult<TeamUsageResponse> {
        let stats = self
            .usage_event_svc
            .team_stats_by_team(&quota.school_name, &quota.team_name)
            .await?;
        let member_count = self
            .list_members_by_normalized_team(&TeamKey {
                school_name: quota.school_name.clone(),
                team_name: quota.team_name.clone(),
            })
            .await?
            .len() as u64;
        Ok(map_team_usage_response(quota, member_count, stats))
    }

    async fn list_members_by_normalized_team(&self, team_key: &TeamKey) -> BizResult<Vec<AppUser>> {
        let members = self.app_user_svc.list_all().await?;
        Ok(members
            .into_iter()
            .filter(|member| team_key_from_app_user(member) == *team_key)
            .collect())
    }

    async fn ensure_quota(&self, team_key: &TeamKey) -> BizResult<TeamUsageQuota> {
        if let Some(quota) = self
            .quota_svc
            .get_by_team(&team_key.school_name, &team_key.team_name)
            .await?
        {
            return Ok(quota);
        }

        self.quota_svc
            .create(CreateTeamUsageQuota {
                id: Uuid::new_v4(),
                school_name: team_key.school_name.clone(),
                team_name: team_key.team_name.clone(),
                question_limit: DEFAULT_QUESTION_LIMIT,
                token_limit: None,
                status: TeamUsageQuotaStatus::Enabled,
                remark: None,
            })
            .await
    }

    async fn ensure_quota_exists(&self, team_id: &str) -> BizResult<TeamUsageQuota> {
        let team_id = parse_uuid(team_id, "team_id", admin_error::ADMIN_USAGE_REQUEST_INVALID)?;
        self.quota_svc.get_by_id(team_id).await?.ok_or_else(|| {
            BizError::new(
                admin_error::ADMIN_USAGE_TEAM_NOT_FOUND,
                "team usage quota not found".to_string(),
            )
        })
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<AppUser> {
        let app_user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if app_user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user disabled".to_string(),
            ));
        }

        Ok(app_user)
    }

    async fn ensure_admin_permission(&self, current_admin_user_id: String) -> BizResult<()> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_TEAM_USAGE)
            .await
    }
}

fn map_team_usage_response(
    quota: TeamUsageQuota,
    member_count: u64,
    stats: TeamUsageEventStats,
) -> TeamUsageResponse {
    let question_remaining = (i64::from(quota.question_limit) - stats.question_used).max(0);

    TeamUsageResponse {
        id: quota.id.to_string(),
        school_name: quota.school_name,
        team_name: quota.team_name,
        member_count,
        question_limit: quota.question_limit,
        question_used: stats.question_used,
        question_remaining,
        token_limit: quota.token_limit,
        token_used: stats.token_used,
        status: quota.status,
        remark: quota.remark,
        last_used_at: stats.last_used_at,
        created_at: quota.created_at,
        updated_at: quota.updated_at,
    }
}

fn map_member_response(app_user: AppUser) -> TeamUsageMemberResponse {
    TeamUsageMemberResponse {
        user_id: app_user.user_id.to_string(),
        display_id: app_user.display_id,
        display_name: app_user.display_name,
        school_name: app_user.school_name,
        team_name: app_user.team_name,
        status: match app_user.status {
            AppUserStatus::Enabled => TeamUsageMemberStatus::Enabled,
            AppUserStatus::Disabled => TeamUsageMemberStatus::Disabled,
        },
        created_at: app_user.created_at,
        updated_at: app_user.updated_at,
    }
}

fn empty_stats(team_key: &TeamKey) -> TeamUsageEventStats {
    TeamUsageEventStats {
        school_name: team_key.school_name.clone(),
        team_name: team_key.team_name.clone(),
        question_used: 0,
        token_used: 0,
        last_used_at: None,
    }
}

fn team_key_from_app_user(app_user: &AppUser) -> TeamKey {
    normalize_team_key(
        app_user.school_name.as_deref(),
        app_user.team_name.as_deref(),
    )
}

fn normalize_team_key(school_name: Option<&str>, team_name: Option<&str>) -> TeamKey {
    TeamKey {
        school_name: normalize_optional(school_name.map(ToOwned::to_owned))
            .unwrap_or_else(|| DEFAULT_UNKNOWN_SCHOOL.to_string()),
        team_name: normalize_optional(team_name.map(ToOwned::to_owned))
            .unwrap_or_else(|| DEFAULT_UNKNOWN_TEAM.to_string()),
    }
}

fn sort_team_usage_rows(
    rows: &mut [TeamUsageResponse],
    sort_by: Option<TeamUsageSortBy>,
    sort_order: Option<SortOrder>,
) {
    match sort_by.unwrap_or(TeamUsageSortBy::UpdatedAt) {
        TeamUsageSortBy::UpdatedAt => rows.sort_by_key(|row| row.updated_at),
        TeamUsageSortBy::QuestionUsed => rows.sort_by_key(|row| row.question_used),
        TeamUsageSortBy::QuestionRemaining => rows.sort_by_key(|row| row.question_remaining),
        TeamUsageSortBy::MemberCount => rows.sort_by_key(|row| row.member_count),
        TeamUsageSortBy::LastUsedAt => rows.sort_by_key(|row| row.last_used_at),
    }

    if !matches!(sort_order, Some(SortOrder::Asc)) {
        rows.reverse();
    }
}

fn paginate_rows(
    rows: Vec<TeamUsageResponse>,
    pagination: PaginationParams,
) -> PaginatedResponse<TeamUsageResponse> {
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

fn parse_uuid(value: &str, label: &str, code: i16) -> BizResult<Uuid> {
    Uuid::parse_str(value)
        .map_err(|_| BizError::new(code, format!("invalid {label}: must be uuid")))
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
