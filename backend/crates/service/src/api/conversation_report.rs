use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    agent_quality_checks::{AgentQualityCheck, AgentQualityCheckFilter, AgentQualityCheckService},
    agent_skill_runs::{AgentSkillRun, AgentSkillRunFilter, AgentSkillRunService},
    app_users::{AppUserService, AppUserStatus},
    conversation_files::{ConversationFile, ConversationFileService},
    conversation_messages::{ConversationMessage, ConversationMessageService},
    conversations::{Conversation, ConversationService},
    workspace_artifacts::{
        CreateWorkspaceArtifact, WorkspaceArtifact, WorkspaceArtifactFilter,
        WorkspaceArtifactService, WorkspaceArtifactStatus, WorkspaceArtifactValidationStatus,
    },
    workspace_run_tasks::{WorkspaceRunTask, WorkspaceRunTaskFilter, WorkspaceRunTaskService},
    workspace_runs::{WorkspaceRun, WorkspaceRunFilter, WorkspaceRunService},
    workspaces::{Workspace, WorkspaceFilter, WorkspaceService, WorkspaceStatus},
};
use serde_json::{Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::dto::{
    conversation_report::{
        ConversationReportMaterialCounts, ConversationTranscriptExportResponse,
        ExportConversationTranscriptRequest, GenerateConversationReportRequest,
        GenerateConversationReportResponse, GeneratedConversationReportExport,
    },
    workspace::{WorkspaceArtifactExportResponse, WorkspaceArtifactResponse},
};

const TRANSCRIPT_SCOPE_ALL: &str = "conversation_all";
const TRANSCRIPT_SCOPE_RECENT: &str = "conversation_recent";
const REPORT_TYPE_CONVERSATION: &str = "conversation_report";
const REPORT_TYPE_COMMERCE: &str = "commerce_conversation_report";
const REPORT_TYPE_TEACHING: &str = "teaching_conversation_report";
const REPORT_TYPE_PROCESS: &str = "process_evidence_pack";
const REPORT_SOURCE: &str = "manual_report_generation";
const MAX_TRANSCRIPT_MESSAGES: usize = 120;
const MAX_REPORT_MESSAGES: usize = 60;
const MAX_REPORT_FILES: usize = 20;
const MAX_REPORT_ARTIFACTS: usize = 20;
const MAX_REPORT_SKILL_RUNS: usize = 100;
const MAX_REPORT_QUALITY_CHECKS: usize = 20;
const MAX_REPORT_RUN_TASKS: usize = 100;
const MAX_ARTIFACT_CHARS: usize = 4_000;
const MAX_FILE_SUMMARY_CHARS: usize = 1_200;
const MAX_REPORT_MATERIAL_CHARS: usize = 30_000;

pub struct ConversationReportApi {
    app_user_svc: AppUserService,
    conversation_svc: ConversationService,
    message_svc: ConversationMessageService,
    file_svc: ConversationFileService,
    workspace_svc: WorkspaceService,
    run_svc: WorkspaceRunService,
    run_task_svc: WorkspaceRunTaskService,
    artifact_svc: WorkspaceArtifactService,
    skill_run_svc: AgentSkillRunService,
    quality_check_svc: AgentQualityCheckService,
}

impl ConversationReportApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db.clone()),
            conversation_svc: ConversationService::new(db.clone()),
            message_svc: ConversationMessageService::new(db.clone()),
            file_svc: ConversationFileService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            run_svc: WorkspaceRunService::new(db.clone()),
            run_task_svc: WorkspaceRunTaskService::new(db.clone()),
            artifact_svc: WorkspaceArtifactService::new(db.clone()),
            skill_run_svc: AgentSkillRunService::new(db.clone()),
            quality_check_svc: AgentQualityCheckService::new(db),
        }
    }

    pub async fn export_transcript(
        &self,
        req: ExportConversationTranscriptRequest,
    ) -> BizResult<ConversationTranscriptExportResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_conversation_owner(user_id, conversation_id)
            .await?;
        let format = normalize_export_format(req.format)?;
        let scope = normalize_transcript_scope(req.scope)?;
        let include_files = req.include_files.unwrap_or(true);
        let include_runtime_events = req.include_runtime_events.unwrap_or(false);
        let mut warnings = Vec::new();
        let messages = self
            .message_svc
            .list_by_conversation_id(conversation_id)
            .await?;
        let scoped_messages =
            scope_messages(messages, &scope, MAX_TRANSCRIPT_MESSAGES, &mut warnings);
        let files = if include_files {
            self.file_svc
                .list_by_conversation_id(conversation_id)
                .await?
        } else {
            Vec::new()
        };
        let material = TranscriptMaterial {
            conversation,
            files,
            include_runtime_events,
            messages: scoped_messages,
            warnings,
        };
        render_transcript_export(conversation_id, format, material)
    }

    pub async fn generate_report(
        &self,
        req: GenerateConversationReportRequest,
    ) -> BizResult<GenerateConversationReportResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let conversation_id = parse_uuid(&req.conversation_id, "conversation_id")?;
        let conversation = self
            .ensure_conversation_owner(user_id, conversation_id)
            .await?;
        let report_type = normalize_report_type(req.report_type)?;
        let scope = normalize_report_scope(req.scope)?;
        let format = normalize_report_format(req.format)?;
        let include_process = req.include_process.unwrap_or(true);
        let include_sources = req.include_sources.unwrap_or(true);
        let include_transcript_appendix = req.include_transcript_appendix.unwrap_or(false);
        let workspace = match parse_optional_uuid(req.workspace_id, "workspace_id")? {
            Some(workspace_id) => {
                self.ensure_active_workspace_owner(user_id, workspace_id)
                    .await?
            }
            None => self.default_workspace(user_id).await?,
        };
        let run_id = parse_optional_uuid(req.run_id, "run_id")?;
        let run = match run_id {
            Some(run_id) => Some(self.ensure_run_owner(user_id, workspace.id, run_id).await?),
            None => self.latest_run_for_workspace(user_id, workspace.id).await?,
        };
        if scope == "workspace_run" && run.is_none() {
            return Err(BizError::new(
                app_error::CONVERSATION_REQUEST_INVALID,
                "run_id is required for workspace_run scope".to_string(),
            ));
        }
        let artifact_ids = parse_uuid_list(req.artifact_ids.unwrap_or_default(), "artifact_ids")?;
        let file_ids = parse_uuid_list(req.file_ids.unwrap_or_default(), "file_ids")?;
        let mut warnings = Vec::new();
        let messages = self
            .message_svc
            .list_by_conversation_id(conversation_id)
            .await?;
        let messages = scope_messages(messages, &scope, MAX_REPORT_MESSAGES, &mut warnings);
        if messages.is_empty() {
            return Err(BizError::new(
                app_error::CONVERSATION_REQUEST_INVALID,
                "conversation has no messages to report".to_string(),
            ));
        }
        let mut files = if include_sources {
            self.file_svc
                .list_by_conversation_id(conversation_id)
                .await?
        } else {
            Vec::new()
        };
        for file_id in file_ids {
            push_unique_file(
                &mut files,
                self.ensure_active_file_owner(user_id, file_id).await?,
            );
        }
        truncate_vec(
            &mut files,
            MAX_REPORT_FILES,
            "conversation files",
            &mut warnings,
        );

        let mut artifacts = Vec::new();
        if include_sources && let Some(run) = run.as_ref() {
            artifacts.extend(
                self.artifact_svc
                    .list(WorkspaceArtifactFilter {
                        workspace_id: Some(workspace.id),
                        run_id: Some(run.id),
                        user_id,
                        status: Some(WorkspaceArtifactStatus::Active),
                        limit: Some(MAX_REPORT_ARTIFACTS as u64),
                        ..Default::default()
                    })
                    .await?,
            );
        }
        for artifact_id in artifact_ids {
            push_unique_artifact(
                &mut artifacts,
                self.ensure_active_artifact_owner(user_id, workspace.id, artifact_id)
                    .await?,
            );
        }
        truncate_vec(
            &mut artifacts,
            MAX_REPORT_ARTIFACTS,
            "workspace artifacts",
            &mut warnings,
        );

        let (run_tasks, skill_runs, quality_checks) = if include_process {
            if let Some(run) = run.as_ref() {
                (
                    self.run_task_svc
                        .list(WorkspaceRunTaskFilter {
                            run_id: Some(run.id),
                            workspace_id: Some(workspace.id),
                            user_id,
                            limit: Some(MAX_REPORT_RUN_TASKS as u64),
                            ..Default::default()
                        })
                        .await?,
                    self.skill_run_svc
                        .list(AgentSkillRunFilter {
                            workspace_id: Some(workspace.id),
                            run_id: Some(run.id),
                            user_id,
                            limit: Some(MAX_REPORT_SKILL_RUNS as u64),
                            ..Default::default()
                        })
                        .await?,
                    self.quality_check_svc
                        .list(AgentQualityCheckFilter {
                            workspace_id: Some(workspace.id),
                            run_id: Some(run.id),
                            user_id,
                            limit: Some(MAX_REPORT_QUALITY_CHECKS as u64),
                            ..Default::default()
                        })
                        .await?,
                )
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            }
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };
        let counts = ConversationReportMaterialCounts {
            messages: messages.len(),
            files: files.len(),
            artifacts: artifacts.len(),
            skill_runs: skill_runs.len(),
            quality_checks: quality_checks.len(),
            run_tasks: run_tasks.len(),
        };
        let material = ReportMaterial {
            artifacts,
            conversation,
            files,
            include_process,
            include_sources,
            include_transcript_appendix,
            messages,
            quality_checks,
            report_type: report_type.clone(),
            run,
            run_tasks,
            scope: scope.clone(),
            skill_runs,
            warnings: warnings.clone(),
            workspace: workspace.clone(),
        };
        let report = render_report_markdown(&material);
        let report = sanitize_user_visible_text(&report);
        let preview = build_preview(&report);
        let artifact_type = report_type.clone();
        let title = report_title(&report_type).to_string();
        let artifact = self
            .artifact_svc
            .create(CreateWorkspaceArtifact {
                id: Uuid::new_v4(),
                workspace_id: workspace.id,
                run_id: material.run.as_ref().map(|item| item.id),
                workspace_task_id: None,
                run_task_id: None,
                user_id,
                artifact_type,
                title,
                content: Some(report.clone()),
                preview: Some(preview),
                mime_type: Some("text/markdown".to_string()),
                format,
                storage_kind: "inline".to_string(),
                bucket: None,
                object_key: None,
                file_name: None,
                file_size: None,
                sha256: None,
                source: REPORT_SOURCE.to_string(),
                status: WorkspaceArtifactStatus::Active,
                validation_status: WorkspaceArtifactValidationStatus::Unchecked,
                validation_report: json!({}),
                metadata: json!({
                    "kind": "manual_conversation_report",
                    "conversationId": conversation_id,
                    "reportType": report_type,
                    "scope": scope,
                    "includeProcess": include_process,
                    "includeSources": include_sources,
                    "includeTranscriptAppendix": include_transcript_appendix,
                    "materialCounts": counts.to_json(),
                    "generatedAt": OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default(),
                }),
            })
            .await?;
        let export = build_artifact_export(&artifact, warnings);

        Ok(GenerateConversationReportResponse {
            artifact: map_artifact(artifact),
            export: GeneratedConversationReportExport::from(export),
        })
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
                "app user disabled".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_conversation_owner(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
    ) -> BizResult<Conversation> {
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
        Ok(conversation)
    }

    async fn default_workspace(&self, user_id: Uuid) -> BizResult<Workspace> {
        let mut workspaces = self
            .workspace_svc
            .list(WorkspaceFilter {
                user_id,
                status: Some(WorkspaceStatus::Active),
                workspace_type: Some("agent_chat".to_string()),
                limit: Some(1),
                ..Default::default()
            })
            .await?;
        if let Some(workspace) = workspaces.pop() {
            return Ok(workspace);
        }
        let mut workspaces = self
            .workspace_svc
            .list(WorkspaceFilter {
                user_id,
                status: Some(WorkspaceStatus::Active),
                limit: Some(1),
                ..Default::default()
            })
            .await?;
        workspaces.pop().ok_or_else(|| {
            BizError::new(
                app_error::WORKSPACE_NOT_FOUND,
                "workspace not found".to_string(),
            )
        })
    }

    async fn ensure_active_workspace_owner(
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
        if workspace.status == WorkspaceStatus::Archived {
            return Err(BizError::new(
                app_error::WORKSPACE_NOT_FOUND,
                "workspace not found".to_string(),
            ));
        }
        Ok(workspace)
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

    async fn latest_run_for_workspace(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> BizResult<Option<WorkspaceRun>> {
        let mut runs = self
            .run_svc
            .list(WorkspaceRunFilter {
                workspace_id: Some(workspace_id),
                user_id,
                limit: Some(1),
                ..Default::default()
            })
            .await?;
        Ok(runs.pop())
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

    async fn ensure_active_file_owner(
        &self,
        user_id: Uuid,
        file_id: Uuid,
    ) -> BizResult<ConversationFile> {
        let file = self.file_svc.get_by_id(file_id).await?.ok_or_else(|| {
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
}

struct TranscriptMaterial {
    conversation: Conversation,
    files: Vec<ConversationFile>,
    include_runtime_events: bool,
    messages: Vec<ConversationMessage>,
    warnings: Vec<String>,
}

struct ReportMaterial {
    artifacts: Vec<WorkspaceArtifact>,
    conversation: Conversation,
    files: Vec<ConversationFile>,
    include_process: bool,
    include_sources: bool,
    include_transcript_appendix: bool,
    messages: Vec<ConversationMessage>,
    quality_checks: Vec<AgentQualityCheck>,
    report_type: String,
    run: Option<WorkspaceRun>,
    run_tasks: Vec<WorkspaceRunTask>,
    scope: String,
    skill_runs: Vec<AgentSkillRun>,
    warnings: Vec<String>,
    workspace: Workspace,
}

fn render_transcript_export(
    conversation_id: Uuid,
    format: String,
    material: TranscriptMaterial,
) -> BizResult<ConversationTranscriptExportResponse> {
    let markdown = render_transcript_markdown(&material);
    let content = match format.as_str() {
        "markdown" => markdown,
        "text" => markdown_to_text(&markdown),
        "json" => serde_json::to_string_pretty(&json!({
            "conversationId": conversation_id,
            "title": material.conversation.title,
            "messages": material.messages.iter().map(message_json).collect::<Vec<_>>(),
            "files": material.files.iter().map(safe_file_json).collect::<Vec<_>>(),
            "warnings": material.warnings,
        }))
        .map_err(|err| {
            BizError::new(
                app_error::CONVERSATION_REQUEST_INVALID,
                format!("failed to render transcript json: {err}"),
            )
        })?,
        _ => unreachable!("format is validated before rendering"),
    };
    let extension = if format == "markdown" {
        "md"
    } else if format == "text" {
        "txt"
    } else {
        "json"
    };
    let mime_type = export_mime_type(&format).to_string();
    let content_chars = content.chars().count();
    Ok(ConversationTranscriptExportResponse {
        conversation_id: conversation_id.to_string(),
        file_name: format!("conversation-transcript.{extension}"),
        mime_type,
        format,
        content,
        content_chars,
        message_count: material.messages.len(),
        file_count: material.files.len(),
        warnings: material.warnings,
    })
}

fn render_transcript_markdown(material: &TranscriptMaterial) -> String {
    let mut out = String::new();
    push_line(&mut out, "# 对话原文");
    push_line(&mut out, "");
    push_line(
        &mut out,
        &format!(
            "- 会话：{}",
            material
                .conversation
                .title
                .as_deref()
                .unwrap_or("未命名会话")
        ),
    );
    push_line(&mut out, &format!("- 消息数：{}", material.messages.len()));
    push_line(&mut out, &format!("- 附件数：{}", material.files.len()));
    push_line(&mut out, "");
    push_line(&mut out, "## 消息");
    for message in &material.messages {
        push_line(&mut out, "");
        push_line(
            &mut out,
            &format!(
                "### {} #{}",
                role_label(&message.role),
                message.sequence.max(0)
            ),
        );
        if let Ok(created_at) = message.created_at.format(&Rfc3339) {
            push_line(&mut out, &format!("- 时间：{created_at}"));
        }
        let text = message_text(message);
        push_line(&mut out, "");
        push_line(
            &mut out,
            if text.is_empty() {
                "（无文本内容）"
            } else {
                &text
            },
        );
        if material.include_runtime_events {
            let runtime_summary = runtime_event_summary(&message.content_json);
            if !runtime_summary.is_empty() {
                push_line(&mut out, "");
                push_line(&mut out, "过程摘要：");
                for line in runtime_summary {
                    push_line(&mut out, &format!("- {line}"));
                }
            }
        }
    }
    if !material.files.is_empty() {
        push_line(&mut out, "");
        push_line(&mut out, "## 附件");
        for file in &material.files {
            push_line(&mut out, &format!("- {}", file_summary(file)));
        }
    }
    if !material.warnings.is_empty() {
        push_line(&mut out, "");
        push_line(&mut out, "## 省略说明");
        for warning in &material.warnings {
            push_line(&mut out, &format!("- {warning}"));
        }
    }
    sanitize_user_visible_text(&out)
}

fn render_report_markdown(material: &ReportMaterial) -> String {
    let mut out = String::new();
    push_line(
        &mut out,
        &format!("# {}", report_title(&material.report_type)),
    );
    push_line(&mut out, "");
    push_line(&mut out, "## 基本信息");
    push_line(
        &mut out,
        &format!(
            "- 会话：{}",
            material
                .conversation
                .title
                .as_deref()
                .unwrap_or("未命名会话")
        ),
    );
    push_line(&mut out, &format!("- 工作区：{}", material.workspace.title));
    if let Some(run) = material.run.as_ref() {
        push_line(
            &mut out,
            &format!("- 任务目标：{}", truncate_text(&run.goal, 240)),
        );
        if let Some(summary) = run
            .summary
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            push_line(
                &mut out,
                &format!("- 运行摘要：{}", truncate_text(summary, 240)),
            );
        }
    }
    push_line(
        &mut out,
        &format!("- 报告范围：{}", scope_label(&material.scope)),
    );
    push_line(&mut out, "");
    push_line(&mut out, "## 对话目标与用户信息");
    for message in material
        .messages
        .iter()
        .filter(|item| item.role == "user")
        .take(12)
    {
        let text = message_text(message);
        if !text.is_empty() {
            push_line(&mut out, &format!("- {}", truncate_text(&text, 360)));
        }
    }
    push_line(&mut out, "");
    push_line(&mut out, "## AI 回答结论");
    for message in material
        .messages
        .iter()
        .filter(|item| item.role == "assistant")
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let text = message_text(message);
        if !text.is_empty() {
            push_line(&mut out, &format!("- {}", truncate_text(&text, 420)));
        }
    }
    match material.report_type.as_str() {
        REPORT_TYPE_COMMERCE => render_commerce_sections(&mut out, material),
        REPORT_TYPE_TEACHING => render_teaching_sections(&mut out, material),
        REPORT_TYPE_PROCESS => render_process_sections(&mut out, material),
        _ => render_general_sections(&mut out, material),
    }
    if material.include_sources {
        render_sources(&mut out, material);
    }
    if material.include_process {
        render_process_summary(&mut out, material);
    }
    if material.include_transcript_appendix {
        push_line(&mut out, "");
        push_line(&mut out, "## 对话附录摘要");
        for message in &material.messages {
            let text = message_text(message);
            if !text.is_empty() {
                push_line(
                    &mut out,
                    &format!(
                        "- {}：{}",
                        role_label(&message.role),
                        truncate_text(&text, 260)
                    ),
                );
            }
        }
    }
    if !material.warnings.is_empty() {
        push_line(&mut out, "");
        push_line(&mut out, "## 省略与限制说明");
        for warning in &material.warnings {
            push_line(&mut out, &format!("- {warning}"));
        }
    }
    truncate_text(&out, MAX_REPORT_MATERIAL_CHARS)
}

fn render_general_sections(out: &mut String, _material: &ReportMaterial) {
    push_line(out, "");
    push_line(out, "## 建议与待办");
    push_line(out, "- 复核以上结论是否覆盖当前任务目标。");
    push_line(out, "- 对缺少数据或未确认假设的部分继续补充资料。");
    push_line(out, "- 将可执行动作拆分为下一轮任务或工作区交付物。");
}

fn render_commerce_sections(out: &mut String, material: &ReportMaterial) {
    push_line(out, "");
    push_line(out, "## 电商经营判断");
    push_line(out, "- 围绕商品、店铺、平台、活动和人群线索整理本次对话。");
    push_line(
        out,
        "- 优先检查卖点、标题、主图、转化、投放、竞品、库存和利润风险。",
    );
    if !material.artifacts.is_empty() {
        push_line(out, "- 已结合现有工作区交付物和搜索证据摘要。");
    }
    push_line(out, "");
    push_line(out, "## 下一步动作");
    push_line(out, "- 明确当前最优先的经营问题。");
    push_line(out, "- 补齐缺失的销量、转化、成本、竞品或投放数据。");
    push_line(
        out,
        "- 将建议落到商品优化、内容优化、投放测试或客服承接动作。",
    );
}

fn render_teaching_sections(out: &mut String, _material: &ReportMaterial) {
    push_line(out, "");
    push_line(out, "## 教学与实训复盘");
    push_line(out, "- 提炼本次学习任务、关键知识点和操作步骤。");
    push_line(
        out,
        "- 区分已经掌握的内容、需要练习的内容和需要教师继续讲解的问题。",
    );
    push_line(out, "");
    push_line(out, "## 后续练习");
    push_line(out, "- 将本次案例改写为可提交成果。");
    push_line(out, "- 针对薄弱知识点安排一次复盘或小测。");
}

fn render_process_sections(out: &mut String, material: &ReportMaterial) {
    push_line(out, "");
    push_line(out, "## 多智能体过程证据");
    push_line(out, &format!("- 运行任务：{} 条", material.run_tasks.len()));
    push_line(
        out,
        &format!("- Skill 调用：{} 条", material.skill_runs.len()),
    );
    push_line(
        out,
        &format!("- 质量检查：{} 条", material.quality_checks.len()),
    );
    push_line(
        out,
        &format!("- 工作区交付物：{} 条", material.artifacts.len()),
    );
}

fn render_sources(out: &mut String, material: &ReportMaterial) {
    push_line(out, "");
    push_line(out, "## 资料与来源");
    if material.files.is_empty() && material.artifacts.is_empty() {
        push_line(out, "- 本次报告没有可纳入的附件或工作区交付物摘要。");
    }
    for file in &material.files {
        push_line(out, &format!("- 附件：{}", file_summary(file)));
        let summary = file_extraction_summary(&file.metadata);
        if !summary.is_empty() {
            push_line(
                out,
                &format!(
                    "  - 摘要：{}",
                    truncate_text(&summary, MAX_FILE_SUMMARY_CHARS)
                ),
            );
        }
    }
    for artifact in &material.artifacts {
        push_line(
            out,
            &format!(
                "- 交付物：{}（{}）",
                artifact.title,
                artifact.artifact_type.replace('_', " ")
            ),
        );
        let text = artifact
            .preview
            .as_deref()
            .or(artifact.content.as_deref())
            .unwrap_or("");
        if !text.trim().is_empty() {
            push_line(
                out,
                &format!("  - 摘要：{}", truncate_text(text, MAX_ARTIFACT_CHARS)),
            );
        }
    }
}

fn render_process_summary(out: &mut String, material: &ReportMaterial) {
    push_line(out, "");
    push_line(out, "## 过程摘要");
    if material.run_tasks.is_empty()
        && material.skill_runs.is_empty()
        && material.quality_checks.is_empty()
    {
        push_line(out, "- 暂无可展示的运行任务、skill 调用或质量检查记录。");
    }
    for task in material.run_tasks.iter().take(20) {
        push_line(
            out,
            &format!(
                "- 任务 {}：{}，角色 {}",
                task.seq,
                task.status.as_str(),
                task.owner_role.as_deref().unwrap_or("未指定")
            ),
        );
    }
    for skill in material.skill_runs.iter().take(20) {
        push_line(
            out,
            &format!(
                "- Skill {}：{}，{}",
                skill.skill_id,
                skill.status.as_str(),
                skill.summary.as_deref().unwrap_or("无摘要")
            ),
        );
    }
    for check in material.quality_checks.iter().take(10) {
        push_line(
            out,
            &format!(
                "- 质量检查：{}，得分 {:.1}",
                if check.passed { "通过" } else { "未通过" },
                check.score
            ),
        );
    }
}

fn build_artifact_export(
    artifact: &WorkspaceArtifact,
    warnings: Vec<String>,
) -> WorkspaceArtifactExportResponse {
    let content = artifact.content.clone().unwrap_or_default();
    WorkspaceArtifactExportResponse {
        artifact_id: artifact.id.to_string(),
        workspace_id: artifact.workspace_id.to_string(),
        title: artifact.title.clone(),
        file_name: format!("{}.md", sanitize_file_base_name(&artifact.title)),
        mime_type: "text/markdown; charset=utf-8".to_string(),
        format: "markdown".to_string(),
        content_chars: content.chars().count(),
        content,
        source_format: artifact.format.clone(),
        source_storage_kind: artifact.storage_kind.clone(),
        metadata: artifact.metadata.clone(),
        warnings,
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

fn scope_messages(
    mut messages: Vec<ConversationMessage>,
    scope: &str,
    limit: usize,
    warnings: &mut Vec<String>,
) -> Vec<ConversationMessage> {
    messages.retain(|message| message.status != "deleted");
    if scope == TRANSCRIPT_SCOPE_RECENT && messages.len() > limit / 2 {
        let omitted = messages.len() - limit / 2;
        warnings.push(format!(
            "{omitted} older message(s) were omitted by recent scope"
        ));
        messages.split_off(omitted)
    } else if messages.len() > limit {
        let omitted = messages.len() - limit;
        warnings.push(format!(
            "{omitted} older message(s) were omitted by material budget"
        ));
        messages.split_off(omitted)
    } else {
        messages
    }
}

fn message_text(message: &ConversationMessage) -> String {
    message
        .content_text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            message
                .content_json
                .as_ref()
                .and_then(|value| value.get("text").or_else(|| value.get("content")))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
        .unwrap_or_default()
}

fn message_json(message: &ConversationMessage) -> Value {
    json!({
        "id": message.id,
        "role": message.role,
        "contentText": message_text(message),
        "sequence": message.sequence,
        "createdAt": message.created_at.format(&Rfc3339).unwrap_or_default(),
    })
}

fn safe_file_json(file: &ConversationFile) -> Value {
    json!({
        "id": file.id,
        "conversationId": file.conversation_id,
        "fileName": file.file_name,
        "mimeType": file.mime_type,
        "fileSize": file.file_size,
        "status": file.status,
        "extractionStatus": file_extraction_status(&file.metadata),
        "summary": truncate_text(&file_extraction_summary(&file.metadata), 500),
    })
}

fn file_summary(file: &ConversationFile) -> String {
    let name = file.file_name.as_deref().unwrap_or("未命名附件");
    let mime = file.mime_type.as_deref().unwrap_or("unknown");
    let size = file
        .file_size
        .map(|value| format!("{value} bytes"))
        .unwrap_or_else(|| "unknown size".to_string());
    format!(
        "{name} / {mime} / {size} / {}",
        file_extraction_status(&file.metadata)
    )
}

fn file_extraction_status(metadata: &Value) -> String {
    metadata
        .get("extraction")
        .and_then(|value| value.get("status"))
        .or_else(|| metadata.get("extractionStatus"))
        .and_then(Value::as_str)
        .unwrap_or("pending")
        .to_string()
}

fn file_extraction_summary(metadata: &Value) -> String {
    metadata
        .get("extraction")
        .and_then(|value| value.get("summary").or_else(|| value.get("text")))
        .or_else(|| metadata.get("extractedText"))
        .or_else(|| metadata.get("textContent"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

fn runtime_event_summary(content_json: &Option<Value>) -> Vec<String> {
    let Some(events) = content_json
        .as_ref()
        .and_then(|value| value.get("runtimeEvents"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    events
        .iter()
        .filter_map(|event| {
            let event_type = event.get("type").and_then(Value::as_str)?;
            match event_type {
                "worker_started" => Some("有智能体开始处理任务".to_string()),
                "worker_finished" => Some("有智能体完成处理任务".to_string()),
                "skill_requested" => event
                    .get("skill_id")
                    .and_then(Value::as_str)
                    .map(|skill| format!("调用能力：{}", safe_label(skill))),
                "skill_result" => event
                    .get("summary")
                    .and_then(Value::as_str)
                    .map(|summary| truncate_text(summary, 160)),
                "quality_checked" => Some("已完成质量检查".to_string()),
                _ => None,
            }
        })
        .take(12)
        .collect()
}

fn sanitize_user_visible_text(text: &str) -> String {
    let mut cleaned = Vec::new();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("objectkey")
            || lower.contains("object_key")
            || lower.contains("diagnostic")
            || lower.contains("failure_kind")
            || lower.contains("provider request")
            || lower.contains("api key")
            || lower.contains("prompt path")
            || lower.contains("stack trace")
        {
            continue;
        }
        cleaned.push(line);
    }
    cleaned.join("\n")
}

fn normalize_export_format(value: Option<String>) -> BizResult<String> {
    let value = normalize_optional(value).unwrap_or_else(|| "markdown".to_string());
    match value.as_str() {
        "markdown" | "text" | "json" => Ok(value),
        _ => Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "unsupported export format".to_string(),
        )),
    }
}

fn normalize_report_format(value: Option<String>) -> BizResult<String> {
    let value = normalize_optional(value).unwrap_or_else(|| "markdown".to_string());
    if value == "markdown" {
        Ok(value)
    } else {
        Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "report format must be markdown".to_string(),
        ))
    }
}

fn normalize_transcript_scope(value: Option<String>) -> BizResult<String> {
    let value = normalize_optional(value).unwrap_or_else(|| TRANSCRIPT_SCOPE_ALL.to_string());
    match value.as_str() {
        TRANSCRIPT_SCOPE_ALL | TRANSCRIPT_SCOPE_RECENT => Ok(value),
        _ => Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "unsupported transcript scope".to_string(),
        )),
    }
}

fn normalize_report_scope(value: Option<String>) -> BizResult<String> {
    let value = normalize_optional(value).unwrap_or_else(|| TRANSCRIPT_SCOPE_ALL.to_string());
    match value.as_str() {
        TRANSCRIPT_SCOPE_ALL | TRANSCRIPT_SCOPE_RECENT | "workspace_run" | "selected_sources" => {
            Ok(value)
        }
        _ => Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "unsupported report scope".to_string(),
        )),
    }
}

fn normalize_report_type(value: Option<String>) -> BizResult<String> {
    let value = normalize_optional(value).unwrap_or_else(|| REPORT_TYPE_CONVERSATION.to_string());
    match value.as_str() {
        REPORT_TYPE_CONVERSATION
        | REPORT_TYPE_COMMERCE
        | REPORT_TYPE_TEACHING
        | REPORT_TYPE_PROCESS => Ok(value),
        _ => Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            "unsupported report type".to_string(),
        )),
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_optional_uuid(value: Option<String>, label: &str) -> BizResult<Option<Uuid>> {
    normalize_optional(value)
        .map(|value| parse_uuid(&value, label))
        .transpose()
}

fn parse_uuid_list(values: Vec<String>, label: &str) -> BizResult<Vec<Uuid>> {
    if values.len() > 20 {
        return Err(BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            format!("{label} exceeds maximum length"),
        ));
    }
    values
        .into_iter()
        .map(|value| parse_uuid(&value, label))
        .collect()
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::CONVERSATION_REQUEST_INVALID,
            format!("{label} is invalid"),
        )
    })
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn truncate_vec<T>(items: &mut Vec<T>, limit: usize, label: &str, warnings: &mut Vec<String>) {
    if items.len() > limit {
        let omitted = items.len() - limit;
        items.truncate(limit);
        warnings.push(format!("{omitted} {label} omitted by material budget"));
    }
}

