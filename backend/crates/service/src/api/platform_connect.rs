use std::collections::HashMap;

use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    channel_connections::{
        ChannelConnection, ChannelConnectionFilter, ChannelConnectionService,
        ChannelConnectionStatus, UpdateChannelConnection, UpsertChannelConnection,
    },
    platform_connections::{
        PlatformConnection, PlatformConnectionFilter, PlatformConnectionService,
        PlatformConnectionStatus, PlatformSyncStatus, UpdatePlatformConnection,
        UpsertPlatformConnection,
    },
    store_metrics::{StoreMetric, StoreMetricFilter, StoreMetricService, UpsertStoreMetric},
};
use serde_json::json;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::platform_execution::{
    PlatformExecutionActionSpec, PlatformExecutionParams, PlatformExecutionReadiness,
    platform_execution_action_specs, platform_execution_details, platform_execution_params,
    platform_execution_readiness_status,
};
use crate::dto::platform_connect::*;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct PlatformConnectApi {
    app_user_svc: AppUserService,
    platform_svc: PlatformConnectionService,
    channel_svc: ChannelConnectionService,
    metric_svc: StoreMetricService,
}

impl PlatformConnectApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            platform_svc: PlatformConnectionService::new(db.clone()),
            channel_svc: ChannelConnectionService::new(db.clone()),
            metric_svc: StoreMetricService::new(db),
        }
    }

    pub fn platform_catalog(&self) -> PlatformCatalogResponse {
        PlatformCatalogResponse {
            platforms: platform_specs(),
        }
    }

    pub async fn execution_adapter_catalog(
        &self,
        user_id: String,
    ) -> BizResult<PlatformExecutionAdapterCatalogResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connections = self
            .platform_svc
            .list(PlatformConnectionFilter {
                user_id,
                platform: None,
                enabled: None,
            })
            .await?;
        let connection_map = connections
            .into_iter()
            .map(|item| (item.platform.clone(), item))
            .collect::<HashMap<_, _>>();
        let adapters = platform_specs()
            .into_iter()
            .map(|spec| {
                let connection = connection_map.get(spec.platform.as_str());
                let connected = connection.is_some();
                let enabled = connection.is_some_and(|item| item.enabled);
                let connection_status = connection
                    .map(|item| item.status.as_str().to_string())
                    .unwrap_or_else(|| "missing".to_string());
                let actions = platform_execution_action_specs()
                    .iter()
                    .copied()
                    .map(|action| {
                        map_platform_execution_action(
                            action,
                            connected,
                            enabled,
                            connection_status.as_str(),
                        )
                    })
                    .collect();
                PlatformExecutionAdapterSpec {
                    platform: spec.platform,
                    display: spec.display,
                    connected,
                    enabled,
                    connection_status,
                    actions,
                }
            })
            .collect();
        Ok(PlatformExecutionAdapterCatalogResponse { adapters })
    }

    pub async fn execution_preflight(
        &self,
        req: PlatformExecutionPreflightRequest,
    ) -> BizResult<PlatformExecutionPreflightResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let skill_name = normalize_required(req.skill_name, "skill_name")?;
        let args = req.args.unwrap_or_else(|| json!({}));
        let params = platform_execution_params(&args);
        let connection = if let Some(platform) = params.platform.as_deref() {
            self.platform_svc
                .get_by_user_platform(user_id, platform)
                .await?
        } else {
            None
        };

        Ok(plan_platform_execution_preflight(
            skill_name.as_str(),
            &params,
            connection.as_ref(),
            OffsetDateTime::now_utc(),
        ))
    }

    pub fn channel_catalog(&self) -> ChannelCatalogResponse {
        ChannelCatalogResponse {
            channels: channel_specs(),
        }
    }

    pub async fn list_platform_connections(
        &self,
        req: ListPlatformConnectionsRequest,
    ) -> BizResult<PlatformConnectionListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let platform = parse_optional_platform(req.platform)?;
        let connections = self
            .platform_svc
            .list(PlatformConnectionFilter {
                user_id,
                platform,
                enabled: req.enabled,
            })
            .await?
            .into_iter()
            .map(map_platform_connection)
            .collect();
        Ok(PlatformConnectionListResponse { connections })
    }

    pub async fn save_platform_connection(
        &self,
        req: SavePlatformConnectionRequest,
    ) -> BizResult<PlatformConnectionResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let platform = parse_platform(req.platform)?;
        let connection = self
            .platform_svc
            .upsert(UpsertPlatformConnection {
                id: Uuid::new_v4(),
                user_id,
                platform,
                app_key: normalize_optional(req.app_key),
                app_secret_enc: normalize_optional(req.app_secret),
                access_token_enc: normalize_optional(req.access_token),
                refresh_token_enc: normalize_optional(req.refresh_token),
                token_expires_at: req.token_expires_at,
                shop_id: normalize_optional(req.shop_id),
                shop_name: normalize_optional(req.shop_name),
                enabled: req.enabled.unwrap_or(true),
                status: if req.enabled == Some(false) {
                    PlatformConnectionStatus::Disabled
                } else {
                    PlatformConnectionStatus::Configured
                },
                last_tested_at: None,
                test_result: req.test_result.unwrap_or_else(|| json!({})),
                last_synced_at: req.last_synced_at,
                sync_status: parse_sync_status(req.sync_status)?
                    .unwrap_or(PlatformSyncStatus::Idle),
                sync_message: normalize_optional(req.sync_message),
                extra: req.extra.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(map_platform_connection(connection))
    }

    pub async fn get_platform_connection(
        &self,
        req: GetPlatformConnectionRequest,
    ) -> BizResult<PlatformConnectionResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connection_id = parse_uuid(&req.connection_id, "connection_id")?;
        let connection = self
            .ensure_platform_connection_owner(user_id, connection_id)
            .await?;
        Ok(map_platform_connection(connection))
    }

    pub async fn update_platform_connection(
        &self,
        req: UpdatePlatformConnectionRequest,
    ) -> BizResult<PlatformConnectionResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connection_id = parse_uuid(&req.connection_id, "connection_id")?;
        let current = self
            .ensure_platform_connection_owner(user_id, connection_id)
            .await?;
        let enabled = req.enabled.unwrap_or(current.enabled);
        let status = if enabled {
            current.status
        } else {
            PlatformConnectionStatus::Disabled
        };
        let connection = self
            .platform_svc
            .update(UpdatePlatformConnection {
                id: current.id,
                platform: req
                    .platform
                    .map(parse_platform)
                    .transpose()?
                    .unwrap_or(current.platform),
                app_key: req.app_key.map(normalize_string).unwrap_or(current.app_key),
                app_secret_enc: req
                    .app_secret
                    .map(normalize_string)
                    .unwrap_or(current.app_secret_enc),
                access_token_enc: req
                    .access_token
                    .map(normalize_string)
                    .unwrap_or(current.access_token_enc),
                refresh_token_enc: req
                    .refresh_token
                    .map(normalize_string)
                    .unwrap_or(current.refresh_token_enc),
                token_expires_at: req.token_expires_at.or(current.token_expires_at),
                shop_id: req.shop_id.map(normalize_string).unwrap_or(current.shop_id),
                shop_name: req
                    .shop_name
                    .map(normalize_string)
                    .unwrap_or(current.shop_name),
                enabled,
                status,
                last_tested_at: current.last_tested_at,
                test_result: req.test_result.unwrap_or(current.test_result),
                last_synced_at: req.last_synced_at.or(current.last_synced_at),
                sync_status: parse_sync_status(req.sync_status)?.unwrap_or(current.sync_status),
                sync_message: req
                    .sync_message
                    .map(normalize_string)
                    .unwrap_or(current.sync_message),
                extra: req.extra.unwrap_or(current.extra),
            })
            .await?;
        Ok(map_platform_connection(connection))
    }

    pub async fn disable_platform_connection(
        &self,
        req: DisablePlatformConnectionRequest,
    ) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connection_id = parse_uuid(&req.connection_id, "connection_id")?;
        let current = self
            .ensure_platform_connection_owner(user_id, connection_id)
            .await?;
        self.platform_svc
            .update(UpdatePlatformConnection {
                id: current.id,
                platform: current.platform,
                app_key: current.app_key,
                app_secret_enc: current.app_secret_enc,
                access_token_enc: current.access_token_enc,
                refresh_token_enc: current.refresh_token_enc,
                token_expires_at: current.token_expires_at,
                shop_id: current.shop_id,
                shop_name: current.shop_name,
                enabled: false,
                status: PlatformConnectionStatus::Disabled,
                last_tested_at: current.last_tested_at,
                test_result: current.test_result,
                last_synced_at: current.last_synced_at,
                sync_status: current.sync_status,
                sync_message: current.sync_message,
                extra: current.extra,
            })
            .await?;
        Ok(())
    }

    pub async fn list_channel_connections(
        &self,
        req: ListChannelConnectionsRequest,
    ) -> BizResult<ChannelConnectionListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let channel = parse_optional_channel(req.channel)?;
        let connections = self
            .channel_svc
            .list(ChannelConnectionFilter {
                user_id,
                channel,
                enabled: req.enabled,
            })
            .await?
            .into_iter()
            .map(map_channel_connection)
            .collect();
        Ok(ChannelConnectionListResponse { connections })
    }

    pub async fn save_channel_connection(
        &self,
        req: SaveChannelConnectionRequest,
    ) -> BizResult<ChannelConnectionResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let channel = parse_channel(req.channel)?;
        let connection = self
            .channel_svc
            .upsert(UpsertChannelConnection {
                id: Uuid::new_v4(),
                user_id,
                channel,
                webhook_url: normalize_optional(req.webhook_url),
                bot_token_enc: normalize_optional(req.bot_token),
                chat_id: normalize_optional(req.chat_id),
                secret_enc: normalize_optional(req.secret),
                extra_credentials: req.extra_credentials.unwrap_or_else(|| json!({})),
                enabled: req.enabled.unwrap_or(true),
                status: if req.enabled == Some(false) {
                    ChannelConnectionStatus::Disabled
                } else {
                    ChannelConnectionStatus::Configured
                },
                last_tested_at: None,
                last_error: normalize_optional(req.last_error),
            })
            .await?;
        Ok(map_channel_connection(connection))
    }

    pub async fn get_channel_connection(
        &self,
        req: GetChannelConnectionRequest,
    ) -> BizResult<ChannelConnectionResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connection_id = parse_uuid(&req.connection_id, "connection_id")?;
        let connection = self
            .ensure_channel_connection_owner(user_id, connection_id)
            .await?;
        Ok(map_channel_connection(connection))
    }

    pub async fn update_channel_connection(
        &self,
        req: UpdateChannelConnectionRequest,
    ) -> BizResult<ChannelConnectionResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connection_id = parse_uuid(&req.connection_id, "connection_id")?;
        let current = self
            .ensure_channel_connection_owner(user_id, connection_id)
            .await?;
        let enabled = req.enabled.unwrap_or(current.enabled);
        let status = if enabled {
            current.status
        } else {
            ChannelConnectionStatus::Disabled
        };
        let connection = self
            .channel_svc
            .update(UpdateChannelConnection {
                id: current.id,
                channel: req
                    .channel
                    .map(parse_channel)
                    .transpose()?
                    .unwrap_or(current.channel),
                webhook_url: req
                    .webhook_url
                    .map(normalize_string)
                    .unwrap_or(current.webhook_url),
                bot_token_enc: req
                    .bot_token
                    .map(normalize_string)
                    .unwrap_or(current.bot_token_enc),
                chat_id: req.chat_id.map(normalize_string).unwrap_or(current.chat_id),
                secret_enc: req
                    .secret
                    .map(normalize_string)
                    .unwrap_or(current.secret_enc),
                extra_credentials: req.extra_credentials.unwrap_or(current.extra_credentials),
                enabled,
                status,
                last_tested_at: current.last_tested_at,
                last_error: req
                    .last_error
                    .map(normalize_string)
                    .unwrap_or(current.last_error),
            })
            .await?;
        Ok(map_channel_connection(connection))
    }

    pub async fn disable_channel_connection(
        &self,
        req: DisableChannelConnectionRequest,
    ) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connection_id = parse_uuid(&req.connection_id, "connection_id")?;
        let current = self
            .ensure_channel_connection_owner(user_id, connection_id)
            .await?;
        self.channel_svc
            .update(UpdateChannelConnection {
                id: current.id,
                channel: current.channel,
                webhook_url: current.webhook_url,
                bot_token_enc: current.bot_token_enc,
                chat_id: current.chat_id,
                secret_enc: current.secret_enc,
                extra_credentials: current.extra_credentials,
                enabled: false,
                status: ChannelConnectionStatus::Disabled,
                last_tested_at: current.last_tested_at,
                last_error: current.last_error,
            })
            .await?;
        Ok(())
    }

    pub async fn list_metrics(
        &self,
        req: ListStoreMetricsRequest,
    ) -> BizResult<StoreMetricListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = StoreMetricFilter {
            user_id,
            platform: req.platform.and_then(normalize_string),
            metric_key: req.metric_key.and_then(normalize_string),
            metric_date_from: req.metric_date_from.and_then(normalize_string),
            metric_date_to: req.metric_date_to.and_then(normalize_string),
            source: req.source.and_then(normalize_string),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = StoreMetricFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.metric_svc.count(count_filter).await?;
        let metrics = self
            .metric_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_store_metric)
            .collect();
        Ok(StoreMetricListResponse {
            metrics,
            total,
            page,
            page_size,
        })
    }

    pub async fn save_metric(&self, req: SaveStoreMetricRequest) -> BizResult<StoreMetricResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let platform = normalize_required(req.platform, "platform")?;
        let metric_date = normalize_required(req.metric_date, "metric_date")?;
        let metric_key = normalize_required(req.metric_key, "metric_key")?;
        let metric = self
            .metric_svc
            .upsert(UpsertStoreMetric {
                id: Uuid::new_v4(),
                user_id,
                platform,
                metric_date,
                metric_key,
                metric_value: req.metric_value,
                dimension_key: normalize_optional(req.dimension_key).unwrap_or_default(),
                dimension_value: normalize_optional(req.dimension_value).unwrap_or_default(),
                source: normalize_optional(req.source).unwrap_or_else(|| "import".to_string()),
            })
            .await?;
        Ok(map_store_metric(metric))
    }

    pub async fn sync_status(&self, user_id: String) -> BizResult<PlatformSyncStatusListResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connections = self
            .platform_svc
            .list(PlatformConnectionFilter {
                user_id,
                platform: None,
                enabled: None,
            })
            .await?;
        let mut by_platform = connections
            .into_iter()
            .map(|item| (item.platform.clone(), item))
            .collect::<HashMap<_, _>>();

        let mut platforms = Vec::new();
        for spec in platform_specs() {
            let connection = by_platform.remove(spec.platform.as_str());
            let latest_metric_date = match connection.as_ref() {
                Some(connection) => {
                    self.metric_svc
                        .get_latest_metric_date(user_id, connection.platform.as_str())
                        .await?
                }
                None => None,
            };
            platforms.push(PlatformSyncStatusResponse {
                platform: spec.platform,
                display: spec.display,
                connected: connection.is_some(),
                enabled: connection.as_ref().is_some_and(|item| item.enabled),
                sync_status: connection
                    .as_ref()
                    .map(|item| item.sync_status.as_str().to_string())
                    .unwrap_or_else(|| "idle".to_string()),
                sync_message: connection
                    .as_ref()
                    .and_then(|item| item.sync_message.clone()),
                last_tested_at: connection.as_ref().and_then(|item| item.last_tested_at),
                last_synced_at: connection.as_ref().and_then(|item| item.last_synced_at),
                latest_metric_date,
            });
        }
        Ok(PlatformSyncStatusListResponse { platforms })
    }

    pub async fn summary(&self, user_id: String) -> BizResult<PlatformSummaryResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let connections = self
            .platform_svc
            .list(PlatformConnectionFilter {
                user_id,
                platform: None,
                enabled: None,
            })
            .await?;
        let connection_map = connections
            .into_iter()
            .map(|item| (item.platform.clone(), item))
            .collect::<HashMap<_, _>>();
        let now = OffsetDateTime::now_utc();
        let since_7d = format_date(now - Duration::days(7));
        let since_30d = format_date(now - Duration::days(30));
        let metrics_7d = self.metric_svc.aggregate_since(user_id, &since_7d).await?;
        let metrics_30d = self.metric_svc.aggregate_since(user_id, &since_30d).await?;
        let map_7d = aggregate_map(metrics_7d);
        let map_30d = aggregate_map(metrics_30d);

        let platforms = platform_specs()
            .into_iter()
            .map(|spec| {
                let connection = connection_map.get(spec.platform.as_str());
                let gmv_30d = metric_total(&map_30d, spec.platform.as_str(), "gmv");
                let ad_spend_30d = metric_total(&map_30d, spec.platform.as_str(), "ad_spend");
                let platform = spec.platform;
                PlatformSummaryItem {
                    gmv_7d: metric_total(&map_7d, platform.as_str(), "gmv"),
                    orders_7d: metric_total(&map_7d, platform.as_str(), "orders"),
                    gmv_30d,
                    orders_30d: metric_total(&map_30d, platform.as_str(), "orders"),
                    uv_30d: metric_total(&map_30d, platform.as_str(), "uv"),
                    ad_roi_30d: (ad_spend_30d > 0.0).then_some(gmv_30d / ad_spend_30d),
                    platform,
                    display: spec.display,
                    connected: connection.is_some(),
                    enabled: connection.is_some_and(|item| item.enabled),
                    last_tested_at: connection.and_then(|item| item.last_tested_at),
                }
            })
            .collect();

        Ok(PlatformSummaryResponse { platforms })
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

    async fn ensure_platform_connection_owner(
        &self,
        user_id: Uuid,
        connection_id: Uuid,
    ) -> BizResult<PlatformConnection> {
        let connection = self
            .platform_svc
            .get_by_id(connection_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PLATFORM_CONNECTION_NOT_FOUND,
                    "platform connection not found".to_string(),
                )
            })?;
        if connection.user_id != user_id {
            return Err(BizError::new(
                app_error::PLATFORM_CONNECTION_FORBIDDEN,
                "platform connection does not belong to current user".to_string(),
            ));
        }
        Ok(connection)
    }

    async fn ensure_channel_connection_owner(
        &self,
        user_id: Uuid,
        connection_id: Uuid,
    ) -> BizResult<ChannelConnection> {
        let connection = self
            .channel_svc
            .get_by_id(connection_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::CHANNEL_CONNECTION_NOT_FOUND,
                    "channel connection not found".to_string(),
                )
            })?;
        if connection.user_id != user_id {
            return Err(BizError::new(
                app_error::CHANNEL_CONNECTION_FORBIDDEN,
                "channel connection does not belong to current user".to_string(),
            ));
        }
        Ok(connection)
    }
}

