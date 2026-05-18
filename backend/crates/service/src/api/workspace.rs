use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    agent_learning_records::{
        AgentLearningRecord, AgentLearningRecordFilter, AgentLearningRecordService,
        AgentLearningRecordStatus, CreateAgentLearningRecord,
    },
    agent_quality_checks::{
        AgentQualityCheck, AgentQualityCheckFilter, AgentQualityCheckService,
        CreateAgentQualityCheck,
    },
    agent_skill_runs::{
        AgentSkillRun, AgentSkillRunFilter, AgentSkillRunService, AgentSkillRunStatus,
        CreateAgentSkillRun, UpdateAgentSkillRun,
    },
    agent_trust_scores::{
        AgentTrustLevel, AgentTrustScore, AgentTrustScoreFilter, AgentTrustScoreService,
        CreateAgentTrustScore, UpdateAgentTrustScore,
    },
    app_users::{AppUserService, AppUserStatus},
    audit_logs::{AuditLogService, CreateAuditLog},
    campaigns::{Campaign, CampaignService, CampaignStatus, CreateCampaign},
    conversation_files::{ConversationFile, ConversationFileService},
    conversations::ConversationService,
    execution_queue::{
        CreateExecutionQueueItem, ExecutionQueueFilter, ExecutionQueueItem, ExecutionQueueService,
        ExecutionQueueStatus, UpdateExecutionQueueItem,
    },
    product_assets::{ProductAsset, ProductAssetFilter, ProductAssetService, ProductAssetStatus},
    product_materials::{
        ProductMaterial, ProductMaterialFilter, ProductMaterialService, ProductMaterialStatus,
    },
    products::{ProductService, ProductStatus},
    workspace_artifacts::{
        CreateWorkspaceArtifact, UpdateWorkspaceArtifact, WorkspaceArtifact,
        WorkspaceArtifactFilter, WorkspaceArtifactService, WorkspaceArtifactStatus,
        WorkspaceArtifactValidationStatus,
    },
    workspace_run_tasks::{
        CreateWorkspaceRunTask, UpdateWorkspaceRunTask, WorkspaceRunTask, WorkspaceRunTaskFilter,
        WorkspaceRunTaskService, WorkspaceRunTaskStatus,
    },
    workspace_runs::{
        CreateWorkspaceRun, UpdateWorkspaceRun, WorkspaceRun, WorkspaceRunFilter,
        WorkspaceRunService, WorkspaceRunStatus,
    },
    workspace_tasks::{
        CreateWorkspaceTask, UpdateWorkspaceTask, WorkspaceTask, WorkspaceTaskFilter,
        WorkspaceTaskService, WorkspaceTaskStatus,
    },
    workspaces::{
        CreateWorkspace, UpdateWorkspace, Workspace, WorkspaceFilter, WorkspaceService,
        WorkspaceStatus,
    },
};
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::{
    campaign::{CampaignDraftResponse, CampaignDraftSourceResponse, CampaignResponse},
    workspace::*,
};

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;
const AGENT_WORKSPACE_TYPE: &str = "agent_chat";
const AGENT_RUN_SOURCE: &str = "agent_session";
const AGENT_RUN_TITLE: &str = "Agent chat workspace";
const DEFAULT_CONTEXT_PACK_MAX_ITEMS: usize = 20;
const MAX_CONTEXT_PACK_MAX_ITEMS: usize = 50;
const DEFAULT_CONTEXT_PACK_MAX_CHARS: usize = 12_000;
const MAX_CONTEXT_PACK_MAX_CHARS: usize = 24_000;
const DEFAULT_CONTEXT_PACK_ITEM_CHARS: usize = 2_400;
const DEFAULT_ARTIFACT_EXPORT_FORMAT: &str = "markdown";
const MAX_CONTEXT_REFS: usize = 20;
const MAX_CONTEXT_REF_CHARS: usize = 160;

pub struct WorkspaceApi {
    db: DbContext,
    app_user_svc: AppUserService,
    conversation_svc: ConversationService,
    conversation_file_svc: ConversationFileService,
    product_svc: ProductService,
    product_asset_svc: ProductAssetService,
    product_material_svc: ProductMaterialService,
    campaign_svc: CampaignService,
    workspace_svc: WorkspaceService,
    task_svc: WorkspaceTaskService,
    run_svc: WorkspaceRunService,
    run_task_svc: WorkspaceRunTaskService,
    artifact_svc: WorkspaceArtifactService,
    execution_svc: ExecutionQueueService,
    quality_svc: AgentQualityCheckService,
    skill_run_svc: AgentSkillRunService,
    trust_svc: AgentTrustScoreService,
    learning_svc: AgentLearningRecordService,
}

