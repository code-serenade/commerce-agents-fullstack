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
    agent_trust_scores::{
        AgentTrustLevel, AgentTrustScore, AgentTrustScoreFilter, AgentTrustScoreService,
        CreateAgentTrustScore, UpdateAgentTrustScore,
    },
    app_users::{AppUserService, AppUserStatus},
    audit_logs::{AuditLogService, CreateAuditLog},
    campaigns::CampaignService,
    execution_queue::{
        CreateExecutionQueueItem, ExecutionQueueFilter, ExecutionQueueItem, ExecutionQueueService,
        ExecutionQueueStatus, UpdateExecutionQueueItem,
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

use crate::dto::workspace::*;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;
const AGENT_WORKSPACE_TYPE: &str = "agent_chat";
const AGENT_RUN_SOURCE: &str = "agent_session";
const AGENT_RUN_TITLE: &str = "Agent chat workspace";

pub struct WorkspaceApi {
    db: DbContext,
    app_user_svc: AppUserService,
    product_svc: ProductService,
    campaign_svc: CampaignService,
    workspace_svc: WorkspaceService,
    task_svc: WorkspaceTaskService,
    run_svc: WorkspaceRunService,
    run_task_svc: WorkspaceRunTaskService,
    artifact_svc: WorkspaceArtifactService,
    execution_svc: ExecutionQueueService,
    quality_svc: AgentQualityCheckService,
    trust_svc: AgentTrustScoreService,
    learning_svc: AgentLearningRecordService,
}

impl WorkspaceApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            app_user_svc: AppUserService::new(db.clone()),
            product_svc: ProductService::new(db.clone()),
            campaign_svc: CampaignService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            task_svc: WorkspaceTaskService::new(db.clone()),
            run_svc: WorkspaceRunService::new(db.clone()),
            run_task_svc: WorkspaceRunTaskService::new(db.clone()),
            artifact_svc: WorkspaceArtifactService::new(db.clone()),
            execution_svc: ExecutionQueueService::new(db.clone()),
            quality_svc: AgentQualityCheckService::new(db.clone()),
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
        let role = normalize_optional(req.role).unwrap_or_else(|| "agent".to_string());
        let limit = req.limit.unwrap_or(5).clamp(1, 10);
        let records = self
            .learning_svc
            .list(AgentLearningRecordFilter {
                user_id,
                workspace_id: None,
                run_id: None,
                quality_check_id: None,
                role: Some(role.clone()),
                category: None,
                status: Some(AgentLearningRecordStatus::Active),
                limit: Some(limit),
                offset: Some(0),
            })
            .await?;
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

fn evaluate_output_quality(
    input: &str,
    output: &str,
    tasks: &[WorkspaceRunTask],
) -> QualityEvaluationDraft {
    let output_chars = output.chars().count();
    let input_chars = input.chars().count();
    let lower = output.to_ascii_lowercase();
    let completeness = if output_chars == 0 {
        0.0
    } else if output_chars >= 800 {
        0.95
    } else if output_chars >= 300 {
        0.82
    } else if output_chars >= 120 {
        0.65
    } else {
        0.35
    };
    let actionability = keyword_score(
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
    );
    let relevance = relevance_score(input, output);
    let clarity = {
        let has_structure = output.contains('\n') || output.contains("1.") || output.contains("- ");
        let sentence_count = output.matches('。').count()
            + output.matches('.').count()
            + output.matches('\n').count();
        let base: f64 = if has_structure { 0.78 } else { 0.55 };
        (base + (sentence_count as f64 * 0.025)).clamp(0.2_f64, 1.0_f64)
    };
    let risk_awareness = keyword_score(
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
    );
    let value_density = {
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
    };
    let confidence_marking = keyword_score(
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
    );
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
                "Answer quality was acceptable; continue improving clarity and actionability for goal: {}",
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
            "基于现有信息，建议先按用户、运营、转化和成本四个维度复盘。\n\
             1. 数据：检查曝光、点击、转化、客单价和退款率，确认主要问题。\n\
             2. 执行：优先调整预算和素材，下一步把低转化计划降预算。\n\
             3. 风险：当前数据可能受活动周期影响，需确认平台口径和样本量。\n\
             4. 智能体岗位：运营负责节奏，数据岗位负责指标，创意岗位负责素材方案。\n\
             这是一份可执行方案，可以先验证三天后再复盘。",
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
    fn trust_ema_moves_previous_score_toward_quality_score() {
        let updated = trust_ema(0.5, 0.8);

        assert!((updated - 0.59).abs() < 0.0001);
    }
}
