use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::table::package_registry::{PackageStatus, PackageType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStateHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub package_registry_id: Option<Uuid>,
    pub package_type: PackageType,
    pub package_id: String,
    pub from_status: PackageStatus,
    pub to_status: PackageStatus,
    pub reason: Option<String>,
    pub changed_by: Option<Uuid>,
    pub metadata: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePackageStateHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub package_registry_id: Option<Uuid>,
    pub package_type: PackageType,
    pub package_id: String,
    pub from_status: PackageStatus,
    pub to_status: PackageStatus,
    pub reason: Option<String>,
    pub changed_by: Option<Uuid>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct PackageStateHistoryFilter {
    pub user_id: Uuid,
    pub package_type: Option<PackageType>,
    pub package_id: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