impl WorkspaceApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            app_user_svc: AppUserService::new(db.clone()),
            conversation_svc: ConversationService::new(db.clone()),
            conversation_file_svc: ConversationFileService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            product_asset_svc: ProductAssetService::new(db.clone()),
            product_material_svc: ProductMaterialService::new(db.clone()),
            campaign_svc: CampaignService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            task_svc: WorkspaceTaskService::new(db.clone()),
            run_svc: WorkspaceRunService::new(db.clone()),
            run_task_svc: WorkspaceRunTaskService::new(db.clone()),
            artifact_svc: WorkspaceArtifactService::new(db.clone()),
            execution_svc: ExecutionQueueService::new(db.clone()),
            quality_svc: AgentQualityCheckService::new(db.clone()),
            skill_run_svc: AgentSkillRunService::new(db.clone()),
            trust_svc: AgentTrustScoreService::new(db.clone()),
            learning_svc: AgentLearningRecordService::new(db),
        }
    }

    pub async fn list_workspaces(
        &self,
        req: ListWorkspacesRequest,
    ) -> BizResult<WorkspaceListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let status = parse_workspace_status(req.status)?;
        let product_id = parse_optional_uuid(req.product_id, "product_id")?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let campaign_id = parse_optional_uuid(req.campaign_id, "campaign_id")?;
        if let Some(campaign_id) = campaign_id {
            self.ensure_campaign_owner(user_id, campaign_id).await?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = WorkspaceFilter {
            user_id,
            status,
            product_id,
            campaign_id,
            workspace_type: normalize_optional(req.workspace_type),
            keyword: normalize_optional(req.keyword),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = WorkspaceFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };

        let total = self.workspace_svc.count(count_filter).await?;
        let workspaces = self
            .workspace_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_workspace)
            .collect::<Vec<_>>();

        Ok(WorkspaceListResponse {
            workspaces,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_workspace(
        &self,
        req: CreateWorkspaceRequest,
    ) -> BizResult<WorkspaceResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let product_id = parse_optional_uuid(req.product_id, "product_id")?;
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let campaign_id = parse_optional_uuid(req.campaign_id, "campaign_id")?;
        if let Some(campaign_id) = campaign_id {
            self.ensure_campaign_owner(user_id, campaign_id).await?;
        }
        let title = normalize_required(req.title, "title")?;

        let workspace = self
            .workspace_svc
            .create(CreateWorkspace {
                id: Uuid::new_v4(),
                user_id,
                product_id,
                campaign_id,
                title,
                workspace_type: normalize_optional(req.workspace_type)
                    .unwrap_or_else(|| "general".to_string()),
                status: WorkspaceStatus::Active,
                config: req.config.unwrap_or_else(|| json!({})),
            })
            .await?;

        Ok(Self::map_workspace(workspace))
    }

    pub async fn get_workspace(&self, req: GetWorkspaceRequest) -> BizResult<WorkspaceResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_uuid(&req.workspace_id, "workspace_id")?;
        let workspace = self
            .ensure_active_workspace_owner(user_id, workspace_id)
            .await?;
        Ok(Self::map_workspace(workspace))
    }

    pub async fn update_workspace(
        &self,
        req: UpdateWorkspaceRequest,
    ) -> BizResult<WorkspaceResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_uuid(&req.workspace_id, "workspace_id")?;
        let current = self.ensure_workspace_owner(user_id, workspace_id).await?;
        let product_id = parse_optional_uuid(req.product_id, "product_id")?.or(current.product_id);
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let campaign_id =
            parse_optional_uuid(req.campaign_id, "campaign_id")?.or(current.campaign_id);
        if let Some(campaign_id) = campaign_id {
            self.ensure_campaign_owner(user_id, campaign_id).await?;
        }

        let workspace = self
            .workspace_svc
            .update(UpdateWorkspace {
                id: current.id,
                product_id,
                campaign_id,
                title: req
                    .title
                    .map(|value| normalize_required(value, "title"))
                    .transpose()?
                    .unwrap_or(current.title),
                workspace_type: req
                    .workspace_type
                    .and_then(normalize_string)
                    .unwrap_or(current.workspace_type),
                status: parse_workspace_status(req.status)?.unwrap_or(current.status),
                config: req.config.unwrap_or(current.config),
            })
            .await?;

        Ok(Self::map_workspace(workspace))
    }

    pub async fn delete_workspace(&self, req: DeleteWorkspaceRequest) -> BizResult<()> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_uuid(&req.workspace_id, "workspace_id")?;
        let current = self.ensure_workspace_owner(user_id, workspace_id).await?;
        self.workspace_svc
            .update(UpdateWorkspace {
                id: current.id,
                product_id: current.product_id,
                campaign_id: current.campaign_id,
                title: current.title,
                workspace_type: current.workspace_type,
                status: WorkspaceStatus::Archived,
                config: current.config,
            })
            .await?;
        Ok(())
    }

    pub async fn list_tasks(
        &self,
        req: ListWorkspaceTasksRequest,
    ) -> BizResult<WorkspaceTaskListResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let status = parse_task_status(req.status)?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = WorkspaceTaskFilter {
            workspace_id: Some(workspace_id),
            user_id,
            status,
            owner_role: normalize_optional(req.owner_role),
            keyword: normalize_optional(req.keyword),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = WorkspaceTaskFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.task_svc.count(count_filter).await?;
        let tasks = self
            .task_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_task)
            .collect();
        Ok(WorkspaceTaskListResponse {
            tasks,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_task(
        &self,
        req: CreateWorkspaceTaskRequest,
    ) -> BizResult<WorkspaceTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let title = normalize_required(req.title, "title")?;
        let task = self
            .task_svc
            .create(CreateWorkspaceTask {
                id: Uuid::new_v4(),
                workspace_id,
                user_id,
                title,
                description: normalize_optional(req.description),
                owner_role: normalize_optional(req.owner_role),
                status: parse_task_status(req.status)?.unwrap_or(WorkspaceTaskStatus::Pending),
                priority: clamp_priority(req.priority.unwrap_or_default()),
                depends_on: req.depends_on.unwrap_or_else(|| json!([])),
                acceptance_criteria: normalize_optional(req.acceptance_criteria),
                result: normalize_optional(req.result),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(Self::map_task(task))
    }

    pub async fn get_task(&self, req: GetWorkspaceTaskRequest) -> BizResult<WorkspaceTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let task_id = parse_uuid(&req.task_id, "task_id")?;
        let task = self
            .ensure_task_owner(user_id, workspace_id, task_id)
            .await?;
        Ok(Self::map_task(task))
    }

    pub async fn update_task(
        &self,
        req: UpdateWorkspaceTaskRequest,
    ) -> BizResult<WorkspaceTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let task_id = parse_uuid(&req.task_id, "task_id")?;
        let current = self
            .ensure_task_owner(user_id, workspace_id, task_id)
            .await?;
        let task = self
            .task_svc
            .update(UpdateWorkspaceTask {
                id: current.id,
                title: req
                    .title
                    .map(|value| normalize_required(value, "title"))
                    .transpose()?
                    .unwrap_or(current.title),
                description: req
                    .description
                    .map(normalize_string)
                    .unwrap_or(current.description),
                owner_role: req
                    .owner_role
                    .map(normalize_string)
                    .unwrap_or(current.owner_role),
                status: parse_task_status(req.status)?.unwrap_or(current.status),
                priority: req.priority.map(clamp_priority).unwrap_or(current.priority),
                depends_on: req.depends_on.unwrap_or(current.depends_on),
                acceptance_criteria: req
                    .acceptance_criteria
                    .map(normalize_string)
                    .unwrap_or(current.acceptance_criteria),
                result: req.result.map(normalize_string).unwrap_or(current.result),
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;
        Ok(Self::map_task(task))
    }

    pub async fn delete_task(&self, req: DeleteWorkspaceTaskRequest) -> BizResult<()> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let task_id = parse_uuid(&req.task_id, "task_id")?;
        let current = self
            .ensure_task_owner(user_id, workspace_id, task_id)
            .await?;
        self.task_svc
            .update(UpdateWorkspaceTask {
                id: current.id,
                title: current.title,
                description: current.description,
                owner_role: current.owner_role,
                status: WorkspaceTaskStatus::Archived,
                priority: current.priority,
                depends_on: current.depends_on,
                acceptance_criteria: current.acceptance_criteria,
                result: current.result,
                metadata: current.metadata,
            })
            .await?;
        Ok(())
    }

    pub async fn list_runs(
        &self,
        req: ListWorkspaceRunsRequest,
    ) -> BizResult<WorkspaceRunListResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let status = parse_run_status(req.status)?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = WorkspaceRunFilter {
            workspace_id: Some(workspace_id),
            user_id,
            status,
            source: normalize_optional(req.source),
            keyword: normalize_optional(req.keyword),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = WorkspaceRunFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.run_svc.count(count_filter).await?;
        let runs = self
            .run_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_run)
            .collect();
        Ok(WorkspaceRunListResponse {
            runs,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_run(
        &self,
        req: CreateWorkspaceRunRequest,
    ) -> BizResult<WorkspaceRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let goal = normalize_required(req.goal, "goal")?;
        let run = self
            .run_svc
            .create(CreateWorkspaceRun {
                id: Uuid::new_v4(),
                workspace_id,
                user_id,
                goal,
                requirements: req.requirements.unwrap_or_else(|| json!([])),
                status: parse_run_status(req.status)?.unwrap_or(WorkspaceRunStatus::Planned),
                source: normalize_optional(req.source).unwrap_or_else(|| "manual".to_string()),
                summary: normalize_optional(req.summary),
                report: req.report.unwrap_or_else(|| json!({})),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(Self::map_run(run))
    }

    pub async fn get_run(&self, req: GetWorkspaceRunRequest) -> BizResult<WorkspaceRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        let run = self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        Ok(Self::map_run(run))
    }

    pub async fn update_run(
        &self,
        req: UpdateWorkspaceRunRequest,
    ) -> BizResult<WorkspaceRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        let current = self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let run = self
            .run_svc
            .update(UpdateWorkspaceRun {
                id: current.id,
                goal: req
                    .goal
                    .map(|value| normalize_required(value, "goal"))
                    .transpose()?
                    .unwrap_or(current.goal),
                requirements: req.requirements.unwrap_or(current.requirements),
                status: parse_run_status(req.status)?.unwrap_or(current.status),
                source: req
                    .source
                    .and_then(normalize_string)
                    .unwrap_or(current.source),
                summary: req.summary.map(normalize_string).unwrap_or(current.summary),
                report: req.report.unwrap_or(current.report),
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;
        Ok(Self::map_run(run))
    }

    pub async fn list_run_tasks(
        &self,
        req: ListWorkspaceRunTasksRequest,
    ) -> BizResult<Vec<WorkspaceRunTaskResponse>> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let tasks = self
            .run_task_svc
            .list(WorkspaceRunTaskFilter {
                run_id: Some(run_id),
                workspace_id: Some(workspace_id),
                user_id,
                status: parse_run_task_status(req.status)?,
                limit: None,
                offset: None,
            })
            .await?
            .into_iter()
            .map(Self::map_run_task)
            .collect();
        Ok(tasks)
    }

    pub async fn create_run_task(
        &self,
        req: CreateWorkspaceRunTaskRequest,
    ) -> BizResult<WorkspaceRunTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let workspace_task_id = parse_optional_uuid(req.workspace_task_id, "workspace_task_id")?;
        if let Some(task_id) = workspace_task_id {
            self.ensure_task_owner(user_id, workspace_id, task_id)
                .await?;
        }
        let task = self
            .run_task_svc
            .create(CreateWorkspaceRunTask {
                id: Uuid::new_v4(),
                run_id,
                workspace_id,
                workspace_task_id,
                user_id,
                seq: req.seq.unwrap_or_default(),
                owner_role: normalize_optional(req.owner_role),
                status: parse_run_task_status(req.status)?
                    .unwrap_or(WorkspaceRunTaskStatus::Pending),
                attempts: req.attempts.unwrap_or_default().max(0),
                acceptance_score: req.acceptance_score.unwrap_or_default().clamp(0.0, 1.0),
                acceptance_passed: req.acceptance_passed.unwrap_or_default(),
                acceptance_notes: normalize_optional(req.acceptance_notes),
                execution_notes: normalize_optional(req.execution_notes),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(Self::map_run_task(task))
    }

    pub async fn update_run_task(
        &self,
        req: UpdateWorkspaceRunTaskRequest,
    ) -> BizResult<WorkspaceRunTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let run_task_id = parse_uuid(&req.run_task_id, "run_task_id")?;
        let current = self
            .ensure_run_task_owner(user_id, workspace_id, run_id, run_task_id)
            .await?;
        let task = self
            .run_task_svc
            .update(UpdateWorkspaceRunTask {
                id: current.id,
                seq: req.seq.unwrap_or(current.seq),
                owner_role: req
                    .owner_role
                    .map(normalize_string)
                    .unwrap_or(current.owner_role),
                status: parse_run_task_status(req.status)?.unwrap_or(current.status),
                attempts: req.attempts.unwrap_or(current.attempts).max(0),
                acceptance_score: req
                    .acceptance_score
                    .unwrap_or(current.acceptance_score)
                    .clamp(0.0, 1.0),
                acceptance_passed: req.acceptance_passed.unwrap_or(current.acceptance_passed),
                acceptance_notes: req
                    .acceptance_notes
                    .map(normalize_string)
                    .unwrap_or(current.acceptance_notes),
                execution_notes: req
                    .execution_notes
                    .map(normalize_string)
                    .unwrap_or(current.execution_notes),
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;
        Ok(Self::map_run_task(task))
    }

    pub async fn list_artifacts(
        &self,
        req: ListWorkspaceArtifactsRequest,
    ) -> BizResult<WorkspaceArtifactListResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let workspace_task_id = parse_optional_uuid(req.workspace_task_id, "workspace_task_id")?;
        if let Some(task_id) = workspace_task_id {
            self.ensure_task_owner(user_id, workspace_id, task_id)
                .await?;
        }
        let run_task_id = parse_optional_uuid(req.run_task_id, "run_task_id")?;
        if let Some(run_task_id) = run_task_id
            && let Some(run_id) = run_id
        {
            self.ensure_run_task_owner(user_id, workspace_id, run_id, run_task_id)
                .await?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = WorkspaceArtifactFilter {
            workspace_id: Some(workspace_id),
            run_id,
            workspace_task_id,
            run_task_id,
            user_id,
            artifact_type: normalize_optional(req.artifact_type),
            status: parse_artifact_status(req.status)?,
            keyword: normalize_optional(req.keyword),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = WorkspaceArtifactFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.artifact_svc.count(count_filter).await?;
        let artifacts = self
            .artifact_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_artifact)
            .collect();
        Ok(WorkspaceArtifactListResponse {
            artifacts,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_artifact(
        &self,
        req: CreateWorkspaceArtifactRequest,
    ) -> BizResult<WorkspaceArtifactResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let (run_id, workspace_task_id, run_task_id) = self
            .resolve_artifact_links(
                user_id,
                workspace_id,
                req.run_id,
                req.workspace_task_id,
                req.run_task_id,
            )
            .await?;
        let title = normalize_required(req.title, "title")?;
        let artifact_type = normalize_required(req.artifact_type, "artifact_type")?;
        let content = normalize_optional(req.content);
        let preview = req
            .preview
            .and_then(normalize_string)
            .or_else(|| content.as_deref().map(|value| compact_text(value, 600)));
        let artifact = self
            .artifact_svc
            .create(CreateWorkspaceArtifact {
                id: Uuid::new_v4(),
                workspace_id,
                run_id,
                workspace_task_id,
                run_task_id,
                user_id,
                artifact_type,
                title,
                content,
                preview,
                mime_type: normalize_optional(req.mime_type),
                format: normalize_optional(req.format).unwrap_or_else(|| "markdown".to_string()),
                storage_kind: normalize_optional(req.storage_kind)
                    .unwrap_or_else(|| "inline".to_string()),
                bucket: normalize_optional(req.bucket),
                object_key: normalize_optional(req.object_key),
                file_name: normalize_optional(req.file_name),
                file_size: req.file_size,
                sha256: normalize_optional(req.sha256),
                source: normalize_optional(req.source).unwrap_or_else(|| "manual".to_string()),
                status: WorkspaceArtifactStatus::Active,
                validation_status: parse_artifact_validation_status(req.validation_status)?
                    .unwrap_or(WorkspaceArtifactValidationStatus::Unchecked),
                validation_report: req.validation_report.unwrap_or_else(|| json!({})),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(Self::map_artifact(artifact))
    }

    pub async fn get_artifact(
        &self,
        req: GetWorkspaceArtifactRequest,
    ) -> BizResult<WorkspaceArtifactResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let artifact_id = parse_uuid(&req.artifact_id, "artifact_id")?;
        let artifact = self
            .ensure_artifact_owner(user_id, workspace_id, artifact_id)
            .await?;
        Ok(Self::map_artifact(artifact))
    }

    pub async fn update_artifact(
        &self,
        req: UpdateWorkspaceArtifactRequest,
    ) -> BizResult<WorkspaceArtifactResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let artifact_id = parse_uuid(&req.artifact_id, "artifact_id")?;
        let current = self
            .ensure_artifact_owner(user_id, workspace_id, artifact_id)
            .await?;
        let content = req.content.map(normalize_string).unwrap_or(current.content);
        let preview = req
            .preview
            .map(normalize_string)
            .unwrap_or(current.preview)
            .or_else(|| content.as_deref().map(|value| compact_text(value, 600)));
        let artifact = self
            .artifact_svc
            .update(UpdateWorkspaceArtifact {
                id: current.id,
                title: req
                    .title
                    .map(|value| normalize_required(value, "title"))
                    .transpose()?
                    .unwrap_or(current.title),
                content,
                preview,
                mime_type: req
                    .mime_type
                    .map(normalize_string)
                    .unwrap_or(current.mime_type),
                format: req
                    .format
                    .and_then(normalize_string)
                    .unwrap_or(current.format),
                storage_kind: req
                    .storage_kind
                    .and_then(normalize_string)
                    .unwrap_or(current.storage_kind),
                bucket: req.bucket.map(normalize_string).unwrap_or(current.bucket),
                object_key: req
                    .object_key
                    .map(normalize_string)
                    .unwrap_or(current.object_key),
                file_name: req
                    .file_name
                    .map(normalize_string)
                    .unwrap_or(current.file_name),
                file_size: req.file_size.or(current.file_size),
                sha256: req.sha256.map(normalize_string).unwrap_or(current.sha256),
                status: parse_artifact_status(req.status)?.unwrap_or(current.status),
                validation_status: parse_artifact_validation_status(req.validation_status)?
                    .unwrap_or(current.validation_status),
                validation_report: req.validation_report.unwrap_or(current.validation_report),
                metadata: req.metadata.unwrap_or(current.metadata),
            })
            .await?;
        Ok(Self::map_artifact(artifact))
    }

    pub async fn delete_artifact(&self, req: DeleteWorkspaceArtifactRequest) -> BizResult<()> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let artifact_id = parse_uuid(&req.artifact_id, "artifact_id")?;
        let current = self
            .ensure_artifact_owner(user_id, workspace_id, artifact_id)
            .await?;
        self.artifact_svc
            .update(UpdateWorkspaceArtifact {
                id: current.id,
                title: current.title,
                content: current.content,
                preview: current.preview,
                mime_type: current.mime_type,
                format: current.format,
                storage_kind: current.storage_kind,
                bucket: current.bucket,
                object_key: current.object_key,
                file_name: current.file_name,
                file_size: current.file_size,
                sha256: current.sha256,
                status: WorkspaceArtifactStatus::Archived,
                validation_status: current.validation_status,
                validation_report: current.validation_report,
                metadata: current.metadata,
            })
            .await?;
        Ok(())
    }

    pub async fn export_artifact(
        &self,
        req: ExportWorkspaceArtifactRequest,
    ) -> BizResult<WorkspaceArtifactExportResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let artifact_id = parse_uuid(&req.artifact_id, "artifact_id")?;
        let artifact = self
            .ensure_active_artifact_owner(user_id, workspace_id, artifact_id)
            .await?;
        build_artifact_export(artifact, req.format, req.include_metadata.unwrap_or(false))
    }

    pub async fn context_summary(
        &self,
        req: GetWorkspaceContextSummaryRequest,
    ) -> BizResult<WorkspaceContextSummaryResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let workspace = self.ensure_workspace_owner(user_id, workspace_id).await?;
        let conversation_id = parse_optional_uuid(req.conversation_id, "conversation_id")?;
        if let Some(conversation_id) = conversation_id {
            self.ensure_conversation_owner(user_id, conversation_id)
                .await?;
        }
        let requested_product_id = parse_optional_uuid(req.product_id, "product_id")?;
        let product_id = if req.include_workspace_product_context {
            requested_product_id.or(workspace.product_id)
        } else {
            requested_product_id
        };
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let explicit_context_refs = explicit_workspace_context_refs(req.context_refs)?;

        let mut conversation_files = if let Some(conversation_id) = conversation_id {
            self.conversation_file_svc
                .list_by_conversation_id(conversation_id)
                .await?
        } else {
            Vec::new()
        };
        for file_id in explicit_context_refs.conversation_file_ids {
            let file = self
                .ensure_active_conversation_file_owner(user_id, file_id)
                .await?;
            push_unique_conversation_file(&mut conversation_files, file);
        }
        let attachment_sources = conversation_files
            .into_iter()
            .map(map_context_conversation_file)
            .collect::<Vec<_>>();

        let mut product_assets = if let Some(product_id) = product_id {
            self.product_asset_svc
                .list(ProductAssetFilter {
                    user_id,
                    product_id,
                    status: Some(ProductAssetStatus::Active),
                    asset_type: None,
                    keyword: None,
                    limit: Some(200),
                })
                .await?
        } else {
            Vec::new()
        };
        for asset_id in explicit_context_refs.product_asset_ids {
            let asset = self
                .ensure_active_product_asset_owner(user_id, asset_id)
                .await?;
            push_unique_product_asset(&mut product_assets, asset);
        }
        let product_asset_sources = product_assets
            .into_iter()
            .map(map_context_product_asset)
            .collect::<Vec<_>>();

        let mut product_materials = if let Some(product_id) = product_id {
            self.product_material_svc
                .list(ProductMaterialFilter {
                    user_id,
                    product_id,
                    status: Some(ProductMaterialStatus::Active),
                    material_type: None,
                    keyword: None,
                    limit: Some(200),
                })
                .await?
        } else {
            Vec::new()
        };
        for material_id in explicit_context_refs.product_material_ids {
            let material = self
                .ensure_active_product_material_owner(user_id, material_id)
                .await?;
            push_unique_product_material(&mut product_materials, material);
        }
        let product_material_sources = product_materials
            .into_iter()
            .map(map_context_product_material)
            .collect::<Vec<_>>();

        let mut workspace_artifacts = self
            .artifact_svc
            .list(WorkspaceArtifactFilter {
                workspace_id: Some(workspace_id),
                run_id,
                workspace_task_id: None,
                run_task_id: None,
                user_id,
                artifact_type: None,
                status: Some(WorkspaceArtifactStatus::Active),
                keyword: None,
                limit: Some(200),
                offset: Some(0),
            })
            .await?;
        for artifact_id in explicit_context_refs.workspace_artifact_ids {
            let artifact = self
                .ensure_active_artifact_owner(user_id, workspace_id, artifact_id)
                .await?;
            push_unique_artifact(&mut workspace_artifacts, artifact);
        }
        let artifact_sources = workspace_artifacts
            .into_iter()
            .map(map_context_artifact)
            .collect::<Vec<_>>();

        let mut campaigns = Vec::new();
        for campaign_id in explicit_context_refs.campaign_ids {
            let campaign = self
                .ensure_active_campaign_owner(user_id, campaign_id)
                .await?;
            push_unique_campaign(&mut campaigns, campaign);
        }
        let campaign_sources = campaigns
            .into_iter()
            .map(map_context_campaign)
            .collect::<Vec<_>>();

        let counts = WorkspaceContextSummaryCounts {
            campaigns: campaign_sources.len(),
            conversation_files: attachment_sources.len(),
            product_assets: product_asset_sources.len(),
            product_materials: product_material_sources.len(),
            workspace_artifacts: artifact_sources.len(),
        };
        let mut missing = Vec::new();
        if conversation_id.is_none() {
            missing.push("conversation_id".to_string());
        }
        if product_id.is_none() {
            missing.push("product_id".to_string());
        }
        if counts.total() == 0 {
            missing.push("context_sources".to_string());
        }
        let mut warnings = Vec::new();
        if attachment_sources.is_empty() {
            warnings.push("no active conversation attachments were found".to_string());
        }
        if product_asset_sources.is_empty() {
            warnings.push("no active product assets were found".to_string());
        }
        if product_material_sources.is_empty() {
            warnings.push("no active product materials were found".to_string());
        }
        if artifact_sources.is_empty() {
            warnings.push("no active workspace artifacts were found".to_string());
        }
        let readiness = if counts.total() == 0 {
            "empty"
        } else if missing.is_empty() {
            "ready"
        } else {
            "partial"
        }
        .to_string();
        let summary = build_context_summary_text(&counts, &readiness);
        let agent_context = json!({
            "kind": "workspace_context_summary",
            "readiness": readiness,
            "workspaceId": workspace_id,
            "conversationId": conversation_id,
            "productId": product_id,
            "runId": run_id,
            "summary": summary,
            "counts": {
                "conversationFiles": counts.conversation_files,
                "campaigns": counts.campaigns,
                "productAssets": counts.product_assets,
                "productMaterials": counts.product_materials,
                "workspaceArtifacts": counts.workspace_artifacts,
            },
            "refs": context_ref_keys(
                &attachment_sources,
                &campaign_sources,
                &product_asset_sources,
                &product_material_sources,
                &artifact_sources,
            ),
            "missing": missing,
            "warnings": warnings,
        });

        Ok(WorkspaceContextSummaryResponse {
            workspace_id: workspace_id.to_string(),
            conversation_id: conversation_id.map(|id| id.to_string()),
            product_id: product_id.map(|id| id.to_string()),
            run_id: run_id.map(|id| id.to_string()),
            readiness,
            summary,
            counts,
            attachment_sources,
            campaign_sources,
            product_asset_sources,
            product_material_sources,
            artifact_sources,
            missing,
            warnings,
            agent_context,
        })
    }

    pub async fn context_pack(
        &self,
        req: GetWorkspaceContextPackRequest,
    ) -> BizResult<WorkspaceContextPackResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let workspace = self.ensure_workspace_owner(user_id, workspace_id).await?;
        let conversation_id = parse_optional_uuid(req.conversation_id, "conversation_id")?;
        if let Some(conversation_id) = conversation_id {
            self.ensure_conversation_owner(user_id, conversation_id)
                .await?;
        }
        let requested_product_id = parse_optional_uuid(req.product_id, "product_id")?;
        let product_id = if req.include_workspace_product_context {
            requested_product_id.or(workspace.product_id)
        } else {
            requested_product_id
        };
        if let Some(product_id) = product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let explicit_context_refs = explicit_workspace_context_refs(req.context_refs)?;

        let mut conversation_files = if let Some(conversation_id) = conversation_id {
            self.conversation_file_svc
                .list_by_conversation_id(conversation_id)
                .await?
        } else {
            Vec::new()
        };
        for file_id in explicit_context_refs.conversation_file_ids {
            let file = self
                .ensure_active_conversation_file_owner(user_id, file_id)
                .await?;
            push_unique_conversation_file(&mut conversation_files, file);
        }

        let mut product_assets = if let Some(product_id) = product_id {
            self.product_asset_svc
                .list(ProductAssetFilter {
                    user_id,
                    product_id,
                    status: Some(ProductAssetStatus::Active),
                    asset_type: None,
                    keyword: None,
                    limit: Some(200),
                })
                .await?
        } else {
            Vec::new()
        };
        for asset_id in explicit_context_refs.product_asset_ids {
            let asset = self
                .ensure_active_product_asset_owner(user_id, asset_id)
                .await?;
            push_unique_product_asset(&mut product_assets, asset);
        }

        let mut product_materials = if let Some(product_id) = product_id {
            self.product_material_svc
                .list(ProductMaterialFilter {
                    user_id,
                    product_id,
                    status: Some(ProductMaterialStatus::Active),
                    material_type: None,
                    keyword: None,
                    limit: Some(200),
                })
                .await?
        } else {
            Vec::new()
        };
        for material_id in explicit_context_refs.product_material_ids {
            let material = self
                .ensure_active_product_material_owner(user_id, material_id)
                .await?;
            push_unique_product_material(&mut product_materials, material);
        }

        let mut workspace_artifacts = self
            .artifact_svc
            .list(WorkspaceArtifactFilter {
                workspace_id: Some(workspace_id),
                run_id,
                workspace_task_id: None,
                run_task_id: None,
                user_id,
                artifact_type: None,
                status: Some(WorkspaceArtifactStatus::Active),
                keyword: None,
                limit: Some(200),
                offset: Some(0),
            })
            .await?;
        for artifact_id in explicit_context_refs.workspace_artifact_ids {
            let artifact = self
                .ensure_active_artifact_owner(user_id, workspace_id, artifact_id)
                .await?;
            push_unique_artifact(&mut workspace_artifacts, artifact);
        }

        let mut campaigns = Vec::new();
        for campaign_id in explicit_context_refs.campaign_ids {
            let campaign = self
                .ensure_active_campaign_owner(user_id, campaign_id)
                .await?;
            push_unique_campaign(&mut campaigns, campaign);
        }

        let counts = WorkspaceContextSummaryCounts {
            campaigns: campaigns.len(),
            conversation_files: conversation_files.len(),
            product_assets: product_assets.len(),
            product_materials: product_materials.len(),
            workspace_artifacts: workspace_artifacts.len(),
        };
        let max_items = req
            .max_items
            .unwrap_or(DEFAULT_CONTEXT_PACK_MAX_ITEMS)
            .clamp(1, MAX_CONTEXT_PACK_MAX_ITEMS);
        let max_chars = req
            .max_chars
            .unwrap_or(DEFAULT_CONTEXT_PACK_MAX_CHARS)
            .clamp(1, MAX_CONTEXT_PACK_MAX_CHARS);
        let candidates = conversation_files
            .into_iter()
            .map(context_pack_conversation_file_candidate)
            .chain(campaigns.into_iter().map(context_pack_campaign_candidate))
            .chain(
                product_assets
                    .into_iter()
                    .map(context_pack_product_asset_candidate),
            )
            .chain(
                product_materials
                    .into_iter()
                    .map(context_pack_product_material_candidate),
            )
            .chain(
                workspace_artifacts
                    .into_iter()
                    .map(context_pack_artifact_candidate),
            )
            .collect::<Vec<_>>();
        let pack = build_context_pack_items(candidates, max_items, max_chars);
        let mut warnings = Vec::new();
        if counts.total() == 0 {
            warnings.push("no active workspace context sources were found".to_string());
        }
        if counts.conversation_files > 0 {
            warnings.extend(conversation_file_context_warnings(&pack.items));
        }
        if pack.readable_items == 0 && counts.total() > 0 {
            warnings.push(
                "workspace context sources exist, but no readable text content is available"
                    .to_string(),
            );
        }
        if pack.truncated {
            warnings.push("workspace context pack text was truncated by safety limits".to_string());
        }
        if !pack.omitted.is_empty() {
            warnings.push(format!(
                "{} workspace context source(s) were omitted by item or character limits",
                pack.omitted.len()
            ));
        }
        let readiness = if counts.total() == 0 {
            "empty"
        } else if pack.readable_items == 0 || !pack.omitted.is_empty() {
            "partial"
        } else {
            "ready"
        }
        .to_string();
        let summary = build_context_pack_summary_text(
            &counts,
            &readiness,
            pack.readable_items,
            pack.text_chars,
            pack.omitted.len(),
        );
        let refs = pack
            .items
            .iter()
            .map(|item| item.ref_key.clone())
            .collect::<Vec<_>>();
        let content_refs = pack
            .items
            .iter()
            .filter(|item| item.text.is_some())
            .map(|item| item.ref_key.clone())
            .collect::<Vec<_>>();
        let agent_context = json!({
            "kind": "workspace_context_pack",
            "readiness": readiness,
            "workspaceId": workspace_id,
            "conversationId": conversation_id,
            "productId": product_id,
            "runId": run_id,
            "summary": summary,
            "counts": {
                "conversationFiles": counts.conversation_files,
                "campaigns": counts.campaigns,
                "productAssets": counts.product_assets,
                "productMaterials": counts.product_materials,
                "workspaceArtifacts": counts.workspace_artifacts,
            },
            "refs": refs,
            "contentRefs": content_refs,
            "items": pack.items,
            "omitted": pack.omitted,
            "warnings": warnings,
        });

        Ok(WorkspaceContextPackResponse {
            workspace_id: workspace_id.to_string(),
            conversation_id: conversation_id.map(|id| id.to_string()),
            product_id: product_id.map(|id| id.to_string()),
            run_id: run_id.map(|id| id.to_string()),
            readiness,
            summary,
            counts,
            items: pack.items,
            omitted: pack.omitted,
            warnings,
            agent_context,
        })
    }

    pub async fn link_agent_run(
        &self,
        req: LinkAgentWorkspaceRunRequest,
    ) -> BizResult<LinkedAgentWorkspaceRunResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let (workspace, created_workspace) = self
            .ensure_agent_workspace(user_id, req.requested_workspace_id)
            .await?;
        let goal = compact_text(&req.input, 240);
        let run = self
            .run_svc
            .create(CreateWorkspaceRun {
                id: Uuid::new_v4(),
                workspace_id: workspace.id,
                user_id,
                goal: goal.clone(),
                requirements: json!({
                    "input": req.input,
                    "sessionId": req.session_id,
                    "agentId": req.agent_id,
                    "metadata": req.metadata,
                }),
                status: WorkspaceRunStatus::Running,
                source: AGENT_RUN_SOURCE.to_string(),
                summary: Some(goal),
                report: json!({
                    "status": "running",
                    "sessionId": req.session_id,
                    "output": null,
                    "tasks": [],
                }),
                metadata: json!({
                    "kind": "agent_session",
                    "sessionId": req.session_id,
                    "agentId": req.agent_id,
                }),
            })
            .await?;

        Ok(LinkedAgentWorkspaceRunResponse {
            workspace: Self::map_workspace(workspace),
            run: Self::map_run(run),
            created_workspace,
        })
    }

    pub async fn link_agent_tasks(
        &self,
        req: LinkAgentWorkspaceTasksRequest,
    ) -> BizResult<LinkedAgentWorkspaceTasksResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        let run = self.ensure_run_owner(user_id, workspace_id, run_id).await?;

        let tx = self.db.begin().await?;
        let task_tx_svc = WorkspaceTaskService::new(tx.clone());
        let run_task_tx_svc = WorkspaceRunTaskService::new(tx.clone());
        let run_tx_svc = WorkspaceRunService::new(tx.clone());
        let write_result: BizResult<(WorkspaceRun, Vec<WorkspaceRunTask>)> = async {
            let mut run_tasks = Vec::new();
            for (index, node) in req.nodes.iter().enumerate() {
                let workspace_task = task_tx_svc
                    .create(CreateWorkspaceTask {
                        id: Uuid::new_v4(),
                        workspace_id,
                        user_id,
                        title: normalize_required(node.title.clone(), "title")?,
                        description: Some(compact_text(&node.objective, 2000)),
                        owner_role: normalize_string(node.role.clone()),
                        status: WorkspaceTaskStatus::Pending,
                        priority: (10 - index as i32).clamp(0, 10),
                        depends_on: json!(node.input_refs),
                        acceptance_criteria: non_empty_join(&node.acceptance),
                        result: None,
                        metadata: json!({
                            "kind": "agent_task_node",
                            "sessionId": req.session_id,
                            "graphId": node.graph_id,
                            "nodeId": node.node_id,
                            "workerId": node.worker_id,
                            "rootTask": req.root_task,
                            "maxAttempts": node.max_attempts,
                        }),
                    })
                    .await?;

                let run_task = run_task_tx_svc
                    .create(CreateWorkspaceRunTask {
                        id: Uuid::new_v4(),
                        run_id,
                        workspace_id,
                        workspace_task_id: Some(workspace_task.id),
                        user_id,
                        seq: index as i32,
                        owner_role: normalize_string(node.role.clone()),
                        status: WorkspaceRunTaskStatus::Pending,
                        attempts: 0,
                        acceptance_score: 0.0,
                        acceptance_passed: false,
                        acceptance_notes: None,
                        execution_notes: None,
                        metadata: json!({
                            "kind": "agent_task_node",
                            "sessionId": req.session_id,
                            "graphId": node.graph_id,
                            "nodeId": node.node_id,
                            "workerId": node.worker_id,
                            "title": node.title,
                            "objective": node.objective,
                            "inputRefs": node.input_refs,
                            "acceptance": node.acceptance,
                            "maxAttempts": node.max_attempts,
                        }),
                    })
                    .await?;
                run_tasks.push(run_task);
            }

            let run = run_tx_svc
                .update(UpdateWorkspaceRun {
                    id: run.id,
                    goal: run.goal,
                    requirements: run.requirements,
                    status: WorkspaceRunStatus::Running,
                    source: run.source,
                    summary: run.summary,
                    report: merge_json_object(
                        run.report,
                        json!({
                            "status": "running",
                            "rootTask": req.root_task,
                            "plannedTaskCount": run_tasks.len(),
                            "tasks": run_tasks.iter().map(Self::run_task_report_item).collect::<Vec<_>>(),
                        }),
                    ),
                    metadata: merge_json_object(
                        run.metadata,
                        json!({
                            "lastGraphId": req.nodes.first().map(|node| node.graph_id.clone()),
                            "plannedTaskCount": run_tasks.len(),
                        }),
                    ),
                })
                .await?;
            Ok((run, run_tasks))
        }
        .await;

        let (_, tasks) = match write_result {
            Ok(value) => {
                tx.commit().await?;
                value
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };

        Ok(LinkedAgentWorkspaceTasksResponse {
            tasks: tasks.into_iter().map(Self::map_run_task).collect(),
        })
    }

    pub async fn mark_agent_task_started(
        &self,
        req: MarkAgentWorkspaceTaskStartedRequest,
    ) -> BizResult<WorkspaceRunTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let current = self
            .find_agent_run_task(
                user_id,
                workspace_id,
                run_id,
                &req.graph_id,
                &req.node_id,
                Some(&req.worker_id),
            )
            .await?;

        let task = self
            .run_task_svc
            .update(UpdateWorkspaceRunTask {
                id: current.id,
                seq: current.seq,
                owner_role: current.owner_role,
                status: WorkspaceRunTaskStatus::Running,
                attempts: i32::from(req.attempt).max(current.attempts),
                acceptance_score: current.acceptance_score,
                acceptance_passed: current.acceptance_passed,
                acceptance_notes: current.acceptance_notes,
                execution_notes: Some(format!("Agent worker {} started.", req.worker_id)),
                metadata: merge_json_object(
                    current.metadata,
                    json!({
                        "attempt": req.attempt,
                        "started": true,
                    }),
                ),
            })
            .await?;
        if let Some(task_id) = task.workspace_task_id {
            self.update_workspace_task_status(
                user_id,
                workspace_id,
                task_id,
                WorkspaceTaskStatus::InProgress,
                None,
                None,
            )
            .await?;
        }
        Ok(Self::map_run_task(task))
    }

    pub async fn record_agent_task_report(
        &self,
        req: RecordAgentWorkspaceTaskReportRequest,
    ) -> BizResult<WorkspaceRunTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let current = self
            .find_agent_run_task(
                user_id,
                workspace_id,
                run_id,
                &req.graph_id,
                &req.node_id,
                Some(&req.worker_id),
            )
            .await?;
        let status = if req.status == "failed" {
            WorkspaceRunTaskStatus::Failed
        } else {
            WorkspaceRunTaskStatus::Succeeded
        };
        let acceptance_notes = req
            .role_output
            .as_ref()
            .and_then(|value| value.get("summary"))
            .and_then(Value::as_str)
            .map(compact_optional_text)
            .or_else(|| Some(compact_text(&req.content, 1000)));
        let report_json = json!({
            "taskId": req.task_id,
            "graphId": req.graph_id,
            "nodeId": req.node_id,
            "workerId": req.worker_id,
            "role": req.role,
            "status": req.status,
            "content": req.content,
            "roleOutput": req.role_output,
            "evidence": req.evidence,
            "risks": req.risks,
            "openQuestions": req.open_questions,
        });

        let task = self
            .run_task_svc
            .update(UpdateWorkspaceRunTask {
                id: current.id,
                seq: current.seq,
                owner_role: normalize_string(req.role.clone()).or(current.owner_role),
                status,
                attempts: current.attempts.max(1),
                acceptance_score: if status == WorkspaceRunTaskStatus::Succeeded {
                    current.acceptance_score.max(0.8)
                } else {
                    current.acceptance_score
                },
                acceptance_passed: status == WorkspaceRunTaskStatus::Succeeded,
                acceptance_notes: acceptance_notes.clone(),
                execution_notes: Some(compact_text(&req.content, 4000)),
                metadata: merge_json_object(
                    current.metadata,
                    json!({
                        "report": report_json,
                    }),
                ),
            })
            .await?;

        if let Some(task_id) = task.workspace_task_id {
            self.update_workspace_task_status(
                user_id,
                workspace_id,
                task_id,
                if status == WorkspaceRunTaskStatus::Succeeded {
                    WorkspaceTaskStatus::Done
                } else {
                    WorkspaceTaskStatus::Blocked
                },
                Some(compact_text(&req.content, 4000)),
                Some(json!({
                    "agentReport": report_json,
                })),
            )
            .await?;
        }
        Ok(Self::map_run_task(task))
    }

    pub async fn record_agent_task_evaluation(
        &self,
        req: RecordAgentWorkspaceTaskEvaluationRequest,
    ) -> BizResult<WorkspaceRunTaskResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let current = self
            .find_agent_run_task(
                user_id,
                workspace_id,
                run_id,
                &req.graph_id,
                &req.node_id,
                None,
            )
            .await?;
        let task = self
            .run_task_svc
            .update(UpdateWorkspaceRunTask {
                id: current.id,
                seq: current.seq,
                owner_role: current.owner_role,
                status: if req.passed {
                    WorkspaceRunTaskStatus::Succeeded
                } else {
                    current.status
                },
                attempts: current.attempts,
                acceptance_score: req.score.clamp(0.0, 1.0),
                acceptance_passed: req.passed,
                acceptance_notes: Some(compact_text(&req.reasons.join("\n"), 1000)),
                execution_notes: current.execution_notes,
                metadata: merge_json_object(
                    current.metadata,
                    json!({
                        "evaluation": {
                            "passed": req.passed,
                            "score": req.score,
                            "reasons": req.reasons,
                            "reworkInstruction": req.rework_instruction,
                        },
                    }),
                ),
            })
            .await?;
        Ok(Self::map_run_task(task))
    }

    pub async fn record_agent_search_evidence(
        &self,
        req: RecordAgentSearchEvidenceRequest,
    ) -> BizResult<Option<WorkspaceArtifactResponse>> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;

        let Some(draft) = build_search_evidence_artifact(&req) else {
            return Ok(None);
        };

        let artifact = self
            .artifact_svc
            .create(CreateWorkspaceArtifact {
                id: Uuid::new_v4(),
                workspace_id,
                run_id: Some(run_id),
                workspace_task_id: None,
                run_task_id: None,
                user_id,
                artifact_type: "search_evidence".to_string(),
                title: draft.title,
                content: Some(req.output.to_string()),
                preview: Some(draft.preview),
                mime_type: Some("application/json".to_string()),
                format: "json".to_string(),
                storage_kind: "inline".to_string(),
                bucket: None,
                object_key: None,
                file_name: None,
                file_size: None,
                sha256: None,
                source: AGENT_RUN_SOURCE.to_string(),
                status: WorkspaceArtifactStatus::Active,
                validation_status: WorkspaceArtifactValidationStatus::Unchecked,
                validation_report: json!({}),
                metadata: draft.metadata,
            })
            .await?;

        Ok(Some(Self::map_artifact(artifact)))
    }

    pub async fn list_agent_skill_runs(
        &self,
        req: ListAgentSkillRunsRequest,
    ) -> BizResult<AgentSkillRunListResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let status = parse_skill_run_status(req.status)?;
        let skill_id = normalize_optional(req.skill_id);
        let role = normalize_optional(req.role);
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = AgentSkillRunFilter {
            user_id,
            workspace_id: Some(workspace_id),
            run_id: Some(run_id),
            status,
            skill_id,
            role,
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = AgentSkillRunFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.skill_run_svc.count(count_filter).await?;
        let skill_runs = self
            .skill_run_svc
            .list(filter.clone())
            .await?
            .into_iter()
            .map(Self::map_skill_run)
            .collect::<Vec<_>>();

        Ok(AgentSkillRunListResponse {
            skill_runs,
            total,
            page,
            page_size,
            agent_context: json!({
                "kind": "agent_skill_runs",
                "workspaceId": workspace_id,
                "runId": run_id,
                "filters": {
                    "status": filter.status.map(|status| status.as_str()),
                    "skillId": filter.skill_id,
                    "role": filter.role,
                },
                "capabilities": [
                    "skill_audit",
                    "sse_skill_requested",
                    "sse_skill_result",
                    "approval_trace"
                ],
            }),
        })
    }

    pub async fn record_agent_skill_requested(
        &self,
        req: RecordAgentSkillRequestedRequest,
    ) -> BizResult<AgentSkillRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let request_id = normalize_required(req.request_id, "request_id")?;

        if let Some(current) = self
            .skill_run_svc
            .get_by_request(user_id, workspace_id, run_id, &request_id)
            .await?
        {
            return Ok(Self::map_skill_run(current));
        }

        let now = OffsetDateTime::now_utc();
        let skill_run = self
            .skill_run_svc
            .create(CreateAgentSkillRun {
                id: Uuid::new_v4(),
                user_id,
                workspace_id,
                run_id,
                session_id: normalize_optional(req.session_id),
                worker_id: normalize_required(req.worker_id, "worker_id")?,
                task_id: normalize_required(req.task_id, "task_id")?,
                request_id,
                skill_id: normalize_required(req.skill_id, "skill_id")?,
                role: normalize_required(req.role, "role")?,
                status: AgentSkillRunStatus::Requested,
                reason: req.reason.map(|value| compact_optional_text(&value)),
                summary: None,
                error: None,
                input: json!({}),
                output: json!({}),
                execution_mode: None,
                risk_level: None,
                approval_required: false,
                metadata: json!({
                    "source": "agent_sse",
                    "sourceEvent": "skill_requested",
                    "recordedAt": now,
                }),
                requested_at: now,
                completed_at: None,
            })
            .await?;

        Ok(Self::map_skill_run(skill_run))
    }

    pub async fn record_agent_skill_result(
        &self,
        req: RecordAgentSkillResultRequest,
    ) -> BizResult<AgentSkillRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let request_id = normalize_required(req.request_id, "request_id")?;
        let status = parse_required_skill_run_status(req.status)?;
        let now = OffsetDateTime::now_utc();
        let output = req.output.unwrap_or_else(|| json!({}));
        let summary = req.summary.map(|value| compact_optional_text(&value));
        let error = req.error.map(|value| compact_optional_text(&value));
        let execution_mode = normalize_optional(req.execution_mode);
        let risk_level = normalize_optional(req.risk_level);
        let approval_required = req.approval_required.unwrap_or(false);
        let metadata_patch = json!({
            "source": "agent_sse",
            "sourceEvent": "skill_result",
            "recordedAt": now,
            "executionMode": execution_mode,
            "riskLevel": risk_level,
            "approvalRequired": approval_required,
        });

        let skill_run = if let Some(current) = self
            .skill_run_svc
            .get_by_request(user_id, workspace_id, run_id, &request_id)
            .await?
        {
            self.skill_run_svc
                .update(UpdateAgentSkillRun {
                    id: current.id,
                    status,
                    reason: current.reason,
                    summary: summary.clone(),
                    error: error.clone(),
                    input: current.input,
                    output,
                    execution_mode: execution_mode.clone(),
                    risk_level: risk_level.clone(),
                    approval_required,
                    metadata: merge_json_object(current.metadata, metadata_patch),
                    completed_at: Some(now),
                })
                .await?
        } else {
            self.skill_run_svc
                .create(CreateAgentSkillRun {
                    id: Uuid::new_v4(),
                    user_id,
                    workspace_id,
                    run_id,
                    session_id: normalize_optional(req.session_id),
                    worker_id: normalize_required(req.worker_id, "worker_id")?,
                    task_id: normalize_required(req.task_id, "task_id")?,
                    request_id,
                    skill_id: normalize_required(req.skill_id, "skill_id")?,
                    role: normalize_required(req.role, "role")?,
                    status,
                    reason: None,
                    summary,
                    error,
                    input: json!({}),
                    output,
                    execution_mode,
                    risk_level,
                    approval_required,
                    metadata: merge_json_object(
                        json!({
                            "source": "agent_sse",
                            "sourceEvent": "skill_result_without_request",
                        }),
                        metadata_patch,
                    ),
                    requested_at: now,
                    completed_at: Some(now),
                })
                .await?
        };

        Ok(Self::map_skill_run(skill_run))
    }

    pub async fn materialize_agent_workspace_task(
        &self,
        req: MaterializeAgentWorkspaceTaskRequest,
    ) -> BizResult<Option<WorkspaceTaskResponse>> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;

        let request_id = normalize_required(req.request_id, "request_id")?;
        let skill_id = normalize_required(req.skill_id, "skill_id")?;
        if skill_id != "coordination_create_task" {
            return Ok(None);
        }

        let Some(draft) = build_workspace_task_from_skill_output(&req.output) else {
            return Ok(None);
        };

        let current_skill_run = match self
            .skill_run_svc
            .get_by_request(user_id, workspace_id, run_id, &request_id)
            .await?
        {
            Some(item) => item,
            None => return Ok(None),
        };

        if let Some(existing_task_id) = current_skill_run
            .metadata
            .get("workspaceTaskId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let task_id = parse_uuid(existing_task_id, "workspace_task_id")?;
            let task = self
                .ensure_task_owner(user_id, workspace_id, task_id)
                .await?;
            return Ok(Some(Self::map_task(task)));
        }

        let tx = self.db.begin().await?;
        let task_tx_svc = WorkspaceTaskService::new(tx.clone());
        let skill_run_tx_svc = AgentSkillRunService::new(tx.clone());
        let write_result: BizResult<(WorkspaceTask, AgentSkillRun)> = async {
            let task = task_tx_svc
                .create(CreateWorkspaceTask {
                    id: Uuid::new_v4(),
                    workspace_id,
                    user_id,
                    title: draft.title.clone(),
                    description: draft.description.clone(),
                    owner_role: draft.owner_role.clone(),
                    status: WorkspaceTaskStatus::Pending,
                    priority: draft.priority,
                    depends_on: json!([]),
                    acceptance_criteria: draft.acceptance_criteria.clone(),
                    result: None,
                    metadata: json!({
                        "kind": "agent_created_workspace_task",
                        "source": "agent_skill_result",
                        "runId": run_id,
                        "requestId": request_id,
                        "skillId": skill_id,
                        "role": req.role,
                        "workspaceId": workspace_id,
                    }),
                })
                .await?;

            let skill_run = skill_run_tx_svc
                .update(UpdateAgentSkillRun {
                    id: current_skill_run.id,
                    status: current_skill_run.status,
                    reason: current_skill_run.reason.clone(),
                    summary: current_skill_run.summary.clone(),
                    error: current_skill_run.error.clone(),
                    input: current_skill_run.input.clone(),
                    output: current_skill_run.output.clone(),
                    execution_mode: current_skill_run.execution_mode.clone(),
                    risk_level: current_skill_run.risk_level.clone(),
                    approval_required: current_skill_run.approval_required,
                    metadata: merge_json_object(
                        current_skill_run.metadata.clone(),
                        json!({
                            "workspaceTaskId": task.id,
                            "workspaceTaskCreated": true,
                            "workspaceTaskSource": "coordination_create_task",
                        }),
                    ),
                    completed_at: current_skill_run.completed_at,
                })
                .await?;

            Ok((task, skill_run))
        }
        .await;

        let (task, _) = match write_result {
            Ok(value) => {
                tx.commit().await?;
                value
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };

        Ok(Some(Self::map_task(task)))
    }

    pub async fn materialize_agent_workspace_memory(
        &self,
        req: MaterializeAgentWorkspaceMemoryRequest,
    ) -> BizResult<Option<AgentLearningRecordResponse>> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;

        let request_id = normalize_required(req.request_id, "request_id")?;
        let skill_id = normalize_required(req.skill_id, "skill_id")?;
        if skill_id != "coordination_save_memory" {
            return Ok(None);
        }

        let Some(draft) = build_workspace_memory_from_skill_output(&req.output) else {
            return Ok(None);
        };

        let current_skill_run = match self
            .skill_run_svc
            .get_by_request(user_id, workspace_id, run_id, &request_id)
            .await?
        {
            Some(item) => item,
            None => return Ok(None),
        };

        if let Some(existing_learning_record_id) = current_skill_run
            .metadata
            .get("learningRecordId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let learning_record_id = parse_uuid(existing_learning_record_id, "learning_record_id")?;
            if let Some(record) = self.learning_svc.get_by_id(learning_record_id).await? {
                return Ok(Some(Self::map_learning_record(record)));
            }
        }

        let existing_record = self
            .learning_svc
            .list(AgentLearningRecordFilter {
                user_id,
                workspace_id: Some(workspace_id),
                run_id: None,
                quality_check_id: None,
                role: Some("agent".to_string()),
                category: Some(draft.memory_type.clone()),
                source_action: Some("workspace_memory_manual".to_string()),
                status: Some(AgentLearningRecordStatus::Active),
                limit: Some(100),
                offset: Some(0),
            })
            .await?
            .into_iter()
            .find(|record| learning_record_memory_key(record) == Some(draft.key.as_str()));

        let tx = self.db.begin().await?;
        let learning_tx_svc = AgentLearningRecordService::new(tx.clone());
        let skill_run_tx_svc = AgentSkillRunService::new(tx.clone());
        let write_result: BizResult<(AgentLearningRecord, AgentSkillRun, &'static str)> = async {
            let upsert_mode = if let Some(existing) = existing_record {
                let record = learning_tx_svc
                    .update(
                        repo::table::agent_learning_records::UpdateAgentLearningRecord {
                            id: existing.id,
                            workspace_id: Some(workspace_id),
                            run_id: Some(run_id),
                            quality_check_id: existing.quality_check_id,
                            role: existing.role.clone(),
                            category: draft.memory_type.clone(),
                            content: draft.content.clone(),
                            confidence: draft.confidence,
                            source_action: "workspace_memory_manual".to_string(),
                            feedback_rating: existing.feedback_rating,
                            feedback_comment: existing.feedback_comment.clone(),
                            status: AgentLearningRecordStatus::Active,
                            metadata: merge_json_object(
                                existing.metadata.clone(),
                                json!({
                                    "kind": "workspace_memory",
                                    "memoryKey": draft.key,
                                    "memoryType": draft.memory_type,
                                    "source": "agent_skill_result",
                                    "sourceRole": req.role,
                                    "requestId": request_id,
                                    "skillId": skill_id,
                                    "workspaceId": workspace_id,
                                    "runId": run_id,
                                    "upsertMode": "update",
                                }),
                            ),
                        },
                    )
                    .await?;
                (record, current_skill_run.clone(), "update")
            } else {
                let record = learning_tx_svc
                    .create(CreateAgentLearningRecord {
                        id: Uuid::new_v4(),
                        user_id,
                        workspace_id: Some(workspace_id),
                        run_id: Some(run_id),
                        quality_check_id: None,
                        role: "agent".to_string(),
                        category: draft.memory_type.clone(),
                        content: draft.content.clone(),
                        confidence: draft.confidence,
                        source_action: "workspace_memory_manual".to_string(),
                        feedback_rating: None,
                        feedback_comment: None,
                        status: AgentLearningRecordStatus::Active,
                        metadata: json!({
                            "kind": "workspace_memory",
                            "memoryKey": draft.key,
                            "memoryType": draft.memory_type,
                            "source": "agent_skill_result",
                            "sourceRole": req.role,
                            "requestId": request_id,
                            "skillId": skill_id,
                            "workspaceId": workspace_id,
                            "runId": run_id,
                            "upsertMode": "create",
                        }),
                    })
                    .await?;
                (record, current_skill_run.clone(), "create")
            };

            let (record, skill_run, upsert_mode) = upsert_mode;
            let updated_skill_run = skill_run_tx_svc
                .update(UpdateAgentSkillRun {
                    id: skill_run.id,
                    status: skill_run.status,
                    reason: skill_run.reason.clone(),
                    summary: skill_run.summary.clone(),
                    error: skill_run.error.clone(),
                    input: skill_run.input.clone(),
                    output: skill_run.output.clone(),
                    execution_mode: skill_run.execution_mode.clone(),
                    risk_level: skill_run.risk_level.clone(),
                    approval_required: skill_run.approval_required,
                    metadata: merge_json_object(
                        skill_run.metadata.clone(),
                        json!({
                            "learningRecordId": record.id,
                            "workspaceMemoryKey": draft.key,
                            "workspaceMemorySaved": true,
                            "workspaceMemoryUpsertMode": upsert_mode,
                        }),
                    ),
                    completed_at: skill_run.completed_at,
                })
                .await?;

            Ok((record, updated_skill_run, upsert_mode))
        }
        .await;

        match write_result {
            Ok((record, _, _)) => {
                tx.commit().await?;
                Ok(Some(Self::map_learning_record(record)))
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    pub async fn materialize_agent_campaign_draft(
        &self,
        req: MaterializeAgentCampaignDraftRequest,
    ) -> BizResult<Option<MaterializedAgentCampaignDraftResponse>> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;

        let request_id = normalize_required(req.request_id, "request_id")?;
        let skill_id = normalize_required(req.skill_id, "skill_id")?;
        if skill_id != "coordination_create_campaign" {
            return Ok(None);
        }

        let Some(draft) = build_campaign_draft_from_skill_output(&req.output) else {
            return Ok(None);
        };
        if let Some(product_id) = draft.product_id {
            self.ensure_active_product_owner(user_id, product_id)
                .await?;
        }

        let current_skill_run = match self
            .skill_run_svc
            .get_by_request(user_id, workspace_id, run_id, &request_id)
            .await?
        {
            Some(item) => item,
            None => return Ok(None),
        };

        if let Some(existing_campaign_id) = current_skill_run
            .metadata
            .get("campaignId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let campaign_id = parse_uuid(existing_campaign_id, "campaign_id")?;
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

            let artifact = if let Some(artifact_id) = current_skill_run
                .metadata
                .get("campaignArtifactId")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let artifact_id = parse_uuid(artifact_id, "campaign_artifact_id")?;
                Some(Self::map_artifact(
                    self.ensure_artifact_owner(user_id, workspace_id, artifact_id)
                        .await?,
                ))
            } else {
                None
            };

            return Ok(Some(MaterializedAgentCampaignDraftResponse {
                campaign: Self::map_materialized_campaign_draft(campaign, &draft),
                artifact,
            }));
        }

        let tx = self.db.begin().await?;
        let campaign_tx_svc = CampaignService::new(tx.clone());
        let artifact_tx_svc = WorkspaceArtifactService::new(tx.clone());
        let skill_run_tx_svc = AgentSkillRunService::new(tx.clone());
        let write_result: BizResult<(Campaign, WorkspaceArtifact, AgentSkillRun)> = async {
            let campaign = campaign_tx_svc
                .create(CreateCampaign {
                    id: Uuid::new_v4(),
                    user_id,
                    product_id: draft.product_id,
                    name: draft.name.clone(),
                    campaign_type: Some(draft.campaign_type.clone()),
                    description: draft.description.clone(),
                    budget: draft.budget.unwrap_or_default(),
                    start_date: draft.start_date.clone(),
                    end_date: draft.end_date.clone(),
                    status: CampaignStatus::Draft,
                    metadata: draft.metadata.clone(),
                    output: draft.output.clone(),
                    action_metadata: draft.action_metadata.clone(),
                })
                .await?;

            let artifact = artifact_tx_svc
                .create(CreateWorkspaceArtifact {
                    id: Uuid::new_v4(),
                    workspace_id,
                    run_id: Some(run_id),
                    workspace_task_id: None,
                    run_task_id: None,
                    user_id,
                    artifact_type: "campaign_draft".to_string(),
                    title: format!("Campaign draft - {}", campaign.name),
                    content: Some(
                        serde_json::to_string_pretty(&json!({
                            "campaign": {
                                "id": campaign.id,
                                "name": campaign.name,
                                "campaignType": campaign.campaign_type,
                                "description": campaign.description,
                                "budget": campaign.budget,
                                "startDate": campaign.start_date,
                                "endDate": campaign.end_date,
                                "status": campaign.status.as_str(),
                            },
                            "source": draft.source,
                            "warnings": draft.warnings,
                            "agentContext": draft.agent_context,
                            "planning": draft.output.get("planning").cloned().unwrap_or_else(|| json!({})),
                            "draft": draft.output.get("draft").cloned().unwrap_or_else(|| json!({})),
                        }))
                        .unwrap_or_else(|_| "{}".to_string()),
                    ),
                    preview: Some(compact_text(
                        &format!(
                            "{} | {} | budget={}",
                            campaign.name, draft.campaign_type, campaign.budget
                        ),
                        500,
                    )),
                    mime_type: Some("application/json".to_string()),
                    format: "json".to_string(),
                    storage_kind: "inline".to_string(),
                    bucket: None,
                    object_key: None,
                    file_name: Some(format!("campaign-draft-{}.json", campaign.id)),
                    file_size: None,
                    sha256: None,
                    source: AGENT_RUN_SOURCE.to_string(),
                    status: WorkspaceArtifactStatus::Active,
                    validation_status: WorkspaceArtifactValidationStatus::Unchecked,
                    validation_report: json!({}),
                    metadata: json!({
                        "kind": "agent_campaign_draft",
                        "campaignId": campaign.id,
                        "runId": run_id,
                        "requestId": request_id,
                        "skillId": skill_id,
                        "role": req.role,
                        "workspaceId": workspace_id,
                        "source": draft.source,
                        "warnings": draft.warnings,
                    }),
                })
                .await?;

            let skill_run = skill_run_tx_svc
                .update(UpdateAgentSkillRun {
                    id: current_skill_run.id,
                    status: current_skill_run.status,
                    reason: current_skill_run.reason.clone(),
                    summary: current_skill_run.summary.clone(),
                    error: current_skill_run.error.clone(),
                    input: current_skill_run.input.clone(),
                    output: current_skill_run.output.clone(),
                    execution_mode: current_skill_run.execution_mode.clone(),
                    risk_level: current_skill_run.risk_level.clone(),
                    approval_required: current_skill_run.approval_required,
                    metadata: merge_json_object(
                        current_skill_run.metadata.clone(),
                        json!({
                            "campaignId": campaign.id,
                            "campaignCreated": true,
                            "campaignSource": "coordination_create_campaign",
                            "campaignArtifactId": artifact.id,
                        }),
                    ),
                    completed_at: current_skill_run.completed_at,
                })
                .await?;

            Ok((campaign, artifact, skill_run))
        }
        .await;

        match write_result {
            Ok((campaign, artifact, _)) => {
                tx.commit().await?;
                Ok(Some(MaterializedAgentCampaignDraftResponse {
                    campaign: Self::map_materialized_campaign_draft(campaign, &draft),
                    artifact: Some(Self::map_artifact(artifact)),
                }))
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    pub async fn materialize_agent_content_artifact(
        &self,
        req: MaterializeAgentContentArtifactRequest,
    ) -> BizResult<Option<WorkspaceArtifactResponse>> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;

        let request_id = normalize_required(req.request_id, "request_id")?;
        let skill_id = normalize_required(req.skill_id, "skill_id")?;
        let Some(draft) = build_content_artifact_from_skill_output(&skill_id, &req.output) else {
            return Ok(None);
        };

        let current_skill_run = match self
            .skill_run_svc
            .get_by_request(user_id, workspace_id, run_id, &request_id)
            .await?
        {
            Some(item) => item,
            None => return Ok(None),
        };

        if let Some(existing_artifact_id) = current_skill_run
            .metadata
            .get("contentArtifactId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let artifact_id = parse_uuid(existing_artifact_id, "content_artifact_id")?;
            let artifact = self
                .ensure_artifact_owner(user_id, workspace_id, artifact_id)
                .await?;
            return Ok(Some(Self::map_artifact(artifact)));
        }

        let tx = self.db.begin().await?;
        let artifact_tx_svc = WorkspaceArtifactService::new(tx.clone());
        let skill_run_tx_svc = AgentSkillRunService::new(tx.clone());
        let write_result: BizResult<(WorkspaceArtifact, AgentSkillRun)> = async {
            let artifact = artifact_tx_svc
                .create(CreateWorkspaceArtifact {
                    id: Uuid::new_v4(),
                    workspace_id,
                    run_id: Some(run_id),
                    workspace_task_id: None,
                    run_task_id: None,
                    user_id,
                    artifact_type: draft.artifact_type.clone(),
                    title: draft.title.clone(),
                    content: Some(draft.content.clone()),
                    preview: Some(draft.preview.clone()),
                    mime_type: Some("text/markdown".to_string()),
                    format: "markdown".to_string(),
                    storage_kind: "inline".to_string(),
                    bucket: None,
                    object_key: None,
                    file_name: Some(draft.file_name.clone()),
                    file_size: None,
                    sha256: None,
                    source: AGENT_RUN_SOURCE.to_string(),
                    status: WorkspaceArtifactStatus::Active,
                    validation_status: WorkspaceArtifactValidationStatus::Unchecked,
                    validation_report: json!({}),
                    metadata: json!({
                        "kind": "agent_generated_content",
                        "contentCategory": draft.content_category,
                        "artifactType": draft.artifact_type,
                        "skillId": skill_id,
                        "requestId": request_id,
                        "role": req.role,
                        "summary": draft.summary,
                    }),
                })
                .await?;

            let skill_run = skill_run_tx_svc
                .update(UpdateAgentSkillRun {
                    id: current_skill_run.id,
                    status: current_skill_run.status,
                    reason: current_skill_run.reason.clone(),
                    summary: current_skill_run.summary.clone(),
                    error: current_skill_run.error.clone(),
                    input: current_skill_run.input.clone(),
                    output: current_skill_run.output.clone(),
                    execution_mode: current_skill_run.execution_mode.clone(),
                    risk_level: current_skill_run.risk_level.clone(),
                    approval_required: current_skill_run.approval_required,
                    metadata: merge_json_object(
                        current_skill_run.metadata.clone(),
                        json!({
                            "contentArtifactId": artifact.id,
                            "contentArtifactCreated": true,
                            "contentArtifactType": artifact.artifact_type,
                            "contentArtifactSource": skill_id,
                        }),
                    ),
                    completed_at: current_skill_run.completed_at,
                })
                .await?;

            Ok((artifact, skill_run))
        }
        .await;

        match write_result {
            Ok((artifact, _)) => {
                tx.commit().await?;
                Ok(Some(Self::map_artifact(artifact)))
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }

    pub async fn complete_agent_run(
        &self,
        req: CompleteAgentWorkspaceRunRequest,
    ) -> BizResult<CompleteAgentWorkspaceRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        let current = self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let tasks = self
            .run_task_svc
            .list(WorkspaceRunTaskFilter {
                run_id: Some(run_id),
                workspace_id: Some(workspace_id),
                user_id,
                status: None,
                limit: None,
                offset: None,
            })
            .await?;
        let output = req.output;
        let summary = compact_text(&output, 800);
        let preview = compact_text(&output, 600);
        let task_count = tasks.len();
        let task_report_items = tasks
            .iter()
            .map(Self::run_task_report_item)
            .collect::<Vec<_>>();

        let tx = self.db.begin().await?;
        let run_tx_svc = WorkspaceRunService::new(tx.clone());
        let artifact_tx_svc = WorkspaceArtifactService::new(tx.clone());
        let write_result: BizResult<(WorkspaceRun, WorkspaceArtifact)> = async {
            let artifact_id = Uuid::new_v4();
            let run = run_tx_svc
                .update(UpdateWorkspaceRun {
                    id: current.id,
                    goal: current.goal,
                    requirements: current.requirements,
                    status: WorkspaceRunStatus::Succeeded,
                    source: current.source,
                    summary: Some(summary.clone()),
                    report: merge_json_object(
                        current.report,
                        json!({
                            "status": "succeeded",
                            "output": output,
                            "artifactId": artifact_id,
                            "taskCount": task_count,
                            "tasks": task_report_items,
                        }),
                    ),
                    metadata: merge_json_object(
                        current.metadata,
                        json!({
                            "completed": true,
                            "artifactId": artifact_id,
                        }),
                    ),
                })
                .await?;

            let artifact = artifact_tx_svc
                .create(CreateWorkspaceArtifact {
                    id: artifact_id,
                    workspace_id,
                    run_id: Some(run_id),
                    workspace_task_id: None,
                    run_task_id: None,
                    user_id,
                    artifact_type: "final_report".to_string(),
                    title: format!("Agent final report - {}", compact_text(&run.goal, 80)),
                    content: Some(output),
                    preview: Some(preview),
                    mime_type: Some("text/markdown".to_string()),
                    format: "markdown".to_string(),
                    storage_kind: "inline".to_string(),
                    bucket: None,
                    object_key: None,
                    file_name: None,
                    file_size: None,
                    sha256: None,
                    source: AGENT_RUN_SOURCE.to_string(),
                    status: WorkspaceArtifactStatus::Active,
                    validation_status: WorkspaceArtifactValidationStatus::Unchecked,
                    validation_report: json!({}),
                    metadata: json!({
                        "kind": "agent_final_report",
                        "runId": run_id,
                        "workspaceId": workspace_id,
                        "taskCount": task_count,
                    }),
                })
                .await?;
            Ok((run, artifact))
        }
        .await;

        let (run, artifact) = match write_result {
            Ok(value) => {
                tx.commit().await?;
                value
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };

        Ok(CompleteAgentWorkspaceRunResponse {
            run: Self::map_run(run),
            artifact: Some(Self::map_artifact(artifact)),
        })
    }

    pub async fn list_agent_quality_checks(
        &self,
        req: ListAgentQualityChecksRequest,
    ) -> BizResult<AgentQualityCheckListResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = AgentQualityCheckFilter {
            workspace_id: Some(workspace_id),
            run_id: Some(run_id),
            artifact_id: None,
            user_id,
            role: None,
            passed: None,
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = AgentQualityCheckFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.quality_svc.count(count_filter).await?;
        let checks = self
            .quality_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_quality_check)
            .collect();
        Ok(AgentQualityCheckListResponse {
            checks,
            total,
            page,
            page_size,
        })
    }

    pub async fn evaluate_agent_run(
        &self,
        req: EvaluateAgentWorkspaceRunRequest,
    ) -> BizResult<AgentQualityEvaluationResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        let run = self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let artifact_id = parse_optional_uuid(req.artifact_id, "artifact_id")?;
        let artifact = if let Some(artifact_id) = artifact_id {
            let artifact = self
                .ensure_artifact_owner(user_id, workspace_id, artifact_id)
                .await?;
            if artifact.run_id != Some(run_id) {
                return Err(BizError::new(
                    app_error::AGENT_QUALITY_REQUEST_INVALID,
                    "artifact does not belong to current run".to_string(),
                ));
            }
            Some(artifact)
        } else {
            None
        };
        let output = req
            .output
            .and_then(normalize_string)
            .or_else(|| {
                artifact
                    .as_ref()
                    .and_then(|item| item.content.clone().or(item.preview.clone()))
            })
            .or_else(|| {
                run.report
                    .get("output")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_default();
        let tasks = self
            .run_task_svc
            .list(WorkspaceRunTaskFilter {
                run_id: Some(run_id),
                workspace_id: Some(workspace_id),
                user_id,
                status: None,
                limit: None,
                offset: None,
            })
            .await?;
        let input = run
            .requirements
            .get("input")
            .and_then(Value::as_str)
            .unwrap_or(&run.goal)
            .to_string();
        let role = "agent".to_string();
        let quality = evaluate_output_quality(&input, &output, &tasks);
        let quality_check_id = Uuid::new_v4();

        let tx = self.db.begin().await?;
        let quality_tx_svc = AgentQualityCheckService::new(tx.clone());
        let trust_tx_svc = AgentTrustScoreService::new(tx.clone());
        let learning_tx_svc = AgentLearningRecordService::new(tx.clone());
        let run_tx_svc = WorkspaceRunService::new(tx.clone());
        let artifact_tx_svc = WorkspaceArtifactService::new(tx.clone());
        let write_result: BizResult<(
            AgentQualityCheck,
            AgentTrustScore,
            AgentTrustUpdateResponse,
            Vec<AgentLearningRecord>,
        )> = async {
            let quality_check = quality_tx_svc
                .create(CreateAgentQualityCheck {
                    id: quality_check_id,
                    workspace_id,
                    run_id,
                    artifact_id,
                    user_id,
                    role: role.clone(),
                    score: quality.score,
                    passed: quality.passed,
                    dimensions: quality.dimensions.clone(),
                    issues: json!(quality.issues),
                    suggestions: json!(quality.suggestions),
                    source: AGENT_RUN_SOURCE.to_string(),
                    metadata: json!({
                        "inputChars": input.chars().count(),
                        "outputChars": output.chars().count(),
                        "taskCount": tasks.len(),
                        "source": "rule_based_quality_checker",
                    }),
                })
                .await?;

            let current_trust = trust_tx_svc.find_by_role(user_id, &role, None).await?;
            let previous_score = current_trust.as_ref().map(|item| item.score).unwrap_or(0.5);
            let previous_level = current_trust
                .as_ref()
                .map(|item| item.trust_level.as_str().to_string())
                .unwrap_or_else(|| AgentTrustLevel::Moderate.as_str().to_string());
            let new_score = trust_ema(previous_score, quality.score);
            let new_level = AgentTrustLevel::from_score(new_score);
            let delta = new_score - previous_score;
            let trust_reason = if quality.passed {
                "quality check passed; trust score moved toward the answer quality".to_string()
            } else {
                "quality check found improvement issues; trust score moved down".to_string()
            };
            let trust_score = match current_trust {
                Some(current) => {
                    trust_tx_svc
                        .update(UpdateAgentTrustScore {
                            id: current.id,
                            score: new_score,
                            trust_level: new_level,
                            previous_score: Some(previous_score),
                            previous_level: Some(previous_level.clone()),
                            delta,
                            reason: Some(trust_reason.clone()),
                            source: "quality_check".to_string(),
                            quality_check_id: Some(quality_check.id),
                            metadata: merge_json_object(
                                current.metadata,
                                json!({
                                    "lastRunId": run_id,
                                    "lastWorkspaceId": workspace_id,
                                    "lastQualityScore": quality.score,
                                }),
                            ),
                        })
                        .await?
                }
                None => {
                    trust_tx_svc
                        .create(CreateAgentTrustScore {
                            id: Uuid::new_v4(),
                            user_id,
                            role: role.clone(),
                            skill_id: None,
                            score: new_score,
                            trust_level: new_level,
                            previous_score: Some(previous_score),
                            previous_level: Some(previous_level.clone()),
                            delta,
                            reason: Some(trust_reason.clone()),
                            source: "quality_check".to_string(),
                            quality_check_id: Some(quality_check.id),
                            metadata: json!({
                                            "lastRunId": run_id,
                                            "lastWorkspaceId": workspace_id,
                            "lastQualityScore": quality.score,
                                        }),
                        })
                        .await?
                }
            };
            let new_level_text = new_level.as_str().to_string();
            let trust_update = AgentTrustUpdateResponse {
                role: role.clone(),
                skill_id: None,
                previous_score,
                new_score,
                previous_level,
                new_level: new_level_text.clone(),
                delta,
                reason: trust_reason,
            };

            let mut learning_records = Vec::new();
            for draft in build_quality_learning_drafts(&quality, &run, &output) {
                let record = learning_tx_svc
                    .create(CreateAgentLearningRecord {
                        id: Uuid::new_v4(),
                        user_id,
                        workspace_id: Some(workspace_id),
                        run_id: Some(run_id),
                        quality_check_id: Some(quality_check.id),
                        role: role.clone(),
                        category: draft.category,
                        content: draft.content,
                        confidence: draft.confidence,
                        source_action: draft.source_action,
                        feedback_rating: None,
                        feedback_comment: None,
                        status: AgentLearningRecordStatus::Active,
                        metadata: draft.metadata,
                    })
                    .await?;
                learning_records.push(record);
            }

            run_tx_svc
                .update(UpdateWorkspaceRun {
                    id: run.id,
                    goal: run.goal,
                    requirements: run.requirements,
                    status: run.status,
                    source: run.source,
                    summary: run.summary,
                    report: merge_json_object(
                        run.report,
                        json!({
                            "quality": {
                                "qualityCheckId": quality_check.id,
                                "score": quality.score,
                                "passed": quality.passed,
                                "dimensions": quality.dimensions.clone(),
                                "issues": quality.issues.clone(),
                                "suggestions": quality.suggestions.clone(),
                                "trustScore": new_score,
                                "trustLevel": new_level_text,
                            },
                        }),
                    ),
                    metadata: merge_json_object(
                        run.metadata,
                        json!({
                            "qualityCheckId": quality_check.id,
                            "qualityScore": quality.score,
                            "qualityPassed": quality.passed,
                            "trustScore": new_score,
                            "trustLevel": new_level_text,
                        }),
                    ),
                })
                .await?;

            if let Some(mut artifact) = artifact {
                artifact.validation_status = if quality.passed {
                    WorkspaceArtifactValidationStatus::Passed
                } else {
                    WorkspaceArtifactValidationStatus::Failed
                };
                artifact.validation_report = json!({
                    "qualityCheckId": quality_check.id,
                    "score": quality.score,
                    "passed": quality.passed,
                    "dimensions": quality.dimensions.clone(),
                    "issues": quality.issues.clone(),
                    "suggestions": quality.suggestions.clone(),
                });
                artifact.metadata = merge_json_object(
                    artifact.metadata,
                    json!({
                        "qualityCheckId": quality_check.id,
                        "qualityScore": quality.score,
                        "qualityPassed": quality.passed,
                    }),
                );
                artifact_tx_svc
                    .update(UpdateWorkspaceArtifact {
                        id: artifact.id,
                        title: artifact.title,
                        content: artifact.content,
                        preview: artifact.preview,
                        mime_type: artifact.mime_type,
                        format: artifact.format,
                        storage_kind: artifact.storage_kind,
                        bucket: artifact.bucket,
                        object_key: artifact.object_key,
                        file_name: artifact.file_name,
                        file_size: artifact.file_size,
                        sha256: artifact.sha256,
                        status: artifact.status,
                        validation_status: artifact.validation_status,
                        validation_report: artifact.validation_report,
                        metadata: artifact.metadata,
                    })
                    .await?;
            }

            Ok((quality_check, trust_score, trust_update, learning_records))
        }
        .await;

        let (quality_check, _trust_score, trust_update, learning_records) = match write_result {
            Ok(value) => {
                tx.commit().await?;
                value
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };

        Ok(AgentQualityEvaluationResponse {
            quality_check: Self::map_quality_check(quality_check),
            trust_update,
            learning_records: learning_records
                .into_iter()
                .map(Self::map_learning_record)
                .collect(),
        })
    }

    pub async fn submit_agent_run_feedback(
        &self,
        req: SubmitAgentRunFeedbackRequest,
    ) -> BizResult<AgentRunFeedbackResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let rating = req.rating.clamp(1, 5);
        let category = if rating >= 4 {
            "feedback_positive"
        } else if rating <= 2 {
            "feedback_negative"
        } else {
            "feedback_neutral"
        };
        let comment = normalize_optional(req.comment);
        let content = comment
            .clone()
            .unwrap_or_else(|| format!("User feedback rating: {rating}/5"));
        let record = self
            .learning_svc
            .create(CreateAgentLearningRecord {
                id: Uuid::new_v4(),
                user_id,
                workspace_id: Some(workspace_id),
                run_id: Some(run_id),
                quality_check_id: None,
                role: "agent".to_string(),
                category: category.to_string(),
                content: compact_text(&content, 2000),
                confidence: feedback_confidence(rating),
                source_action: "user_feedback".to_string(),
                feedback_rating: Some(rating),
                feedback_comment: comment,
                status: AgentLearningRecordStatus::Active,
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(AgentRunFeedbackResponse {
            learning_record: Self::map_learning_record(record),
        })
    }

    pub async fn list_agent_learning_records(
        &self,
        req: ListAgentLearningRecordsRequest,
    ) -> BizResult<AgentLearningRecordListResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = AgentLearningRecordFilter {
            user_id,
            workspace_id: Some(workspace_id),
            run_id,
            quality_check_id: None,
            role: normalize_optional(req.role),
            category: normalize_optional(req.category),
            source_action: None,
            status: Some(AgentLearningRecordStatus::Active),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = AgentLearningRecordFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.learning_svc.count(count_filter).await?;
        let records = self
            .learning_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_learning_record)
            .collect();
        Ok(AgentLearningRecordListResponse {
            records,
            total,
            page,
            page_size,
        })
    }

    pub async fn list_agent_trust_scores(
        &self,
        req: ListAgentTrustScoresRequest,
    ) -> BizResult<AgentTrustScoreListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = AgentTrustScoreFilter {
            user_id,
            role: normalize_optional(req.role),
            skill_id: None,
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = AgentTrustScoreFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.trust_svc.count(count_filter).await?;
        let scores = self
            .trust_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_trust_score)
            .collect::<BizResult<Vec<_>>>()?;
        Ok(AgentTrustScoreListResponse {
            scores,
            total,
            page,
            page_size,
        })
    }

    pub async fn build_agent_learning_context(
        &self,
        req: BuildAgentLearningContextRequest,
    ) -> BizResult<AgentLearningContextResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_optional_uuid(req.workspace_id, "workspace_id")?;
        if let Some(workspace_id) = workspace_id {
            self.ensure_workspace_owner(user_id, workspace_id).await?;
        }
        let role = normalize_optional(req.role).unwrap_or_else(|| "agent".to_string());
        let limit = req.limit.unwrap_or(5).clamp(1, 10);
        let mut records = self
            .learning_svc
            .list(AgentLearningRecordFilter {
                user_id,
                workspace_id,
                run_id: None,
                quality_check_id: None,
                role: Some(role.clone()),
                category: None,
                source_action: None,
                status: Some(AgentLearningRecordStatus::Active),
                limit: Some(limit),
                offset: Some(0),
            })
            .await?;
        if records.is_empty() && workspace_id.is_some() {
            records = self
                .learning_svc
                .list(AgentLearningRecordFilter {
                    user_id,
                    workspace_id: None,
                    run_id: None,
                    quality_check_id: None,
                    role: Some(role.clone()),
                    category: None,
                    source_action: None,
                    status: Some(AgentLearningRecordStatus::Active),
                    limit: Some(limit),
                    offset: Some(0),
                })
                .await?;
        }
        let trust_scores = self
            .trust_svc
            .list(AgentTrustScoreFilter {
                user_id,
                role: Some(role),
                skill_id: None,
                limit: Some(5),
                offset: Some(0),
            })
            .await?;
        let learning_summary = records
            .iter()
            .take(5)
            .map(|record| {
                format!(
                    "[{} {:.2}] {}",
                    record.category,
                    record.confidence,
                    compact_text(&record.content, 160)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let trust_summary = json!({
            "scores": trust_scores
                .iter()
                .map(|score| json!({
                    "role": score.role,
                    "skillId": score.skill_id,
                    "score": score.score,
                    "trustLevel": score.trust_level.as_str(),
                    "delta": score.delta,
                    "reason": score.reason,
                }))
                .collect::<Vec<_>>(),
        });
        Ok(AgentLearningContextResponse {
            trust_summary,
            learning_summary,
            records: records.into_iter().map(Self::map_learning_record).collect(),
        })
    }

    pub async fn fail_agent_run(
        &self,
        req: FailAgentWorkspaceRunRequest,
    ) -> BizResult<WorkspaceRunResponse> {
        let (user_id, workspace_id) = self
            .ensure_request_workspace(&req.user_id, &req.workspace_id)
            .await?;
        let run_id = parse_uuid(&req.run_id, "run_id")?;
        let current = self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        let run = self
            .run_svc
            .update(UpdateWorkspaceRun {
                id: current.id,
                goal: current.goal,
                requirements: current.requirements,
                status: WorkspaceRunStatus::Failed,
                source: current.source,
                summary: Some(compact_text(&req.reason, 800)),
                report: merge_json_object(
                    current.report,
                    json!({
                        "status": "failed",
                        "reason": req.reason,
                    }),
                ),
                metadata: merge_json_object(
                    current.metadata,
                    json!({
                        "failed": true,
                    }),
                ),
            })
            .await?;
        Ok(Self::map_run(run))
    }

    pub async fn list_execution_queue(
        &self,
        req: ListExecutionQueueRequest,
    ) -> BizResult<ExecutionQueueListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_optional_uuid(req.workspace_id, "workspace_id")?;
        if let Some(workspace_id) = workspace_id {
            self.ensure_active_workspace_owner(user_id, workspace_id)
                .await?;
        }
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::EXECUTION_QUEUE_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = ExecutionQueueFilter {
            user_id,
            workspace_id,
            run_id,
            status: parse_execution_status(req.status)?,
            action_type: normalize_optional(req.action_type),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = ExecutionQueueFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.execution_svc.count(count_filter).await?;
        let items = self
            .execution_svc
            .list(filter)
            .await?
            .into_iter()
            .map(Self::map_execution)
            .collect();
        Ok(ExecutionQueueListResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_execution_queue(
        &self,
        req: CreateExecutionQueueRequest,
    ) -> BizResult<ExecutionQueueResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_optional_uuid(req.workspace_id, "workspace_id")?;
        if let Some(workspace_id) = workspace_id {
            self.ensure_active_workspace_owner(user_id, workspace_id)
                .await?;
        }
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::EXECUTION_QUEUE_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let workspace_task_id = parse_optional_uuid(req.workspace_task_id, "workspace_task_id")?;
        if let Some(task_id) = workspace_task_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::EXECUTION_QUEUE_REQUEST_INVALID,
                    "workspace_id is required when workspace_task_id is provided".to_string(),
                )
            })?;
            self.ensure_task_owner(user_id, workspace_id, task_id)
                .await?;
        }
        let item = self
            .execution_svc
            .create(CreateExecutionQueueItem {
                id: Uuid::new_v4(),
                user_id,
                workspace_id,
                run_id,
                workspace_task_id,
                action_type: normalize_required(req.action_type, "action_type")?,
                description: normalize_required(req.description, "description")?,
                platform: normalize_optional(req.platform),
                payload: req.payload.unwrap_or_else(|| json!({})),
                expected_impact: normalize_optional(req.expected_impact),
                status: ExecutionQueueStatus::Pending,
                decision_reason: None,
                result: json!({}),
                decided_at: None,
            })
            .await?;
        Ok(Self::map_execution(item))
    }

    pub async fn get_execution_queue(
        &self,
        req: GetExecutionQueueRequest,
    ) -> BizResult<ExecutionQueueResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let execution_id = parse_uuid(&req.execution_id, "execution_id")?;
        let item = self.ensure_execution_owner(user_id, execution_id).await?;
        Ok(Self::map_execution(item))
    }

    pub async fn approve_execution_queue(
        &self,
        req: DecideExecutionQueueRequest,
    ) -> BizResult<ExecutionQueueResponse> {
        self.decide_execution(req, ExecutionQueueStatus::Approved)
            .await
    }

    pub async fn reject_execution_queue(
        &self,
        req: DecideExecutionQueueRequest,
    ) -> BizResult<ExecutionQueueResponse> {
        self.decide_execution(req, ExecutionQueueStatus::Rejected)
            .await
    }

    async fn decide_execution(
        &self,
        req: DecideExecutionQueueRequest,
        status: ExecutionQueueStatus,
    ) -> BizResult<ExecutionQueueResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let execution_id = parse_uuid(&req.execution_id, "execution_id")?;
        let current = self.ensure_execution_owner(user_id, execution_id).await?;
        if current.status != ExecutionQueueStatus::Pending {
            return Err(BizError::new(
                app_error::EXECUTION_QUEUE_REQUEST_INVALID,
                "execution suggestion is not pending".to_string(),
            ));
        }
        let reason = normalize_optional(req.reason);
        let result = req.result.unwrap_or(current.result.clone());
        let now = OffsetDateTime::now_utc();
        let tx = self.db.begin().await?;
        let execution_tx_svc = ExecutionQueueService::new(tx.clone());
        let audit_tx_svc = AuditLogService::new(tx.clone());
        let update_result: BizResult<ExecutionQueueItem> = async {
            let item = execution_tx_svc
                .update(UpdateExecutionQueueItem {
                    id: current.id,
                    status,
                    decision_reason: reason.clone(),
                    result: result.clone(),
                    decided_at: Some(now),
                })
                .await?;
            audit_tx_svc
                .create(CreateAuditLog {
                    id: Uuid::new_v4(),
                    user_id,
                    actor_user_id: user_id,
                    workspace_id: current.workspace_id,
                    run_id: current.run_id,
                    execution_queue_id: Some(current.id),
                    tool_approval_id: None,
                    action: match status {
                        ExecutionQueueStatus::Approved => "execution_queue.approve",
                        ExecutionQueueStatus::Rejected => "execution_queue.reject",
                        _ => "execution_queue.decide",
                    }
                    .to_string(),
                    target_type: "execution_queue".to_string(),
                    target_id: Some(current.id.to_string()),
                    status: "success".to_string(),
                    reason,
                    request_id: None,
                    trace_id: None,
                    metadata: json!({
                        "actionType": current.action_type,
                        "platform": current.platform,
                        "decisionStatus": status.as_str(),
                        "result": result,
                    }),
                })
                .await?;
            Ok(item)
        }
        .await;
        let item = match update_result {
            Ok(item) => {
                tx.commit().await?;
                item
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };
        Ok(Self::map_execution(item))
    }

    async fn ensure_request_workspace(
        &self,
        user_id: &str,
        workspace_id: &str,
    ) -> BizResult<(Uuid, Uuid)> {
        let user_id = parse_user_id(user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_uuid(workspace_id, "workspace_id")?;
        self.ensure_active_workspace_owner(user_id, workspace_id)
            .await?;
        Ok((user_id, workspace_id))
    }

    async fn ensure_agent_workspace(
        &self,
        user_id: Uuid,
        requested_workspace_id: Option<String>,
    ) -> BizResult<(Workspace, bool)> {
        if let Some(workspace_id) = parse_optional_uuid(requested_workspace_id, "workspace_id")? {
            return Ok((
                self.ensure_active_workspace_owner(user_id, workspace_id)
                    .await?,
                false,
            ));
        }

        let mut workspaces = self
            .workspace_svc
            .list(WorkspaceFilter {
                user_id,
                status: Some(WorkspaceStatus::Active),
                product_id: None,
                campaign_id: None,
                workspace_type: Some(AGENT_WORKSPACE_TYPE.to_string()),
                keyword: None,
                limit: Some(1),
                offset: Some(0),
            })
            .await?;
        if let Some(workspace) = workspaces.pop() {
            return Ok((workspace, false));
        }

        let workspace = self
            .workspace_svc
            .create(CreateWorkspace {
                id: Uuid::new_v4(),
                user_id,
                product_id: None,
                campaign_id: None,
                title: AGENT_RUN_TITLE.to_string(),
                workspace_type: AGENT_WORKSPACE_TYPE.to_string(),
                status: WorkspaceStatus::Active,
                config: json!({
                    "createdBy": "agent_session",
                    "default": true,
                }),
            })
            .await?;
        Ok((workspace, true))
    }

    async fn find_agent_run_task(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        run_id: Uuid,
        graph_id: &str,
        node_id: &str,
        worker_id: Option<&str>,
    ) -> BizResult<WorkspaceRunTask> {
        let tasks = self
            .run_task_svc
            .list(WorkspaceRunTaskFilter {
                run_id: Some(run_id),
                workspace_id: Some(workspace_id),
                user_id,
                status: None,
                limit: None,
                offset: None,
            })
            .await?;
        tasks
            .into_iter()
            .find(|task| {
                metadata_str_eq(&task.metadata, "graphId", graph_id)
                    && metadata_str_eq(&task.metadata, "nodeId", node_id)
                    && worker_id
                        .map(|worker_id| metadata_str_eq(&task.metadata, "workerId", worker_id))
                        .unwrap_or(true)
            })
            .ok_or_else(|| {
                BizError::new(
                    app_error::WORKSPACE_RUN_TASK_NOT_FOUND,
                    "workspace run task not found".to_string(),
                )
            })
    }

    async fn update_workspace_task_status(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        task_id: Uuid,
        status: WorkspaceTaskStatus,
        result: Option<String>,
        metadata_patch: Option<Value>,
    ) -> BizResult<WorkspaceTask> {
        let current = self
            .ensure_task_owner(user_id, workspace_id, task_id)
            .await?;
        self.task_svc
            .update(UpdateWorkspaceTask {
                id: current.id,
                title: current.title,
                description: current.description,
                owner_role: current.owner_role,
                status,
                priority: current.priority,
                depends_on: current.depends_on,
                acceptance_criteria: current.acceptance_criteria,
                result: result.or(current.result),
                metadata: metadata_patch
                    .map(|patch| merge_json_object(current.metadata.clone(), patch))
                    .unwrap_or(current.metadata),
            })
            .await
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

    async fn ensure_active_product_owner(&self, user_id: Uuid, product_id: Uuid) -> BizResult<()> {
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
        if product.status == ProductStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_NOT_FOUND,
                "product not found".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_campaign_owner(&self, user_id: Uuid, campaign_id: Uuid) -> BizResult<()> {
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
        Ok(())
    }

    async fn ensure_active_campaign_owner(
        &self,
        user_id: Uuid,
        campaign_id: Uuid,
    ) -> BizResult<Campaign> {
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
        if campaign.status == CampaignStatus::Archived {
            return Err(BizError::new(
                app_error::CAMPAIGN_NOT_FOUND,
                "campaign not found".to_string(),
            ));
        }
        Ok(campaign)
    }

    async fn ensure_conversation_owner(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
    ) -> BizResult<()> {
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
        if conversation.status == "deleted" {
            return Err(BizError::new(
                app_error::CONVERSATION_NOT_FOUND,
                "conversation not found".to_string(),
            ));
        }
        Ok(())
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

    async fn ensure_task_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        task_id: Uuid,
    ) -> BizResult<WorkspaceTask> {
        let task = self.task_svc.get_by_id(task_id).await?.ok_or_else(|| {
            BizError::new(
                app_error::WORKSPACE_TASK_NOT_FOUND,
                "workspace task not found".to_string(),
            )
        })?;
        if task.user_id != user_id || task.workspace_id != workspace_id {
            return Err(BizError::new(
                app_error::WORKSPACE_TASK_FORBIDDEN,
                "workspace task does not belong to current workspace".to_string(),
            ));
        }
        Ok(task)
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

    async fn ensure_run_task_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        run_id: Uuid,
        run_task_id: Uuid,
    ) -> BizResult<WorkspaceRunTask> {
        let task = self
            .run_task_svc
            .get_by_id(run_task_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::WORKSPACE_RUN_TASK_NOT_FOUND,
                    "workspace run task not found".to_string(),
                )
            })?;
        if task.user_id != user_id || task.workspace_id != workspace_id || task.run_id != run_id {
            return Err(BizError::new(
                app_error::WORKSPACE_FORBIDDEN,
                "workspace run task does not belong to current run".to_string(),
            ));
        }
        Ok(task)
    }

    async fn ensure_artifact_owner(
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
        Ok(artifact)
    }

    async fn ensure_active_artifact_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        artifact_id: Uuid,
    ) -> BizResult<WorkspaceArtifact> {
        let artifact = self
            .ensure_artifact_owner(user_id, workspace_id, artifact_id)
            .await?;
        if artifact.status == WorkspaceArtifactStatus::Archived {
            return Err(BizError::new(
                app_error::WORKSPACE_ARTIFACT_NOT_FOUND,
                "workspace artifact not found".to_string(),
            ));
        }
        Ok(artifact)
    }

    async fn ensure_active_conversation_file_owner(
        &self,
        user_id: Uuid,
        file_id: Uuid,
    ) -> BizResult<ConversationFile> {
        let file = self
            .conversation_file_svc
            .get_by_id(file_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::CONVERSATION_FILE_NOT_FOUND,
                    "conversation file not found".to_string(),
                )
            })?;
        if file.user_id != user_id {
            return Err(BizError::new(
                app_error::CONVERSATION_FILE_FORBIDDEN,
                "conversation file does not belong to current user".to_string(),
            ));
        }
        if file.status != "active" {
            return Err(BizError::new(
                app_error::CONVERSATION_FILE_NOT_FOUND,
                "conversation file not found".to_string(),
            ));
        }
        self.ensure_conversation_owner(user_id, file.conversation_id)
            .await?;
        Ok(file)
    }

    async fn ensure_active_product_asset_owner(
        &self,
        user_id: Uuid,
        asset_id: Uuid,
    ) -> BizResult<ProductAsset> {
        let asset = self
            .product_asset_svc
            .get_by_id(asset_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_ASSET_NOT_FOUND,
                    "product asset not found".to_string(),
                )
            })?;
        if asset.user_id != user_id {
            return Err(BizError::new(
                app_error::PRODUCT_ASSET_FORBIDDEN,
                "product asset does not belong to current user".to_string(),
            ));
        }
        if asset.status == ProductAssetStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_ASSET_NOT_FOUND,
                "product asset not found".to_string(),
            ));
        }
        self.ensure_active_product_owner(user_id, asset.product_id)
            .await?;
        Ok(asset)
    }

    async fn ensure_active_product_material_owner(
        &self,
        user_id: Uuid,
        material_id: Uuid,
    ) -> BizResult<ProductMaterial> {
        let material = self
            .product_material_svc
            .get_by_id(material_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PRODUCT_MATERIAL_NOT_FOUND,
                    "product material not found".to_string(),
                )
            })?;
        if material.user_id != user_id {
            return Err(BizError::new(
                app_error::PRODUCT_MATERIAL_FORBIDDEN,
                "product material does not belong to current user".to_string(),
            ));
        }
        if material.status == ProductMaterialStatus::Archived {
            return Err(BizError::new(
                app_error::PRODUCT_MATERIAL_NOT_FOUND,
                "product material not found".to_string(),
            ));
        }
        self.ensure_active_product_owner(user_id, material.product_id)
            .await?;
        Ok(material)
    }

    async fn resolve_artifact_links(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        run_id: Option<String>,
        workspace_task_id: Option<String>,
        run_task_id: Option<String>,
    ) -> BizResult<(Option<Uuid>, Option<Uuid>, Option<Uuid>)> {
        let run_id = parse_optional_uuid(run_id, "run_id")?;
        if let Some(run_id) = run_id {
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let workspace_task_id = parse_optional_uuid(workspace_task_id, "workspace_task_id")?;
        if let Some(task_id) = workspace_task_id {
            self.ensure_task_owner(user_id, workspace_id, task_id)
                .await?;
        }
        let run_task_id = parse_optional_uuid(run_task_id, "run_task_id")?;
        if let Some(run_task_id) = run_task_id {
            let Some(run_id) = run_id else {
                return Err(BizError::new(
                    app_error::WORKSPACE_ARTIFACT_REQUEST_INVALID,
                    "run_id is required when run_task_id is provided".to_string(),
                ));
            };
            self.ensure_run_task_owner(user_id, workspace_id, run_id, run_task_id)
                .await?;
        }
        Ok((run_id, workspace_task_id, run_task_id))
    }

    async fn ensure_execution_owner(
        &self,
        user_id: Uuid,
        execution_id: Uuid,
    ) -> BizResult<ExecutionQueueItem> {
        let item = self
            .execution_svc
            .get_by_id(execution_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::EXECUTION_QUEUE_NOT_FOUND,
                    "execution suggestion not found".to_string(),
                )
            })?;
        if item.user_id != user_id {
            return Err(BizError::new(
                app_error::EXECUTION_QUEUE_FORBIDDEN,
                "execution suggestion does not belong to current user".to_string(),
            ));
        }
        Ok(item)
    }

    fn map_workspace(item: Workspace) -> WorkspaceResponse {
        WorkspaceResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            product_id: item.product_id.map(|id| id.to_string()),
            campaign_id: item.campaign_id.map(|id| id.to_string()),
            title: item.title,
            workspace_type: item.workspace_type,
            status: item.status.as_str().to_string(),
            config: item.config,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_task(item: WorkspaceTask) -> WorkspaceTaskResponse {
        WorkspaceTaskResponse {
            id: item.id.to_string(),
            workspace_id: item.workspace_id.to_string(),
            user_id: item.user_id.to_string(),
            title: item.title,
            description: item.description,
            owner_role: item.owner_role,
            status: item.status.as_str().to_string(),
            priority: item.priority,
            depends_on: item.depends_on,
            acceptance_criteria: item.acceptance_criteria,
            result: item.result,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_run(item: WorkspaceRun) -> WorkspaceRunResponse {
        WorkspaceRunResponse {
            id: item.id.to_string(),
            workspace_id: item.workspace_id.to_string(),
            user_id: item.user_id.to_string(),
            goal: item.goal,
            requirements: item.requirements,
            status: item.status.as_str().to_string(),
            source: item.source,
            summary: item.summary,
            report: item.report,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_run_task(item: WorkspaceRunTask) -> WorkspaceRunTaskResponse {
        WorkspaceRunTaskResponse {
            id: item.id.to_string(),
            run_id: item.run_id.to_string(),
            workspace_id: item.workspace_id.to_string(),
            workspace_task_id: item.workspace_task_id.map(|id| id.to_string()),
            user_id: item.user_id.to_string(),
            seq: item.seq,
            owner_role: item.owner_role,
            status: item.status.as_str().to_string(),
            attempts: item.attempts,
            acceptance_score: item.acceptance_score,
            acceptance_passed: item.acceptance_passed,
            acceptance_notes: item.acceptance_notes,
            execution_notes: item.execution_notes,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_skill_run(item: AgentSkillRun) -> AgentSkillRunResponse {
        AgentSkillRunResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            workspace_id: item.workspace_id.to_string(),
            run_id: item.run_id.to_string(),
            session_id: item.session_id,
            worker_id: item.worker_id,
            task_id: item.task_id,
            request_id: item.request_id,
            skill_id: item.skill_id,
            role: item.role,
            status: item.status.as_str().to_string(),
            reason: item.reason,
            summary: item.summary,
            error: item.error,
            input: item.input,
            output: item.output,
            execution_mode: item.execution_mode,
            risk_level: item.risk_level,
            approval_required: item.approval_required,
            metadata: item.metadata,
            requested_at: item.requested_at,
            completed_at: item.completed_at,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_artifact(item: WorkspaceArtifact) -> WorkspaceArtifactResponse {
        WorkspaceArtifactResponse {
            id: item.id.to_string(),
            workspace_id: item.workspace_id.to_string(),
            run_id: item.run_id.map(|id| id.to_string()),
            workspace_task_id: item.workspace_task_id.map(|id| id.to_string()),
            run_task_id: item.run_task_id.map(|id| id.to_string()),
            user_id: item.user_id.to_string(),
            artifact_type: item.artifact_type,
            title: item.title,
            content: item.content,
            preview: item.preview,
            mime_type: item.mime_type,
            format: item.format,
            storage_kind: item.storage_kind,
            bucket: item.bucket,
            object_key: item.object_key,
            file_name: item.file_name,
            file_size: item.file_size,
            sha256: item.sha256,
            source: item.source,
            status: item.status.as_str().to_string(),
            validation_status: item.validation_status.as_str().to_string(),
            validation_report: item.validation_report,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn map_materialized_campaign_draft(
        item: Campaign,
        draft: &CampaignDraftFromSkill,
    ) -> CampaignDraftResponse {
        CampaignDraftResponse {
            campaign: CampaignResponse {
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
            },
            source: draft.source.clone(),
            warnings: draft.warnings.clone(),
            agent_context: draft.agent_context.clone(),
        }
    }

    fn map_execution(item: ExecutionQueueItem) -> ExecutionQueueResponse {
        ExecutionQueueResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            workspace_id: item.workspace_id.map(|id| id.to_string()),
            run_id: item.run_id.map(|id| id.to_string()),
            workspace_task_id: item.workspace_task_id.map(|id| id.to_string()),
            action_type: item.action_type,
            description: item.description,
            platform: item.platform,
            payload: item.payload,
            expected_impact: item.expected_impact,
            status: item.status.as_str().to_string(),
            decision_reason: item.decision_reason,
            result: item.result,
            created_at: item.created_at,
            decided_at: item.decided_at,
        }
    }

    fn map_quality_check(item: AgentQualityCheck) -> AgentQualityCheckResponse {
        AgentQualityCheckResponse {
            id: item.id.to_string(),
            workspace_id: item.workspace_id.to_string(),
            run_id: item.run_id.to_string(),
            artifact_id: item.artifact_id.map(|id| id.to_string()),
            user_id: item.user_id.to_string(),
            role: item.role,
            score: item.score,
            passed: item.passed,
            dimensions: item.dimensions,
            issues: item.issues,
            suggestions: item.suggestions,
            source: item.source,
            metadata: item.metadata,
            created_at: item.created_at,
        }
    }

    fn map_trust_score(item: AgentTrustScore) -> BizResult<AgentTrustScoreResponse> {
        Ok(AgentTrustScoreResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            role: item.role,
            skill_id: item.skill_id,
            score: item.score,
            trust_level: item.trust_level.as_str().to_string(),
            previous_score: item.previous_score,
            previous_level: item.previous_level,
            delta: item.delta,
            reason: item.reason,
            source: item.source,
            quality_check_id: item.quality_check_id.map(|id| id.to_string()),
            metadata: item.metadata,
            updated_at: item.updated_at,
        })
    }

    fn map_learning_record(item: AgentLearningRecord) -> AgentLearningRecordResponse {
        AgentLearningRecordResponse {
            id: item.id.to_string(),
            user_id: item.user_id.to_string(),
            workspace_id: item.workspace_id.map(|id| id.to_string()),
            run_id: item.run_id.map(|id| id.to_string()),
            quality_check_id: item.quality_check_id.map(|id| id.to_string()),
            role: item.role,
            category: item.category,
            content: item.content,
            confidence: item.confidence,
            source_action: item.source_action,
            feedback_rating: item.feedback_rating,
            feedback_comment: item.feedback_comment,
            status: item.status.as_str().to_string(),
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn run_task_report_item(item: &WorkspaceRunTask) -> Value {
        json!({
            "id": item.id,
            "runId": item.run_id,
            "workspaceTaskId": item.workspace_task_id,
            "seq": item.seq,
            "ownerRole": item.owner_role,
            "status": item.status.as_str(),
            "attempts": item.attempts,
            "acceptanceScore": item.acceptance_score,
            "acceptancePassed": item.acceptance_passed,
            "acceptanceNotes": item.acceptance_notes,
            "executionNotes": item.execution_notes,
            "metadata": item.metadata,
        })
    }
}

fn map_context_conversation_file(item: ConversationFile) -> WorkspaceContextSourceResponse {
    let id = item.id.to_string();
    WorkspaceContextSourceResponse {
        source_type: "conversation_file".to_string(),
        id: id.clone(),
        title: item
            .file_name
            .clone()
            .or_else(|| Some(item.object_key.clone())),
        mime_type: item.mime_type,
        file_size: item.file_size,
        status: item.status,
        ref_key: format!("conversation_file:{id}"),
        preview: item.file_name,
        metadata: json!({
            "conversationId": item.conversation_id,
            "messageId": item.message_id,
            "bucket": item.bucket,
            "objectKey": item.object_key,
            "sha256": item.sha256,
            "visibility": item.visibility,
            "storageProvider": item.storage_provider,
        }),
    }
}

fn map_context_product_asset(item: ProductAsset) -> WorkspaceContextSourceResponse {
    let id = item.id.to_string();
    WorkspaceContextSourceResponse {
        source_type: "product_asset".to_string(),
        id: id.clone(),
        title: item.file_name.clone().or_else(|| item.description.clone()),
        mime_type: item.mime_type,
        file_size: item.file_size,
        status: item.status.as_str().to_string(),
        ref_key: format!("product_asset:{id}"),
        preview: item.description,
        metadata: json!({
            "productId": item.product_id,
            "assetType": item.asset_type,
            "bucket": item.bucket,
            "objectKey": item.object_key,
            "metadata": item.metadata,
        }),
    }
}

fn map_context_product_material(item: ProductMaterial) -> WorkspaceContextSourceResponse {
    let id = item.id.to_string();
    WorkspaceContextSourceResponse {
        source_type: "product_material".to_string(),
        id: id.clone(),
        title: item.title.clone(),
        mime_type: Some("text/plain".to_string()),
        file_size: None,
        status: item.status.as_str().to_string(),
        ref_key: format!("product_material:{id}"),
        preview: Some(compact_text(&item.content, 240)),
        metadata: json!({
            "productId": item.product_id,
            "materialType": item.material_type,
            "version": item.version,
            "metadata": item.metadata,
        }),
    }
}

fn map_context_artifact(item: WorkspaceArtifact) -> WorkspaceContextSourceResponse {
    let id = item.id.to_string();
    WorkspaceContextSourceResponse {
        source_type: "workspace_artifact".to_string(),
        id: id.clone(),
        title: Some(item.title),
        mime_type: item.mime_type,
        file_size: item.file_size,
        status: item.status.as_str().to_string(),
        ref_key: format!("workspace_artifact:{id}"),
        preview: item.preview.or_else(|| {
            item.content
                .as_deref()
                .map(|content| compact_text(content, 240))
        }),
        metadata: json!({
            "workspaceId": item.workspace_id,
            "runId": item.run_id,
            "workspaceTaskId": item.workspace_task_id,
            "runTaskId": item.run_task_id,
            "artifactType": item.artifact_type,
            "format": item.format,
            "storageKind": item.storage_kind,
            "bucket": item.bucket,
            "objectKey": item.object_key,
            "fileName": item.file_name,
            "sha256": item.sha256,
            "source": item.source,
            "validationStatus": item.validation_status.as_str(),
            "validationReport": item.validation_report,
            "metadata": item.metadata,
        }),
    }
}

fn map_context_campaign(item: Campaign) -> WorkspaceContextSourceResponse {
    let id = item.id.to_string();
    WorkspaceContextSourceResponse {
        source_type: "campaign".to_string(),
        id: id.clone(),
        title: Some(item.name.clone()),
        mime_type: Some("application/json".to_string()),
        file_size: None,
        status: item.status.as_str().to_string(),
        ref_key: format!("campaign:{id}"),
        preview: campaign_preview(&item),
        metadata: json!({
            "productId": item.product_id,
            "campaignType": item.campaign_type,
            "budget": item.budget,
            "startDate": item.start_date,
            "endDate": item.end_date,
            "metadata": item.metadata,
            "actionMetadata": item.action_metadata,
        }),
    }
}

#[derive(Debug, Clone)]
struct WorkspaceContextPackCandidate {
    source_type: String,
    id: String,
    title: Option<String>,
    ref_key: String,
    content_kind: String,
    text: Option<String>,
    extraction_status: Option<String>,
    metadata: Value,
}

#[derive(Debug, Clone, Default)]
struct BuiltWorkspaceContextPack {
    items: Vec<WorkspaceContextPackItemResponse>,
    omitted: Vec<String>,
    readable_items: usize,
    text_chars: usize,
    truncated: bool,
}

fn context_pack_conversation_file_candidate(
    item: ConversationFile,
) -> WorkspaceContextPackCandidate {
    let id = item.id.to_string();
    let extraction_status = conversation_file_extraction_status(&item.metadata);
    let text = if extraction_status == "extracted" {
        conversation_file_extracted_text(&item.metadata)
    } else {
        None
    };
    let content_kind = match (extraction_status.as_str(), text.is_some()) {
        ("extracted", true) => "extracted_text".to_string(),
        ("extracted", false) => "metadata_only".to_string(),
        ("failed", _) => "extraction_failed".to_string(),
        ("unsupported", _) => "extraction_unsupported".to_string(),
        ("pending", _) => "extraction_pending".to_string(),
        _ => "metadata_only".to_string(),
    };
    WorkspaceContextPackCandidate {
        source_type: "conversation_file".to_string(),
        id: id.clone(),
        title: item
            .file_name
            .clone()
            .or_else(|| Some(item.object_key.clone())),
        ref_key: format!("conversation_file:{id}"),
        content_kind,
        text,
        extraction_status: Some(extraction_status),
        metadata: json!({
            "conversationId": item.conversation_id,
            "messageId": item.message_id,
            "bucket": item.bucket,
            "objectKey": item.object_key,
            "fileName": item.file_name,
            "mimeType": item.mime_type,
            "fileSize": item.file_size,
            "sha256": item.sha256,
            "visibility": item.visibility,
            "storageProvider": item.storage_provider,
            "status": item.status,
            "metadata": item.metadata,
        }),
    }
}

fn context_pack_product_asset_candidate(item: ProductAsset) -> WorkspaceContextPackCandidate {
    let id = item.id.to_string();
    let text = item.description.clone().and_then(normalize_string);
    WorkspaceContextPackCandidate {
        source_type: "product_asset".to_string(),
        id: id.clone(),
        title: item.file_name.clone().or_else(|| item.description.clone()),
        ref_key: format!("product_asset:{id}"),
        content_kind: if text.is_some() {
            "description".to_string()
        } else {
            "unavailable".to_string()
        },
        text,
        extraction_status: None,
        metadata: json!({
            "productId": item.product_id,
            "assetType": item.asset_type,
            "bucket": item.bucket,
            "objectKey": item.object_key,
            "fileName": item.file_name,
            "mimeType": item.mime_type,
            "fileSize": item.file_size,
            "metadata": item.metadata,
        }),
    }
}

fn context_pack_campaign_candidate(item: Campaign) -> WorkspaceContextPackCandidate {
    let id = item.id.to_string();
    WorkspaceContextPackCandidate {
        source_type: "campaign".to_string(),
        id: id.clone(),
        title: Some(item.name.clone()),
        ref_key: format!("campaign:{id}"),
        content_kind: "campaign_summary".to_string(),
        text: Some(campaign_context_text(&item)),
        extraction_status: None,
        metadata: json!({
            "productId": item.product_id,
            "campaignType": item.campaign_type,
            "budget": item.budget,
            "startDate": item.start_date,
            "endDate": item.end_date,
            "status": item.status.as_str(),
            "metadata": item.metadata,
            "output": item.output,
            "actionMetadata": item.action_metadata,
        }),
    }
}

fn context_pack_product_material_candidate(item: ProductMaterial) -> WorkspaceContextPackCandidate {
    let id = item.id.to_string();
    WorkspaceContextPackCandidate {
        source_type: "product_material".to_string(),
        id: id.clone(),
        title: item.title.clone(),
        ref_key: format!("product_material:{id}"),
        content_kind: "content".to_string(),
        text: normalize_string(item.content),
        extraction_status: None,
        metadata: json!({
            "productId": item.product_id,
            "materialType": item.material_type,
            "version": item.version,
            "metadata": item.metadata,
        }),
    }
}

fn context_pack_artifact_candidate(item: WorkspaceArtifact) -> WorkspaceContextPackCandidate {
    let id = item.id.to_string();
    let (content_kind, text) = context_pack_artifact_text(&item);
    WorkspaceContextPackCandidate {
        source_type: "workspace_artifact".to_string(),
        id: id.clone(),
        title: Some(item.title),
        ref_key: format!("workspace_artifact:{id}"),
        content_kind,
        text,
        extraction_status: None,
        metadata: json!({
            "workspaceId": item.workspace_id,
            "runId": item.run_id,
            "workspaceTaskId": item.workspace_task_id,
            "runTaskId": item.run_task_id,
            "artifactType": item.artifact_type,
            "format": item.format,
            "storageKind": item.storage_kind,
            "bucket": item.bucket,
            "objectKey": item.object_key,
            "fileName": item.file_name,
            "mimeType": item.mime_type,
            "fileSize": item.file_size,
            "sha256": item.sha256,
            "source": item.source,
            "validationStatus": item.validation_status.as_str(),
            "validationReport": item.validation_report,
            "metadata": item.metadata,
        }),
    }
}

fn context_pack_artifact_text(item: &WorkspaceArtifact) -> (String, Option<String>) {
    if item.artifact_type == "search_evidence"
        && metadata_str_eq(&item.metadata, "kind", "agent_search_evidence")
        && let Some(summary) = search_evidence_context_pack_text(item)
    {
        return ("search_evidence_summary".to_string(), Some(summary));
    }
    match item.content.clone().and_then(normalize_string) {
        Some(content) => ("content".to_string(), Some(content)),
        None => match item.preview.clone().and_then(normalize_string) {
            Some(preview) => ("preview".to_string(), Some(preview)),
            None => ("unavailable".to_string(), None),
        },
    }
}

fn campaign_preview(item: &Campaign) -> Option<String> {
    Some(compact_text(&campaign_context_text(item), 240))
}

fn campaign_context_text(item: &Campaign) -> String {
    let mut lines = vec![
        "Campaign context".to_string(),
        format!("Name: {}", item.name),
        format!("Status: {}", item.status.as_str()),
        format!("Budget: {}", item.budget),
    ];
    if let Some(campaign_type) = item.campaign_type.as_deref() {
        lines.push(format!("Type: {campaign_type}"));
    }
    if let Some(product_id) = item.product_id {
        lines.push(format!("Product ref: product:{product_id}"));
    }
    if let Some(start_date) = item.start_date.as_deref() {
        lines.push(format!("Start date: {start_date}"));
    }
    if let Some(end_date) = item.end_date.as_deref() {
        lines.push(format!("End date: {end_date}"));
    }
    if let Some(description) = item
        .description
        .as_deref()
        .and_then(normalize_borrowed_string)
    {
        lines.push(format!("Description: {}", compact_text(&description, 1200)));
    }
    if let Some(metadata_summary) = readable_json_context("Metadata", &item.metadata) {
        lines.push(metadata_summary);
    }
    if let Some(output_summary) = readable_json_context("Saved output", &item.output) {
        lines.push(output_summary);
    }
    if let Some(action_summary) = readable_json_context("Action metadata", &item.action_metadata) {
        lines.push(action_summary);
    }
    lines.join("\n")
}

fn readable_json_context(label: &str, value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Object(map) if map.is_empty() => None,
        Value::Array(items) if items.is_empty() => None,
        _ => serde_json::to_string(value)
            .ok()
            .and_then(normalize_string)
            .map(|text| format!("{label}: {}", compact_text(&text, 2400))),
    }
}

fn normalize_borrowed_string(value: &str) -> Option<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

fn search_evidence_context_pack_text(item: &WorkspaceArtifact) -> Option<String> {
    let output = item
        .content
        .as_deref()
        .and_then(|content| serde_json::from_str::<Value>(content).ok());
    let output = output.as_ref();
    let mut lines = Vec::new();
    lines.push("Search evidence summary".to_string());
    if let Some(query) = search_evidence_context_query(output, &item.metadata) {
        lines.push(format!("Query: {}", compact_text(&query, 220)));
    }
    if let Some(product_line) = search_evidence_context_product_line(output, &item.metadata) {
        lines.push(product_line);
    }
    if let Some(summary) = search_evidence_context_text_field(output, "/evidence/summary")
        .or_else(|| search_evidence_context_text_field(output, "/evidence/llm_context"))
    {
        lines.push(format!("Evidence: {}", compact_text(&summary, 700)));
    }
    let confidence = search_evidence_context_f64(output, "/confidence")
        .or_else(|| item.metadata.get("confidence").and_then(Value::as_f64));
    let source_count = search_evidence_context_u64(output, "/provenance/source_count")
        .or_else(|| item.metadata.get("sourceCount").and_then(Value::as_u64));
    let domains = search_evidence_context_string_array(output, "/provenance/domains")
        .or_else(|| {
            item.metadata.get("domains").and_then(|value| {
                value.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                })
            })
        })
        .unwrap_or_default();
    let mut metrics = Vec::new();
    if let Some(source_count) = source_count {
        metrics.push(format!("sources={source_count}"));
    }
    if let Some(confidence) = confidence {
        metrics.push(format!("confidence={confidence:.2}"));
    }
    if !domains.is_empty() {
        metrics.push(format!("domains={}", domains.join(", ")));
    }
    if !metrics.is_empty() {
        lines.push(format!("Metrics: {}", metrics.join("; ")));
    }
    let sources = search_evidence_context_sources(output);
    for (index, source) in sources.iter().take(5).enumerate() {
        let title = source
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled source");
        let domain = source
            .get("domain")
            .and_then(Value::as_str)
            .or_else(|| source.get("url").and_then(Value::as_str))
            .unwrap_or("-");
        let url = source.get("url").and_then(Value::as_str).unwrap_or("-");
        let snippet = source
            .get("snippet")
            .and_then(Value::as_str)
            .map(|value| format!(" - {}", compact_text(value, 260)))
            .unwrap_or_default();
        lines.push(format!(
            "Source {}: {} ({}) {}{}",
            index + 1,
            compact_text(title, 180),
            compact_text(domain, 120),
            compact_text(url, 240),
            snippet
        ));
    }
    if lines.len() > 1 {
        return Some(lines.join("\n"));
    }
    item.preview.clone().and_then(normalize_string)
}

fn search_evidence_context_query(output: Option<&Value>, metadata: &Value) -> Option<String> {
    search_evidence_context_text_field(output, "/query/planned")
        .or_else(|| search_evidence_context_text_field(output, "/query/query"))
        .or_else(|| search_evidence_context_text_field(output, "/query"))
        .or_else(|| {
            metadata
                .get("query")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

fn search_evidence_context_product_line(
    output: Option<&Value>,
    metadata: &Value,
) -> Option<String> {
    let product_ref = search_evidence_context_text_field(output, "/query/product_ref")
        .or_else(|| {
            metadata
                .pointer("/productContext/productRef")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| {
            metadata
                .get("productRef")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let product_name = search_evidence_context_text_field(output, "/query/product_name")
        .or_else(|| {
            metadata
                .pointer("/productContext/productName")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| {
            metadata
                .get("productName")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let product_category = search_evidence_context_text_field(output, "/query/product_category")
        .or_else(|| {
            metadata
                .pointer("/productContext/productCategory")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .or_else(|| {
            metadata
                .get("productCategory")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let parts = [product_name, product_category, product_ref]
        .into_iter()
        .flatten()
        .filter_map(normalize_string)
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| format!("Product: {}", compact_text(&parts.join(" / "), 220)))
}

fn search_evidence_context_text_field(output: Option<&Value>, pointer: &str) -> Option<String> {
    output
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .and_then(normalize_string)
}

fn search_evidence_context_f64(output: Option<&Value>, pointer: &str) -> Option<f64> {
    output
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_f64)
}

fn search_evidence_context_u64(output: Option<&Value>, pointer: &str) -> Option<u64> {
    output
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_u64)
}

fn search_evidence_context_string_array(
    output: Option<&Value>,
    pointer: &str,
) -> Option<Vec<String>> {
    output
        .and_then(|value| value.pointer(pointer))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter_map(|value| normalize_string(value.to_string()))
                .collect::<Vec<_>>()
        })
}

fn search_evidence_context_sources(output: Option<&Value>) -> Vec<Value> {
    output
        .and_then(|value| {
            value
                .get("sources")
                .or_else(|| value.pointer("/data/items"))
                .and_then(Value::as_array)
        })
        .map(|items| items.iter().take(5).cloned().collect::<Vec<_>>())
        .unwrap_or_default()
}

fn conversation_file_extracted_text(metadata: &Value) -> Option<String> {
    metadata_text_field(
        metadata,
        &[
            "extractedText",
            "extracted_text",
            "textContent",
            "text_content",
            "plainText",
            "plain_text",
            "markdown",
            "text",
            "content",
            "preview",
            "summary",
        ],
    )
    .or_else(|| metadata_nested_text_field(metadata, "extraction"))
    .or_else(|| metadata_nested_text_field(metadata, "parsed"))
}

fn conversation_file_extraction_status(metadata: &Value) -> String {
    metadata
        .get("extractionStatus")
        .and_then(Value::as_str)
        .or_else(|| {
            metadata
                .get("extraction")
                .and_then(|value| value.get("status"))
                .and_then(Value::as_str)
        })
        .map(normalize_conversation_file_extraction_status)
        .unwrap_or_else(|| {
            if conversation_file_extracted_text(metadata).is_some() {
                "extracted".to_string()
            } else {
                "pending".to_string()
            }
        })
}

fn normalize_conversation_file_extraction_status(value: &str) -> String {
    match value.trim() {
        "ready" | "success" | "extracted" => "extracted".to_string(),
        "failed" | "error" => "failed".to_string(),
        "unsupported" => "unsupported".to_string(),
        "pending" | "queued" | "processing" => "pending".to_string(),
        _ => "pending".to_string(),
    }
}

fn metadata_nested_text_field(metadata: &Value, key: &str) -> Option<String> {
    metadata.get(key).and_then(|value| {
        metadata_text_field(
            value,
            &[
                "extractedText",
                "extracted_text",
                "textContent",
                "text_content",
                "plainText",
                "plain_text",
                "markdown",
                "text",
                "content",
                "preview",
                "summary",
            ],
        )
    })
}

fn metadata_text_field(metadata: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        metadata
            .get(*key)
            .and_then(Value::as_str)
            .and_then(|value| normalize_string(value.to_string()))
    })
}

fn build_artifact_export(
    artifact: WorkspaceArtifact,
    requested_format: Option<String>,
    include_metadata: bool,
) -> BizResult<WorkspaceArtifactExportResponse> {
    let source_format = normalize_export_label(&artifact.format)
        .unwrap_or_else(|| DEFAULT_ARTIFACT_EXPORT_FORMAT.to_string());
    let format = resolve_artifact_export_format(requested_format, &source_format)?;
    let mut warnings = Vec::new();
    let source_text = artifact
        .content
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            artifact
                .preview
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| {
                    warnings
                        .push("artifact content is empty; exported preview instead".to_string());
                    value.to_string()
                })
        })
        .unwrap_or_else(|| {
            warnings.push("artifact has no inline content or preview".to_string());
            String::new()
        });
    if artifact.storage_kind.trim() != "inline" {
        warnings.push(format!(
            "artifact storage kind is {}; object storage content was not read",
            artifact.storage_kind
        ));
    }

    let metadata = build_artifact_export_metadata(&artifact, include_metadata);
    let content = match format.as_str() {
        "markdown" => build_artifact_markdown_export(&artifact, &source_text, &metadata),
        "text" => build_artifact_text_export(&artifact, &source_text, &metadata),
        "json" => build_artifact_json_export(&artifact, &source_text, &metadata)?,
        _ => unreachable!("export format should be validated before rendering"),
    };
    let file_name = build_artifact_export_file_name(&artifact, &format);
    let mime_type = artifact_export_mime_type(&format).to_string();
    let content_chars = content.chars().count();

    Ok(WorkspaceArtifactExportResponse {
        artifact_id: artifact.id.to_string(),
        workspace_id: artifact.workspace_id.to_string(),
        title: artifact.title,
        file_name,
        mime_type,
        format,
        content,
        content_chars,
        source_format,
        source_storage_kind: artifact.storage_kind,
        metadata,
        warnings,
    })
}

fn resolve_artifact_export_format(
    requested_format: Option<String>,
    source_format: &str,
) -> BizResult<String> {
    let requested_format = requested_format
        .and_then(normalize_string)
        .or_else(|| normalize_export_label(source_format))
        .unwrap_or_else(|| DEFAULT_ARTIFACT_EXPORT_FORMAT.to_string());
    let format = match requested_format.to_ascii_lowercase().as_str() {
        "md" | "markdown" => "markdown",
        "txt" | "text" | "plain" | "plaintext" => "text",
        "json" => "json",
        _ => {
            return Err(BizError::new(
                app_error::WORKSPACE_ARTIFACT_REQUEST_INVALID,
                format!("unsupported artifact export format: {requested_format}"),
            ));
        }
    };
    Ok(format.to_string())
}

fn normalize_export_label(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    (!value.is_empty()).then_some(value)
}

fn build_artifact_export_metadata(artifact: &WorkspaceArtifact, include_metadata: bool) -> Value {
    if !include_metadata {
        return json!({});
    }
    json!({
        "artifactId": artifact.id,
        "workspaceId": artifact.workspace_id,
        "runId": artifact.run_id,
        "workspaceTaskId": artifact.workspace_task_id,
        "runTaskId": artifact.run_task_id,
        "artifactType": artifact.artifact_type,
        "source": artifact.source,
        "sourceFormat": artifact.format,
        "sourceStorageKind": artifact.storage_kind,
        "validationStatus": artifact.validation_status.as_str(),
        "validationReport": artifact.validation_report,
        "metadata": artifact.metadata,
        "createdAt": artifact.created_at,
        "updatedAt": artifact.updated_at,
    })
}

fn build_artifact_markdown_export(
    artifact: &WorkspaceArtifact,
    source_text: &str,
    metadata: &Value,
) -> String {
    let mut sections = vec![format!("# {}", artifact.title.trim())];
    if !metadata
        .as_object()
        .map(|item| item.is_empty())
        .unwrap_or(true)
    {
        sections.push(format!(
            "```json\n{}\n```",
            serde_json::to_string_pretty(metadata).unwrap_or_else(|_| "{}".to_string())
        ));
    }
    if !source_text.trim().is_empty() {
        sections.push(source_text.trim().to_string());
    }
    sections.join("\n\n")
}

fn build_artifact_text_export(
    artifact: &WorkspaceArtifact,
    source_text: &str,
    metadata: &Value,
) -> String {
    let mut sections = vec![artifact.title.trim().to_string()];
    if !metadata
        .as_object()
        .map(|item| item.is_empty())
        .unwrap_or(true)
    {
        sections.push(serde_json::to_string_pretty(metadata).unwrap_or_else(|_| "{}".to_string()));
    }
    if !source_text.trim().is_empty() {
        sections.push(source_text.trim().to_string());
    }
    sections.join("\n\n")
}

fn build_artifact_json_export(
    artifact: &WorkspaceArtifact,
    source_text: &str,
    metadata: &Value,
) -> BizResult<String> {
    serde_json::to_string_pretty(&json!({
        "artifactId": artifact.id,
        "workspaceId": artifact.workspace_id,
        "title": artifact.title,
        "artifactType": artifact.artifact_type,
        "sourceFormat": artifact.format,
        "sourceStorageKind": artifact.storage_kind,
        "content": source_text,
        "metadata": metadata,
    }))
    .map_err(|err| {
        BizError::new(
            app_error::WORKSPACE_ARTIFACT_REQUEST_INVALID,
            format!("failed to render artifact json export: {err}"),
        )
    })
}

fn build_artifact_export_file_name(artifact: &WorkspaceArtifact, format: &str) -> String {
    let extension = artifact_export_extension(format);
    if let Some(file_name) = artifact
        .file_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return replace_file_extension(file_name, extension);
    }
    let stem = artifact
        .title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let stem = if stem.is_empty() {
        artifact.id.to_string()
    } else {
        compact_text(&stem, 80)
    };
    format!("{stem}.{extension}")
}

fn replace_file_extension(file_name: &str, extension: &str) -> String {
    let trimmed = file_name.trim();
    let Some((stem, _)) = trimmed.rsplit_once('.') else {
        return format!("{trimmed}.{extension}");
    };
    if stem.trim().is_empty() {
        return format!("{trimmed}.{extension}");
    }
    format!("{stem}.{extension}")
}

fn artifact_export_extension(format: &str) -> &'static str {
    match format {
        "json" => "json",
        "text" => "txt",
        _ => "md",
    }
}

fn artifact_export_mime_type(format: &str) -> &'static str {
    match format {
        "json" => "application/json; charset=utf-8",
        "text" => "text/plain; charset=utf-8",
        _ => "text/markdown; charset=utf-8",
    }
}

fn build_context_pack_items(
    candidates: Vec<WorkspaceContextPackCandidate>,
    max_items: usize,
    max_chars: usize,
) -> BuiltWorkspaceContextPack {
    let mut pack = BuiltWorkspaceContextPack::default();
    let mut ordered = candidates
        .iter()
        .filter(|candidate| candidate.text.is_some())
        .chain(
            candidates
                .iter()
                .filter(|candidate| candidate.text.is_none()),
        )
        .collect::<Vec<_>>();
    ordered.dedup_by(|left, right| left.ref_key == right.ref_key);

    for candidate in ordered {
        if pack.items.len() >= max_items {
            pack.omitted.push(candidate.ref_key.clone());
            continue;
        }
        if pack.text_chars >= max_chars && candidate.text.is_some() {
            pack.omitted.push(candidate.ref_key.clone());
            pack.truncated = true;
            continue;
        }
        let (text, text_chars, truncated) = candidate
            .text
            .as_deref()
            .map(|text| truncate_context_pack_text(text, max_chars - pack.text_chars))
            .unwrap_or((None, 0, false));
        if truncated {
            pack.truncated = true;
        }
        if text.is_some() {
            pack.readable_items += 1;
            pack.text_chars += text_chars;
        }
        pack.items.push(WorkspaceContextPackItemResponse {
            source_type: candidate.source_type.clone(),
            id: candidate.id.clone(),
            title: candidate.title.clone(),
            ref_key: candidate.ref_key.clone(),
            content_kind: candidate.content_kind.clone(),
            text,
            text_chars,
            truncated,
            metadata: match &candidate.extraction_status {
                Some(status) => merge_json_object(
                    candidate.metadata.clone(),
                    json!({ "extractionStatus": status }),
                ),
                None => candidate.metadata.clone(),
            },
        });
    }
    pack
}

fn conversation_file_context_warnings(items: &[WorkspaceContextPackItemResponse]) -> Vec<String> {
    let mut metadata_only = 0;
    let mut pending = 0;
    let mut failed = 0;
    let mut unsupported = 0;
    for item in items
        .iter()
        .filter(|item| item.source_type == "conversation_file")
    {
        match item.content_kind.as_str() {
            "metadata_only" => metadata_only += 1,
            "extraction_pending" => pending += 1,
            "extraction_failed" => failed += 1,
            "extraction_unsupported" => unsupported += 1,
            _ => {}
        }
    }

    let mut warnings = Vec::new();
    if metadata_only > 0 {
        warnings.push(format!(
            "{metadata_only} conversation file(s) only have metadata; no extracted text was \
             registered"
        ));
    }
    if pending > 0 {
        warnings.push(format!(
            "{pending} conversation file(s) are still pending text extraction"
        ));
    }
    if failed > 0 {
        warnings.push(format!(
            "{failed} conversation file(s) failed text extraction; see extraction.failureReason"
        ));
    }
    if unsupported > 0 {
        warnings.push(format!(
            "{unsupported} conversation file(s) are not supported by the current extraction \
             adapter"
        ));
    }
    warnings
}

fn truncate_context_pack_text(
    value: &str,
    remaining_chars: usize,
) -> (Option<String>, usize, bool) {
    let item_limit = remaining_chars.min(DEFAULT_CONTEXT_PACK_ITEM_CHARS);
    if item_limit == 0 {
        return (None, 0, true);
    }
    let value = value.trim();
    let value_chars = value.chars().count();
    if value_chars <= item_limit {
        return (Some(value.to_string()), value_chars, false);
    }
    let truncated = value.chars().take(item_limit).collect::<String>();
    (Some(format!("{truncated}...")), item_limit, true)
}

fn build_context_summary_text(counts: &WorkspaceContextSummaryCounts, readiness: &str) -> String {
    if counts.total() == 0 {
        return "No active workspace context sources are ready for agent use.".to_string();
    }
    format!(
        "Workspace context is {readiness}: {} conversation files, {} campaigns, {} product \
         assets, {} product materials, {} workspace artifacts are available for agent grounding \
         and delivery.",
        counts.conversation_files,
        counts.campaigns,
        counts.product_assets,
        counts.product_materials,
        counts.workspace_artifacts
    )
}

fn build_context_pack_summary_text(
    counts: &WorkspaceContextSummaryCounts,
    readiness: &str,
    readable_items: usize,
    text_chars: usize,
    omitted_items: usize,
) -> String {
    if counts.total() == 0 {
        return "No active workspace context text is ready for agent use.".to_string();
    }
    format!(
        "Workspace context pack is {readiness}: {readable_items} readable item(s), {text_chars} \
         text chars, {omitted_items} omitted item(s), from {} conversation files, {} campaigns, \
         {} product assets, {} product materials, {} workspace artifacts.",
        counts.conversation_files,
        counts.campaigns,
        counts.product_assets,
        counts.product_materials,
        counts.workspace_artifacts
    )
}

fn context_ref_keys(
    attachments: &[WorkspaceContextSourceResponse],
    campaigns: &[WorkspaceContextSourceResponse],
    assets: &[WorkspaceContextSourceResponse],
    materials: &[WorkspaceContextSourceResponse],
    artifacts: &[WorkspaceContextSourceResponse],
) -> Vec<String> {
    attachments
        .iter()
        .chain(campaigns.iter())
        .chain(assets.iter())
        .chain(materials.iter())
        .chain(artifacts.iter())
        .map(|source| source.ref_key.clone())
        .collect()
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::WORKSPACE_REQUEST_INVALID,
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

fn normalize_context_refs(context_refs: Vec<String>) -> BizResult<Vec<String>> {
    if context_refs.len() > MAX_CONTEXT_REFS {
        return Err(BizError::new(
            app_error::WORKSPACE_REQUEST_INVALID,
            format!("context_refs cannot exceed {MAX_CONTEXT_REFS} items"),
        ));
    }

    let mut refs = Vec::new();
    for item in context_refs {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if item.chars().count() > MAX_CONTEXT_REF_CHARS {
            return Err(BizError::new(
                app_error::WORKSPACE_REQUEST_INVALID,
                format!("context ref must be at most {MAX_CONTEXT_REF_CHARS} characters"),
            ));
        }
        if !refs.iter().any(|existing| existing == item) {
            refs.push(item.to_string());
        }
    }
    Ok(refs)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExplicitWorkspaceContextRefs {
    campaign_ids: Vec<Uuid>,
    conversation_file_ids: Vec<Uuid>,
    product_asset_ids: Vec<Uuid>,
    product_material_ids: Vec<Uuid>,
    workspace_artifact_ids: Vec<Uuid>,
}

fn explicit_workspace_context_refs(
    context_refs: Vec<String>,
) -> BizResult<ExplicitWorkspaceContextRefs> {
    let mut refs = ExplicitWorkspaceContextRefs::default();
    for context_ref in normalize_context_refs(context_refs)? {
        if let Some(file_id) = conversation_file_id_from_context_ref(&context_ref) {
            let file_id = parse_uuid(file_id, "context_ref conversation file id")?;
            push_unique_uuid(&mut refs.conversation_file_ids, file_id);
            continue;
        }
        if let Some(asset_id) = product_asset_id_from_context_ref(&context_ref) {
            let asset_id = parse_uuid(asset_id, "context_ref product asset id")?;
            push_unique_uuid(&mut refs.product_asset_ids, asset_id);
            continue;
        }
        if let Some(material_id) = product_material_id_from_context_ref(&context_ref) {
            let material_id = parse_uuid(material_id, "context_ref product material id")?;
            push_unique_uuid(&mut refs.product_material_ids, material_id);
            continue;
        }
        if let Some(artifact_id) = artifact_id_from_context_ref(&context_ref) {
            let artifact_id = parse_uuid(artifact_id, "context_ref artifact id")?;
            push_unique_uuid(&mut refs.workspace_artifact_ids, artifact_id);
            continue;
        }
        if let Some(campaign_id) = campaign_id_from_context_ref(&context_ref) {
            let campaign_id = parse_uuid(campaign_id, "context_ref campaign id")?;
            push_unique_uuid(&mut refs.campaign_ids, campaign_id);
        }
    }
    Ok(refs)
}

fn conversation_file_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("conversation_file:")
        .or_else(|| value.strip_prefix("conversation_file/"))
        .or_else(|| value.strip_prefix("file:"))
        .or_else(|| value.strip_prefix("file/"))
}

fn product_asset_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("product_asset:")
        .or_else(|| value.strip_prefix("product_asset/"))
        .or_else(|| value.strip_prefix("asset:"))
        .or_else(|| value.strip_prefix("asset/"))
}

fn product_material_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("product_material:")
        .or_else(|| value.strip_prefix("product_material/"))
        .or_else(|| value.strip_prefix("material:"))
        .or_else(|| value.strip_prefix("material/"))
}

fn artifact_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("workspace_artifact:")
        .or_else(|| value.strip_prefix("workspace_artifact/"))
        .or_else(|| value.strip_prefix("artifact:"))
        .or_else(|| value.strip_prefix("artifact/"))
}

fn campaign_id_from_context_ref(value: &str) -> Option<&str> {
    value
        .strip_prefix("campaign:")
        .or_else(|| value.strip_prefix("campaign/"))
}

fn push_unique_uuid(items: &mut Vec<Uuid>, id: Uuid) {
    if !items.contains(&id) {
        items.push(id);
    }
}

fn push_unique_conversation_file(items: &mut Vec<ConversationFile>, file: ConversationFile) {
    if !items.iter().any(|item| item.id == file.id) {
        items.push(file);
    }
}

fn push_unique_product_asset(items: &mut Vec<ProductAsset>, asset: ProductAsset) {
    if !items.iter().any(|item| item.id == asset.id) {
        items.push(asset);
    }
}

fn push_unique_product_material(items: &mut Vec<ProductMaterial>, material: ProductMaterial) {
    if !items.iter().any(|item| item.id == material.id) {
        items.push(material);
    }
}

fn push_unique_artifact(items: &mut Vec<WorkspaceArtifact>, artifact: WorkspaceArtifact) {
    if !items.iter().any(|item| item.id == artifact.id) {
        items.push(artifact);
    }
}

fn push_unique_campaign(items: &mut Vec<Campaign>, campaign: Campaign) {
    if !items.iter().any(|item| item.id == campaign.id) {
        items.push(campaign);
    }
}

#[derive(Debug, Clone)]
struct SearchEvidenceArtifactDraft {
    title: String,
    preview: String,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct SearchEvidenceProductContext {
    product_ref: Option<String>,
    product_name: Option<String>,
    product_category: Option<String>,
}

#[derive(Debug, Clone)]
struct WorkspaceTaskFromSkillDraft {
    title: String,
    description: Option<String>,
    owner_role: Option<String>,
    priority: i32,
    acceptance_criteria: Option<String>,
}

fn build_search_evidence_artifact(
    req: &RecordAgentSearchEvidenceRequest,
) -> Option<SearchEvidenceArtifactDraft> {
    if !req.skill_id.starts_with("search_")
        || req.status != "success"
        || req.execution_mode.as_deref() != Some("adapter")
        || req.output.get("status").and_then(Value::as_str) != Some("live")
    {
        return None;
    }
    let sources = req.output.get("sources")?.as_array()?;
    if sources.is_empty() {
        return None;
    }

    let source_count = sources.len();
    let query = req
        .output
        .pointer("/query/planned")
        .and_then(Value::as_str)
        .or_else(|| req.output.get("planned_query").and_then(Value::as_str))
        .or_else(|| req.output.pointer("/query/raw").and_then(Value::as_str))
        .unwrap_or_default()
        .trim()
        .to_string();
    let engine = req
        .output
        .get("engine")
        .and_then(Value::as_str)
        .or_else(|| {
            req.output
                .pointer("/adapter/provider")
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("external_search")
        .to_string();
    let confidence = req.output.get("confidence").and_then(Value::as_f64);
    let domains = req
        .output
        .pointer("/provenance/domains")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let title_query = if query.is_empty() {
        req.skill_id.clone()
    } else {
        compact_text(&query, 80)
    };
    let product_context = search_evidence_product_context(&req.output);
    let title = format!("Search evidence - {title_query}");
    let preview = format_search_evidence_preview(sources, &query, product_context.as_ref());
    let mut metadata = json!({
        "kind": "agent_search_evidence",
        "workerId": req.worker_id,
        "taskId": req.task_id,
        "requestId": req.request_id,
        "skillId": req.skill_id,
        "role": req.role,
        "status": req.status,
        "summary": req.summary,
        "executionMode": req.execution_mode,
        "adapterStatus": req.output.get("adapter_status").cloned(),
        "engine": engine,
        "query": query,
        "sourceCount": source_count,
        "domains": domains,
        "confidence": confidence,
        "provenance": req.output.get("provenance").cloned(),
        "freshness": req.output.get("freshness").cloned(),
        "evidence": req.output.get("evidence").cloned(),
    });
    if let (Some(context), Some(metadata)) = (product_context, metadata.as_object_mut()) {
        metadata.insert("productContext".to_string(), context.to_json());
        if let Some(product_ref) = context.product_ref {
            metadata.insert("productRef".to_string(), json!(product_ref));
        }
        if let Some(product_name) = context.product_name {
            metadata.insert("productName".to_string(), json!(product_name));
        }
        if let Some(product_category) = context.product_category {
            metadata.insert("productCategory".to_string(), json!(product_category));
        }
    }

    Some(SearchEvidenceArtifactDraft {
        title,
        preview,
        metadata,
    })
}

fn build_workspace_task_from_skill_output(output: &Value) -> Option<WorkspaceTaskFromSkillDraft> {
    if output.get("status").and_then(Value::as_str) != Some("success") {
        return None;
    }
    let task_creation = output.get("task_creation")?.as_object()?;
    if task_creation.get("should_persist").and_then(Value::as_bool) != Some(true) {
        return None;
    }

    let title = task_creation
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 255))?;
    let description = task_creation
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000));
    let owner_role = task_creation
        .get("owner_role")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 64));
    let priority = task_creation
        .get("priority")
        .and_then(Value::as_i64)
        .map(|value| clamp_priority(value as i32))
        .unwrap_or(3);
    let acceptance_criteria = task_creation
        .get("acceptance_criteria")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000));

    Some(WorkspaceTaskFromSkillDraft {
        title,
        description,
        owner_role,
        priority,
        acceptance_criteria,
    })
}

