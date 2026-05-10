use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    campaigns::{
        Campaign, CampaignFilter, CampaignService, CampaignStatus, CreateCampaign, UpdateCampaign,
    },
    products::{Product, ProductService, ProductStatus},
    workspace_artifacts::{WorkspaceArtifact, WorkspaceArtifactService, WorkspaceArtifactStatus},
    workspaces::{Workspace, WorkspaceService, WorkspaceStatus},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::dto::campaign::{
    CampaignDraftResponse, CampaignDraftSourceResponse, CampaignListResponse, CampaignResponse,
    CreateCampaignDraftRequest, CreateCampaignRequest, DeleteCampaignRequest, GetCampaignRequest,
    ListCampaignsRequest, UpdateCampaignRequest,
};

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct CampaignApi {
    app_user_svc: AppUserService,
    product_svc: ProductService,
    campaign_svc: CampaignService,
    workspace_svc: WorkspaceService,
    artifact_svc: WorkspaceArtifactService,
}

impl CampaignApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            campaign_svc: CampaignService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            artifact_svc: WorkspaceArtifactService::new(db),
        }
    }

    pub async fn list_campaigns(
        &self,
        req: ListCampaignsRequest,
    ) -> BizResult<CampaignListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let status = parse_optional_status(req.status)?;
        let product_id = req
            .product_id
            .map(|value| parse_uuid(&value, "product_id"))
            .transpose()?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }

        let page = req.page.unwrap_or(DEFAULT_PAGE).max(1);
        let page_size = req
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        let campaign_type = normalize_optional(req.campaign_type);
        let keyword = normalize_optional(req.keyword);
        let filter = CampaignFilter {
            user_id,
            product_id,
            status,
            campaign_type,
            keyword,
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = CampaignFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };

        let total = self.campaign_svc.count(count_filter).await?;
        let campaigns = self
            .campaign_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_campaign)
            .collect::<BizResult<Vec<_>>>()?;

        Ok(CampaignListResponse {
            campaigns,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_campaign(&self, req: CreateCampaignRequest) -> BizResult<CampaignResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = req
            .product_id
            .map(|value| parse_uuid(&value, "product_id"))
            .transpose()?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let name = normalize_required(req.name, "name")?;
        let status = parse_optional_status(req.status)?.unwrap_or(CampaignStatus::Draft);

        let campaign = self
            .campaign_svc
            .create(CreateCampaign {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                name,
                campaign_type: normalize_optional(req.campaign_type),
                description: normalize_optional(req.description),
                budget: req.budget.unwrap_or_default(),
                start_date: normalize_optional(req.start_date),
                end_date: normalize_optional(req.end_date),
                status,
                metadata: req.metadata.unwrap_or_else(|| json!({})),
                output: req.output.unwrap_or_else(|| json!({})),
                action_metadata: req.action_metadata.unwrap_or_else(|| json!({})),
            })
            .await?;

        Self::map_campaign(campaign)
    }

    pub async fn create_campaign_draft(
        &self,
        req: CreateCampaignDraftRequest,
    ) -> BizResult<CampaignDraftResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let mut product_id = parse_optional_uuid(req.product_id, "product_id")?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let workspace_id = parse_optional_uuid(req.workspace_id, "workspace_id")?;
        let workspace = if let Some(workspace_id) = workspace_id {
            let workspace = self
                .ensure_active_workspace_owner(user_id, workspace_id)
                .await?;
            if let Some(workspace_product_id) = workspace.product_id {
                if let Some(request_product_id) = product_id {
                    if request_product_id != workspace_product_id {
                        return Err(BizError::new(
                            app_error::CAMPAIGN_REQUEST_INVALID,
                            "workspace product does not match requested product".to_string(),
                        ));
                    }
                } else {
                    product_id = Some(workspace_product_id);
                }
            }
            Some(workspace)
        } else {
            None
        };
        let source_artifact_id = parse_optional_uuid(req.source_artifact_id, "source_artifact_id")?;
        let source_artifact = if let Some(source_artifact_id) = source_artifact_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::CAMPAIGN_REQUEST_INVALID,
                    "workspace_id is required when source_artifact_id is provided".to_string(),
                )
            })?;
            Some(
                self.ensure_active_artifact_owner(user_id, workspace_id, source_artifact_id)
                    .await?,
            )
        } else {
            None
        };
        let name = normalize_required(req.name, "name")?;
        let campaign_type = normalize_optional(req.campaign_type)
            .or_else(|| Some(default_campaign_draft_type(&req.source_type).to_string()));
        let description = normalize_optional(req.description);
        let source_type = normalize_optional(req.source_type)
            .unwrap_or_else(|| default_campaign_draft_source_type(source_artifact_id).to_string());
        let source_summary = normalize_optional(req.source_summary)
            .or_else(|| {
                source_artifact
                    .as_ref()
                    .and_then(|item| item.preview.clone())
            })
            .or_else(|| source_artifact.as_ref().map(|item| item.title.clone()));
        let channels = normalize_string_vec(req.channels);
        let planning = CampaignDraftPlanningInput {
            goal: normalize_optional(req.goal),
            target_audience: normalize_optional(req.target_audience),
            channels,
            strategy: normalize_optional(req.strategy),
        };
        let source = CampaignDraftSourceResponse {
            source_type,
            product_id: product_id.map(|id| id.to_string()),
            workspace_id: workspace_id.map(|id| id.to_string()),
            source_artifact_id: source_artifact_id.map(|id| id.to_string()),
            source_summary,
            ref_keys: build_campaign_draft_ref_keys(product_id, workspace_id, source_artifact_id),
        };
        let docs = build_campaign_draft_documents(CampaignDraftDocumentInput {
            source: source.clone(),
            planning,
            draft: req.draft,
            metadata: req.metadata,
            action_metadata: req.action_metadata,
            budget: req.budget,
            start_date: normalize_optional(req.start_date.clone()),
            end_date: normalize_optional(req.end_date.clone()),
            workspace,
            source_artifact,
        });

        let campaign = self
            .campaign_svc
            .create(CreateCampaign {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                name,
                campaign_type,
                description,
                budget: req.budget.unwrap_or_default(),
                start_date: normalize_optional(req.start_date),
                end_date: normalize_optional(req.end_date),
                status: CampaignStatus::Draft,
                metadata: docs.metadata,
                output: docs.output,
                action_metadata: docs.action_metadata,
            })
            .await?;
        let campaign = Self::map_campaign(campaign)?;

        Ok(CampaignDraftResponse {
            campaign,
            source,
            warnings: docs.warnings,
            agent_context: docs.agent_context,
        })
    }

    pub async fn get_campaign(&self, req: GetCampaignRequest) -> BizResult<CampaignResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let campaign_id = parse_uuid(&req.campaign_id, "campaign_id")?;
        let campaign = self.ensure_campaign_owner(user_id, campaign_id).await?;
        Self::map_campaign(campaign)
    }

    pub async fn update_campaign(&self, req: UpdateCampaignRequest) -> BizResult<CampaignResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let campaign_id = parse_uuid(&req.campaign_id, "campaign_id")?;
        let current = self.ensure_campaign_owner(user_id, campaign_id).await?;

        let product_id = req
            .product_id
            .map(|value| parse_uuid(&value, "product_id"))
            .transpose()?
            .or(current.product_id);
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }

        let status = parse_optional_status(req.status)?.unwrap_or(current.status);
        let campaign = self
            .campaign_svc
            .update(UpdateCampaign {
                id: current.id,
                product_id,
                name: req
                    .name
                    .map(|value| normalize_required(value, "name"))
                    .transpose()?
                    .unwrap_or(current.name),
                campaign_type: req
                    .campaign_type
                    .map(normalize_string)
                    .unwrap_or(current.campaign_type),
                description: req
                    .description
                    .map(normalize_string)
                    .unwrap_or(current.description),
                budget: req.budget.unwrap_or(current.budget),
                start_date: req
                    .start_date
                    .map(normalize_string)
                    .unwrap_or(current.start_date),
                end_date: req
                    .end_date
                    .map(normalize_string)
                    .unwrap_or(current.end_date),
                status,
                metadata: req.metadata.unwrap_or(current.metadata),
                output: req.output.unwrap_or(current.output),
                action_metadata: req.action_metadata.unwrap_or(current.action_metadata),
            })
            .await?;

        Self::map_campaign(campaign)
    }

    pub async fn delete_campaign(&self, req: DeleteCampaignRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let campaign_id = parse_uuid(&req.campaign_id, "campaign_id")?;
        let current = self.ensure_campaign_owner(user_id, campaign_id).await?;

        self.campaign_svc
            .update(UpdateCampaign {
                id: current.id,
                product_id: current.product_id,
                name: current.name,
                campaign_type: current.campaign_type,
                description: current.description,
                budget: current.budget,
                start_date: current.start_date,
                end_date: current.end_date,
                status: CampaignStatus::Archived,
                metadata: current.metadata,
                output: current.output,
                action_metadata: current.action_metadata,
            })
            .await?;

        Ok(())
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

    async fn ensure_product_owner(&self, user_id: Uuid, product_id: Uuid) -> BizResult<Product> {
        let product = self
            .product_svc
            .get_by_id(product_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_NOT_FOUND,
                    "product not found".to_string(),
                )
            })?;
        if product.user_id != user_id {
            return Err(BizError::new(
                app_error::PRODUCT_FORBIDDEN,
                "product does not belong to current user".to_string(),
            ));
        }
        Ok(product)
    }

    async fn ensure_active_product_owner(
        &self,
        user_id: Uuid,
        product_id: Uuid,
    ) -> BizResult<Product> {
        let product = self.ensure_product_owner(user_id, product_id).await?;
        if product.status == ProductStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_NOT_FOUND,
                "product not found".to_string(),
            ));
        }
        Ok(product)
    }

    async fn ensure_campaign_owner(&self, user_id: Uuid, campaign_id: Uuid) -> BizResult<Campaign> {
        let campaign = self
            .campaign_svc
            .get_by_id(campaign_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::CAMPAIGN_NOT_FOUND,
                    "campaign not found".to_string(),
                )
            })?;
        if campaign.user_id != user_id {
            return Err(BizError::new(
                app_error::CAMPAIGN_FORBIDDEN,
                "campaign does not belong to current user".to_string(),
            ));
        }
        Ok(campaign)
    }

    async fn ensure_workspace_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> BizResult<Workspace> {
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
        Ok(workspace)
    }

    async fn ensure_active_workspace_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> BizResult<Workspace> {
        let workspace = self.ensure_workspace_owner(user_id, workspace_id).await?;
        if workspace.status == WorkspaceStatus::Archived {
            return Err(BizError::new(
                app_error::WORKSPACE_NOT_FOUND,
                "workspace not found".to_string(),
            ));
        }
        Ok(workspace)
    }

    async fn ensure_active_artifact_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        artifact_id: Uuid,
    ) -> BizResult<WorkspaceArtifact> {
        let artifact = self
            .artifact_svc
            .get_by_id(artifact_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::WORKSPACE_ARTIFACT_NOT_FOUND,
                    "workspace artifact not found".to_string(),
                )
            })?;
        if artifact.user_id != user_id || artifact.workspace_id != workspace_id {
            return Err(BizError::new(
                app_error::WORKSPACE_ARTIFACT_FORBIDDEN,
                "workspace artifact does not belong to current workspace".to_string(),
            ));
        }
        if artifact.status == WorkspaceArtifactStatus::Archived {
            return Err(BizError::new(
                app_error::WORKSPACE_ARTIFACT_NOT_FOUND,
                "workspace artifact not found".to_string(),
            ));
        }
        Ok(artifact)
    }

    fn map_campaign(item: Campaign) -> BizResult<CampaignResponse> {
        Ok(CampaignResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.map(|id| id.to_string()),
            name: item.name,
            campaign_type: item.campaign_type,
            description: item.description,
            budget: item.budget,
            start_date: item.start_date,
            end_date: item.end_date,
            status: item.status.as_str().to_string(),
            metadata: item.metadata,
            output: item.output,
            action_metadata: item.action_metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}

struct CampaignDraftPlanningInput {
    goal: Option<String>,
    target_audience: Option<String>,
    channels: Vec<String>,
    strategy: Option<String>,
}

struct CampaignDraftDocumentInput {
    source: CampaignDraftSourceResponse,
    planning: CampaignDraftPlanningInput,
    draft: Option<Value>,
    metadata: Option<Value>,
    action_metadata: Option<Value>,
    budget: Option<f64>,
    start_date: Option<String>,
    end_date: Option<String>,
    workspace: Option<Workspace>,
    source_artifact: Option<WorkspaceArtifact>,
}

struct CampaignDraftDocuments {
    metadata: Value,
    output: Value,
    action_metadata: Value,
    warnings: Vec<String>,
    agent_context: Value,
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::CAMPAIGN_REQUEST_INVALID,
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

fn parse_optional_status(value: Option<String>) -> BizResult<Option<CampaignStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "draft" => Ok(CampaignStatus::Draft),
            "active" => Ok(CampaignStatus::Active),
            "paused" => Ok(CampaignStatus::Paused),
            "ended" => Ok(CampaignStatus::Ended),
            "archived" => Ok(CampaignStatus::Archived),
            _ => Err(BizError::new(
                app_error::CAMPAIGN_REQUEST_INVALID,
                format!("invalid campaign status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::CAMPAIGN_REQUEST_INVALID,
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

fn normalize_string_vec(value: Option<Vec<String>>) -> Vec<String> {
    value
        .unwrap_or_default()
        .into_iter()
        .filter_map(normalize_string)
        .collect()
}

fn default_campaign_draft_type(source_type: &Option<String>) -> &'static str {
    match source_type.as_deref() {
        Some("ad_review") => "ad_review",
        Some("launch") => "launch",
        Some("promotion") => "promotion",
        _ => "agent_draft",
    }
}

fn default_campaign_draft_source_type(source_artifact_id: Option<Uuid>) -> &'static str {
    if source_artifact_id.is_some() {
        "workspace_artifact"
    } else {
        "manual_draft"
    }
}

fn build_campaign_draft_ref_keys(
    product_id: Option<Uuid>,
    workspace_id: Option<Uuid>,
    source_artifact_id: Option<Uuid>,
) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(product_id) = product_id {
        refs.push(format!("product:{product_id}"));
    }
    if let Some(workspace_id) = workspace_id {
        refs.push(format!("workspace:{workspace_id}"));
    }
    if let Some(source_artifact_id) = source_artifact_id {
        refs.push(format!("workspace_artifact:{source_artifact_id}"));
    }
    refs
}

fn build_campaign_draft_documents(input: CampaignDraftDocumentInput) -> CampaignDraftDocuments {
    let warnings = build_campaign_draft_warnings(&input);
    let ref_keys = input.source.ref_keys.clone();
    let source_summary = input.source.source_summary.clone();
    let workspace_context = input.workspace.as_ref().map(|workspace| {
        json!({
            "id": workspace.id,
            "title": workspace.title,
            "workspaceType": workspace.workspace_type,
        })
    });
    let artifact_context = input.source_artifact.as_ref().map(|artifact| {
        json!({
            "id": artifact.id,
            "title": artifact.title,
            "artifactType": artifact.artifact_type,
            "source": artifact.source,
            "validationStatus": artifact.validation_status.as_str(),
        })
    });
    let planning = json!({
        "goal": input.planning.goal,
        "targetAudience": input.planning.target_audience,
        "channels": input.planning.channels,
        "strategy": input.planning.strategy,
        "budget": input.budget,
        "startDate": input.start_date,
        "endDate": input.end_date,
    });
    let source = json!({
        "sourceType": input.source.source_type,
        "productId": input.source.product_id,
        "workspaceId": input.source.workspace_id,
        "sourceArtifactId": input.source.source_artifact_id,
        "sourceSummary": source_summary,
        "refKeys": ref_keys,
    });
    let metadata = json!({
        "source": source.clone(),
        "planning": planning.clone(),
        "refs": ref_keys,
        "workspace": workspace_context,
        "sourceArtifact": artifact_context,
        "extra": input.metadata.unwrap_or_else(|| json!({})),
    });
    let output = json!({
        "kind": "campaign_draft",
        "draft": input.draft.unwrap_or_else(|| json!({})),
        "planning": planning.clone(),
        "sourceSummary": source_summary,
    });
    let action_metadata = json!({
        "action": "create_campaign_draft",
        "executionStatus": "not_submitted",
        "requiresApproval": true,
        "platformExecution": "not_started",
        "approvalReason": "campaign draft is local only; platform execution must be approved separately",
        "source": source.clone(),
        "extra": input.action_metadata.unwrap_or_else(|| json!({})),
    });
    let agent_context = json!({
        "type": "campaign_draft",
        "source": source,
        "planning": planning,
        "warnings": warnings.clone(),
    });

    CampaignDraftDocuments {
        metadata,
        output,
        action_metadata,
        warnings,
        agent_context,
    }
}

fn build_campaign_draft_warnings(input: &CampaignDraftDocumentInput) -> Vec<String> {
    let mut warnings = Vec::new();
    if input.source.product_id.is_none() {
        warnings.push("campaign draft is not bound to a product".to_string());
    }
    if input.budget.map(|budget| budget <= 0.0).unwrap_or(true) {
        warnings.push("campaign draft budget is missing or zero".to_string());
    }
    if input.start_date.is_none() || input.end_date.is_none() {
        warnings.push("campaign draft date range is incomplete".to_string());
    }
    if input.source.source_artifact_id.is_none() {
        warnings.push("campaign draft has no source artifact".to_string());
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_draft_documents_preserve_source_refs() {
        let product_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let workspace_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        let artifact_id = Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();
        let source = CampaignDraftSourceResponse {
            source_type: "agent_output".to_string(),
            product_id: Some(product_id.to_string()),
            workspace_id: Some(workspace_id.to_string()),
            source_artifact_id: Some(artifact_id.to_string()),
            source_summary: Some("launch plan".to_string()),
            ref_keys: build_campaign_draft_ref_keys(
                Some(product_id),
                Some(workspace_id),
                Some(artifact_id),
            ),
        };

        let docs = build_campaign_draft_documents(CampaignDraftDocumentInput {
            source,
            planning: CampaignDraftPlanningInput {
                goal: Some("test launch".to_string()),
                target_audience: Some("coffee buyers".to_string()),
                channels: vec!["douyin".to_string()],
                strategy: Some("small budget test".to_string()),
            },
            draft: Some(json!({"tasks": []})),
            metadata: None,
            action_metadata: None,
            budget: Some(3000.0),
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-05-07".to_string()),
            workspace: None,
            source_artifact: None,
        });

        assert_eq!(docs.warnings, Vec::<String>::new());
        assert_eq!(
            docs.metadata
                .get("source")
                .and_then(|value| value.get("sourceType"))
                .and_then(Value::as_str),
            Some("agent_output")
        );
        assert_eq!(
            docs.action_metadata
                .get("requiresApproval")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn campaign_draft_documents_warn_when_context_is_sparse() {
        let source = CampaignDraftSourceResponse {
            source_type: "manual_draft".to_string(),
            product_id: None,
            workspace_id: None,
            source_artifact_id: None,
            source_summary: None,
            ref_keys: Vec::new(),
        };

        let docs = build_campaign_draft_documents(CampaignDraftDocumentInput {
            source,
            planning: CampaignDraftPlanningInput {
                goal: None,
                target_audience: None,
                channels: Vec::new(),
                strategy: None,
            },
            draft: None,
            metadata: None,
            action_metadata: None,
            budget: None,
            start_date: None,
            end_date: None,
            workspace: None,
            source_artifact: None,
        });

        assert!(docs.warnings.iter().any(|item| item.contains("product")));
        assert!(docs.warnings.iter().any(|item| item.contains("budget")));
        assert!(docs.warnings.iter().any(|item| item.contains("date range")));
        assert!(
            docs.warnings
                .iter()
                .any(|item| item.contains("source artifact"))
        );
    }
}