fn push_unique_file(items: &mut Vec<ConversationFile>, file: ConversationFile) {
    if !items.iter().any(|item| item.id == file.id) {
        items.push(file);
    }
}

fn push_unique_artifact(items: &mut Vec<WorkspaceArtifact>, artifact: WorkspaceArtifact) {
    if !items.iter().any(|item| item.id == artifact.id) {
        items.push(artifact);
    }
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

fn markdown_to_text(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| {
            line.trim_start_matches('#')
                .trim_start_matches("- ")
                .trim()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn export_mime_type(format: &str) -> &'static str {
    match format {
        "json" => "application/json; charset=utf-8",
        "text" => "text/plain; charset=utf-8",
        _ => "text/markdown; charset=utf-8",
    }
}

fn role_label(role: &str) -> &'static str {
    match role {
        "user" => "用户",
        "assistant" => "AI",
        "system" => "系统",
        "tool" => "工具",
        _ => "消息",
    }
}

fn report_title(report_type: &str) -> &'static str {
    match report_type {
        REPORT_TYPE_COMMERCE => "电商对话整理报告",
        REPORT_TYPE_TEACHING => "教学实训复盘报告",
        REPORT_TYPE_PROCESS => "多智能体过程证据报告",
        _ => "对话整理报告",
    }
}