fn platform_specs() -> Vec<PlatformSpec> {
    vec![
        PlatformSpec {
            platform: "taobao".to_string(),
            display: "淘宝/天猫".to_string(),
            required_fields: vec![
                field("app_key", "App Key", false, true),
                field("app_secret", "App Secret", true, true),
                field("access_token", "Session Key / Access Token", true, true),
            ],
            doc_url: Some("https://open.taobao.com/doc.htm".to_string()),
        },
        PlatformSpec {
            platform: "jd".to_string(),
            display: "京东".to_string(),
            required_fields: vec![
                field("app_key", "App Key", false, true),
                field("app_secret", "App Secret", true, true),
                field("access_token", "Access Token", true, true),
            ],
            doc_url: Some("https://jos.jd.com/".to_string()),
        },
        PlatformSpec {
            platform: "pdd".to_string(),
            display: "拼多多".to_string(),
            required_fields: vec![
                field("app_key", "Client ID", false, true),
                field("app_secret", "Client Secret", true, true),
                field("access_token", "Access Token", true, true),
            ],
            doc_url: Some("https://open.pinduoduo.com/".to_string()),
        },
        PlatformSpec {
            platform: "douyin".to_string(),
            display: "抖音电商".to_string(),
            required_fields: vec![
                field("app_key", "App ID", false, true),
                field("app_secret", "App Secret", true, true),
                field("access_token", "Access Token", true, true),
            ],
            doc_url: Some("https://op.jinritemai.com/".to_string()),
        },
    ]
}

