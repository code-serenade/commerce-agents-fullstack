use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    Domain,
    Role,
    Skill,
    Capability,
}

impl PackageType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Domain => "domain",
            Self::Role => "role",
            Self::Skill => "skill",
            Self::Capability => "capability",
        }
    }
}

impl TryFrom<&str> for PackageType {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "domain" => Ok(Self::Domain),
            "role" => Ok(Self::Role),
            "skill" => Ok(Self::Skill),
            "capability" => Ok(Self::Capability),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid package type: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageStatus {
    Active,
    Disabled,
    Deprecated,
}

impl PackageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Deprecated => "deprecated",
        }
    }
}

impl TryFrom<&str> for PackageStatus {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid package status: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRegistry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub package_type: PackageType,
    pub package_id: String,
    pub package_name: String,
    pub version: String,
    pub status: PackageStatus,
    pub manifest_path: Option<String>,
    pub dependencies: Value,
    pub compatibility: Value,
    pub metadata: Value,
    pub updated_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertPackageRegistry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub package_type: PackageType,
    pub package_id: String,
    pub package_name: String,
    pub version: String,
    pub status: PackageStatus,
    pub manifest_path: Option<String>,
    pub dependencies: Value,
    pub compatibility: Value,
    pub metadata: Value,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePackageRegistry {
    pub id: Uuid,
    pub package_name: String,
    pub version: String,
    pub status: PackageStatus,
    pub manifest_path: Option<String>,
    pub dependencies: Value,
    pub compatibility: Value,
    pub metadata: Value,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct PackageRegistryFilter {
    pub user_id: Uuid,
    pub package_type: Option<PackageType>,
    pub status: Option<PackageStatus>,
    pub include_disabled: bool,
    pub keyword: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
