use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    audit_logs::{AuditLog, AuditLogFilter, AuditLogService, CreateAuditLog},
    conversations::ConversationService,
    execution_queue::{
        CreateExecutionQueueItem, ExecutionQueueItem, ExecutionQueueService, ExecutionQueueStatus,
        UpdateExecutionQueueItem,
    },
    platform_connections::{PlatformConnection, PlatformConnectionService},
    tool_approval_queue::{
        CreateToolApproval, ToolApproval, ToolApprovalFilter, ToolApprovalQueueService,
        ToolApprovalStatus, UpdateToolApprovalDecision,
    },
    workspace_runs::WorkspaceRunService,
    workspaces::{WorkspaceService, WorkspaceStatus},
};
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

use super::platform_execution::{
    PlatformExecutionAdapterOutcome, PlatformExecutionParams, PlatformExecutionReadiness,
    is_platform_execution_skill, platform_execution_details, platform_execution_dispatch_outcome,
    platform_execution_params, platform_execution_readiness_status, supported_platform_action,
};
use crate::dto::approval_audit::*;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct ApprovalAuditApi {
    db: DbContext,
    app_user_svc: AppUserService,
    workspace_svc: WorkspaceService,
    run_svc: WorkspaceRunService,
    conversation_svc: ConversationService,
    approval_svc: ToolApprovalQueueService,
    platform_svc: PlatformConnectionService,
    audit_svc: AuditLogService,
}

