use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    Active,
    Archived,
}

impl WorkspaceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for WorkspaceStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid workspace status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Option<Uuid>,
    pub campaign_id: Option<Uuid>,
    pub title: String,
    pub workspace_type: String,
    pub status: WorkspaceStatus,
    pub config: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspace {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Option<Uuid>,
    pub campaign_id: Option<Uuid>,
    pub title: String,
    pub workspace_type: String,
    pub status: WorkspaceStatus,
    pub config: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkspace {
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub campaign_id: Option<Uuid>,
    pub title: String,
    pub workspace_type: String,
    pub status: WorkspaceStatus,
    pub config: Value,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceFilter {
    pub user_id: Uuid,
    pub status: Option<WorkspaceStatus>,
    pub product_id: Option<Uuid>,
    pub campaign_id: Option<Uuid>,
    pub workspace_type: Option<String>,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
