use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    audit_logs::{AuditLogService, CreateAuditLog},
    campaigns::CampaignService,
    execution_queue::{
        CreateExecutionQueueItem, ExecutionQueueFilter, ExecutionQueueItem, ExecutionQueueService,
        ExecutionQueueStatus, UpdateExecutionQueueItem,
    },
    products::{ProductService, ProductStatus},
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
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::workspace::*;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct WorkspaceApi {
    db: DbContext,
    app_user_svc: AppUserService,
    product_svc: ProductService,
    campaign_svc: CampaignService,
    workspace_svc: WorkspaceService,
    task_svc: WorkspaceTaskService,
    run_svc: WorkspaceRunService,
    run_task_svc: WorkspaceRunTaskService,
    execution_svc: ExecutionQueueService,
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
            execution_svc: ExecutionQueueService::new(db),
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