impl ApprovalAuditApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            run_svc: WorkspaceRunService::new(db.clone()),
            conversation_svc: ConversationService::new(db.clone()),
            approval_svc: ToolApprovalQueueService::new(db.clone()),
            platform_svc: PlatformConnectionService::new(db.clone()),
            audit_svc: AuditLogService::new(db.clone()),
            db,
        }
    }

    pub async fn list_tool_approvals(
        &self,
        req: ListToolApprovalsRequest,
    ) -> BizResult<ToolApprovalListResponse> {
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
                    app_error::APPROVAL_AUDIT_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = ToolApprovalFilter {
            user_id,
            workspace_id,
            run_id,
            status: parse_tool_approval_status(req.status)?,
            skill_name: normalize_optional(req.skill_name),
            risk_level: normalize_optional(req.risk_level).map(normalize_risk_level),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = ToolApprovalFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.approval_svc.count(count_filter).await?;
        let items = self
            .approval_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_tool_approval)
            .collect();
        Ok(ToolApprovalListResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_tool_approval(
        &self,
        req: CreateToolApprovalRequest,
    ) -> BizResult<ToolApprovalResponse> {
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
                    app_error::APPROVAL_AUDIT_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let conversation_id = parse_optional_uuid(req.conversation_id, "conversation_id")?;
        if let Some(conversation_id) = conversation_id {
            self.ensure_conversation_owner(user_id, conversation_id)
                .await?;
        }
        let action_key = normalize_optional(req.action_key);
        if let Some(action_key) = action_key.as_deref()
            && let Some(existing) = self.approval_svc.get_by_action_key(action_key).await?
        {
            if existing.user_id == user_id {
                return Ok(map_tool_approval(existing));
            }
            return Err(BizError::new(
                app_error::TOOL_APPROVAL_FORBIDDEN,
                "tool approval action key belongs to another user".to_string(),
            ));
        }
        let skill_name = normalize_required(req.skill_name, "skill_name")?;
        let args = req.args.unwrap_or_else(|| json!({}));
        let initial_result = self
            .tool_approval_initial_result(user_id, skill_name.as_str(), &args)
            .await?;
        let approval = self
            .approval_svc
            .create(CreateToolApproval {
                id: Uuid::new_v4(),
                user_id,
                workspace_id,
                run_id,
                conversation_id,
                request_role: normalize_optional(req.request_role),
                skill_name,
                tool_call_id: normalize_optional(req.tool_call_id),
                risk_level: normalize_risk_level(
                    req.risk_level.unwrap_or_else(|| "L1".to_string()),
                ),
                args,
                status: ToolApprovalStatus::Pending,
                reason: normalize_optional(req.reason),
                action_key,
                result: initial_result,
                approved_by: None,
                approved_at: None,
                executed_at: None,
            })
            .await?;
        Ok(map_tool_approval(approval))
    }

    pub async fn get_tool_approval(
        &self,
        req: GetToolApprovalRequest,
    ) -> BizResult<ToolApprovalResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let approval_id = parse_uuid(&req.approval_id, "approval_id")?;
        let approval = self
            .ensure_tool_approval_owner(user_id, approval_id)
            .await?;
        Ok(map_tool_approval(approval))
    }

    pub async fn approve_tool_approval(
        &self,
        req: DecideToolApprovalRequest,
    ) -> BizResult<ToolApprovalResponse> {
        self.decide_tool_approval(req, ToolApprovalStatus::Approved)
            .await
    }

    pub async fn reject_tool_approval(
        &self,
        req: DecideToolApprovalRequest,
    ) -> BizResult<ToolApprovalResponse> {
        self.decide_tool_approval(req, ToolApprovalStatus::Rejected)
            .await
    }

    pub async fn list_audit_logs(
        &self,
        req: ListAuditLogsRequest,
    ) -> BizResult<AuditLogListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_optional_uuid(req.workspace_id, "workspace_id")?;
        if let Some(workspace_id) = workspace_id {
            self.ensure_workspace_owner(user_id, workspace_id).await?;
        }
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::APPROVAL_AUDIT_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = AuditLogFilter {
            user_id,
            actor_user_id: parse_optional_uuid(req.actor_user_id, "actor_user_id")?,
            workspace_id,
            run_id,
            execution_queue_id: parse_optional_uuid(req.execution_queue_id, "execution_queue_id")?,
            tool_approval_id: parse_optional_uuid(req.tool_approval_id, "tool_approval_id")?,
            action: normalize_optional(req.action),
            target_type: normalize_optional(req.target_type),
            status: normalize_optional(req.status),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = AuditLogFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.audit_svc.count(count_filter).await?;
        let logs = self
            .audit_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_audit_log)
            .collect();
        Ok(AuditLogListResponse {
            logs,
            total,
            page,
            page_size,
        })
    }

    pub async fn create_audit_log(
        &self,
        req: CreateAuditLogRequest,
    ) -> BizResult<AuditLogResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let actor_user_id = req
            .actor_user_id
            .map(|value| parse_uuid(&value, "actor_user_id"))
            .transpose()?
            .unwrap_or(user_id);
        if actor_user_id != user_id {
            self.ensure_app_user_enabled(actor_user_id).await?;
        }
        let workspace_id = parse_optional_uuid(req.workspace_id, "workspace_id")?;
        if let Some(workspace_id) = workspace_id {
            self.ensure_workspace_owner(user_id, workspace_id).await?;
        }
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        if let Some(run_id) = run_id {
            let workspace_id = workspace_id.ok_or_else(|| {
                BizError::new(
                    app_error::APPROVAL_AUDIT_REQUEST_INVALID,
                    "workspace_id is required when run_id is provided".to_string(),
                )
            })?;
            self.ensure_run_owner(user_id, workspace_id, run_id).await?;
        }
        let execution_queue_id = parse_optional_uuid(req.execution_queue_id, "execution_queue_id")?;
        let tool_approval_id = parse_optional_uuid(req.tool_approval_id, "tool_approval_id")?;
        if let Some(tool_approval_id) = tool_approval_id {
            self.ensure_tool_approval_owner(user_id, tool_approval_id)
                .await?;
        }
        let log = self
            .audit_svc
            .create(CreateAuditLog {
                id: Uuid::new_v4(),
                user_id,
                actor_user_id,
                workspace_id,
                run_id,
                execution_queue_id,
                tool_approval_id,
                action: normalize_required(req.action, "action")?,
                target_type: normalize_required(req.target_type, "target_type")?,
                target_id: normalize_optional(req.target_id),
                status: normalize_optional(req.status).unwrap_or_else(|| "success".to_string()),
                reason: normalize_optional(req.reason),
                request_id: normalize_optional(req.request_id),
                trace_id: normalize_optional(req.trace_id),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
            })
            .await?;
        Ok(map_audit_log(log))
    }

    async fn decide_tool_approval(
        &self,
        req: DecideToolApprovalRequest,
        status: ToolApprovalStatus,
    ) -> BizResult<ToolApprovalResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let approval_id = parse_uuid(&req.approval_id, "approval_id")?;
        let current = self
            .ensure_tool_approval_owner(user_id, approval_id)
            .await?;
        if current.status != ToolApprovalStatus::Pending {
            return Err(BizError::new(
                app_error::APPROVAL_AUDIT_REQUEST_INVALID,
                "tool approval request is not pending".to_string(),
            ));
        }
        let now = OffsetDateTime::now_utc();
        let result = req.result.unwrap_or_else(|| json!({}));
        let reason = normalize_optional(req.reason);
        let action = match status {
            ToolApprovalStatus::Approved => "tool_approval.approve",
            ToolApprovalStatus::Rejected => "tool_approval.reject",
            _ => "tool_approval.decide",
        };
        let tx = self.db.begin().await?;
        let approval_tx_svc = ToolApprovalQueueService::new(tx.clone());
        let execution_tx_svc = ExecutionQueueService::new(tx.clone());
        let platform_tx_svc = PlatformConnectionService::new(tx.clone());
        let audit_tx_svc = AuditLogService::new(tx.clone());
        let update_result: BizResult<ToolApproval> = async {
            let approval = if status == ToolApprovalStatus::Approved {
                self.execute_tool_approval_after_approve(
                    &approval_tx_svc,
                    &execution_tx_svc,
                    &platform_tx_svc,
                    &audit_tx_svc,
                    &current,
                    ApprovalExecutionContext {
                        actor_user_id: user_id,
                        now,
                        decision_reason: reason.clone(),
                        manual_result: result.clone(),
                    },
                )
                .await?
            } else {
                let approval = approval_tx_svc
                    .update_decision(UpdateToolApprovalDecision {
                        id: current.id,
                        status,
                        reason: reason.clone(),
                        result: result.clone(),
                        approved_by: Some(user_id),
                        approved_at: Some(now),
                        executed_at: None,
                    })
                    .await?;
                audit_tx_svc
                    .create(CreateAuditLog {
                        id: Uuid::new_v4(),
                        user_id: current.user_id,
                        actor_user_id: user_id,
                        workspace_id: current.workspace_id,
                        run_id: current.run_id,
                        execution_queue_id: None,
                        tool_approval_id: Some(current.id),
                        action: action.to_string(),
                        target_type: "tool_approval".to_string(),
                        target_id: Some(current.id.to_string()),
                        status: "success".to_string(),
                        reason,
                        request_id: None,
                        trace_id: None,
                        metadata: json!({
                            "skillName": current.skill_name,
                            "toolCallId": current.tool_call_id,
                            "riskLevel": current.risk_level,
                            "actionKey": current.action_key,
                            "decisionStatus": status.as_str(),
                            "result": result,
                        }),
                    })
                    .await?;
                approval
            };
            Ok(approval)
        }
        .await;
        let approval = match update_result {
            Ok(approval) => {
                tx.commit().await?;
                approval
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };
        Ok(map_tool_approval(approval))
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

    async fn ensure_workspace_owner(&self, user_id: Uuid, workspace_id: Uuid) -> BizResult<()> {
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
        Ok(())
    }

    async fn ensure_active_workspace_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> BizResult<()> {
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
        Ok(())
    }

    async fn ensure_run_owner(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        run_id: Uuid,
    ) -> BizResult<()> {
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
        Ok(())
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
        Ok(())
    }

    async fn ensure_tool_approval_owner(
        &self,
        user_id: Uuid,
        approval_id: Uuid,
    ) -> BizResult<ToolApproval> {
        let approval = self
            .approval_svc
            .get_by_id(approval_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::TOOL_APPROVAL_NOT_FOUND,
                    "tool approval request not found".to_string(),
                )
            })?;
        if approval.user_id != user_id {
            return Err(BizError::new(
                app_error::TOOL_APPROVAL_FORBIDDEN,
                "tool approval request does not belong to current user".to_string(),
            ));
        }
        Ok(approval)
    }

    async fn tool_approval_initial_result(
        &self,
        user_id: Uuid,
        skill_name: &str,
        args: &Value,
    ) -> BizResult<Value> {
        if !is_platform_execution_skill(skill_name) {
            return Ok(json!({}));
        }
        let now = OffsetDateTime::now_utc();
        let params = platform_execution_params(args);
        let connection = match params.platform.as_deref() {
            Some(platform) if supported_platform_action(skill_name) => {
                self.platform_svc
                    .get_by_user_platform(user_id, platform)
                    .await?
            }
            _ => None,
        };
        Ok(json!({
            "status": "pending",
            "source": "tool_approval_create",
            "executionReadiness": plan_platform_execution_readiness(
                skill_name,
                &params,
                connection.as_ref(),
                now,
            ),
        }))
    }

    async fn execute_tool_approval_after_approve(
        &self,
        approval_tx_svc: &ToolApprovalQueueService,
        execution_tx_svc: &ExecutionQueueService,
        platform_tx_svc: &PlatformConnectionService,
        audit_tx_svc: &AuditLogService,
        current: &ToolApproval,
        ctx: ApprovalExecutionContext,
    ) -> BizResult<ToolApproval> {
        let ApprovalExecutionContext {
            actor_user_id,
            now,
            decision_reason,
            manual_result,
        } = ctx;
        let approved_reason = decision_reason.clone().or_else(|| current.reason.clone());
        let approved = approval_tx_svc
            .update_decision(UpdateToolApprovalDecision {
                id: current.id,
                status: ToolApprovalStatus::Approved,
                reason: approved_reason.clone(),
                result: json!({
                    "status": "approved",
                    "source": "tool_approval_approve",
                    "manualResult": manual_result,
                }),
                approved_by: Some(actor_user_id),
                approved_at: Some(now),
                executed_at: None,
            })
            .await?;
        audit_tx_svc
            .create(CreateAuditLog {
                id: Uuid::new_v4(),
                user_id: current.user_id,
                actor_user_id,
                workspace_id: current.workspace_id,
                run_id: current.run_id,
                execution_queue_id: None,
                tool_approval_id: Some(current.id),
                action: "tool_approval.approve".to_string(),
                target_type: "tool_approval".to_string(),
                target_id: Some(current.id.to_string()),
                status: "success".to_string(),
                reason: approved_reason.clone(),
                request_id: current.tool_call_id.clone(),
                trace_id: None,
                metadata: json!({
                    "skillName": current.skill_name,
                    "toolCallId": current.tool_call_id,
                    "riskLevel": current.risk_level,
                    "actionKey": current.action_key,
                    "decisionStatus": ToolApprovalStatus::Approved.as_str(),
                    "manualResult": manual_result,
                }),
            })
            .await?;

        let params = platform_execution_params(&current.args);
        let execution_item = execution_tx_svc
            .create(CreateExecutionQueueItem {
                id: Uuid::new_v4(),
                user_id: current.user_id,
                workspace_id: current.workspace_id,
                run_id: current.run_id,
                workspace_task_id: None,
                action_type: current.skill_name.clone(),
                description: build_execution_description(current, &params),
                platform: params.platform.clone(),
                payload: current.args.clone(),
                expected_impact: None,
                status: ExecutionQueueStatus::Approved,
                decision_reason: approved_reason.clone(),
                result: json!({
                    "status": "approved",
                    "source": "tool_approval",
                    "approvalId": current.id,
                    "skillName": current.skill_name,
                }),
                decided_at: Some(now),
            })
            .await?;
        audit_tx_svc
            .create(CreateAuditLog {
                id: Uuid::new_v4(),
                user_id: current.user_id,
                actor_user_id,
                workspace_id: current.workspace_id,
                run_id: current.run_id,
                execution_queue_id: Some(execution_item.id),
                tool_approval_id: Some(current.id),
                action: "execution_queue.create_from_tool_approval".to_string(),
                target_type: "execution_queue".to_string(),
                target_id: Some(execution_item.id.to_string()),
                status: "success".to_string(),
                reason: approved_reason.clone(),
                request_id: current.tool_call_id.clone(),
                trace_id: None,
                metadata: json!({
                    "approvalId": current.id,
                    "skillName": current.skill_name,
                    "platform": params.platform,
                    "actionKey": current.action_key,
                }),
            })
            .await?;

        let attempt = attempt_platform_execution(
            platform_tx_svc,
            current,
            &execution_item,
            &params,
            now,
            &manual_result,
        )
        .await?;

        let final_execution = execution_tx_svc
            .update(UpdateExecutionQueueItem {
                id: execution_item.id,
                status: attempt.execution_status,
                decision_reason: attempt.decision_reason.clone(),
                result: attempt.result.clone(),
                decided_at: Some(now),
            })
            .await?;
        audit_tx_svc
            .create(CreateAuditLog {
                id: Uuid::new_v4(),
                user_id: current.user_id,
                actor_user_id,
                workspace_id: current.workspace_id,
                run_id: current.run_id,
                execution_queue_id: Some(execution_item.id),
                tool_approval_id: Some(current.id),
                action: attempt.execution_audit_action.to_string(),
                target_type: "execution_queue".to_string(),
                target_id: Some(execution_item.id.to_string()),
                status: "success".to_string(),
                reason: attempt.decision_reason.clone(),
                request_id: current.tool_call_id.clone(),
                trace_id: None,
                metadata: json!({
                    "approvalId": current.id,
                    "skillName": current.skill_name,
                    "platform": params.platform,
                    "executionStatus": attempt.execution_status.as_str(),
                    "result": final_execution.result,
                }),
            })
            .await?;

        let final_approval = approval_tx_svc
            .update_decision(UpdateToolApprovalDecision {
                id: approved.id,
                status: attempt.approval_status,
                reason: attempt.decision_reason.clone(),
                result: attempt.result.clone(),
                approved_by: Some(actor_user_id),
                approved_at: Some(now),
                executed_at: attempt.executed_at,
            })
            .await?;
        audit_tx_svc
            .create(CreateAuditLog {
                id: Uuid::new_v4(),
                user_id: current.user_id,
                actor_user_id,
                workspace_id: current.workspace_id,
                run_id: current.run_id,
                execution_queue_id: Some(execution_item.id),
                tool_approval_id: Some(current.id),
                action: attempt.approval_audit_action.to_string(),
                target_type: "tool_approval".to_string(),
                target_id: Some(current.id.to_string()),
                status: "success".to_string(),
                reason: attempt.decision_reason,
                request_id: current.tool_call_id.clone(),
                trace_id: None,
                metadata: json!({
                    "executionQueueId": execution_item.id,
                    "skillName": current.skill_name,
                    "platform": params.platform,
                    "finalApprovalStatus": final_approval.status.as_str(),
                    "result": final_approval.result,
                }),
            })
            .await?;

        Ok(final_approval)
    }
}

