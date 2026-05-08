use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::table::package_registry::{PackageStatus, PackageType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRunPackageLock {
    pub id: Uuid,
    pub run_id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub package_registry_id: Option<Uuid>,
    pub package_type: PackageType,
    pub package_id: String,
    pub package_name: String,
    pub locked_version: String,
    pub locked_status: PackageStatus,
    pub required: bool,
    pub lock_source: Option<String>,
    pub manifest_path: Option<String>,
    pub notes: Option<String>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceRunPackageLock {
    pub id: Uuid,
    pub run_id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub package_registry_id: Option<Uuid>,
    pub package_type: PackageType,
    pub package_id: String,
    pub package_name: String,
    pub locked_version: String,
    pub locked_status: PackageStatus,
    pub required: bool,
    pub lock_source: Option<String>,
    pub manifest_path: Option<String>,
    pub notes: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceRunPackageLockFilter {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub run_id: Uuid,
}