fn build_workspace_memory_from_skill_output(
    output: &Value,
) -> Option<WorkspaceMemoryFromSkillDraft> {
    if output.get("status").and_then(Value::as_str) != Some("success") {
        return None;
    }
    let memory_save = output.get("memory_save")?.as_object()?;
    if memory_save.get("should_persist").and_then(Value::as_bool) != Some(true) {
        return None;
    }

    let key = memory_save
        .get("key")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 120))?;
    let content = memory_save
        .get("content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000))?;
    let memory_type = memory_save
        .get("memory_type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_workspace_memory_type)
        .unwrap_or_else(|| "fact".to_string());
    let confidence = memory_confidence_for_type(&memory_type);

    Some(WorkspaceMemoryFromSkillDraft {
        key,
        content,
        memory_type,
        confidence,
    })
}

fn build_content_artifact_from_skill_output(
    skill_id: &str,
    output: &Value,
) -> Option<ContentArtifactFromSkillDraft> {
    if output.get("status").and_then(Value::as_str) != Some("success") {
        return None;
    }

    let (content_category, artifact_type, title, summary) = match skill_id {
        "creative_video_script" => (
            "creative",
            "video_script",
            "Video script",
            "creative video script artifact generated",
        ),
        "creative_seeding_copy" => (
            "creative",
            "seeding_copy",
            "Seeding copy",
            "creative seeding copy artifact generated",
        ),
        "creative_ip_branding" => (
            "creative",
            "ip_branding",
            "IP branding brief",
            "creative IP branding artifact generated",
        ),
        "creative_content_calendar" => (
            "creative",
            "content_calendar",
            "Content calendar",
            "creative content calendar artifact generated",
        ),
        "creative_live_script" => (
            "creative",
            "live_script",
            "Live script",
            "creative live script artifact generated",
        ),
        "creative_product_article" => (
            "creative",
            "product_article",
            "Product article",
            "creative product article artifact generated",
        ),
        "creative_trend_catch" => (
            "creative",
            "creative_trend_report",
            "Creative trend report",
            "creative trend report artifact generated",
        ),
        "creative_title_ctr_scorer" => (
            "creative",
            "creative_title_ctr_score",
            "Creative title CTR score",
            "creative title CTR score artifact generated",
        ),
        "design_main_image" => (
            "design",
            "design_main_image_brief",
            "Main image design brief",
            "design main image artifact generated",
        ),
        "design_detail_page" => (
            "design",
            "design_detail_page_brief",
            "Detail page design brief",
            "design detail page artifact generated",
        ),
        "design_color_scheme" => (
            "design",
            "design_color_scheme",
            "Design color scheme",
            "design color scheme artifact generated",
        ),
        "design_material_spec" => (
            "design",
            "design_material_spec",
            "Design material spec",
            "design material spec artifact generated",
        ),
        "design_campaign_poster" => (
            "design",
            "design_campaign_poster_brief",
            "Campaign poster design brief",
            "design campaign poster artifact generated",
        ),
        "data_dashboard" => (
            "data",
            "data_dashboard",
            "Data dashboard",
            "data dashboard artifact generated",
        ),
        "data_funnel_analysis" => (
            "data",
            "data_funnel_report",
            "Data funnel report",
            "data funnel analysis artifact generated",
        ),
        "data_channel_roi" => (
            "data",
            "data_channel_roi_report",
            "Data channel ROI report",
            "data channel ROI artifact generated",
        ),
        "data_refund_decomposition" => (
            "data",
            "data_refund_report",
            "Data refund decomposition report",
            "data refund decomposition artifact generated",
        ),
        "data_anomaly_diagnosis" => (
            "data",
            "data_anomaly_report",
            "Data anomaly report",
            "data anomaly diagnosis artifact generated",
        ),
        "data_trend_forecast" => (
            "data",
            "data_trend_forecast_report",
            "Data trend forecast report",
            "data trend forecast artifact generated",
        ),
        "data_customer_segmentation" => (
            "data",
            "data_customer_segmentation_report",
            "Data customer segmentation report",
            "data customer segmentation artifact generated",
        ),
        "data_competitor_analysis" => (
            "data",
            "data_competitor_report",
            "Data competitor report",
            "data competitor analysis artifact generated",
        ),
        "data_ab_test_analyzer" => (
            "data",
            "data_ab_test_report",
            "Data A/B test report",
            "data A/B test artifact generated",
        ),
        "data_attribution_analysis" => (
            "data",
            "data_attribution_report",
            "Data attribution report",
            "data attribution analysis artifact generated",
        ),
        "data_multi_period_trend" => (
            "data",
            "data_multi_period_trend_report",
            "Data multi-period trend report",
            "data multi-period trend artifact generated",
        ),
        "data_ltv_calculator" => (
            "data",
            "data_ltv_report",
            "Data LTV report",
            "data LTV calculation artifact generated",
        ),
        "data_cohort_analysis" => (
            "data",
            "data_cohort_report",
            "Data cohort report",
            "data cohort analysis artifact generated",
        ),
        "data_seasonal_decompose" => (
            "data",
            "data_seasonal_decompose_report",
            "Data seasonal decomposition report",
            "data seasonal decomposition artifact generated",
        ),
        "data_price_elasticity" => (
            "data",
            "data_price_elasticity_report",
            "Data price elasticity report",
            "data price elasticity artifact generated",
        ),
        "data_demand_forecast" => (
            "data",
            "data_demand_forecast_report",
            "Data demand forecast report",
            "data demand forecast artifact generated",
        ),
        "data_comprehensive_diagnosis" => (
            "data",
            "data_comprehensive_diagnosis_report",
            "Data comprehensive diagnosis report",
            "data comprehensive diagnosis artifact generated",
        ),
        "query_store_metrics" => (
            "data",
            "data_store_metrics_report",
            "Data store metrics report",
            "data store metrics artifact generated",
        ),
        "query_product_metrics" => (
            "data",
            "data_product_metrics_report",
            "Data product metrics report",
            "data product metrics artifact generated",
        ),
        "query_campaign_metrics" => (
            "data",
            "data_campaign_metrics_report",
            "Data campaign metrics report",
            "data campaign metrics artifact generated",
        ),
        "accounting_cost_calc" => (
            "accounting",
            "accounting_cost_report",
            "Accounting cost report",
            "accounting cost calculation artifact generated",
        ),
        "accounting_profit_analysis" => (
            "accounting",
            "accounting_profit_report",
            "Accounting profit report",
            "accounting profit analysis artifact generated",
        ),
        "accounting_roi_calc" => (
            "accounting",
            "accounting_roi_report",
            "Accounting ROI report",
            "accounting ROI calculation artifact generated",
        ),
        "accounting_budget_plan" => (
            "accounting",
            "accounting_budget_plan",
            "Accounting budget plan",
            "accounting budget plan artifact generated",
        ),
        "accounting_break_even_calc" => (
            "accounting",
            "accounting_break_even_report",
            "Accounting break-even report",
            "accounting break-even artifact generated",
        ),
        "accounting_compliance_check" => (
            "accounting",
            "accounting_compliance_report",
            "Accounting compliance report",
            "accounting compliance artifact generated",
        ),
        "accounting_pl_statement" => (
            "accounting",
            "accounting_pl_statement",
            "Accounting P&L statement",
            "accounting P&L statement artifact generated",
        ),
        "accounting_budget_vs_actual" => (
            "accounting",
            "accounting_budget_vs_actual_report",
            "Accounting budget vs actual report",
            "accounting budget vs actual artifact generated",
        ),
        "accounting_gmv_waterfall" => (
            "accounting",
            "accounting_gmv_waterfall_report",
            "Accounting GMV waterfall report",
            "accounting GMV waterfall artifact generated",
        ),
        "accounting_scenario_analysis" => (
            "accounting",
            "accounting_scenario_report",
            "Accounting scenario report",
            "accounting scenario analysis artifact generated",
        ),
        "accounting_cash_flow_forecast" => (
            "accounting",
            "accounting_cash_flow_forecast_report",
            "Accounting cash flow forecast",
            "accounting cash flow forecast artifact generated",
        ),
        "accounting_financial_narrative" => (
            "accounting",
            "accounting_financial_narrative_report",
            "Accounting financial narrative",
            "accounting financial narrative artifact generated",
        ),
        "service_ticket_handler" => (
            "service",
            "service_ticket_playbook",
            "Service ticket playbook",
            "service ticket handling artifact generated",
        ),
        "service_faq_playbook" => (
            "service",
            "service_faq_playbook",
            "Service FAQ playbook",
            "service FAQ playbook artifact generated",
        ),
        "service_escalation_flow" => (
            "service",
            "service_escalation_playbook",
            "Service escalation playbook",
            "service escalation flow artifact generated",
        ),
        "service_dsr_improvement" => (
            "service",
            "service_dsr_improvement_plan",
            "Service DSR improvement plan",
            "service DSR improvement artifact generated",
        ),
        "service_return_handler" => (
            "service",
            "service_return_plan",
            "Service return plan",
            "service return handling artifact generated",
        ),
        "service_nps_analyzer" => (
            "service",
            "service_nps_report",
            "Service NPS report",
            "service NPS analysis artifact generated",
        ),
        "service_sentiment_analyzer" => (
            "service",
            "service_sentiment_report",
            "Service sentiment report",
            "service sentiment analysis artifact generated",
        ),
        "service_nps_driver_analysis" => (
            "service",
            "service_nps_driver_report",
            "Service NPS driver report",
            "service NPS driver analysis artifact generated",
        ),
        "ops_promo_planning" => (
            "ops",
            "ops_promo_plan",
            "Ops promotion plan",
            "ops promotion planning artifact generated",
        ),
        "ops_pricing_strategy" => (
            "ops",
            "ops_pricing_strategy",
            "Ops pricing strategy",
            "ops pricing strategy artifact generated",
        ),
        "ops_channel_strategy" => (
            "ops",
            "ops_channel_strategy",
            "Ops channel strategy",
            "ops channel strategy artifact generated",
        ),
        "ops_inventory_planning" => (
            "ops",
            "ops_inventory_plan",
            "Ops inventory plan",
            "ops inventory planning artifact generated",
        ),
        "ops_listing_copy" => (
            "ops",
            "ops_listing_copy",
            "Ops listing copy",
            "ops listing copy artifact generated",
        ),
        "ops_assortment_planning" => (
            "ops",
            "ops_assortment_plan",
            "Ops assortment plan",
            "ops assortment planning artifact generated",
        ),
        "ops_execution_plan" => (
            "ops",
            "ops_execution_plan",
            "Ops execution plan",
            "ops execution plan artifact generated",
        ),
        "ops_smart_pricing" => (
            "ops",
            "ops_smart_pricing_report",
            "Ops smart pricing report",
            "ops smart pricing artifact generated",
        ),
        "ops_inventory_optimizer" => (
            "ops",
            "ops_inventory_optimization_report",
            "Ops inventory optimization report",
            "ops inventory optimization artifact generated",
        ),
        "ops_ad_fatigue_detector" => (
            "ops",
            "ops_ad_fatigue_report",
            "Ops ad fatigue report",
            "ops ad fatigue detection artifact generated",
        ),
        "ops_abc_xyz_classifier" => (
            "ops",
            "ops_abc_xyz_report",
            "Ops ABC-XYZ report",
            "ops ABC-XYZ classification artifact generated",
        ),
        "douyin_ads_plan_structure" => (
            "douyin_ads",
            "douyin_ads_plan_structure",
            "Douyin ads plan structure",
            "Douyin ads plan structure artifact generated",
        ),
        "douyin_ads_audience_package" => (
            "douyin_ads",
            "douyin_ads_audience_package",
            "Douyin ads audience package",
            "Douyin ads audience package artifact generated",
        ),
        "douyin_ads_budget_pacing" => (
            "douyin_ads",
            "douyin_ads_budget_pacing",
            "Douyin ads budget pacing",
            "Douyin ads budget pacing artifact generated",
        ),
        "douyin_ads_live_funnel_diagnosis" => (
            "douyin_ads",
            "douyin_ads_live_funnel_report",
            "Douyin ads live funnel report",
            "Douyin ads live funnel diagnosis artifact generated",
        ),
        "web_seo_optimize" => (
            "web",
            "web_seo_plan",
            "Web SEO plan",
            "web SEO optimization artifact generated",
        ),
        "web_title_generator" => (
            "web",
            "web_title_candidates",
            "Web title candidates",
            "web title generation artifact generated",
        ),
        "web_page_conversion" => (
            "web",
            "web_conversion_plan",
            "Web conversion plan",
            "web page conversion artifact generated",
        ),
        "web_store_design" => (
            "web",
            "web_store_design_brief",
            "Web store design brief",
            "web store design artifact generated",
        ),
        "web_keyword_research" => (
            "web",
            "web_keyword_matrix",
            "Web keyword matrix",
            "web keyword research artifact generated",
        ),
        "web_product_description" => (
            "web",
            "web_product_description",
            "Web product description",
            "web product description artifact generated",
        ),
        "web_title_seo_scorer" => (
            "web",
            "web_title_seo_score",
            "Web title SEO score",
            "web title SEO scoring artifact generated",
        ),
        "engineering_arch_review" => (
            "engineering",
            "engineering_arch_review_report",
            "Engineering architecture review",
            "engineering architecture review artifact generated",
        ),
        "engineering_bug_analysis" => (
            "engineering",
            "engineering_bug_analysis_report",
            "Engineering bug analysis report",
            "engineering bug analysis artifact generated",
        ),
        "engineering_perf_optimize" => (
            "engineering",
            "engineering_perf_plan",
            "Engineering performance plan",
            "engineering performance optimization artifact generated",
        ),
        "engineering_tech_selection" => (
            "engineering",
            "engineering_tech_selection_report",
            "Engineering tech selection report",
            "engineering tech selection artifact generated",
        ),
        "engineering_sla_monitor" => (
            "engineering",
            "engineering_sla_report",
            "Engineering SLA report",
            "engineering SLA monitor artifact generated",
        ),
        "learning_teaching_explain" => (
            "learning",
            "learning_teaching_explain",
            "Learning teaching explain",
            "learning teaching explanation artifact generated",
        ),
        _ => return None,
    };

    let json_body = serde_json::to_string_pretty(output).ok()?;
    let preview = output
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 500))
        .unwrap_or_else(|| compact_text(&json_body, 500));

    Some(ContentArtifactFromSkillDraft {
        content_category: content_category.to_string(),
        artifact_type: artifact_type.to_string(),
        title: title.to_string(),
        file_name: format!("{artifact_type}.md"),
        preview,
        content: format!("# {title}\n\n```json\n{json_body}\n```"),
        summary: summary.to_string(),
    })
}