pub fn map_audit_log(item: AuditLog) -> AuditLogResponse {
    AuditLogResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        actor_user_id: item.actor_user_id.to_string(),
        workspace_id: item.workspace_id.map(|id| id.to_string()),
        run_id: item.run_id.map(|id| id.to_string()),
        execution_queue_id: item.execution_queue_id.map(|id| id.to_string()),
        tool_approval_id: item.tool_approval_id.map(|id| id.to_string()),
        action: item.action,
        target_type: item.target_type,
        target_id: item.target_id,
        status: item.status,
        reason: item.reason,
        request_id: item.request_id,
        trace_id: item.trace_id,
        metadata: item.metadata,
        created_at: item.created_at,
    }
}

pub fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::APPROVAL_AUDIT_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

pub fn parse_optional_uuid(value: Option<String>, label: &str) -> BizResult<Option<Uuid>> {
    value
        .and_then(normalize_string)
        .map(|value| parse_uuid(&value, label))
        .transpose()
}

pub fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::APPROVAL_AUDIT_REQUEST_INVALID,
            format!("{label} cannot be empty"),
        ));
    }
    Ok(value)
}

pub fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(normalize_string)
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_tool_approval_status(value: Option<String>) -> BizResult<Option<ToolApprovalStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| match value.as_str() {
            "pending" => Ok(ToolApprovalStatus::Pending),
            "approved" => Ok(ToolApprovalStatus::Approved),
            "executed" => Ok(ToolApprovalStatus::Executed),
            "failed" => Ok(ToolApprovalStatus::Failed),
            "rejected" => Ok(ToolApprovalStatus::Rejected),
            "cancelled" => Ok(ToolApprovalStatus::Cancelled),
            _ => Err(BizError::new(
                app_error::APPROVAL_AUDIT_REQUEST_INVALID,
                format!("invalid status: {value}"),
            )),
        })
        .transpose()
}

