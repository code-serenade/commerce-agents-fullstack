use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ListPackageCatalogRequest {
    pub user_id: String,
    pub package_type: Option<String>,
    pub status: Option<String>,
    pub include_disabled: Option<bool>,
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertPackageRequest {
    pub user_id: String,
    pub package_type: String,
    pub package_id: String,
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub manifest_path: Option<String>,
    pub dependencies: Option<Value>,
    pub compatibility: Option<Value>,
    pub metadata: Option<Value>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPackageRequest {
    pub user_id: String,
    pub package_type: String,
    pub package_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePackageRequest {
    pub user_id: String,
    pub package_type: String,
    pub package_id: String,
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub manifest_path: Option<String>,
    pub dependencies: Option<Value>,
    pub compatibility: Option<Value>,
    pub metadata: Option<Value>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageStatusChangeRequest {
    pub user_id: String,
    pub package_type: String,
    pub package_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPackageHistoryRequest {
    pub user_id: String,
    pub package_type: String,
    pub package_id: String,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPackageLocksRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SavePackageLocksRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
    pub entries: Vec<PackageLockInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageLockInput {
    pub package_type: String,
    pub package_id: String,
    pub package_name: Option<String>,
    pub locked_version: Option<String>,
    pub locked_status: Option<String>,
    pub required: Option<bool>,
    pub lock_source: Option<String>,
    pub manifest_path: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageLockDriftRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCatalogResponse {
    pub packages: Vec<PackageResponse>,
    pub sections: Vec<PackageCatalogSection>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCatalogSection {
    pub package_type: String,
    pub items: Vec<PackageResponse>,
    pub active_count: u64,
    pub disabled_count: u64,
    pub deprecated_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageResponse {
    pub id: String,
    pub user_id: String,
    pub package_type: String,
    pub package_id: String,
    pub package_name: String,
    pub version: String,
    pub status: String,
    pub manifest_path: Option<String>,
    pub dependencies: Value,
    pub compatibility: Value,
    pub metadata: Value,
    pub updated_by: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageValidationResponse {
    pub package: PackageResponse,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub dependencies: Vec<PackageDependencyValidation>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDependencyValidation {
    pub package_type: String,
    pub package_id: String,
    pub required: bool,
    pub found: bool,
    pub status: Option<String>,
    pub valid: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageHistoryListResponse {
    pub history: Vec<PackageHistoryResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageHistoryResponse {
    pub id: String,
    pub user_id: String,
    pub package_registry_id: Option<String>,
    pub package_type: String,
    pub package_id: String,
    pub from_status: String,
    pub to_status: String,
    pub reason: Option<String>,
    pub changed_by: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageLockListResponse {
    pub locks: Vec<PackageLockResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageLockResponse {
    pub id: String,
    pub run_id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub package_registry_id: Option<String>,
    pub package_type: String,
    pub package_id: String,
    pub package_name: String,
    pub locked_version: String,
    pub locked_status: String,
    pub required: bool,
    pub lock_source: Option<String>,
    pub manifest_path: Option<String>,
    pub notes: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageLockDriftResponse {
    pub locks: Vec<PackageLockResponse>,
    pub drifts: Vec<PackageLockDrift>,
    pub summary: PackageLockDriftSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageLockDrift {
    pub package_type: String,
    pub package_id: String,
    pub drift_type: String,
    pub severity: String,
    pub blocking: bool,
    pub message: String,
    pub locked_version: Option<String>,
    pub current_version: Option<String>,
    pub locked_status: Option<String>,
    pub current_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageLockDriftSummary {
    pub total: u64,
    pub blocking_count: u64,
    pub warning_count: u64,
    pub has_blocking: bool,
}
