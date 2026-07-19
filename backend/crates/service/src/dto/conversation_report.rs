use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::workspace::{WorkspaceArtifactExportResponse, WorkspaceArtifactResponse};

#[derive(Debug, Clone, Deserialize)]
pub struct ExportConversationTranscriptRequest {
    pub user_id: String,
    pub conversation_id: String,
    pub format: Option<String>,
    pub include_files: Option<bool>,
    pub include_runtime_events: Option<bool>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationTranscriptExportResponse {
    pub conversation_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub format: String,
    pub content: String,
    pub content_chars: usize,
    pub message_count: usize,
    pub file_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateConversationReportRequest {
    pub user_id: String,
    pub conversation_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub report_type: Option<String>,
    pub scope: Option<String>,
    pub format: Option<String>,
    pub include_process: Option<bool>,
    pub include_sources: Option<bool>,
    pub include_transcript_appendix: Option<bool>,
    pub artifact_ids: Option<Vec<String>>,
    pub file_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateConversationReportResponse {
    pub artifact: WorkspaceArtifactResponse,
    pub export: GeneratedConversationReportExport,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedConversationReportExport {
    pub file_name: String,
    pub mime_type: String,
    pub format: String,
    pub content: String,
    pub content_chars: usize,
    pub warnings: Vec<String>,
}

impl From<WorkspaceArtifactExportResponse> for GeneratedConversationReportExport {
    fn from(value: WorkspaceArtifactExportResponse) -> Self {
        Self {
            file_name: value.file_name,
            mime_type: value.mime_type,
            format: value.format,
            content: value.content,
            content_chars: value.content_chars,
            warnings: value.warnings,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationReportMaterialCounts {
    pub messages: usize,
    pub files: usize,
    pub artifacts: usize,
    pub skill_runs: usize,
    pub quality_checks: usize,
    pub run_tasks: usize,
}

impl ConversationReportMaterialCounts {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "messages": self.messages,
            "files": self.files,
            "artifacts": self.artifacts,
            "skillRuns": self.skill_runs,
            "qualityChecks": self.quality_checks,
            "runTasks": self.run_tasks,
        })
    }
}