fn map_platform_execution_action(
    action: PlatformExecutionActionSpec,
    connected: bool,
    enabled: bool,
    connection_status: &str,
) -> PlatformExecutionAdapterAction {
    let (readiness, readiness_message) = if !connected {
        (
            "blocked",
            "No enabled platform connection was found for this user and platform.",
        )
    } else if !enabled || connection_status == "disabled" {
        ("blocked", "Platform connection exists but is disabled.")
    } else {
        ("blocked", action.adapter_message)
    };
    PlatformExecutionAdapterAction {
        skill_name: action.skill_name.to_string(),
        display: action.display.to_string(),
        description: action.description.to_string(),
        risk_level: "L3".to_string(),
        approval_required: true,
        execution_mode: "adapter".to_string(),
        adapter_status: "adapter_not_implemented".to_string(),
        can_execute_now: false,
        readiness: readiness.to_string(),
        readiness_message: readiness_message.to_string(),
        required_params: action
            .required_params
            .iter()
            .map(|item| item.to_string())
            .collect(),
        optional_params: action
            .optional_params
            .iter()
            .map(|item| item.to_string())
            .collect(),
        idempotency_key_fields: action
            .idempotency_key_fields
            .iter()
            .map(|item| item.to_string())
            .collect(),
    }
}