fn build_campaign_draft_from_skill_output(output: &Value) -> Option<CampaignDraftFromSkill> {
    if output.get("status").and_then(Value::as_str) != Some("success") {
        return None;
    }
    let campaign_creation = output.get("campaign_creation")?.as_object()?;
    if campaign_creation
        .get("should_persist")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return None;
    }

    let name = campaign_creation
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 255))?;
    let description = campaign_creation
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000));
    let campaign_type = campaign_creation
        .get("campaign_type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 64))
        .unwrap_or_else(|| "agent_draft".to_string());
    let budget = campaign_creation.get("budget").and_then(Value::as_f64);
    let product_id = campaign_creation
        .get("product_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| parse_uuid(value, "product_id"))
        .transpose()
        .ok()
        .flatten();
    let start_date = campaign_creation
        .get("start_date")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 64));
    let end_date = campaign_creation
        .get("end_date")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 64));
    let goal = campaign_creation
        .get("goal")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000));
    let target_audience = campaign_creation
        .get("target_audience")
        .or_else(|| campaign_creation.get("targetAudience"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000));
    let channels = campaign_creation
        .get("channels")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| compact_text(value, 64))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let strategy = campaign_creation
        .get("strategy")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 2_000));
    let source_summary = campaign_creation
        .get("source_summary")
        .or_else(|| campaign_creation.get("sourceSummary"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| compact_text(value, 4_000));
    let workspace_id = campaign_creation
        .get("workspace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let source = CampaignDraftSourceResponse {
        source_type: "agent_skill_result".to_string(),
        product_id: product_id.map(|id| id.to_string()),
        workspace_id: workspace_id.clone(),
        source_artifact_id: None,
        source_summary: source_summary.clone(),
        ref_keys: build_campaign_draft_ref_keys_from_skill(product_id, source_summary.as_deref()),
    };
    let warnings = build_campaign_draft_warnings_from_skill(
        source.product_id.is_some(),
        budget,
        start_date.as_deref(),
        end_date.as_deref(),
        source_summary.as_deref(),
    );
    let planning = json!({
        "goal": goal,
        "targetAudience": target_audience,
        "channels": channels,
        "strategy": strategy,
        "budget": budget,
        "startDate": start_date,
        "endDate": end_date,
    });
    let draft = campaign_creation
        .get("draft")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let metadata = json!({
        "source": {
            "sourceType": source.source_type,
            "productId": source.product_id,
            "workspaceId": source.workspace_id,
            "sourceArtifactId": source.source_artifact_id,
            "sourceSummary": source.source_summary,
            "refKeys": source.ref_keys,
        },
        "planning": planning.clone(),
        "refs": source.ref_keys.clone(),
        "workspace": {
            "id": workspace_id,
        },
        "sourceArtifact": Value::Null,
        "extra": {
            "materializedFrom": "coordination_create_campaign",
        },
    });
    let output_json = json!({
        "kind": "campaign_draft",
        "draft": draft,
        "planning": planning.clone(),
        "sourceSummary": source_summary,
    });
    let action_metadata = json!({
        "action": "create_campaign_draft",
        "executionStatus": "not_submitted",
        "requiresApproval": true,
        "platformExecution": "not_started",
        "approvalReason": "campaign draft is local only; platform execution must be approved separately",
        "source": {
            "sourceType": source.source_type,
            "productId": source.product_id,
            "workspaceId": source.workspace_id,
            "sourceArtifactId": source.source_artifact_id,
            "sourceSummary": source.source_summary,
            "refKeys": source.ref_keys,
        },
        "extra": {
            "materializedFrom": "coordination_create_campaign",
        },
    });
    let agent_context = json!({
        "type": "campaign_draft",
        "source": {
            "sourceType": source.source_type,
            "productId": source.product_id,
            "workspaceId": source.workspace_id,
            "sourceArtifactId": source.source_artifact_id,
            "sourceSummary": source.source_summary,
            "refKeys": source.ref_keys,
        },
        "planning": planning,
        "warnings": warnings.clone(),
    });

    Some(CampaignDraftFromSkill {
        name,
        description,
        campaign_type,
        budget,
        product_id,
        start_date,
        end_date,
        metadata,
        output: output_json,
        action_metadata,
        source,
        warnings,
        agent_context,
    })
}