fn scope_label(scope: &str) -> &'static str {
    match scope {
        TRANSCRIPT_SCOPE_RECENT => "最近会话",
        "workspace_run" => "指定工作区运行",
        "selected_sources" => "选定材料",
        _ => "全部会话",
    }
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let text = value.trim();
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut truncated = text.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn build_preview(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}

fn sanitize_file_base_name(value: &str) -> String {
    let text = value
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => ch,
        })
        .collect::<String>();
    let text = text.trim();
    if text.is_empty() {
        "conversation-report".to_string()
    } else {
        text.chars().take(48).collect()
    }
}

fn safe_label(value: &str) -> String {
    value.replace('_', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use time::macros::datetime;

    #[test]
    fn normalize_report_rejects_unknown_type() {
        assert!(normalize_report_type(Some("unknown".to_string())).is_err());
        assert_eq!(
            normalize_report_type(Some(REPORT_TYPE_COMMERCE.to_string())).unwrap(),
            REPORT_TYPE_COMMERCE
        );
    }

    #[test]
    fn transcript_renderer_hides_raw_runtime_json() {
        let message = test_message(
            "assistant",
            "done",
            Some(json!({
                "runtimeEvents": [
                    {"type": "skill_requested", "request_id": "internal", "skill_id": "search_market"},
                    {"type": "unknown", "objectKey": "secret"}
                ]
            })),
        );
        let material = TranscriptMaterial {
            conversation: test_conversation(),
            files: Vec::new(),
            include_runtime_events: true,
            messages: vec![message],
            warnings: Vec::new(),
        };
        let rendered = render_transcript_markdown(&material);
        assert!(rendered.contains("调用能力"));
        assert!(!rendered.contains("request_id"));
        assert!(!rendered.contains("objectKey"));
    }

    #[test]
    fn report_sanitizer_removes_internal_lines() {
        let input = "visible\nobjectKey: secret\ndiagnostic JSON: {}\nfailure_kind=bad\nvisible 2";
        let rendered = sanitize_user_visible_text(input);
        assert!(rendered.contains("visible"));
        assert!(rendered.contains("visible 2"));
        assert!(!rendered.contains("objectKey"));
        assert!(!rendered.contains("diagnostic"));
        assert!(!rendered.contains("failure_kind"));
    }

    #[test]
    fn report_renderer_includes_commerce_sections() {
        let material = ReportMaterial {
            artifacts: Vec::new(),
            conversation: test_conversation(),
            files: Vec::new(),
            include_process: true,
            include_sources: true,
            include_transcript_appendix: false,
            messages: vec![test_message("user", "怎么优化标题", None)],
            quality_checks: Vec::new(),
            report_type: REPORT_TYPE_COMMERCE.to_string(),
            run: None,
            run_tasks: Vec::new(),
            scope: TRANSCRIPT_SCOPE_ALL.to_string(),
            skill_runs: Vec::new(),
            warnings: Vec::new(),
            workspace: test_workspace(),
        };
        let rendered = render_report_markdown(&material);
        assert!(rendered.contains("电商经营判断"));
        assert!(rendered.contains("怎么优化标题"));
    }

    fn test_conversation() -> Conversation {
        Conversation {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: Some("测试会话".to_string()),
            status: "active".to_string(),
            source: "web".to_string(),
            agent_id: None,
            model_name: None,
            metadata: json!({}),
            created_at: datetime!(2026-07-08 00:00 UTC),
            updated_at: datetime!(2026-07-08 00:00 UTC),
            archived_at: None,
        }
    }

    fn test_workspace() -> Workspace {
        Workspace {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            product_id: None,
            campaign_id: None,
            title: "测试工作区".to_string(),
            workspace_type: "agent_chat".to_string(),
            status: WorkspaceStatus::Active,
            config: json!({}),
            created_at: datetime!(2026-07-08 00:00 UTC),
            updated_at: datetime!(2026-07-08 00:00 UTC),
        }
    }

    fn test_message(role: &str, content: &str, content_json: Option<Value>) -> ConversationMessage {
        ConversationMessage {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: role.to_string(),
            content_text: Some(content.to_string()),
            content_json,
            status: "completed".to_string(),
            sequence: 1,
            token_count: None,
            metadata: json!({}),
            strategy_id: None,
            response_mode: None,
            routing_mode: None,
            collaboration_mode: None,
            learning_level: None,
            created_at: datetime!(2026-07-08 00:00 UTC),
            updated_at: datetime!(2026-07-08 00:00 UTC),
        }
    }
}