fn normalize_risk_level(value: String) -> String {
    let value = value.trim().to_uppercase();
    if value.is_empty() {
        "L1".to_string()
    } else {
        value
    }
}

fn normalize_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn pagination(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    (page, page_size, offset)
}

fn map_tool_approval(item: ToolApproval) -> ToolApprovalResponse {
    ToolApprovalResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        workspace_id: item.workspace_id.map(|id| id.to_string()),
        run_id: item.run_id.map(|id| id.to_string()),
        conversation_id: item.conversation_id.map(|id| id.to_string()),
        request_role: item.request_role,
        skill_name: item.skill_name,
        tool_call_id: item.tool_call_id,
        risk_level: item.risk_level,
        args: item.args,
        status: item.status.as_str().to_string(),
        reason: item.reason,
        action_key: item.action_key,
        result: item.result,
        approved_by: item.approved_by.map(|id| id.to_string()),
        approved_at: item.approved_at,
        executed_at: item.executed_at,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

#[derive(Debug, Clone)]
struct PlatformExecutionAttempt {
    approval_status: ToolApprovalStatus,
    execution_status: ExecutionQueueStatus,
    decision_reason: Option<String>,
    result: Value,
    executed_at: Option<OffsetDateTime>,
    execution_audit_action: &'static str,
    approval_audit_action: &'static str,
}

#[derive(Debug, Clone)]
struct ApprovalExecutionContext {
    actor_user_id: Uuid,
    now: OffsetDateTime,
    decision_reason: Option<String>,
    manual_result: Value,
}

async fn attempt_platform_execution(
    platform_svc: &PlatformConnectionService,
    approval: &ToolApproval,
    execution_item: &ExecutionQueueItem,
    params: &PlatformExecutionParams,
    now: OffsetDateTime,
    manual_result: &Value,
) -> BizResult<PlatformExecutionAttempt> {
    let connection = if let Some(platform) = params.platform.as_deref() {
        platform_svc
            .get_by_user_platform(approval.user_id, platform)
            .await?
    } else {
        None
    };

    Ok(plan_platform_execution(
        approval,
        execution_item,
        params,
        connection.as_ref(),
        manual_result,
        now,
    ))
}

fn plan_platform_execution(
    approval: &ToolApproval,
    execution_item: &ExecutionQueueItem,
    params: &PlatformExecutionParams,
    connection: Option<&PlatformConnection>,
    manual_result: &Value,
    now: OffsetDateTime,
) -> PlatformExecutionAttempt {
    let outcome =
        platform_execution_dispatch_outcome(approval.skill_name.as_str(), params, connection);
    platform_execution_failed(
        approval,
        execution_item,
        outcome,
        params,
        manual_result,
        now,
    )
}

fn plan_platform_execution_readiness(
    skill_name: &str,
    params: &PlatformExecutionParams,
    connection: Option<&PlatformConnection>,
    now: OffsetDateTime,
) -> Value {
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

    json!({
        "status": status,
        "adapterStatus": adapter_status,
        "canApprove": false,
        "canExecuteNow": false,
        "willFailIfApproved": true,
        "platform": params.platform,
        "skillName": skill_name,
        "message": message,
        "details": platform_execution_details(params),
        "checkedAt": now,
    })
}

fn platform_execution_failed(
    approval: &ToolApproval,
    execution_item: &ExecutionQueueItem,
    outcome: PlatformExecutionAdapterOutcome,
    params: &PlatformExecutionParams,
    manual_result: &Value,
    now: OffsetDateTime,
) -> PlatformExecutionAttempt {
    let decision_reason = outcome.message.clone();
    let result = json!({
        "status": outcome.status,
        "source": "tool_approval_execution",
        "approvalId": approval.id,
        "executionQueueId": execution_item.id,
        "approvalStatus": "failed",
        "executionStatus": "failed",
        "adapterStatus": outcome.adapter_status,
        "platform": params.platform,
        "skillName": approval.skill_name,
        "message": outcome.message,
        "details": platform_execution_details(params),
        "adapterRequest": outcome.adapter_request,
        "providerResult": outcome.provider_result,
        "retryable": outcome.retryable,
        "manualResult": manual_result,
        "attemptedAt": now,
    });
    PlatformExecutionAttempt {
        approval_status: ToolApprovalStatus::Failed,
        execution_status: ExecutionQueueStatus::Failed,
        decision_reason: Some(decision_reason),
        result,
        executed_at: Some(now),
        execution_audit_action: "execution_queue.failed",
        approval_audit_action: "tool_approval.execute_failed",
    }
}

fn build_execution_description(
    approval: &ToolApproval,
    params: &PlatformExecutionParams,
) -> String {
    let platform = params.platform.as_deref().unwrap_or("unknown-platform");
    let target = params
        .product_id
        .as_deref()
        .map(|product_id| format!(" product {product_id}"))
        .unwrap_or_default();
    format!(
        "Approved execution for {} on {}{}",
        approval.skill_name, platform, target
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo::table::platform_connections::{
        PlatformConnection, PlatformConnectionStatus, PlatformSyncStatus,
    };

    #[test]
    fn approval_execution_params_reads_normalized_inputs_from_skill_output() {
        let args = json!({
            "source": "agent_sse",
            "skillId": "platform_update_price",
            "output": {
                "normalized_inputs": {
                    "platform": "douyin",
                    "product_id": "prod-1",
                    "new_price": 129.5,
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
    fn plan_platform_execution_reports_missing_connection() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let approval = test_tool_approval("platform_update_price");
        let execution = test_execution_queue();
        let params = PlatformExecutionParams {
            platform: Some("taobao".to_string()),
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };

        let attempt =
            plan_platform_execution(&approval, &execution, &params, None, &json!({}), now);

        assert_eq!(attempt.approval_status, ToolApprovalStatus::Failed);
        assert_eq!(attempt.execution_status, ExecutionQueueStatus::Failed);
        assert_eq!(
            attempt.result.get("adapterStatus").and_then(Value::as_str),
            Some("connection_missing")
        );
    }

    #[test]
    fn plan_platform_execution_reports_adapter_not_implemented_when_connected() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let approval = test_tool_approval("platform_update_inventory");
        let execution = test_execution_queue();
        let params = PlatformExecutionParams {
            platform: Some("jd".to_string()),
            product_id: Some("prod-1".to_string()),
            quantity: Some(12),
            ..Default::default()
        };
        let connection = test_platform_connection("jd");

        let attempt = plan_platform_execution(
            &approval,
            &execution,
            &params,
            Some(&connection),
            &json!({"operator": "manual"}),
            now,
        );

        assert_eq!(attempt.approval_status, ToolApprovalStatus::Failed);
        assert_eq!(
            attempt.result.get("adapterStatus").and_then(Value::as_str),
            Some("adapter_not_implemented")
        );
        assert_eq!(
            attempt
                .result
                .pointer("/details/quantity")
                .and_then(Value::as_i64),
            Some(12)
        );
        assert_eq!(
            attempt
                .result
                .pointer("/adapterRequest/idempotencyKey")
                .and_then(Value::as_str),
            Some("platform_update_inventory:platform=jd|product_id=prod-1|sku_id=null|quantity=12")
        );
        assert_eq!(
            attempt
                .result
                .pointer("/adapterRequest/args/quantity")
                .and_then(Value::as_i64),
            Some(12)
        );
        assert_eq!(
            attempt.result.get("retryable").and_then(Value::as_bool),
            Some(false)
        );
        assert!(
            attempt
                .result
                .get("providerResult")
                .is_some_and(Value::is_null)
        );
    }

    #[test]
    fn approval_execution_readiness_reports_missing_platform() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };

        let readiness =
            plan_platform_execution_readiness("platform_update_price", &params, None, now);

        assert_eq!(
            readiness.get("status").and_then(Value::as_str),
            Some("blocked")
        );
        assert_eq!(
            readiness.get("adapterStatus").and_then(Value::as_str),
            Some("validation_error")
        );
        assert_eq!(
            readiness.get("willFailIfApproved").and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn approval_execution_readiness_reports_missing_connection() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            platform: Some("taobao".to_string()),
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };

        let readiness =
            plan_platform_execution_readiness("platform_update_price", &params, None, now);

        assert_eq!(
            readiness.get("status").and_then(Value::as_str),
            Some("blocked")
        );
        assert_eq!(
            readiness.get("adapterStatus").and_then(Value::as_str),
            Some("connection_missing")
        );
        assert_eq!(
            readiness
                .pointer("/details/productId")
                .and_then(Value::as_str),
            Some("prod-1")
        );
    }

    #[test]
    fn approval_execution_readiness_reports_adapter_not_implemented() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            platform: Some("jd".to_string()),
            product_id: Some("prod-1".to_string()),
            quantity: Some(12),
            ..Default::default()
        };
        let connection = test_platform_connection("jd");

        let readiness = plan_platform_execution_readiness(
            "platform_update_inventory",
            &params,
            Some(&connection),
            now,
        );

        assert_eq!(
            readiness.get("status").and_then(Value::as_str),
            Some("blocked")
        );
        assert_eq!(
            readiness.get("adapterStatus").and_then(Value::as_str),
            Some("adapter_not_implemented")
        );
        assert_eq!(
            readiness.get("canExecuteNow").and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn plan_platform_execution_reports_unsupported_platform_action() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let approval = test_tool_approval("platform_publish_campaign");
        let execution = test_execution_queue();
        let params = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            ..Default::default()
        };
        let connection = test_platform_connection("douyin");

        let attempt = plan_platform_execution(
            &approval,
            &execution,
            &params,
            Some(&connection),
            &json!({}),
            now,
        );

        assert_eq!(attempt.approval_status, ToolApprovalStatus::Failed);
        assert_eq!(
            attempt.result.get("adapterStatus").and_then(Value::as_str),
            Some("unsupported_action")
        );
        assert!(
            attempt
                .result
                .get("adapterRequest")
                .is_some_and(Value::is_null)
        );
    }

    #[test]
    fn plan_platform_execution_reports_unsupported_action_before_connection() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let approval = test_tool_approval("platform_publish_campaign");
        let execution = test_execution_queue();
        let params = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            ..Default::default()
        };

        let attempt =
            plan_platform_execution(&approval, &execution, &params, None, &json!({}), now);

        assert_eq!(attempt.approval_status, ToolApprovalStatus::Failed);
        assert_eq!(
            attempt.result.get("adapterStatus").and_then(Value::as_str),
            Some("unsupported_action")
        );
    }

    #[test]
    fn approval_execution_readiness_never_marks_unimplemented_adapter_ready() {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        let params = PlatformExecutionParams {
            platform: Some("jd".to_string()),
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };
        let connection = test_platform_connection("jd");

        let readiness = plan_platform_execution_readiness(
            "platform_update_price",
            &params,
            Some(&connection),
            now,
        );

        assert_eq!(
            readiness.get("adapterStatus").and_then(Value::as_str),
            Some("adapter_not_implemented")
        );
        assert_eq!(
            readiness.get("canApprove").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            readiness.get("willFailIfApproved").and_then(Value::as_bool),
            Some(true)
        );
    }

    fn test_tool_approval(skill_name: &str) -> ToolApproval {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        ToolApproval {
            id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            user_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            workspace_id: Some(Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()),
            run_id: Some(Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap()),
            conversation_id: None,
            request_role: Some("ops".to_string()),
            skill_name: skill_name.to_string(),
            tool_call_id: Some("tool-call-1".to_string()),
            risk_level: "L3".to_string(),
            args: json!({}),
            status: ToolApprovalStatus::Pending,
            reason: Some("需要人工确认".to_string()),
            action_key: Some("agent:run:test".to_string()),
            result: json!({}),
            approved_by: None,
            approved_at: None,
            executed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn test_execution_queue() -> ExecutionQueueItem {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        ExecutionQueueItem {
            id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
            user_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            workspace_id: Some(Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()),
            run_id: Some(Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap()),
            workspace_task_id: None,
            action_type: "platform_update_price".to_string(),
            description: "test execution".to_string(),
            platform: Some("taobao".to_string()),
            payload: json!({}),
            expected_impact: None,
            status: ExecutionQueueStatus::Approved,
            decision_reason: None,
            result: json!({}),
            created_at: now,
            decided_at: Some(now),
        }
    }

    fn test_platform_connection(platform: &str) -> PlatformConnection {
        let now = OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap();
        PlatformConnection {
            id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
            user_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            platform: platform.to_string(),
            app_key: Some("ak".to_string()),
            app_secret_enc: Some("secret".to_string()),
            access_token_enc: Some("token".to_string()),
            refresh_token_enc: Some("refresh".to_string()),
            token_expires_at: None,
            shop_id: Some("shop-1".to_string()),
            shop_name: Some("Demo Shop".to_string()),
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
}