fn search_evidence_product_context(output: &Value) -> Option<SearchEvidenceProductContext> {
    let product_ref = normalized_json_string(output.pointer("/query/product_ref"));
    let product_name = normalized_json_string(output.pointer("/query/product_name"));
    let product_category = normalized_json_string(output.pointer("/query/product_category"));
    if product_ref.is_none() && product_name.is_none() && product_category.is_none() {
        return None;
    }
    Some(SearchEvidenceProductContext {
        product_ref,
        product_name,
        product_category,
    })
}

fn normalized_json_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .and_then(|value| normalize_string(value.to_string()))
}

impl SearchEvidenceProductContext {
    fn to_json(&self) -> Value {
        let mut value = serde_json::Map::new();
        if let Some(product_ref) = &self.product_ref {
            value.insert("productRef".to_string(), json!(product_ref));
        }
        if let Some(product_name) = &self.product_name {
            value.insert("productName".to_string(), json!(product_name));
        }
        if let Some(product_category) = &self.product_category {
            value.insert("productCategory".to_string(), json!(product_category));
        }
        Value::Object(value)
    }

    fn preview_line(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(product_name) = &self.product_name {
            parts.push(product_name.as_str());
        }
        if let Some(product_category) = &self.product_category {
            parts.push(product_category.as_str());
        }
        if let Some(product_ref) = &self.product_ref {
            parts.push(product_ref.as_str());
        }
        (!parts.is_empty()).then(|| format!("Product: {}", compact_text(&parts.join(" / "), 180)))
    }
}

