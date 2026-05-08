use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    audit_logs::{AuditLog, AuditLogFilter, AuditLogService, CreateAuditLog},
    conversations::ConversationService,
    tool_approval_queue::{
        CreateToolApproval, ToolApproval, ToolApprovalFilter, ToolApprovalQueueService,
        ToolApprovalStatus, UpdateToolApprovalDecision,
    },
    workspace_runs::WorkspaceRunService,
    workspaces::{WorkspaceService, WorkspaceStatus},
};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

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
        let approval = self
            .approval_svc
            .create(CreateToolApproval {
                id: Uuid::new_v4(),
                user_id,
                workspace_id,
                run_id,
                conversation_id,
                request_role: normalize_optional(req.request_role),
                skill_name: normalize_required(req.skill_name, "skill_name")?,
                tool_call_id: normalize_optional(req.tool_call_id),
                risk_level: normalize_risk_level(
                    req.risk_level.unwrap_or_else(|| "L1".to_string()),
                ),
                args: req.args.unwrap_or_else(|| json!({})),
                status: ToolApprovalStatus::Pending,
                reason: normalize_optional(req.reason),
                action_key,
                result: json!({}),
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
        let audit_tx_svc = AuditLogService::new(tx.clone());
        let update_result: BizResult<ToolApproval> = async {
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