fn plan_platform_execution_preflight(
    skill_name: &str,
    params: &PlatformExecutionParams,
    connection: Option<&PlatformConnection>,
    checked_at: OffsetDateTime,
) -> PlatformExecutionPreflightResponse {
    let (status, adapter_status, message) =
        match platform_execution_readiness_status(skill_name, params, connection) {
            PlatformExecutionReadiness::Blocked {
                adapter_status,
                message,
            } => ("blocked", adapter_status, message),
            PlatformExecutionReadiness::Unsupported { message } => {
                ("unsupported", "unsupported_action", message)
            }
        };

    PlatformExecutionPreflightResponse {
        status: status.to_string(),
        adapter_status: adapter_status.to_string(),
        can_approve: false,
        can_execute_now: false,
        will_fail_if_approved: true,
        platform: params.platform.clone(),
        skill_name: skill_name.to_string(),
        message: message.to_string(),
        details: platform_execution_details(params),
        checked_at,
    }
}

fn channel_specs() -> Vec<ChannelSpec> {
    vec![
        ChannelSpec {
            channel: "wecom".to_string(),
            display: "企业微信机器人".to_string(),
            required_fields: vec![
                field("webhook_url", "Webhook URL", false, true),
                field("secret", "签名密钥", true, false),
            ],
            doc_url: Some("https://developer.work.weixin.qq.com/document/path/91770".to_string()),
        },
        ChannelSpec {
            channel: "feishu".to_string(),
            display: "飞书机器人".to_string(),
            required_fields: vec![
                field("webhook_url", "Webhook URL", false, true),
                field("secret", "签名密钥", true, false),
            ],
            doc_url: Some(
                "https://open.feishu.cn/document/client-docs/bot-v3/add-custom-bot".to_string(),
            ),
        },
        ChannelSpec {
            channel: "telegram".to_string(),
            display: "Telegram Bot".to_string(),
            required_fields: vec![
                field("bot_token", "Bot Token", true, true),
                field("chat_id", "Chat ID", false, true),
            ],
            doc_url: Some("https://core.telegram.org/bots/api#sendmessage".to_string()),
        },
        ChannelSpec {
            channel: "discord".to_string(),
            display: "Discord Webhook".to_string(),
            required_fields: vec![field("webhook_url", "Webhook URL", false, true)],
            doc_url: Some(
                "https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks"
                    .to_string(),
            ),
        },
    ]
}