fn format_search_evidence_preview(
    sources: &[Value],
    query: &str,
    product_context: Option<&SearchEvidenceProductContext>,
) -> String {
    let mut lines = Vec::new();
    if !query.trim().is_empty() {
        lines.push(format!("Query: {}", compact_text(query, 160)));
    }
    if let Some(product_line) = product_context.and_then(SearchEvidenceProductContext::preview_line)
    {
        lines.push(product_line);
    }
    lines.push(format!("Sources: {}", sources.len()));
    for (index, source) in sources.iter().take(5).enumerate() {
        let title = source
            .get("title")
            .and_then(Value::as_str)
            .or_else(|| source.get("name").and_then(Value::as_str))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("untitled");
        let domain = source
            .get("domain")
            .and_then(Value::as_str)
            .or_else(|| source.get("url").and_then(Value::as_str))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown");
        lines.push(format!(
            "{}. {} ({})",
            index + 1,
            compact_text(title, 120),
            compact_text(domain, 80)
        ));
    }
    compact_text(&lines.join("\n"), 1000)
}

fn parse_workspace_status(value: Option<String>) -> BizResult<Option<WorkspaceStatus>> {
    parse_status(value, |value| match value {
        "active" => Ok(WorkspaceStatus::Active),
        "archived" => Ok(WorkspaceStatus::Archived),
        _ => Err(()),
    })
}

fn parse_task_status(value: Option<String>) -> BizResult<Option<WorkspaceTaskStatus>> {
    parse_status(value, |value| match value {
        "pending" => Ok(WorkspaceTaskStatus::Pending),
        "in_progress" => Ok(WorkspaceTaskStatus::InProgress),
        "done" => Ok(WorkspaceTaskStatus::Done),
        "blocked" => Ok(WorkspaceTaskStatus::Blocked),
        "cancelled" => Ok(WorkspaceTaskStatus::Cancelled),
        "archived" => Ok(WorkspaceTaskStatus::Archived),
        _ => Err(()),
    })
}

fn parse_run_status(value: Option<String>) -> BizResult<Option<WorkspaceRunStatus>> {
    parse_status(value, |value| match value {
        "planned" => Ok(WorkspaceRunStatus::Planned),
        "running" => Ok(WorkspaceRunStatus::Running),
        "succeeded" => Ok(WorkspaceRunStatus::Succeeded),
        "failed" => Ok(WorkspaceRunStatus::Failed),
        "cancelled" => Ok(WorkspaceRunStatus::Cancelled),
        "archived" => Ok(WorkspaceRunStatus::Archived),
        _ => Err(()),
    })
}

fn parse_run_task_status(value: Option<String>) -> BizResult<Option<WorkspaceRunTaskStatus>> {
    parse_status(value, |value| match value {
        "pending" => Ok(WorkspaceRunTaskStatus::Pending),
        "running" => Ok(WorkspaceRunTaskStatus::Running),
        "succeeded" => Ok(WorkspaceRunTaskStatus::Succeeded),
        "failed" => Ok(WorkspaceRunTaskStatus::Failed),
        "skipped" => Ok(WorkspaceRunTaskStatus::Skipped),
        "cancelled" => Ok(WorkspaceRunTaskStatus::Cancelled),
        _ => Err(()),
    })
}

fn parse_skill_run_status(value: Option<String>) -> BizResult<Option<AgentSkillRunStatus>> {
    parse_status(value, parse_skill_run_status_value)
}

fn parse_required_skill_run_status(value: String) -> BizResult<AgentSkillRunStatus> {
    let value = normalize_required(value, "status")?;
    parse_skill_run_status_value(value.as_str()).map_err(|_| {
        BizError::new(
            app_error::WORKSPACE_REQUEST_INVALID,
            format!("invalid skill run status: {value}"),
        )
    })
}

fn parse_skill_run_status_value(value: &str) -> Result<AgentSkillRunStatus, ()> {
    match value {
        "requested" => Ok(AgentSkillRunStatus::Requested),
        "success" => Ok(AgentSkillRunStatus::Success),
        "deferred" => Ok(AgentSkillRunStatus::Deferred),
        "pending_approval" => Ok(AgentSkillRunStatus::PendingApproval),
        "not_found" => Ok(AgentSkillRunStatus::NotFound),
        "validation_error" => Ok(AgentSkillRunStatus::ValidationError),
        "failed" => Ok(AgentSkillRunStatus::Failed),
        _ => Err(()),
    }
}

fn parse_artifact_status(value: Option<String>) -> BizResult<Option<WorkspaceArtifactStatus>> {
    parse_status(value, |value| match value {
        "active" => Ok(WorkspaceArtifactStatus::Active),
        "archived" => Ok(WorkspaceArtifactStatus::Archived),
        _ => Err(()),
    })
}

fn parse_artifact_validation_status(
    value: Option<String>,
) -> BizResult<Option<WorkspaceArtifactValidationStatus>> {
    parse_status(value, |value| match value {
        "unchecked" => Ok(WorkspaceArtifactValidationStatus::Unchecked),
        "passed" => Ok(WorkspaceArtifactValidationStatus::Passed),
        "failed" => Ok(WorkspaceArtifactValidationStatus::Failed),
        _ => Err(()),
    })
}

fn parse_execution_status(value: Option<String>) -> BizResult<Option<ExecutionQueueStatus>> {
    parse_status(value, |value| match value {
        "pending" => Ok(ExecutionQueueStatus::Pending),
        "approved" => Ok(ExecutionQueueStatus::Approved),
        "rejected" => Ok(ExecutionQueueStatus::Rejected),
        "executed" => Ok(ExecutionQueueStatus::Executed),
        "failed" => Ok(ExecutionQueueStatus::Failed),
        "cancelled" => Ok(ExecutionQueueStatus::Cancelled),
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
                    app_error::WORKSPACE_REQUEST_INVALID,
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
            app_error::WORKSPACE_REQUEST_INVALID,
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

fn compact_optional_text(value: &str) -> String {
    compact_text(value, 1000)
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

fn normalize_workspace_memory_type(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "fact" | "事实" => "fact".to_string(),
        "decision" | "决策" => "decision".to_string(),
        "insight" | "洞察" => "insight".to_string(),
        "warning" | "风险" => "warning".to_string(),
        _ => "fact".to_string(),
    }
}

fn memory_confidence_for_type(memory_type: &str) -> f64 {
    match memory_type {
        "decision" => 0.88,
        "warning" => 0.84,
        "insight" => 0.8,
        _ => 0.76,
    }
}

fn learning_record_memory_key(record: &AgentLearningRecord) -> Option<&str> {
    record
        .metadata
        .get("memoryKey")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn build_campaign_draft_ref_keys_from_skill(
    product_id: Option<Uuid>,
    source_summary: Option<&str>,
) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(product_id) = product_id {
        refs.push(format!("product:{product_id}"));
    }
    if source_summary.is_some() {
        refs.push("agent_output:coordination_create_campaign".to_string());
    }
    refs
}

fn build_campaign_draft_warnings_from_skill(
    has_product: bool,
    budget: Option<f64>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    source_summary: Option<&str>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if !has_product {
        warnings.push("campaign draft is not bound to a product".to_string());
    }
    if budget.map(|value| value <= 0.0).unwrap_or(true) {
        warnings.push("campaign draft budget is missing or zero".to_string());
    }
    if start_date.is_none() || end_date.is_none() {
        warnings.push("campaign draft date range is incomplete".to_string());
    }
    if source_summary.is_none() {
        warnings.push("campaign draft has no source artifact".to_string());
    }
    warnings
}

fn non_empty_join(values: &[String]) -> Option<String> {
    let value = values
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    (!value.is_empty()).then_some(value)
}

fn metadata_str_eq(metadata: &Value, key: &str, expected: &str) -> bool {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(|value| value == expected)
        .unwrap_or(false)
}

fn merge_json_object(base: Value, patch: Value) -> Value {
    match (base, patch) {
        (Value::Object(mut base), Value::Object(patch)) => {
            for (key, value) in patch {
                base.insert(key, value);
            }
            Value::Object(base)
        }
        (_, patch) => patch,
    }
}

#[derive(Debug, Clone)]
struct QualityEvaluationDraft {
    score: f64,
    passed: bool,
    dimensions: Value,
    issues: Vec<String>,
    suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
struct LearningDraft {
    category: String,
    content: String,
    confidence: f64,
    source_action: String,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct CampaignDraftFromSkill {
    name: String,
    description: Option<String>,
    campaign_type: String,
    budget: Option<f64>,
    product_id: Option<Uuid>,
    start_date: Option<String>,
    end_date: Option<String>,
    metadata: Value,
    output: Value,
    action_metadata: Value,
    source: CampaignDraftSourceResponse,
    warnings: Vec<String>,
    agent_context: Value,
}

#[derive(Debug, Clone)]
struct WorkspaceMemoryFromSkillDraft {
    key: String,
    content: String,
    memory_type: String,
    confidence: f64,
}

#[derive(Debug, Clone)]
struct ContentArtifactFromSkillDraft {
    content_category: String,
    artifact_type: String,
    title: String,
    file_name: String,
    preview: String,
    content: String,
    summary: String,
}

fn evaluate_output_quality(
    input: &str,
    output: &str,
    tasks: &[WorkspaceRunTask],
) -> QualityEvaluationDraft {
    let output_chars = output.chars().count();
    let input_chars = input.chars().count();
    let lower = output.to_ascii_lowercase();
    let direct_answer = is_direct_answer_quality_case(input, output, output_chars);
    let completeness = if output_chars == 0 {
        0.0
    } else if direct_answer {
        0.9
    } else if output_chars >= 800 {
        0.95
    } else if output_chars >= 300 {
        0.82
    } else if output_chars >= 120 {
        0.65
    } else {
        0.35
    };
    let actionability = if direct_answer {
        0.82
    } else {
        keyword_score(
            output,
            &[
                "建议",
                "步骤",
                "执行",
                "下一步",
                "可以",
                "优先",
                "方案",
                "checklist",
                "todo",
            ],
        )
    };
    let relevance = if direct_answer {
        0.88
    } else {
        relevance_score(input, output)
    };
    let clarity = {
        if direct_answer {
            0.88
        } else {
            let has_structure =
                output.contains('\n') || output.contains("1.") || output.contains("- ");
            let sentence_count = output.matches('。').count()
                + output.matches('.').count()
                + output.matches('\n').count();
            let base: f64 = if has_structure { 0.78 } else { 0.55 };
            (base + (sentence_count as f64 * 0.025)).clamp(0.2_f64, 1.0_f64)
        }
    };
    let risk_awareness = if direct_answer {
        0.82
    } else {
        keyword_score(
            output,
            &[
                "风险",
                "边界",
                "假设",
                "注意",
                "需确认",
                "验证",
                "不确定",
                "限制",
            ],
        )
    };
    let value_density = {
        if direct_answer {
            0.78
        } else {
            let business_hits = keyword_hits(
                output,
                &[
                    "用户",
                    "业务",
                    "运营",
                    "电商",
                    "转化",
                    "成本",
                    "效率",
                    "数据",
                    "策略",
                    "执行",
                    "岗位",
                    "智能体",
                ],
            );
            let density = business_hits as f64 / (output_chars.max(1) as f64 / 240.0).max(1.0);
            (0.45 + density * 0.13).clamp(0.2_f64, 1.0_f64)
        }
    };
    let confidence_marking = if direct_answer {
        0.82
    } else {
        keyword_score(
            output,
            &[
                "基于",
                "现有信息",
                "可能",
                "建议",
                "需要确认",
                "我会",
                "可以先",
            ],
        )
    };
    let role_delivery = if tasks.is_empty() {
        0.72
    } else {
        let avg = tasks
            .iter()
            .map(|task| task.acceptance_score.clamp(0.0, 1.0))
            .sum::<f64>()
            / tasks.len() as f64;
        let passed_ratio =
            tasks.iter().filter(|task| task.acceptance_passed).count() as f64 / tasks.len() as f64;
        (avg * 0.7 + passed_ratio * 0.3).clamp(0.0, 1.0)
    };
    let fabrication = {
        if direct_answer {
            0.86
        } else {
            let numeric_claims = lower
                .split_whitespace()
                .filter(|part| part.chars().any(|ch| ch.is_ascii_digit()))
                .count();
            let has_caution = ["约", "大概", "估算", "需验证", "来源", "基于"]
                .iter()
                .any(|word| output.contains(word));
            if numeric_claims > 8 && !has_caution {
                0.45
            } else if numeric_claims > 3 && !has_caution {
                0.65
            } else {
                0.86
            }
        }
    };
    let dimensions = json!({
        "completeness": completeness,
        "actionability": actionability,
        "relevance": relevance,
        "clarity": clarity,
        "riskAwareness": risk_awareness,
        "valueDensity": value_density,
        "confidenceMarking": confidence_marking,
        "roleDelivery": role_delivery,
        "fabricationGuard": fabrication,
        "directAnswer": direct_answer,
        "inputChars": input_chars,
        "outputChars": output_chars,
    });
    let values = [
        completeness,
        actionability,
        relevance,
        clarity,
        risk_awareness,
        value_density,
        confidence_marking,
        role_delivery,
        fabrication,
    ];
    let score = (values.iter().sum::<f64>() / values.len() as f64).clamp(0.0, 1.0);
    let passed = score >= 0.7;
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();
    push_quality_issue(
        completeness,
        "回答完整度偏低",
        "补齐背景、结论、操作路径和交付物说明。",
        &mut issues,
        &mut suggestions,
    );
    push_quality_issue(
        actionability,
        "可执行性不足",
        "增加可直接执行的步骤、优先级和下一步动作。",
        &mut issues,
        &mut suggestions,
    );
    push_quality_issue(
        relevance,
        "和用户问题的关键词关联偏弱",
        "围绕用户原问题补充更直接的回答。",
        &mut issues,
        &mut suggestions,
    );
    push_quality_issue(
        risk_awareness,
        "风险和边界提示不足",
        "说明假设、风险、依赖条件和需要用户确认的内容。",
        &mut issues,
        &mut suggestions,
    );
    push_quality_issue(
        confidence_marking,
        "确定性标注不足",
        "区分确定事实、推断和需要进一步验证的判断。",
        &mut issues,
        &mut suggestions,
    );

    QualityEvaluationDraft {
        score,
        passed,
        dimensions,
        issues,
        suggestions,
    }
}

fn push_quality_issue(
    value: f64,
    issue: &str,
    suggestion: &str,
    issues: &mut Vec<String>,
    suggestions: &mut Vec<String>,
) {
    if value < 0.65 {
        issues.push(issue.to_string());
        suggestions.push(suggestion.to_string());
    }
}

fn is_direct_answer_quality_case(input: &str, output: &str, output_chars: usize) -> bool {
    if output.trim().is_empty() || output_chars > 120 {
        return false;
    }

    let input_lower = input.to_ascii_lowercase();
    let explicit_direct = [
        "只输出",
        "只给",
        "只回答",
        "只回复",
        "不要解释",
        "不用解释",
        "无需解释",
        "直接回答",
        "直接给",
        "简短回答",
        "简要回答",
        "一句话",
        "only output",
        "answer only",
        "no explanation",
    ]
    .iter()
    .any(|term| input.contains(term) || input_lower.contains(term));

    let has_input_digit = input.chars().any(|ch| ch.is_ascii_digit());
    let arithmetic_request = has_input_digit
        && [
            "+", "-", "*", "/", "×", "÷", "加", "减", "乘", "除", "计算", "等于", "结果",
        ]
        .iter()
        .any(|term| input.contains(term));
    let short_question = ["是多少", "多少", "几个", "几"]
        .iter()
        .any(|term| input.contains(term));
    let concise_output = output.matches('\n').count() <= 1
        && output.matches('。').count() + output.matches('.').count() <= 2;

    (explicit_direct || arithmetic_request || short_question) && concise_output
}

fn keyword_score(text: &str, keywords: &[&str]) -> f64 {
    let hits = keyword_hits(text, keywords);
    match hits {
        0 => 0.35,
        1 => 0.62,
        2 => 0.76,
        3 => 0.86,
        _ => 0.95,
    }
}

fn keyword_hits(text: &str, keywords: &[&str]) -> usize {
    keywords
        .iter()
        .filter(|keyword| text.contains(**keyword))
        .count()
}

fn relevance_score(input: &str, output: &str) -> f64 {
    let input_keywords = extract_keywords(input);
    if input_keywords.is_empty() {
        return 0.72;
    }
    let matched = input_keywords
        .iter()
        .filter(|keyword| output.contains(keyword.as_str()))
        .count();
    (0.35 + matched as f64 / input_keywords.len() as f64 * 0.6).clamp(0.2, 1.0)
}

fn extract_keywords(text: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    for part in text.split(|ch: char| {
        ch.is_whitespace()
            || [
                ',', '.', '，', '。', '？', '?', '！', '!', '、', ';', '；', ':', '：',
            ]
            .contains(&ch)
    }) {
        let part = part.trim();
        let char_count = part.chars().count();
        if char_count < 2 {
            continue;
        }
        if char_count <= 8 || !contains_cjk(part) {
            push_unique_keyword(&mut keywords, part);
            continue;
        }
        for term in commerce_relevance_terms() {
            if part.contains(term) {
                push_unique_keyword(&mut keywords, term);
            }
        }
    }
    keywords.truncate(16);
    keywords
}

fn contains_cjk(text: &str) -> bool {
    text.chars()
        .any(|ch| matches!(ch, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}'))
}

fn push_unique_keyword(keywords: &mut Vec<String>, keyword: &str) {
    if !keywords.iter().any(|item| item == keyword) {
        keywords.push(keyword.to_string());
    }
}

fn commerce_relevance_terms() -> &'static [&'static str] {
    &[
        "电商",
        "商品",
        "新品",
        "营销",
        "活动",
        "投放",
        "复盘",
        "方案",
        "风险",
        "下一步",
        "执行",
        "用户",
        "运营",
        "转化",
        "成本",
        "效率",
        "数据",
        "策略",
        "预算",
        "素材",
        "平台",
        "客单价",
        "退款",
        "利润",
        "库存",
        "选品",
        "定价",
        "客服",
        "客诉",
        "岗位",
        "智能体",
        "GMV",
        "ROI",
    ]
}

fn trust_ema(previous_score: f64, quality_score: f64) -> f64 {
    let alpha = 0.3;
    (previous_score * (1.0 - alpha) + quality_score * alpha).clamp(0.0, 1.0)
}

fn feedback_confidence(rating: i32) -> f64 {
    match rating {
        5 => 0.9,
        4 => 0.75,
        3 => 0.55,
        2 => 0.7,
        _ => 0.85,
    }
}

fn build_quality_learning_drafts(
    quality: &QualityEvaluationDraft,
    run: &WorkspaceRun,
    output: &str,
) -> Vec<LearningDraft> {
    let mut drafts = Vec::new();
    if quality.passed && quality.score >= 0.75 {
        drafts.push(LearningDraft {
            category: "success".to_string(),
            content: format!(
                "High quality answer pattern for similar tasks: {}",
                compact_text(output, 500)
            ),
            confidence: quality.score.min(0.95),
            source_action: "quality_check_success".to_string(),
            metadata: json!({
                "runId": run.id,
                "score": quality.score,
            }),
        });
    }

    if !quality.issues.is_empty() {
        let category = if quality.score < 0.5 {
            "blindspot"
        } else {
            "improvement"
        };
        drafts.push(LearningDraft {
            category: category.to_string(),
            content: format!(
                "Quality issues to avoid next time: {}",
                quality.issues.join("; ")
            ),
            confidence: (1.0 - quality.score).clamp(0.45, 0.9),
            source_action: "quality_check_issue".to_string(),
            metadata: json!({
                "runId": run.id,
                "suggestions": quality.suggestions,
                "score": quality.score,
            }),
        });
    }

    if drafts.is_empty() {
        drafts.push(LearningDraft {
            category: "general".to_string(),
            content: format!(
                "Answer quality was acceptable; continue improving clarity and actionability for \
                 goal: {}",
                compact_text(&run.goal, 240)
            ),
            confidence: quality.score.max(0.5),
            source_action: "quality_check_general".to_string(),
            metadata: json!({
                "runId": run.id,
                "score": quality.score,
            }),
        });
    }

    drafts
}

fn clamp_priority(priority: i32) -> i32 {
    priority.clamp(0, 10)
}

fn pagination(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    (page, page_size, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_checker_passes_actionable_ecommerce_answer() {
        let quality = evaluate_output_quality(
            "请给我一个电商新品投放复盘方案，包含风险和下一步执行。",
            "基于现有信息，建议先按用户、运营、转化和成本四个维度复盘。\n1. \
             数据：检查曝光、点击、转化、客单价和退款率，确认主要问题。\n2. \
             执行：优先调整预算和素材，下一步把低转化计划降预算。\n3. \
             风险：当前数据可能受活动周期影响，需确认平台口径和样本量。\n4. \
             智能体岗位：运营负责节奏，数据岗位负责指标，创意岗位负责素材方案。\\
             n这是一份可执行方案，可以先验证三天后再复盘。",
            &[],
        );

        assert!(quality.passed);
        assert!(quality.score >= 0.7);
        assert!(quality.issues.is_empty());
    }

    #[test]
    fn quality_checker_rejects_empty_answer() {
        let quality = evaluate_output_quality("请给我一个电商运营方案", "", &[]);

        assert!(!quality.passed);
        assert!(quality.score < 0.7);
        assert!(quality.issues.iter().any(|issue| issue == "回答完整度偏低"));
    }

    #[test]
    fn quality_checker_accepts_explicit_short_direct_answer() {
        let quality = evaluate_output_quality(
            "只输出 17+25 的计算结果，不要解释。",
            "17 + 25 等于 42。",
            &[],
        );

        assert!(quality.passed);
        assert!(quality.score >= 0.7);
        assert!(quality.issues.is_empty());
        assert_eq!(
            quality
                .dimensions
                .get("directAnswer")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn trust_ema_moves_previous_score_toward_quality_score() {
        let updated = trust_ema(0.5, 0.8);

        assert!((updated - 0.59).abs() < 0.0001);
    }

    #[test]
    fn search_evidence_artifact_requires_live_sources() {
        let req = RecordAgentSearchEvidenceRequest {
            user_id: "11111111-1111-1111-1111-111111111111".to_string(),
            workspace_id: "22222222-2222-2222-2222-222222222222".to_string(),
            run_id: "33333333-3333-3333-3333-333333333333".to_string(),
            worker_id: "worker.data".to_string(),
            task_id: "task-1".to_string(),
            request_id: "task-1-skill-1".to_string(),
            skill_id: "search_competitor".to_string(),
            role: "data".to_string(),
            status: "success".to_string(),
            summary: Some("外部搜索已返回 1 个可采信来源".to_string()),
            execution_mode: Some("adapter".to_string()),
            output: json!({
                "status": "live",
                "adapter_status": "live",
                "query": {
                    "planned": "冷萃咖啡 竞品 价格"
                },
                "engine": "mock-search",
                "sources": [
                    {
                        "title": "冷萃咖啡竞品价格观察",
                        "url": "https://example.com/a",
                        "domain": "example.com",
                        "snippet": "价格区间集中在 19-29 元。"
                    }
                ],
                "provenance": {
                    "source_count": 1,
                    "domains": ["example.com"]
                },
                "confidence": 0.82,
                "freshness": {
                    "requested_at": "2026-05-10T00:00:00Z"
                },
                "evidence": {
                    "summary": "共 1 条实时来源，覆盖 1 个域名；回答应优先引用这些来源，并注明时效性和置信度。",
                    "source_titles": ["冷萃咖啡竞品价格观察"],
                    "source_domains": ["example.com"],
                    "llm_context": "实时来源摘要"
                }
            }),
        };

        let draft = build_search_evidence_artifact(&req).expect("live search should persist");

        assert_eq!(draft.title, "Search evidence - 冷萃咖啡 竞品 价格");
        assert!(draft.preview.contains("Sources: 1"));
        assert_eq!(
            draft.metadata.get("sourceCount").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            draft.metadata.get("engine").and_then(Value::as_str),
            Some("mock-search")
        );
        assert!(
            draft
                .metadata
                .get("evidence")
                .and_then(Value::as_object)
                .and_then(|value| value.get("summary"))
                .and_then(Value::as_str)
                .is_some_and(|summary| summary.contains("1 条实时来源"))
        );
    }

    #[test]
    fn search_evidence_artifact_preserves_product_context() {
        let req = RecordAgentSearchEvidenceRequest {
            user_id: "11111111-1111-1111-1111-111111111111".to_string(),
            workspace_id: "22222222-2222-2222-2222-222222222222".to_string(),
            run_id: "33333333-3333-3333-3333-333333333333".to_string(),
            worker_id: "worker.data".to_string(),
            task_id: "task-1".to_string(),
            request_id: "task-1-skill-1".to_string(),
            skill_id: "search_competitor".to_string(),
            role: "data".to_string(),
            status: "success".to_string(),
            summary: Some("外部搜索已返回 1 个可采信来源".to_string()),
            execution_mode: Some("adapter".to_string()),
            output: json!({
                "status": "live",
                "adapter_status": "live",
                "query": {
                    "planned": "五一咖啡组合 咖啡饮品",
                    "product_ref": "product:11111111-1111-1111-1111-111111111111",
                    "product_name": "五一咖啡组合",
                    "product_category": "咖啡饮品"
                },
                "engine": "mock-search",
                "sources": [
                    {
                        "title": "五一咖啡组合竞品价格观察",
                        "url": "https://example.com/a",
                        "domain": "example.com",
                        "snippet": "同类组合装集中在 89-129 元。"
                    }
                ],
                "provenance": {
                    "source_count": 1,
                    "domains": ["example.com"]
                },
                "confidence": 0.82,
                "freshness": {
                    "requested_at": "2026-05-10T00:00:00Z"
                },
                "evidence": {
                    "summary": "共 1 条实时来源，覆盖 1 个域名。",
                    "source_titles": ["五一咖啡组合竞品价格观察"],
                    "source_domains": ["example.com"],
                    "llm_context": "实时来源摘要"
                }
            }),
        };

        let draft = build_search_evidence_artifact(&req).expect("live search should persist");

        assert!(draft.preview.contains(
            "Product: 五一咖啡组合 / 咖啡饮品 / product:11111111-1111-1111-1111-111111111111"
        ));
        assert_eq!(
            draft.metadata.pointer("/productContext/productRef"),
            Some(&json!("product:11111111-1111-1111-1111-111111111111"))
        );
        assert_eq!(
            draft.metadata.pointer("/productContext/productName"),
            Some(&json!("五一咖啡组合"))
        );
        assert_eq!(
            draft.metadata.pointer("/productContext/productCategory"),
            Some(&json!("咖啡饮品"))
        );
        assert_eq!(
            draft.metadata.get("productRef"),
            Some(&json!("product:11111111-1111-1111-1111-111111111111"))
        );
        assert_eq!(
            draft.metadata.get("productName"),
            Some(&json!("五一咖啡组合"))
        );
        assert_eq!(
            draft.metadata.get("productCategory"),
            Some(&json!("咖啡饮品"))
        );
    }

    #[test]
    fn search_evidence_artifact_ignores_degraded_output() {
        let req = RecordAgentSearchEvidenceRequest {
            user_id: "11111111-1111-1111-1111-111111111111".to_string(),
            workspace_id: "22222222-2222-2222-2222-222222222222".to_string(),
            run_id: "33333333-3333-3333-3333-333333333333".to_string(),
            worker_id: "worker.data".to_string(),
            task_id: "task-1".to_string(),
            request_id: "task-1-skill-1".to_string(),
            skill_id: "search_trends".to_string(),
            role: "data".to_string(),
            status: "deferred".to_string(),
            summary: Some("adapter 未配置".to_string()),
            execution_mode: Some("adapter".to_string()),
            output: json!({
                "status": "deferred",
                "adapter_status": "adapter_required",
                "sources": []
            }),
        };

        assert!(build_search_evidence_artifact(&req).is_none());
    }

    #[test]
    fn skill_run_status_parser_accepts_agent_runtime_statuses() {
        assert_eq!(
            parse_skill_run_status(Some("requested".to_string())).unwrap(),
            Some(AgentSkillRunStatus::Requested)
        );
        assert_eq!(
            parse_skill_run_status(Some("pending_approval".to_string())).unwrap(),
            Some(AgentSkillRunStatus::PendingApproval)
        );
        assert_eq!(
            parse_required_skill_run_status("validation_error".to_string()).unwrap(),
            AgentSkillRunStatus::ValidationError
        );
    }

    #[test]
    fn skill_run_status_parser_rejects_unknown_values() {
        let err = parse_required_skill_run_status("done".to_string())
            .expect_err("unknown skill run status should fail");

        assert_eq!(err.code(), app_error::WORKSPACE_REQUEST_INVALID);
    }

    #[test]
    fn skill_run_mapper_preserves_audit_fields() {
        let requested_at = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let completed_at = OffsetDateTime::from_unix_timestamp(1_778_544_002).unwrap();
        let item = AgentSkillRun {
            id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            user_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            workspace_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
            run_id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
            session_id: Some("session-1".to_string()),
            worker_id: "worker.ops".to_string(),
            task_id: "task-1".to_string(),
            request_id: "task-1-skill-1".to_string(),
            skill_id: "ops_campaign_plan".to_string(),
            role: "ops".to_string(),
            status: AgentSkillRunStatus::Success,
            reason: Some("需要生成运营计划".to_string()),
            summary: Some("已生成计划".to_string()),
            error: None,
            input: json!({}),
            output: json!({ "plan": "launch" }),
            execution_mode: Some("native".to_string()),
            risk_level: Some("low".to_string()),
            approval_required: false,
            metadata: json!({ "source": "agent_sse" }),
            requested_at,
            completed_at: Some(completed_at),
            created_at: requested_at,
            updated_at: completed_at,
        };

        let response = WorkspaceApi::map_skill_run(item);

        assert_eq!(response.skill_id, "ops_campaign_plan");
        assert_eq!(response.status, "success");
        assert_eq!(response.execution_mode.as_deref(), Some("native"));
        assert_eq!(response.risk_level.as_deref(), Some("low"));
        assert_eq!(
            response.output.get("plan").and_then(Value::as_str),
            Some("launch")
        );
        assert_eq!(response.completed_at, Some(completed_at));
    }

    #[test]
    fn workspace_task_artifact_builds_from_coordination_skill_output() {
        let output = json!({
            "status": "success",
            "task_creation": {
                "should_persist": true,
                "title": "整理五一咖啡活动复盘任务",
                "description": "拆出投放、客服、商品三个方向",
                "owner_role": "ops",
                "priority": 6,
                "acceptance_criteria": "输出复盘结论和后续动作"
            }
        });

        let draft = build_workspace_task_from_skill_output(&output)
            .expect("coordination create task output should be materializable");

        assert_eq!(draft.title, "整理五一咖啡活动复盘任务");
        assert_eq!(draft.owner_role.as_deref(), Some("ops"));
        assert_eq!(draft.priority, 6);
        assert_eq!(
            draft.acceptance_criteria.as_deref(),
            Some("输出复盘结论和后续动作")
        );
    }

    #[test]
    fn workspace_task_artifact_ignores_non_persistable_output() {
        let output = json!({
            "status": "success",
            "task_creation": {
                "should_persist": false,
                "title": "仅建议不落库"
            }
        });

        assert!(build_workspace_task_from_skill_output(&output).is_none());
    }

    #[test]
    fn workspace_memory_artifact_builds_from_coordination_skill_output() {
        let output = json!({
            "status": "success",
            "memory_save": {
                "should_persist": true,
                "key": "pricing_strategy_2024",
                "content": "本品定价区间保持在 159-179 元，避免低于核心竞品均价。",
                "memory_type": "decision"
            }
        });

        let draft = build_workspace_memory_from_skill_output(&output)
            .expect("coordination save memory output should be materializable");

        assert_eq!(draft.key, "pricing_strategy_2024");
        assert_eq!(draft.memory_type, "decision");
        assert!(draft.content.contains("159-179"));
        assert!((draft.confidence - 0.88).abs() < f64::EPSILON);
    }

    #[test]
    fn content_artifact_builds_from_creative_skill_output() {
        let output = json!({
            "status": "success",
            "summary": "短视频脚本已生成",
            "脚本结构": [
                {"阶段": "开场钩子", "话术": "先看结果"}
            ]
        });

        let draft = build_content_artifact_from_skill_output("creative_video_script", &output)
            .expect("creative output should be materializable");

        assert_eq!(draft.artifact_type, "video_script");
        assert_eq!(draft.content_category, "creative");
        assert_eq!(draft.title, "Video script");
        assert_eq!(draft.file_name, "video_script.md");
        assert!(draft.preview.contains("短视频脚本已生成"));
        assert!(draft.content.contains("# Video script"));
        assert!(draft.content.contains("```json"));
    }

    #[test]
    fn content_artifact_builds_from_creative_advanced_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "内容趋势方向已生成",
            "类目": "冷萃咖啡",
            "趋势方向": [
                {"方向": "真实测评透明化", "内容切入": "把优缺点讲清楚"}
            ]
        });

        let draft = build_content_artifact_from_skill_output("creative_trend_catch", &output)
            .expect("creative trend output should be materializable");

        assert_eq!(draft.content_category, "creative");
        assert_eq!(draft.artifact_type, "creative_trend_report");
        assert_eq!(draft.title, "Creative trend report");
        assert_eq!(draft.file_name, "creative_trend_report.md");
        assert!(draft.preview.contains("内容趋势方向已生成"));
        assert!(draft.content.contains("# Creative trend report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );

        let score_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "标题CTR预测评分已完成",
            "标题": "限时新品冷萃咖啡，通勤党立即加购",
            "综合CTR评分": "82/100"
        });

        let score_draft =
            build_content_artifact_from_skill_output("creative_title_ctr_scorer", &score_output)
                .expect("creative title CTR output should be materializable");

        assert_eq!(score_draft.content_category, "creative");
        assert_eq!(score_draft.artifact_type, "creative_title_ctr_score");
        assert_eq!(score_draft.title, "Creative title CTR score");
        assert_eq!(score_draft.file_name, "creative_title_ctr_score.md");
        assert!(score_draft.preview.contains("标题CTR预测评分已完成"));
        assert!(score_draft.content.contains("# Creative title CTR score"));
    }

    #[test]
    fn content_artifact_builds_from_design_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "summary": "主图设计方案已生成",
            "素材清单": ["商品正面透明底图", "场景背景图"]
        });

        let draft = build_content_artifact_from_skill_output("design_main_image", &output)
            .expect("design output should be materializable");

        assert_eq!(draft.content_category, "design");
        assert_eq!(draft.artifact_type, "design_main_image_brief");
        assert_eq!(draft.title, "Main image design brief");
        assert_eq!(draft.file_name, "design_main_image_brief.md");
        assert!(draft.preview.contains("主图设计方案已生成"));
        assert!(draft.content.contains("# Main image design brief"));
    }

    #[test]
    fn content_artifact_builds_from_design_material_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "calculated_from_defaults",
            "summary": "素材规范已生成",
            "用途": "主图套组",
            "平台规格": [
                {"平台": "淘宝", "建议尺寸": "800x800"}
            ]
        });

        let draft = build_content_artifact_from_skill_output("design_material_spec", &output)
            .expect("design material output should be materializable");

        assert_eq!(draft.content_category, "design");
        assert_eq!(draft.artifact_type, "design_material_spec");
        assert_eq!(draft.title, "Design material spec");
        assert_eq!(draft.file_name, "design_material_spec.md");
        assert!(draft.preview.contains("素材规范已生成"));
        assert!(draft.content.contains("# Design material spec"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );

        let attribution_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "渠道归因分析完成，抖音归因贡献最高",
            "各渠道归因明细": [
                {"渠道": "抖音", "实际GMV": 120000.0},
                {"渠道": "淘宝", "实际GMV": 80000.0}
            ]
        });

        let attribution_draft = build_content_artifact_from_skill_output(
            "data_attribution_analysis",
            &attribution_output,
        )
        .expect("data attribution output should be materializable");

        assert_eq!(attribution_draft.content_category, "data");
        assert_eq!(attribution_draft.artifact_type, "data_attribution_report");
        assert_eq!(attribution_draft.title, "Data attribution report");
        assert_eq!(attribution_draft.file_name, "data_attribution_report.md");
        assert!(attribution_draft.preview.contains("渠道归因分析完成"));
    }

    #[test]
    fn content_artifact_builds_from_data_dashboard_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "summary": "经营看板已生成",
            "核心指标": [
                {"name": "GMV", "value": 128000.0, "unit": "CNY"},
                {"name": "订单数", "value": 320, "unit": "orders"}
            ],
            "经营健康度": {
                "score": 82,
                "level": "healthy"
            },
            "下一步动作": ["复盘高转化渠道", "关注退款率"]
        });

        let draft = build_content_artifact_from_skill_output("data_dashboard", &output)
            .expect("data dashboard output should be materializable");

        assert_eq!(draft.content_category, "data");
        assert_eq!(draft.artifact_type, "data_dashboard");
        assert_eq!(draft.title, "Data dashboard");
        assert_eq!(draft.file_name, "data_dashboard.md");
        assert!(draft.preview.contains("经营看板已生成"));
        assert!(draft.content.contains("# Data dashboard"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_data_analysis_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "summary": "漏斗分析完成，瓶颈环节：点击到加购",
            "漏斗分析": [
                {"阶段": "曝光到点击", "转化率": "6.20%"},
                {"阶段": "点击到加购", "转化率": "18.00%"}
            ],
            "优化建议": "改善详情页"
        });

        let draft = build_content_artifact_from_skill_output("data_funnel_analysis", &output)
            .expect("data analysis output should be materializable");

        assert_eq!(draft.content_category, "data");
        assert_eq!(draft.artifact_type, "data_funnel_report");
        assert_eq!(draft.title, "Data funnel report");
        assert_eq!(draft.file_name, "data_funnel_report.md");
        assert!(draft.preview.contains("漏斗分析完成"));
        assert!(draft.content.contains("# Data funnel report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_data_insight_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "异常诊断报告已生成",
            "指标": "GMV",
            "样本数": 7,
            "异常点数": 1,
            "异常详情": [
                {"第几天": 6, "数值": 82000.0, "方向": "激增"}
            ]
        });

        let draft = build_content_artifact_from_skill_output("data_anomaly_diagnosis", &output)
            .expect("data insight output should be materializable");

        assert_eq!(draft.content_category, "data");
        assert_eq!(draft.artifact_type, "data_anomaly_report");
        assert_eq!(draft.title, "Data anomaly report");
        assert_eq!(draft.file_name, "data_anomaly_report.md");
        assert!(draft.preview.contains("异常诊断报告已生成"));
        assert!(draft.content.contains("# Data anomaly report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_data_advanced_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "Demand forecast report generated",
            "metric": "orders",
            "historyPoints": 8,
            "forecast": [
                {"days": 7, "total": 1280.0}
            ]
        });

        let draft = build_content_artifact_from_skill_output("data_demand_forecast", &output)
            .expect("data advanced output should be materializable");

        assert_eq!(draft.content_category, "data");
        assert_eq!(draft.artifact_type, "data_demand_forecast_report");
        assert_eq!(draft.title, "Data demand forecast report");
        assert_eq!(draft.file_name, "data_demand_forecast_report.md");
        assert!(draft.preview.contains("Demand forecast report generated"));
        assert!(draft.content.contains("# Data demand forecast report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_data_comprehensive_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "has_partial_data": true,
            "adapter_status": "partial_manual_input",
            "data_readiness": "partial_user_input",
            "summary": "综合业务诊断完成，健康评分72分，等级需关注",
            "整体健康评分": {"总分": 72, "等级": "需关注"},
            "核心发现": ["经营看板：经营健康度72分，等级需关注"],
            "风险预警": ["广告ROI低于2，建议收缩低效计划并复盘素材"],
            "7天行动计划": [
                {"优先级": "P0", "动作": "补齐缺失数据并重新运行综合诊断"}
            ]
        });

        let draft =
            build_content_artifact_from_skill_output("data_comprehensive_diagnosis", &output)
                .expect("data comprehensive diagnosis output should be materializable");

        assert_eq!(draft.content_category, "data");
        assert_eq!(draft.artifact_type, "data_comprehensive_diagnosis_report");
        assert_eq!(draft.title, "Data comprehensive diagnosis report");
        assert_eq!(draft.file_name, "data_comprehensive_diagnosis_report.md");
        assert!(draft.preview.contains("综合业务诊断完成"));
        assert!(
            draft
                .content
                .contains("# Data comprehensive diagnosis report")
        );
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"partial_manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_store_metrics_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "explicit_metrics",
            "summary": "店铺指标已从本轮显式输入汇总，GMV=180000，订单=1800",
            "platform": "douyin",
            "totals": {
                "gmv": 180000.0,
                "orders": 1800.0,
                "uv": 36000.0,
                "clicks": 7200.0,
                "ad_spend": 36000.0,
                "refund_amount": 5400.0,
                "gross_profit": 54000.0
            },
            "derived_metrics": {
                "conversion_rate": "5.00%",
                "ctr": "20.00%",
                "ad_roi": 5.0,
                "refund_rate": "3.00%"
            },
            "week_over_week": {
                "gmv": {"current": 180000.0, "previous": 150000.0, "change_rate": "20.00%"}
            },
            "anomaly_count": 1
        });

        let draft = build_content_artifact_from_skill_output("query_store_metrics", &output)
            .expect("store metrics output should be materializable");

        assert_eq!(draft.content_category, "data");
        assert_eq!(draft.artifact_type, "data_store_metrics_report");
        assert_eq!(draft.title, "Data store metrics report");
        assert_eq!(draft.file_name, "data_store_metrics_report.md");
        assert!(draft.preview.contains("店铺指标已从本轮显式输入汇总"));
        assert!(draft.content.contains("# Data store metrics report"));
        assert!(
            draft
                .content
                .contains("\"data_readiness\": \"explicit_metrics\"")
        );
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_ignores_deferred_store_metrics() {
        let output = json!({
            "status": "deferred",
            "has_data": false,
            "adapter_status": "store_metrics_required",
            "data_readiness": "missing_user_input",
            "summary": "店铺指标查询等待真实数据输入"
        });

        assert!(build_content_artifact_from_skill_output("query_store_metrics", &output).is_none());
    }

    #[test]
    fn content_artifact_builds_from_product_and_campaign_metrics_skill_output() {
        let product_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "explicit_product_metrics",
            "summary": "商品指标已从本轮显式输入汇总，GMV=96000，订单=1200",
            "query": {
                "product_id": "prod-001",
                "product_name": "冷萃咖啡",
                "sku_id": "sku-a",
                "platform": "抖音"
            },
            "totals": {
                "gmv": 96000.0,
                "orders": 1200.0,
                "uv": 24000.0,
                "ad_spend": 24000.0
            },
            "derived_metrics": {
                "conversion_rate": "5.00%",
                "ad_roi": 4.0
            },
            "answerPolicy": {
                "no_default_product": true,
                "live_platform_lookup": false
            }
        });
        let product_draft =
            build_content_artifact_from_skill_output("query_product_metrics", &product_output)
                .expect("product metrics output should be materializable");

        assert_eq!(product_draft.content_category, "data");
        assert_eq!(product_draft.artifact_type, "data_product_metrics_report");
        assert_eq!(product_draft.title, "Data product metrics report");
        assert_eq!(product_draft.file_name, "data_product_metrics_report.md");
        assert!(
            product_draft
                .preview
                .contains("商品指标已从本轮显式输入汇总")
        );
        assert!(
            product_draft
                .content
                .contains("# Data product metrics report")
        );
        assert!(
            product_draft
                .content
                .contains("\"data_readiness\": \"explicit_product_metrics\"")
        );
        assert!(
            product_draft
                .content
                .contains("\"no_default_product\": true")
        );

        let campaign_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "explicit_campaign_metrics",
            "summary": "活动指标已从本轮显式输入汇总，GMV=210000，花费=42000",
            "query": {
                "campaign_id": "camp-618",
                "campaign_name": "618大促",
                "campaign_type": "promotion",
                "platform": "天猫"
            },
            "totals": {
                "budget": 50000.0,
                "spend": 42000.0,
                "gmv": 210000.0,
                "orders": 1800.0,
                "impressions": 180000.0
            },
            "derived_metrics": {
                "ctr": "4.00%",
                "roas": 5.0
            },
            "answerPolicy": {
                "live_platform_lookup": false
            }
        });
        let campaign_draft =
            build_content_artifact_from_skill_output("query_campaign_metrics", &campaign_output)
                .expect("campaign metrics output should be materializable");

        assert_eq!(campaign_draft.content_category, "data");
        assert_eq!(campaign_draft.artifact_type, "data_campaign_metrics_report");
        assert_eq!(campaign_draft.title, "Data campaign metrics report");
        assert_eq!(campaign_draft.file_name, "data_campaign_metrics_report.md");
        assert!(
            campaign_draft
                .preview
                .contains("活动指标已从本轮显式输入汇总")
        );
        assert!(
            campaign_draft
                .content
                .contains("# Data campaign metrics report")
        );
        assert!(
            campaign_draft
                .content
                .contains("\"data_readiness\": \"explicit_campaign_metrics\"")
        );
    }

    #[test]
    fn content_artifact_ignores_deferred_product_and_campaign_metrics() {
        let product_output = json!({
            "status": "deferred",
            "has_data": false,
            "adapter_status": "product_metrics_required",
            "data_readiness": "missing_user_input",
            "summary": "商品指标查询等待真实数据输入"
        });
        assert!(
            build_content_artifact_from_skill_output("query_product_metrics", &product_output)
                .is_none()
        );

        let campaign_output = json!({
            "status": "deferred",
            "has_data": false,
            "adapter_status": "campaign_metrics_required",
            "data_readiness": "missing_user_input",
            "summary": "活动指标查询等待真实数据输入"
        });
        assert!(
            build_content_artifact_from_skill_output("query_campaign_metrics", &campaign_output)
                .is_none()
        );
    }

    #[test]
    fn content_artifact_builds_from_accounting_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "summary": "利润分析完成，净营收252000元，营业利润92000元",
            "利润分析": {
                "毛利润": 154000.0,
                "营业利润": 92000.0,
                "营业利润率": "36.51%"
            },
            "费用结构": {
                "商品成本": 98000.0,
                "运营费用": 26000.0,
                "广告支出": 36000.0
            }
        });

        let draft = build_content_artifact_from_skill_output("accounting_profit_analysis", &output)
            .expect("accounting output should be materializable");

        assert_eq!(draft.content_category, "accounting");
        assert_eq!(draft.artifact_type, "accounting_profit_report");
        assert_eq!(draft.title, "Accounting profit report");
        assert_eq!(draft.file_name, "accounting_profit_report.md");
        assert!(draft.preview.contains("利润分析完成"));
        assert!(draft.content.contains("# Accounting profit report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );

        let break_even_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "calculated_from_defaults",
            "summary": "盈亏平衡分析完成，需销售433单，需实现营收55424元",
            "盈亏平衡点": {
                "需销售订单数": 433,
                "需实现营收": "¥55424.00"
            }
        });

        let break_even_draft = build_content_artifact_from_skill_output(
            "accounting_break_even_calc",
            &break_even_output,
        )
        .expect("break-even output should be materializable");

        assert_eq!(break_even_draft.content_category, "accounting");
        assert_eq!(
            break_even_draft.artifact_type,
            "accounting_break_even_report"
        );
        assert_eq!(break_even_draft.title, "Accounting break-even report");
        assert_eq!(
            break_even_draft.file_name,
            "accounting_break_even_report.md"
        );
        assert!(break_even_draft.preview.contains("盈亏平衡分析完成"));

        let waterfall_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "calculated_from_defaults",
            "summary": "GMV利润瀑布分析完成，税后净利润¥68000.00",
            "GMV利润瀑布": []
        });

        let waterfall_draft =
            build_content_artifact_from_skill_output("accounting_gmv_waterfall", &waterfall_output)
                .expect("GMV waterfall output should be materializable");

        assert_eq!(
            waterfall_draft.artifact_type,
            "accounting_gmv_waterfall_report"
        );
        assert_eq!(waterfall_draft.title, "Accounting GMV waterfall report");

        let cash_flow_output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "calculated_from_defaults",
            "summary": "3个月现金流预测完成，累计净现金流¥96000.00",
            "3个月现金流预测": []
        });

        let cash_flow_draft = build_content_artifact_from_skill_output(
            "accounting_cash_flow_forecast",
            &cash_flow_output,
        )
        .expect("cash flow output should be materializable");

        assert_eq!(
            cash_flow_draft.artifact_type,
            "accounting_cash_flow_forecast_report"
        );
        assert_eq!(cash_flow_draft.title, "Accounting cash flow forecast");
    }

    #[test]
    fn content_artifact_builds_from_service_support_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "客服升级处理流程已生成",
            "严重度": "S1",
            "主管接管话术": "先承认体验问题，明确接管和下一次同步时间",
            "内部工单模板": {
                "severity": "S1",
                "owner": "客服主管",
                "review_required": true
            }
        });

        let draft = build_content_artifact_from_skill_output("service_escalation_flow", &output)
            .expect("service support output should be materializable");

        assert_eq!(draft.content_category, "service");
        assert_eq!(draft.artifact_type, "service_escalation_playbook");
        assert_eq!(draft.title, "Service escalation playbook");
        assert_eq!(draft.file_name, "service_escalation_playbook.md");
        assert!(draft.preview.contains("客服升级处理流程已生成"));
        assert!(draft.content.contains("# Service escalation playbook"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_service_insight_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "NPS与CSAT分析已完成",
            "NPS净推荐值": 42.0,
            "CSAT满意度": "86.0%（良好）",
            "忠诚度洞察": {
                "转化优先级": "被动者→推荐者成本最低，批评者→中立是当务之急"
            }
        });

        let draft = build_content_artifact_from_skill_output("service_nps_analyzer", &output)
            .expect("service insight output should be materializable");

        assert_eq!(draft.content_category, "service");
        assert_eq!(draft.artifact_type, "service_nps_report");
        assert_eq!(draft.title, "Service NPS report");
        assert_eq!(draft.file_name, "service_nps_report.md");
        assert!(draft.preview.contains("NPS与CSAT分析已完成"));
        assert!(draft.content.contains("# Service NPS report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_ops_core_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "促销策划方案已生成",
            "商品": "冷萃咖啡",
            "总预算": "¥20000.00",
            "活动阶段": [
                {"阶段": "预热期", "预算": "¥3000.00"}
            ],
            "KPI": {
                "目标GMV": "¥100000.00"
            }
        });

        let draft = build_content_artifact_from_skill_output("ops_promo_planning", &output)
            .expect("ops core output should be materializable");

        assert_eq!(draft.content_category, "ops");
        assert_eq!(draft.artifact_type, "ops_promo_plan");
        assert_eq!(draft.title, "Ops promotion plan");
        assert_eq!(draft.file_name, "ops_promo_plan.md");
        assert!(draft.preview.contains("促销策划方案已生成"));
        assert!(draft.content.contains("# Ops promotion plan"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_ops_advanced_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "智能定价方案已生成",
            "推荐定价": "¥98.00",
            "定价方案对比": [
                {"定价": 98.0, "月度总利润": 26000.0}
            ]
        });

        let draft = build_content_artifact_from_skill_output("ops_smart_pricing", &output)
            .expect("ops advanced output should be materializable");

        assert_eq!(draft.content_category, "ops");
        assert_eq!(draft.artifact_type, "ops_smart_pricing_report");
        assert_eq!(draft.title, "Ops smart pricing report");
        assert_eq!(draft.file_name, "ops_smart_pricing_report.md");
        assert!(draft.preview.contains("智能定价方案已生成"));
        assert!(draft.content.contains("# Ops smart pricing report"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_douyin_ads_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "千川计划结构已生成",
            "计划结构": [
                {"层级": "冷启动赛马", "建议预算": "¥6000.00"}
            ]
        });

        let draft = build_content_artifact_from_skill_output("douyin_ads_plan_structure", &output)
            .expect("douyin ads output should be materializable");

        assert_eq!(draft.content_category, "douyin_ads");
        assert_eq!(draft.artifact_type, "douyin_ads_plan_structure");
        assert_eq!(draft.title, "Douyin ads plan structure");
        assert_eq!(draft.file_name, "douyin_ads_plan_structure.md");
        assert!(draft.preview.contains("千川计划结构已生成"));
        assert!(draft.content.contains("# Douyin ads plan structure"));
    }

    #[test]
    fn content_artifact_builds_from_web_growth_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "关键词矩阵已生成",
            "种子词": "冷萃咖啡",
            "关键词矩阵": [
                {"类型": "核心词", "词": ["冷萃咖啡"]}
            ]
        });

        let draft = build_content_artifact_from_skill_output("web_keyword_research", &output)
            .expect("web growth output should be materializable");

        assert_eq!(draft.content_category, "web");
        assert_eq!(draft.artifact_type, "web_keyword_matrix");
        assert_eq!(draft.title, "Web keyword matrix");
        assert_eq!(draft.file_name, "web_keyword_matrix.md");
        assert!(draft.preview.contains("关键词矩阵已生成"));
        assert!(draft.content.contains("# Web keyword matrix"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_engineering_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "工程架构评估报告已生成",
            "系统类型": "电商平台",
            "五维评分": [
                {"维度": "扩展性", "评分": 72}
            ]
        });

        let draft = build_content_artifact_from_skill_output("engineering_arch_review", &output)
            .expect("engineering output should be materializable");

        assert_eq!(draft.content_category, "engineering");
        assert_eq!(draft.artifact_type, "engineering_arch_review_report");
        assert_eq!(draft.title, "Engineering architecture review");
        assert_eq!(draft.file_name, "engineering_arch_review_report.md");
        assert!(draft.preview.contains("工程架构评估报告已生成"));
        assert!(draft.content.contains("# Engineering architecture review"));
        assert!(
            draft
                .content
                .contains("\"adapter_status\": \"manual_input\"")
        );
    }

    #[test]
    fn content_artifact_builds_from_engineering_sla_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "calculated_from_defaults",
            "summary": "SLA监控分析已完成",
            "系统": "电商系统"
        });

        let draft = build_content_artifact_from_skill_output("engineering_sla_monitor", &output)
            .expect("engineering SLA output should be materializable");

        assert_eq!(draft.content_category, "engineering");
        assert_eq!(draft.artifact_type, "engineering_sla_report");
        assert_eq!(draft.title, "Engineering SLA report");
        assert_eq!(draft.file_name, "engineering_sla_report.md");
        assert!(draft.preview.contains("SLA监控分析已完成"));
    }

    #[test]
    fn content_artifact_builds_from_learning_skill_output() {
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": "user_provided",
            "summary": "教学解释已生成：ROI和ROAS的区别",
            "教学主题": "ROI和ROAS的区别",
            "30秒结论": "ROI更偏财务投入产出，ROAS更偏广告投放产出。"
        });

        let draft = build_content_artifact_from_skill_output("learning_teaching_explain", &output)
            .expect("learning teaching output should be materializable");

        assert_eq!(draft.content_category, "learning");
        assert_eq!(draft.artifact_type, "learning_teaching_explain");
        assert_eq!(draft.title, "Learning teaching explain");
        assert_eq!(draft.file_name, "learning_teaching_explain.md");
        assert!(draft.preview.contains("教学解释已生成"));
    }

    #[test]
    fn content_artifact_ignores_non_supported_skill() {
        let output = json!({
            "status": "success",
            "summary": "not supported"
        });

        assert!(build_content_artifact_from_skill_output("ops_unknown_skill", &output).is_none());
    }

    #[test]
    fn campaign_draft_builds_from_coordination_skill_output() {
        let output = json!({
            "status": "success",
            "campaign_creation": {
                "should_persist": true,
                "name": "五一咖啡促销活动",
                "description": "提升五一周末转化率",
                "campaign_type": "promotion",
                "budget": 5000.0,
                "product_id": "11111111-1111-1111-1111-111111111111",
                "goal": "提升五一周末转化率",
                "target_audience": "通勤咖啡用户",
                "channels": ["douyin", "taobao"],
                "strategy": "先小预算测款，再放量高转化素材",
                "workspace_id": "workspace-1",
                "source_summary": "Selected product and workspace context...",
                "draft": {
                    "executionSteps": ["步骤1", "步骤2"]
                }
            }
        });

        let draft = build_campaign_draft_from_skill_output(&output)
            .expect("coordination create campaign output should be materializable");

        assert_eq!(draft.name, "五一咖啡促销活动");
        assert_eq!(draft.campaign_type, "promotion");
        assert_eq!(draft.budget, Some(5000.0));
        assert_eq!(
            draft.source.product_id.as_deref(),
            Some("11111111-1111-1111-1111-111111111111")
        );
        assert!(
            draft
                .output
                .pointer("/planning/channels")
                .and_then(Value::as_array)
                .is_some()
        );
    }

    #[test]
    fn workspace_memory_artifact_ignores_non_persistable_output() {
        let output = json!({
            "status": "success",
            "memory_save": {
                "should_persist": false,
                "key": "temp",
                "content": "仅建议，不落库"
            }
        });

        assert!(build_workspace_memory_from_skill_output(&output).is_none());
    }

    #[test]
    fn artifact_export_renders_markdown_and_metadata() {
        let artifact = test_artifact(WorkspaceArtifactDraft {
            title: "投放复盘报告",
            content: Some("## 结论\n继续保留高 ROI 计划。"),
            preview: Some("报告摘要"),
            format: "markdown",
            storage_kind: "inline",
            file_name: Some("report.txt"),
        });

        let export = build_artifact_export(artifact, Some("markdown".to_string()), true).unwrap();

        assert_eq!(export.format, "markdown");
        assert_eq!(export.file_name, "report.md");
        assert_eq!(export.mime_type, "text/markdown; charset=utf-8");
        assert!(export.content.starts_with("# 投放复盘报告"));
        assert!(export.content.contains("## 结论"));
        assert_eq!(
            export.metadata.get("artifactType").and_then(Value::as_str),
            Some("final_report")
        );
        assert!(export.warnings.is_empty());
    }

    #[test]
    fn artifact_export_uses_preview_when_content_is_empty() {
        let artifact = test_artifact(WorkspaceArtifactDraft {
            title: "search evidence",
            content: None,
            preview: Some("Sources: 3"),
            format: "json",
            storage_kind: "object",
            file_name: None,
        });

        let export = build_artifact_export(artifact, Some("text".to_string()), false).unwrap();

        assert_eq!(export.format, "text");
        assert_eq!(export.file_name, "search_evidence.txt");
        assert!(export.content.contains("Sources: 3"));
        assert!(
            export
                .warnings
                .iter()
                .any(|warning| warning.contains("preview"))
        );
        assert!(
            export
                .warnings
                .iter()
                .any(|warning| warning.contains("object storage content was not read"))
        );
        assert_eq!(export.metadata, json!({}));
    }

    #[test]
    fn artifact_export_rejects_unsupported_format() {
        let err = resolve_artifact_export_format(Some("docx".to_string()), "markdown")
            .expect_err("unsupported format should fail");

        assert_eq!(err.code(), app_error::WORKSPACE_ARTIFACT_REQUEST_INVALID);
    }

    #[derive(Debug, Clone)]
    struct WorkspaceArtifactDraft {
        title: &'static str,
        content: Option<&'static str>,
        preview: Option<&'static str>,
        format: &'static str,
        storage_kind: &'static str,
        file_name: Option<&'static str>,
    }

    fn test_artifact(draft: WorkspaceArtifactDraft) -> WorkspaceArtifact {
        let now = OffsetDateTime::from_unix_timestamp(1_762_646_400).unwrap();
        WorkspaceArtifact {
            id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            workspace_id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
            run_id: Some(Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap()),
            workspace_task_id: None,
            run_task_id: None,
            user_id: Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddddd").unwrap(),
            artifact_type: "final_report".to_string(),
            title: draft.title.to_string(),
            content: draft.content.map(ToString::to_string),
            preview: draft.preview.map(ToString::to_string),
            mime_type: Some("text/markdown".to_string()),
            format: draft.format.to_string(),
            storage_kind: draft.storage_kind.to_string(),
            bucket: None,
            object_key: None,
            file_name: draft.file_name.map(ToString::to_string),
            file_size: None,
            sha256: None,
            source: "agent_session".to_string(),
            status: WorkspaceArtifactStatus::Active,
            validation_status: WorkspaceArtifactValidationStatus::Unchecked,
            validation_report: json!({}),
            metadata: json!({ "kind": "test" }),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn context_summary_counts_total_sources() {
        let counts = WorkspaceContextSummaryCounts {
            campaigns: 1,
            conversation_files: 1,
            product_assets: 2,
            product_materials: 3,
            workspace_artifacts: 4,
        };

        assert_eq!(counts.total(), 11);
    }

    #[test]
    fn context_summary_text_reports_empty_state() {
        let counts = WorkspaceContextSummaryCounts::default();

        assert_eq!(
            build_context_summary_text(&counts, "empty"),
            "No active workspace context sources are ready for agent use."
        );
    }

    #[test]
    fn context_ref_keys_preserve_source_order() {
        let source = |source_type: &str, id: &str| WorkspaceContextSourceResponse {
            source_type: source_type.to_string(),
            id: id.to_string(),
            title: None,
            mime_type: None,
            file_size: None,
            status: "active".to_string(),
            ref_key: format!("{source_type}:{id}"),
            preview: None,
            metadata: json!({}),
        };
        let attachments = vec![source("conversation_file", "a")];
        let campaigns = vec![source("campaign", "campaign-a")];
        let assets = vec![source("product_asset", "b")];
        let materials = vec![source("product_material", "c")];
        let artifacts = vec![source("workspace_artifact", "d")];

        assert_eq!(
            context_ref_keys(&attachments, &campaigns, &assets, &materials, &artifacts),
            vec![
                "conversation_file:a".to_string(),
                "campaign:campaign-a".to_string(),
                "product_asset:b".to_string(),
                "product_material:c".to_string(),
                "workspace_artifact:d".to_string(),
            ]
        );
    }

    #[test]
    fn context_pack_search_evidence_uses_readable_summary() {
        let mut artifact = test_artifact(WorkspaceArtifactDraft {
            title: "Search evidence - 五一咖啡组合",
            content: Some(
                r#"{
                    "status": "live",
                    "query": {
                        "planned": "五一咖啡组合 竞品",
                        "product_ref": "product:11111111-1111-1111-1111-111111111111",
                        "product_name": "五一咖啡组合",
                        "product_category": "咖啡饮品"
                    },
                    "sources": [
                        {
                            "title": "竞品价格观察",
                            "url": "https://example.com/a",
                            "domain": "example.com",
                            "snippet": "同类组合装集中在 89-129 元。"
                        }
                    ],
                    "provenance": {
                        "source_count": 1,
                        "domains": ["example.com"]
                    },
                    "confidence": 0.82,
                    "evidence": {
                        "summary": "共 1 条实时来源，覆盖 1 个域名。",
                        "llm_context": "实时来源摘要"
                    }
                }"#,
            ),
            preview: Some("Query: 五一咖啡组合 竞品"),
            format: "json",
            storage_kind: "inline",
            file_name: Some("search_evidence.json"),
        });
        artifact.artifact_type = "search_evidence".to_string();
        artifact.metadata = json!({
            "kind": "agent_search_evidence",
            "sourceCount": 1,
            "confidence": 0.82
        });

        let candidate = context_pack_artifact_candidate(artifact);

        assert_eq!(candidate.content_kind, "search_evidence_summary");
        let text = candidate
            .text
            .expect("search evidence summary should be readable");
        assert!(text.contains("Search evidence summary"));
        assert!(text.contains("Query: 五一咖啡组合 竞品"));
        assert!(text.contains(
            "Product: 五一咖啡组合 / 咖啡饮品 / product:11111111-1111-1111-1111-111111111111"
        ));
        assert!(text.contains("Evidence: 共 1 条实时来源"));
        assert!(text.contains("Metrics: sources=1; confidence=0.82; domains=example.com"));
        assert!(text.contains("Source 1: 竞品价格观察"));
        assert!(!text.contains("\"sources\""));
    }

    #[test]
    fn context_pack_search_evidence_falls_back_to_preview_when_json_invalid() {
        let mut artifact = test_artifact(WorkspaceArtifactDraft {
            title: "Search evidence - invalid".into(),
            content: Some("{not json"),
            preview: Some("Query: fallback preview\nSources: 1"),
            format: "json",
            storage_kind: "inline",
            file_name: Some("search_evidence.json"),
        });
        artifact.artifact_type = "search_evidence".to_string();
        artifact.metadata = json!({ "kind": "agent_search_evidence" });

        let candidate = context_pack_artifact_candidate(artifact);

        assert_eq!(candidate.content_kind, "search_evidence_summary");
        assert_eq!(
            candidate.text.as_deref(),
            Some("Query: fallback preview\nSources: 1")
        );
    }

    #[test]
    fn explicit_workspace_context_refs_parse_supported_forms() {
        let artifact_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let campaign_id = Uuid::parse_str("eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee").unwrap();
        let file_id = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let asset_id = Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap();
        let material_id = Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddddd").unwrap();
        let refs = explicit_workspace_context_refs(vec![
            format!("workspace_artifact:{artifact_id}"),
            format!("artifact/{artifact_id}"),
            format!("conversation_file:{file_id}"),
            format!("file/{file_id}"),
            format!("product_asset:{asset_id}"),
            format!("asset/{asset_id}"),
            format!("product_material:{material_id}"),
            format!("material/{material_id}"),
            format!("campaign:{campaign_id}"),
            format!("campaign/{campaign_id}"),
            format!("product:{artifact_id}"),
            format!("workspace_artifact:{artifact_id}"),
        ])
        .expect("workspace context refs should parse");

        assert_eq!(refs.workspace_artifact_ids, vec![artifact_id]);
        assert_eq!(refs.conversation_file_ids, vec![file_id]);
        assert_eq!(refs.product_asset_ids, vec![asset_id]);
        assert_eq!(refs.product_material_ids, vec![material_id]);
        assert_eq!(refs.campaign_ids, vec![campaign_id]);
    }

    #[test]
    fn explicit_workspace_context_refs_reject_invalid_uuid() {
        let err = explicit_workspace_context_refs(vec!["product_material:not-a-uuid".to_string()])
            .expect_err("invalid artifact ref should fail");

        assert_eq!(err.code(), app_error::WORKSPACE_REQUEST_INVALID);
    }

    #[test]
    fn explicit_workspace_context_refs_reject_too_many_refs() {
        let refs = (0 .. 21).map(|index| format!("artifact:{index}")).collect();
        let err = explicit_workspace_context_refs(refs).expect_err("too many refs should fail");

        assert_eq!(err.code(), app_error::WORKSPACE_REQUEST_INVALID);
    }

    #[test]
    fn context_pack_prefers_readable_items_and_truncates_text() {
        let candidates = vec![
            WorkspaceContextPackCandidate {
                source_type: "conversation_file".to_string(),
                id: "file-1".to_string(),
                title: Some("brief.pdf".to_string()),
                ref_key: "conversation_file:file-1".to_string(),
                content_kind: "metadata_only".to_string(),
                text: None,
                extraction_status: Some("pending".to_string()),
                metadata: json!({}),
            },
            WorkspaceContextPackCandidate {
                source_type: "product_material".to_string(),
                id: "material-1".to_string(),
                title: Some("卖点".to_string()),
                ref_key: "product_material:material-1".to_string(),
                content_kind: "content".to_string(),
                text: Some("咖啡新品主打低糖和冷萃场景。".repeat(20)),
                extraction_status: None,
                metadata: json!({}),
            },
        ];

        let pack = build_context_pack_items(candidates, 2, 24);

        assert_eq!(pack.items[0].ref_key, "product_material:material-1");
        assert_eq!(pack.readable_items, 1);
        assert!(pack.truncated);
        assert!(
            pack.items[0]
                .text
                .as_deref()
                .is_some_and(|text| text.ends_with("..."))
        );
        assert_eq!(pack.items[1].content_kind, "metadata_only");
        assert!(pack.items[1].text.is_none());
    }

    #[test]
    fn context_pack_uses_attachment_text_only_from_safe_metadata() {
        let metadata = json!({
            "extraction": {
                "status": "ready",
                "summary": "附件里写了活动预算和目标 ROI。"
            }
        });

        assert_eq!(
            conversation_file_extracted_text(&metadata),
            Some("附件里写了活动预算和目标 ROI。".to_string())
        );
        assert!(conversation_file_extracted_text(&json!({"objectKey": "raw.pdf"})).is_none());
    }

    #[test]
    fn context_pack_respects_attachment_extraction_status() {
        let pending = context_pack_conversation_file_candidate(ConversationFile {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            message_id: None,
            bucket: "private".to_string(),
            object_key: "chat/u/c/file.pdf".to_string(),
            file_name: Some("file.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            file_size: Some(128),
            sha256: None,
            visibility: "private".to_string(),
            storage_provider: "s3".to_string(),
            status: "active".to_string(),
            metadata: json!({
                "extractionStatus": "pending",
                "extraction": {
                    "status": "pending",
                    "text": "不应被读取"
                }
            }),
            created_at: OffsetDateTime::now_utc(),
            deleted_at: None,
        });
        assert_eq!(pending.content_kind, "extraction_pending");
        assert!(pending.text.is_none());

        let failed = context_pack_conversation_file_candidate(ConversationFile {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            message_id: None,
            bucket: "private".to_string(),
            object_key: "chat/u/c/file.pdf".to_string(),
            file_name: Some("file.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            file_size: Some(128),
            sha256: None,
            visibility: "private".to_string(),
            storage_provider: "s3".to_string(),
            status: "active".to_string(),
            metadata: json!({
                "extraction": {
                    "status": "failed",
                    "failureReason": "encrypted"
                }
            }),
            created_at: OffsetDateTime::now_utc(),
            deleted_at: None,
        });
        assert_eq!(failed.content_kind, "extraction_failed");
        assert!(failed.text.is_none());

        let extracted = context_pack_conversation_file_candidate(ConversationFile {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            message_id: None,
            bucket: "private".to_string(),
            object_key: "chat/u/c/file.pdf".to_string(),
            file_name: Some("file.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            file_size: Some(128),
            sha256: None,
            visibility: "private".to_string(),
            storage_provider: "s3".to_string(),
            status: "active".to_string(),
            metadata: json!({
                "extractionStatus": "extracted",
                "extraction": {
                    "status": "extracted",
                    "text": "可读正文"
                }
            }),
            created_at: OffsetDateTime::now_utc(),
            deleted_at: None,
        });
        assert_eq!(extracted.content_kind, "extracted_text");
        assert_eq!(extracted.text, Some("可读正文".to_string()));
    }

    #[test]
    fn context_pack_summary_reports_readable_and_omitted_counts() {
        let counts = WorkspaceContextSummaryCounts {
            campaigns: 1,
            conversation_files: 1,
            product_assets: 1,
            product_materials: 2,
            workspace_artifacts: 1,
        };

        let summary = build_context_pack_summary_text(&counts, "partial", 3, 1200, 2);

        assert!(summary.contains("partial"));
        assert!(summary.contains("3 readable"));
        assert!(summary.contains("2 omitted"));
    }

    #[test]
    fn campaign_context_pack_candidate_includes_operational_context() {
        let campaign = Campaign {
            id: Uuid::parse_str("eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee").unwrap(),
            user_id: Uuid::new_v4(),
            product_id: Some(Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()),
            name: "618 campaign".to_string(),
            campaign_type: Some("promotion".to_string()),
            description: Some("Drive preorder conversion".to_string()),
            budget: 3000.0,
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-06-18".to_string()),
            status: CampaignStatus::Active,
            metadata: json!({ "goal": "roi 2.0" }),
            output: json!({ "plan": "short video plus live room" }),
            action_metadata: json!({ "executionStatus": "not_submitted" }),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        let candidate = context_pack_campaign_candidate(campaign);

        assert_eq!(
            candidate.ref_key,
            "campaign:eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"
        );
        let text = candidate.text.expect("campaign context should be readable");
        assert!(text.contains("618 campaign"));
        assert!(text.contains("Budget: 3000"));
        assert!(text.contains("Product ref: product:11111111-1111-1111-1111-111111111111"));
        assert!(text.contains("Saved output"));
    }
}