fn field(key: &str, label: &str, secret: bool, required: bool) -> PlatformFieldSpec {
    PlatformFieldSpec {
        key: key.to_string(),
        label: label.to_string(),
        secret,
        required,
    }
}

fn map_platform_connection(item: PlatformConnection) -> PlatformConnectionResponse {
    PlatformConnectionResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        platform: item.platform.clone(),
        display: platform_display(item.platform.as_str()),
        app_key: item.app_key.as_deref().map(mask_secret),
        app_secret: item.app_secret_enc.as_deref().map(mask_secret),
        access_token: item.access_token_enc.as_deref().map(mask_secret),
        refresh_token: item.refresh_token_enc.as_deref().map(mask_secret),
        token_expires_at: item.token_expires_at,
        shop_id: item.shop_id,
        shop_name: item.shop_name,
        enabled: item.enabled,
        status: item.status.as_str().to_string(),
        last_tested_at: item.last_tested_at,
        test_result: item.test_result,
        last_synced_at: item.last_synced_at,
        sync_status: item.sync_status.as_str().to_string(),
        sync_message: item.sync_message,
        extra: item.extra,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_channel_connection(item: ChannelConnection) -> ChannelConnectionResponse {
    ChannelConnectionResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        channel: item.channel.clone(),
        display: channel_display(item.channel.as_str()),
        webhook_url: item.webhook_url.as_deref().map(mask_secret),
        bot_token: item.bot_token_enc.as_deref().map(mask_secret),
        chat_id: item.chat_id,
        secret: item.secret_enc.as_deref().map(mask_secret),
        enabled: item.enabled,
        status: item.status.as_str().to_string(),
        last_tested_at: item.last_tested_at,
        last_error: item.last_error,
        extra_credentials: item.extra_credentials,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_store_metric(item: StoreMetric) -> StoreMetricResponse {
    StoreMetricResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        platform: item.platform,
        metric_date: item.metric_date,
        metric_key: item.metric_key,
        metric_value: item.metric_value,
        dimension_key: item.dimension_key,
        dimension_value: item.dimension_value,
        source: item.source,
        created_at: item.created_at,
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::PLATFORM_CONNECT_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_optional_platform(value: Option<String>) -> BizResult<Option<String>> {
    value.map(parse_platform).transpose()
}

fn parse_platform(value: String) -> BizResult<String> {
    let value = normalize_required(value, "platform")?.to_lowercase();
    if platform_specs().iter().any(|item| item.platform == value) {
        Ok(value)
    } else {
        Err(BizError::new(
            app_error::PLATFORM_CONNECT_REQUEST_INVALID,
            format!("unsupported platform: {value}"),
        ))
    }
}

fn parse_optional_channel(value: Option<String>) -> BizResult<Option<String>> {
    value.map(parse_channel).transpose()
}

fn parse_channel(value: String) -> BizResult<String> {
    let value = normalize_required(value, "channel")?.to_lowercase();
    if channel_specs().iter().any(|item| item.channel == value) {
        Ok(value)
    } else {
        Err(BizError::new(
            app_error::PLATFORM_CONNECT_REQUEST_INVALID,
            format!("unsupported channel: {value}"),
        ))
    }
}

fn parse_sync_status(value: Option<String>) -> BizResult<Option<PlatformSyncStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| {
            PlatformSyncStatus::try_from(value.as_str()).map_err(|_| {
                BizError::new(
                    app_error::PLATFORM_CONNECT_REQUEST_INVALID,
                    format!("invalid sync_status: {value}"),
                )
            })
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PLATFORM_CONNECT_REQUEST_INVALID,
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

fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    if value.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &value[..4], &value[value.len() - 4..])
}

fn platform_display(platform: &str) -> String {
    platform_specs()
        .into_iter()
        .find(|item| item.platform == platform)
        .map(|item| item.display)
        .unwrap_or_else(|| platform.to_string())
}

fn channel_display(channel: &str) -> String {
    channel_specs()
        .into_iter()
        .find(|item| item.channel == channel)
        .map(|item| item.display)
        .unwrap_or_else(|| channel.to_string())
}

fn pagination(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    (page, page_size, offset)
}

fn format_date(value: OffsetDateTime) -> String {
    let date = value.date();
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

fn aggregate_map(
    items: Vec<repo::table::store_metrics::StoreMetricAggregate>,
) -> HashMap<String, f64> {
    items
        .into_iter()
        .map(|item| (format!("{}:{}", item.platform, item.metric_key), item.total))
        .collect()
}

fn metric_total(map: &HashMap<String, f64>, platform: &str, metric_key: &str) -> f64 {
    map.get(&format!("{platform}:{metric_key}"))
        .copied()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn test_connection(platform: &str) -> PlatformConnection {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        PlatformConnection {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            platform: platform.to_string(),
            app_key: Some("app-key".to_string()),
            app_secret_enc: Some("secret".to_string()),
            access_token_enc: Some("token".to_string()),
            refresh_token_enc: None,
            token_expires_at: None,
            shop_id: Some("shop-1".to_string()),
            shop_name: Some("Test Shop".to_string()),
            enabled: true,
            status: PlatformConnectionStatus::Configured,
            last_tested_at: None,
            test_result: json!({}),
            last_synced_at: None,
            sync_status: PlatformSyncStatus::Idle,
            sync_message: None,
            extra: json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn platform_execution_adapter_catalog_reports_missing_connection() {
        let action = map_platform_execution_action(
            platform_execution_action_specs()[0],
            false,
            false,
            "missing",
        );

        assert_eq!(action.skill_name, "platform_update_price");
        assert_eq!(action.adapter_status, "adapter_not_implemented");
        assert!(!action.can_execute_now);
        assert_eq!(action.readiness, "blocked");
        assert_eq!(
            action.readiness_message,
            "No enabled platform connection was found for this user and platform."
        );
    }

    #[test]
    fn platform_execution_adapter_catalog_reports_disabled_connection() {
        let action = map_platform_execution_action(
            platform_execution_action_specs()[1],
            true,
            false,
            "disabled",
        );

        assert_eq!(action.skill_name, "platform_update_inventory");
        assert!(!action.can_execute_now);
        assert_eq!(action.readiness, "blocked");
        assert_eq!(
            action.readiness_message,
            "Platform connection exists but is disabled."
        );
    }

    #[test]
    fn platform_execution_adapter_catalog_reports_unimplemented_adapter_when_connected() {
        let action = map_platform_execution_action(
            platform_execution_action_specs()[2],
            true,
            true,
            "configured",
        );

        assert_eq!(action.skill_name, "platform_sync_products");
        assert_eq!(action.adapter_status, "adapter_not_implemented");
        assert!(!action.can_execute_now);
        assert_eq!(action.readiness, "blocked");
        assert_eq!(
            action.readiness_message,
            "Platform execution adapter is not implemented yet for product sync."
        );
        assert_eq!(action.required_params, vec!["platform"]);
    }

    #[test]
    fn platform_execution_preflight_reads_normalized_skill_output() {
        let args = json!({
            "output": {
                "normalized_inputs": {
                    "platform": "DouYin",
                    "product_id": "prod-1",
                    "new_price": "129.5",
                    "sku_id": "sku-9"
                }
            }
        });

        let params = platform_execution_params(&args);

        assert_eq!(params.platform.as_deref(), Some("douyin"));
        assert_eq!(params.product_id.as_deref(), Some("prod-1"));
        assert_eq!(params.sku_id.as_deref(), Some("sku-9"));
        assert_eq!(params.new_price, Some(129.5));
    }

    #[test]
    fn platform_execution_preflight_reports_missing_platform() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };

        let preflight =
            plan_platform_execution_preflight("platform_update_price", &params, None, now);

        assert_eq!(preflight.status, "blocked");
        assert_eq!(preflight.adapter_status, "validation_error");
        assert!(!preflight.can_execute_now);
        assert!(preflight.will_fail_if_approved);
    }

    #[test]
    fn platform_execution_preflight_reports_missing_connection() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            platform: Some("taobao".to_string()),
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };

        let preflight =
            plan_platform_execution_preflight("platform_update_price", &params, None, now);

        assert_eq!(preflight.status, "blocked");
        assert_eq!(preflight.adapter_status, "connection_missing");
        assert_eq!(
            preflight
                .details
                .pointer("/productId")
                .and_then(Value::as_str),
            Some("prod-1")
        );
    }

    #[test]
    fn platform_execution_preflight_reports_unimplemented_adapter_when_connected() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            platform: Some("jd".to_string()),
            product_id: Some("prod-1".to_string()),
            quantity: Some(12),
            ..Default::default()
        };
        let connection = test_connection("jd");

        let preflight = plan_platform_execution_preflight(
            "platform_update_inventory",
            &params,
            Some(&connection),
            now,
        );

        assert_eq!(preflight.status, "blocked");
        assert_eq!(preflight.adapter_status, "adapter_not_implemented");
        assert!(!preflight.can_execute_now);
        assert_eq!(
            preflight
                .details
                .pointer("/quantity")
                .and_then(Value::as_i64),
            Some(12)
        );
    }

    #[test]
    fn platform_execution_preflight_reports_unsupported_non_platform_action() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams::default();

        let preflight =
            plan_platform_execution_preflight("ops_pricing_strategy", &params, None, now);

        assert_eq!(preflight.status, "unsupported");
        assert_eq!(preflight.adapter_status, "unsupported_action");
        assert_eq!(
            preflight.message,
            "Tool approval is not a platform execution action."
        );
    }
}
