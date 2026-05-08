use std::collections::BTreeMap;

use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    app_users::{AppUserService, AppUserStatus},
    package_registry::{
        PackageRegistry, PackageRegistryFilter, PackageRegistryService, PackageStatus, PackageType,
        UpdatePackageRegistry, UpsertPackageRegistry,
    },
    package_state_history::{
        CreatePackageStateHistory, PackageStateHistory, PackageStateHistoryFilter,
        PackageStateHistoryService,
    },
    workspace_run_package_locks::{
        CreateWorkspaceRunPackageLock, WorkspaceRunPackageLock, WorkspaceRunPackageLockFilter,
        WorkspaceRunPackageLockService,
    },
    workspace_runs::{WorkspaceRun, WorkspaceRunService},
    workspaces::{WorkspaceService, WorkspaceStatus},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::dto::package_governance::*;

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct PackageGovernanceApi {
    db: DbContext,
    app_user_svc: AppUserService,
    package_svc: PackageRegistryService,
    history_svc: PackageStateHistoryService,
    workspace_svc: WorkspaceService,
    run_svc: WorkspaceRunService,
    lock_svc: WorkspaceRunPackageLockService,
}

impl PackageGovernanceApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            db: db.clone(),
            app_user_svc: AppUserService::new(db.clone()),
            package_svc: PackageRegistryService::new(db.clone()),
            history_svc: PackageStateHistoryService::new(db.clone()),
            workspace_svc: WorkspaceService::new(db.clone()),
            run_svc: WorkspaceRunService::new(db.clone()),
            lock_svc: WorkspaceRunPackageLockService::new(db),
        }
    }

    pub async fn catalog(
        &self,
        req: ListPackageCatalogRequest,
    ) -> BizResult<PackageCatalogResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let package_type = parse_optional_package_type(req.package_type)?;
        let status = parse_optional_package_status(req.status)?;
        let include_disabled = req.include_disabled.unwrap_or(false);
        let (page, page_size, offset) = pagination(req.page, req.page_size);
        let filter = PackageRegistryFilter {
            user_id,
            package_type,
            status,
            include_disabled,
            keyword: normalize_optional(req.keyword.clone()),
            limit: Some(page_size),
            offset: Some(offset),
        };
        let count_filter = PackageRegistryFilter {
            limit: None,
            offset: None,
            ..filter.clone()
        };
        let total = self.package_svc.count(count_filter).await?;
        let packages = self
            .package_svc
            .list(filter)
            .await?
            .into_iter()
            .map(map_package)
            .collect();
        let all_for_sections = self
            .package_svc
            .list(PackageRegistryFilter {
                user_id,
                package_type,
                status: None,
                include_disabled: true,
                keyword: normalize_optional(req.keyword),
                limit: None,
                offset: None,
            })
            .await?;
        let sections = catalog_sections(all_for_sections);
        Ok(PackageCatalogResponse {
            packages,
            sections,
            total,
            page,
            page_size,
        })
    }

    pub async fn upsert_package(&self, req: UpsertPackageRequest) -> BizResult<PackageResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let package_type = parse_package_type(&req.package_type)?;
        let package_id = normalize_required(req.package_id, "package_id")?;
        let current = self
            .package_svc
            .get_by_user_package(user_id, package_type, package_id.as_str())
            .await?;
        let status = parse_optional_package_status(req.status)?.unwrap_or(PackageStatus::Active);
        let package = self
            .package_svc
            .upsert(UpsertPackageRegistry {
                id: Uuid::new_v4(),
                user_id,
                package_type,
                package_id: package_id.clone(),
                package_name: normalize_optional(req.package_name)
                    .unwrap_or_else(|| package_id.clone()),
                version: normalize_optional(req.version).unwrap_or_else(default_package_version),
                status,
                manifest_path: normalize_optional(req.manifest_path),
                dependencies: req.dependencies.unwrap_or_else(|| json!({})),
                compatibility: req.compatibility.unwrap_or_else(|| json!({})),
                metadata: req.metadata.unwrap_or_else(|| json!({})),
                updated_by: Some(user_id),
            })
            .await?;
        if let Some(current) = current
            && current.status != package.status
        {
            self.record_history(
                &package,
                current.status,
                package.status,
                normalize_optional(req.reason),
                json!({"source": "upsert"}),
            )
            .await?;
        }
        Ok(map_package(package))
    }

    pub async fn get_package(&self, req: GetPackageRequest) -> BizResult<PackageResponse> {
        let package = self.ensure_package_owner_from_strings(&req).await?;
        Ok(map_package(package))
    }

    pub async fn update_package(&self, req: UpdatePackageRequest) -> BizResult<PackageResponse> {
        let current = self
            .ensure_package_owner_from_strings(&GetPackageRequest {
                user_id: req.user_id.clone(),
                package_type: req.package_type.clone(),
                package_id: req.package_id.clone(),
            })
            .await?;
        let status = parse_optional_package_status(req.status)?.unwrap_or(current.status);
        let package = self
            .package_svc
            .update(UpdatePackageRegistry {
                id: current.id,
                package_name: req
                    .package_name
                    .and_then(normalize_string)
                    .unwrap_or_else(|| current.package_name.clone()),
                version: req
                    .version
                    .and_then(normalize_string)
                    .unwrap_or_else(|| current.version.clone()),
                status,
                manifest_path: req
                    .manifest_path
                    .map(normalize_string)
                    .unwrap_or(current.manifest_path.clone()),
                dependencies: req
                    .dependencies
                    .unwrap_or_else(|| current.dependencies.clone()),
                compatibility: req
                    .compatibility
                    .unwrap_or_else(|| current.compatibility.clone()),
                metadata: req.metadata.unwrap_or_else(|| current.metadata.clone()),
                updated_by: Some(current.user_id),
            })
            .await?;
        if current.status != package.status {
            self.record_history(
                &package,
                current.status,
                package.status,
                normalize_optional(req.reason),
                json!({"source": "update"}),
            )
            .await?;
        }
        Ok(map_package(package))
    }

    pub async fn validate_package(
        &self,
        req: GetPackageRequest,
    ) -> BizResult<PackageValidationResponse> {
        let package = self.ensure_package_owner_from_strings(&req).await?;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if package.status != PackageStatus::Active {
            errors.push(format!("package status is {}", package.status.as_str()));
        }

        let mut dependency_results = Vec::new();
        for dependency in dependency_entries(&package.dependencies) {
            let current = self
                .package_svc
                .get_by_user_package(
                    package.user_id,
                    dependency.package_type,
                    dependency.package_id.as_str(),
                )
                .await?;
            let result = match current {
                Some(dep) if dep.status == PackageStatus::Active => PackageDependencyValidation {
                    package_type: dependency.package_type.as_str().to_string(),
                    package_id: dependency.package_id.clone(),
                    required: dependency.required,
                    found: true,
                    status: Some(dep.status.as_str().to_string()),
                    valid: true,
                    message: None,
                },
                Some(dep) => {
                    let message = format!(
                        "dependency {}:{} is {}",
                        dependency.package_type.as_str(),
                        dependency.package_id,
                        dep.status.as_str()
                    );
                    if dependency.required {
                        errors.push(message.clone());
                    } else {
                        warnings.push(message.clone());
                    }
                    PackageDependencyValidation {
                        package_type: dependency.package_type.as_str().to_string(),
                        package_id: dependency.package_id.clone(),
                        required: dependency.required,
                        found: true,
                        status: Some(dep.status.as_str().to_string()),
                        valid: !dependency.required,
                        message: Some(message),
                    }
                }
                None => {
                    let message = format!(
                        "dependency {}:{} is missing",
                        dependency.package_type.as_str(),
                        dependency.package_id
                    );
                    if dependency.required {
                        errors.push(message.clone());
                    } else {
                        warnings.push(message.clone());
                    }
                    PackageDependencyValidation {
                        package_type: dependency.package_type.as_str().to_string(),
                        package_id: dependency.package_id.clone(),
                        required: dependency.required,
                        found: false,
                        status: None,
                        valid: !dependency.required,
                        message: Some(message),
                    }
                }
            };
            dependency_results.push(result);
        }

        if compatibility_incompatible(&package.compatibility) {
            errors.push("package compatibility marks this package as incompatible".to_string());
        }

        let valid = errors.is_empty();
        Ok(PackageValidationResponse {
            package: map_package(package),
            valid,
            errors,
            warnings,
            dependencies: dependency_results,
        })
    }

    pub async fn activate_package(
        &self,
        req: PackageStatusChangeRequest,
    ) -> BizResult<PackageResponse> {
        self.transition_package(req, PackageStatus::Active, "activate")
            .await
    }

    pub async fn deactivate_package(
        &self,
        req: PackageStatusChangeRequest,
    ) -> BizResult<PackageResponse> {
        self.transition_package(req, PackageStatus::Disabled, "deactivate")
            .await
    }

    pub async fn rollback_package(
        &self,
        req: PackageStatusChangeRequest,
    ) -> BizResult<PackageResponse> {
        let current = self
            .ensure_package_owner_from_strings(&GetPackageRequest {
                user_id: req.user_id,
                package_type: req.package_type,
                package_id: req.package_id,
            })
            .await?;
        let history = self
            .history_svc
            .list(PackageStateHistoryFilter {
                user_id: current.user_id,
                package_type: Some(current.package_type),
                package_id: Some(current.package_id.clone()),
                limit: Some(1),
                offset: None,
            })
            .await?;
        let previous_status = history
            .first()
            .map(|item| item.from_status)
            .ok_or_else(|| {
                BizError::new(
                    app_error::PACKAGE_REQUEST_INVALID,
                    "package has no status history to rollback".to_string(),
                )
            })?;
        let updated = self
            .package_svc
            .update(UpdatePackageRegistry {
                id: current.id,
                package_name: current.package_name,
                version: current.version,
                status: previous_status,
                manifest_path: current.manifest_path,
                dependencies: current.dependencies,
                compatibility: current.compatibility,
                metadata: current.metadata,
                updated_by: Some(current.user_id),
            })
            .await?;
        self.record_history(
            &updated,
            current.status,
            previous_status,
            normalize_optional(req.reason).or_else(|| Some("rollback".to_string())),
            json!({"source": "rollback"}),
        )
        .await?;
        Ok(map_package(updated))
    }

    pub async fn package_history(
        &self,
        req: ListPackageHistoryRequest,
    ) -> BizResult<PackageHistoryListResponse> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let package_type = parse_package_type(&req.package_type)?;
        let package_id = normalize_required(req.package_id, "package_id")?;
        self.ensure_package_owner(user_id, package_type, package_id.as_str())
            .await?;
        let history = self
            .history_svc
            .list(PackageStateHistoryFilter {
                user_id,
                package_type: Some(package_type),
                package_id: Some(package_id),
                limit: Some(req.limit.unwrap_or(50).clamp(1, 500)),
                offset: None,
            })
            .await?
            .into_iter()
            .map(map_history)
            .collect();
        Ok(PackageHistoryListResponse { history })
    }

    pub async fn list_package_locks(
        &self,
        req: ListPackageLocksRequest,
    ) -> BizResult<PackageLockListResponse> {
        let (user_id, workspace_id, run_id) = self
            .ensure_request_run(&req.user_id, &req.workspace_id, &req.run_id)
            .await?;
        let locks = self
            .lock_svc
            .list(WorkspaceRunPackageLockFilter {
                user_id,
                workspace_id,
                run_id,
            })
            .await?
            .into_iter()
            .map(map_lock)
            .collect();
        Ok(PackageLockListResponse { locks })
    }

    pub async fn save_package_locks(
        &self,
        req: SavePackageLocksRequest,
    ) -> BizResult<PackageLockListResponse> {
        let (user_id, workspace_id, run_id) = self
            .ensure_request_run(&req.user_id, &req.workspace_id, &req.run_id)
            .await?;
        let mut entries = Vec::with_capacity(req.entries.len());
        for entry in req.entries {
            let package_type = parse_package_type(&entry.package_type)?;
            let package_id = normalize_required(entry.package_id, "package_id")?;
            let current = self
                .package_svc
                .get_by_user_package(user_id, package_type, package_id.as_str())
                .await?;
            let package_name = normalize_optional(entry.package_name)
                .or_else(|| current.as_ref().map(|item| item.package_name.clone()))
                .unwrap_or_else(|| package_id.clone());
            let locked_version = normalize_optional(entry.locked_version)
                .or_else(|| current.as_ref().map(|item| item.version.clone()))
                .unwrap_or_else(default_package_version);
            let locked_status = parse_optional_package_status(entry.locked_status)?
                .or_else(|| current.as_ref().map(|item| item.status))
                .unwrap_or(PackageStatus::Active);
            entries.push(CreateWorkspaceRunPackageLock {
                id: Uuid::new_v4(),
                run_id,
                workspace_id,
                user_id,
                package_registry_id: current.as_ref().map(|item| item.id),
                package_type,
                package_id,
                package_name,
                locked_version,
                locked_status,
                required: entry.required.unwrap_or(true),
                lock_source: normalize_optional(entry.lock_source),
                manifest_path: normalize_optional(entry.manifest_path)
                    .or_else(|| current.as_ref().and_then(|item| item.manifest_path.clone())),
                notes: normalize_optional(entry.notes),
                metadata: entry.metadata.unwrap_or_else(|| json!({})),
            });
        }

        let tx = self.db.begin().await?;
        let lock_tx_svc = WorkspaceRunPackageLockService::new(tx.clone());
        let result = lock_tx_svc.replace_for_run(run_id, entries).await;
        let locks = match result {
            Ok(locks) => {
                tx.commit().await?;
                locks
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };
        Ok(PackageLockListResponse {
            locks: locks.into_iter().map(map_lock).collect(),
        })
    }

    pub async fn package_lock_drift(
        &self,
        req: PackageLockDriftRequest,
    ) -> BizResult<PackageLockDriftResponse> {
        let (user_id, workspace_id, run_id) = self
            .ensure_request_run(&req.user_id, &req.workspace_id, &req.run_id)
            .await?;
        let locks = self
            .lock_svc
            .list(WorkspaceRunPackageLockFilter {
                user_id,
                workspace_id,
                run_id,
            })
            .await?;
        let mut drifts = Vec::new();
        for lock in &locks {
            let current = self
                .package_svc
                .get_by_user_package(user_id, lock.package_type, lock.package_id.as_str())
                .await?;
            match current {
                None => drifts.push(lock_drift(
                    lock,
                    "missing",
                    true,
                    "locked package is missing from current registry".to_string(),
                    None,
                    None,
                )),
                Some(package) => {
                    if package.version != lock.locked_version {
                        drifts.push(PackageLockDrift {
                            package_type: lock.package_type.as_str().to_string(),
                            package_id: lock.package_id.clone(),
                            drift_type: "version_changed".to_string(),
                            severity: "warning".to_string(),
                            blocking: false,
                            message: "package version changed after run lock was created"
                                .to_string(),
                            locked_version: Some(lock.locked_version.clone()),
                            current_version: Some(package.version.clone()),
                            locked_status: Some(lock.locked_status.as_str().to_string()),
                            current_status: Some(package.status.as_str().to_string()),
                        });
                    }
                    if package.status != lock.locked_status {
                        let blocking = lock.required && package.status != PackageStatus::Active;
                        drifts.push(PackageLockDrift {
                            package_type: lock.package_type.as_str().to_string(),
                            package_id: lock.package_id.clone(),
                            drift_type: "status_changed".to_string(),
                            severity: if blocking { "blocking" } else { "warning" }.to_string(),
                            blocking,
                            message: "package status changed after run lock was created"
                                .to_string(),
                            locked_version: Some(lock.locked_version.clone()),
                            current_version: Some(package.version.clone()),
                            locked_status: Some(lock.locked_status.as_str().to_string()),
                            current_status: Some(package.status.as_str().to_string()),
                        });
                    }
                    if lock.required && package.status != PackageStatus::Active {
                        drifts.push(PackageLockDrift {
                            package_type: lock.package_type.as_str().to_string(),
                            package_id: lock.package_id.clone(),
                            drift_type: "required_package_inactive".to_string(),
                            severity: "blocking".to_string(),
                            blocking: true,
                            message: "required package is not active".to_string(),
                            locked_version: Some(lock.locked_version.clone()),
                            current_version: Some(package.version),
                            locked_status: Some(lock.locked_status.as_str().to_string()),
                            current_status: Some(package.status.as_str().to_string()),
                        });
                    }
                }
            }
        }
        let blocking_count = drifts.iter().filter(|item| item.blocking).count() as u64;
        let warning_count = drifts.len() as u64 - blocking_count;
        Ok(PackageLockDriftResponse {
            locks: locks.into_iter().map(map_lock).collect(),
            summary: PackageLockDriftSummary {
                total: drifts.len() as u64,
                blocking_count,
                warning_count,
                has_blocking: blocking_count > 0,
            },
            drifts,
        })
    }

    async fn transition_package(
        &self,
        req: PackageStatusChangeRequest,
        target_status: PackageStatus,
        source: &str,
    ) -> BizResult<PackageResponse> {
        let current = self
            .ensure_package_owner_from_strings(&GetPackageRequest {
                user_id: req.user_id,
                package_type: req.package_type,
                package_id: req.package_id,
            })
            .await?;
        if current.status == target_status {
            return Ok(map_package(current));
        }
        let updated = self
            .package_svc
            .update(UpdatePackageRegistry {
                id: current.id,
                package_name: current.package_name,
                version: current.version,
                status: target_status,
                manifest_path: current.manifest_path,
                dependencies: current.dependencies,
                compatibility: current.compatibility,
                metadata: current.metadata,
                updated_by: Some(current.user_id),
            })
            .await?;
        self.record_history(
            &updated,
            current.status,
            target_status,
            normalize_optional(req.reason),
            json!({"source": source}),
        )
        .await?;
        Ok(map_package(updated))
    }

    async fn record_history(
        &self,
        package: &PackageRegistry,
        from_status: PackageStatus,
        to_status: PackageStatus,
        reason: Option<String>,
        metadata: Value,
    ) -> BizResult<PackageStateHistory> {
        self.history_svc
            .create(CreatePackageStateHistory {
                id: Uuid::new_v4(),
                user_id: package.user_id,
                package_registry_id: Some(package.id),
                package_type: package.package_type,
                package_id: package.package_id.clone(),
                from_status,
                to_status,
                reason,
                changed_by: Some(package.user_id),
                metadata,
            })
            .await
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

    async fn ensure_package_owner_from_strings(
        &self,
        req: &GetPackageRequest,
    ) -> BizResult<PackageRegistry> {
        let user_id = parse_user_id(&req.user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let package_type = parse_package_type(&req.package_type)?;
        let package_id = normalize_required(req.package_id.clone(), "package_id")?;
        self.ensure_package_owner(user_id, package_type, package_id.as_str())
            .await
    }

    async fn ensure_package_owner(
        &self,
        user_id: Uuid,
        package_type: PackageType,
        package_id: &str,
    ) -> BizResult<PackageRegistry> {
        self.package_svc
            .get_by_user_package(user_id, package_type, package_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::PACKAGE_NOT_FOUND,
                    "package not found".to_string(),
                )
            })
    }

    async fn ensure_request_run(
        &self,
        user_id: &str,
        workspace_id: &str,
        run_id: &str,
    ) -> BizResult<(Uuid, Uuid, Uuid)> {
        let user_id = parse_user_id(user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        let workspace_id = parse_uuid(workspace_id, "workspace_id")?;
        let run_id = parse_uuid(run_id, "run_id")?;
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
        let run: WorkspaceRun = self.run_svc.get_by_id(run_id).await?.ok_or_else(|| {
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
        Ok((user_id, workspace_id, run_id))
    }
}

#[derive(Debug, Clone)]
struct DependencyEntry {
    package_type: PackageType,
    package_id: String,
    required: bool,
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    parse_uuid(user_id, "user_id")
}

fn parse_uuid(value: &str, label: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| {
        BizError::new(
            app_error::PACKAGE_REQUEST_INVALID,
            format!("invalid {label}: must be uuid"),
        )
    })
}

fn parse_package_type(value: &str) -> BizResult<PackageType> {
    PackageType::try_from(value.trim().to_lowercase().as_str()).map_err(|_| {
        BizError::new(
            app_error::PACKAGE_REQUEST_INVALID,
            format!("invalid package_type: {value}"),
        )
    })
}

fn parse_optional_package_type(value: Option<String>) -> BizResult<Option<PackageType>> {
    value
        .and_then(normalize_string)
        .map(|value| parse_package_type(&value))
        .transpose()
}

fn parse_package_status(value: &str) -> BizResult<PackageStatus> {
    PackageStatus::try_from(value.trim().to_lowercase().as_str()).map_err(|_| {
        BizError::new(
            app_error::PACKAGE_REQUEST_INVALID,
            format!("invalid status: {value}"),
        )
    })
}

fn parse_optional_package_status(value: Option<String>) -> BizResult<Option<PackageStatus>> {
    value
        .and_then(normalize_string)
        .map(|value| parse_package_status(&value))
        .transpose()
}

fn normalize_required(value: String, label: &str) -> BizResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BizError::new(
            app_error::PACKAGE_REQUEST_INVALID,
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

fn default_package_version() -> String {
    "0.1.0".to_string()
}

fn pagination(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    (page, page_size, offset)
}

fn catalog_sections(packages: Vec<PackageRegistry>) -> Vec<PackageCatalogSection> {
    let mut grouped: BTreeMap<String, Vec<PackageRegistry>> = BTreeMap::new();
    for package in packages {
        grouped
            .entry(package.package_type.as_str().to_string())
            .or_default()
            .push(package);
    }
    grouped
        .into_iter()
        .map(|(package_type, items)| {
            let active_count = items
                .iter()
                .filter(|item| item.status == PackageStatus::Active)
                .count() as u64;
            let disabled_count = items
                .iter()
                .filter(|item| item.status == PackageStatus::Disabled)
                .count() as u64;
            let deprecated_count = items
                .iter()
                .filter(|item| item.status == PackageStatus::Deprecated)
                .count() as u64;
            PackageCatalogSection {
                package_type,
                items: items.into_iter().map(map_package).collect(),
                active_count,
                disabled_count,
                deprecated_count,
            }
        })
        .collect()
}

fn dependency_entries(value: &Value) -> Vec<DependencyEntry> {
    match value {
        Value::Array(items) => items.iter().filter_map(dependency_from_value).collect(),
        Value::Object(map) => {
            if let Some(items) = map.get("items").and_then(Value::as_array) {
                return items.iter().filter_map(dependency_from_value).collect();
            }
            let mut entries = Vec::new();
            for package_type in [
                PackageType::Domain,
                PackageType::Role,
                PackageType::Skill,
                PackageType::Capability,
            ] {
                if let Some(items) = map.get(package_type.as_str()) {
                    entries.extend(dependencies_for_type(package_type, items));
                }
            }
            entries
        }
        _ => Vec::new(),
    }
}

fn dependencies_for_type(package_type: PackageType, value: &Value) -> Vec<DependencyEntry> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(package_id) => Some(DependencyEntry {
                    package_type,
                    package_id: package_id.clone(),
                    required: true,
                }),
                Value::Object(_) => dependency_from_value(item),
                _ => None,
            })
            .collect(),
        Value::String(package_id) => vec![DependencyEntry {
            package_type,
            package_id: package_id.clone(),
            required: true,
        }],
        _ => Vec::new(),
    }
}

fn dependency_from_value(value: &Value) -> Option<DependencyEntry> {
    let map = value.as_object()?;
    let package_type = map
        .get("packageType")
        .or_else(|| map.get("package_type"))
        .or_else(|| map.get("type"))
        .and_then(Value::as_str)
        .and_then(|value| parse_package_type(value).ok())?;
    let package_id = map
        .get("packageId")
        .or_else(|| map.get("package_id"))
        .or_else(|| map.get("id"))
        .and_then(Value::as_str)?
        .trim()
        .to_string();
    if package_id.is_empty() {
        return None;
    }
    Some(DependencyEntry {
        package_type,
        package_id,
        required: map.get("required").and_then(Value::as_bool).unwrap_or(true),
    })
}

fn compatibility_incompatible(value: &Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    map.get("compatible").and_then(Value::as_bool) == Some(false)
        || map.get("incompatible").and_then(Value::as_bool) == Some(true)
        || map.get("status").and_then(Value::as_str) == Some("incompatible")
}

fn lock_drift(
    lock: &WorkspaceRunPackageLock,
    drift_type: &str,
    missing_is_blocking: bool,
    message: String,
    current_version: Option<String>,
    current_status: Option<String>,
) -> PackageLockDrift {
    let blocking = lock.required && missing_is_blocking;
    PackageLockDrift {
        package_type: lock.package_type.as_str().to_string(),
        package_id: lock.package_id.clone(),
        drift_type: drift_type.to_string(),
        severity: if blocking { "blocking" } else { "warning" }.to_string(),
        blocking,
        message,
        locked_version: Some(lock.locked_version.clone()),
        current_version,
        locked_status: Some(lock.locked_status.as_str().to_string()),
        current_status,
    }
}

fn map_package(item: PackageRegistry) -> PackageResponse {
    PackageResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        package_type: item.package_type.as_str().to_string(),
        package_id: item.package_id,
        package_name: item.package_name,
        version: item.version,
        status: item.status.as_str().to_string(),
        manifest_path: item.manifest_path,
        dependencies: item.dependencies,
        compatibility: item.compatibility,
        metadata: item.metadata,
        updated_by: item.updated_by.map(|id| id.to_string()),
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn map_history(item: PackageStateHistory) -> PackageHistoryResponse {
    PackageHistoryResponse {
        id: item.id.to_string(),
        user_id: item.user_id.to_string(),
        package_registry_id: item.package_registry_id.map(|id| id.to_string()),
        package_type: item.package_type.as_str().to_string(),
        package_id: item.package_id,
        from_status: item.from_status.as_str().to_string(),
        to_status: item.to_status.as_str().to_string(),
        reason: item.reason,
        changed_by: item.changed_by.map(|id| id.to_string()),
        metadata: item.metadata,
        created_at: item.created_at,
    }
}

fn map_lock(item: WorkspaceRunPackageLock) -> PackageLockResponse {
    PackageLockResponse {
        id: item.id.to_string(),
        run_id: item.run_id.to_string(),
        workspace_id: item.workspace_id.to_string(),
        user_id: item.user_id.to_string(),
        package_registry_id: item.package_registry_id.map(|id| id.to_string()),
        package_type: item.package_type.as_str().to_string(),
        package_id: item.package_id,
        package_name: item.package_name,
        locked_version: item.locked_version,
        locked_status: item.locked_status.as_str().to_string(),
        required: item.required,
        lock_source: item.lock_source,
        manifest_path: item.manifest_path,
        notes: item.notes,
        metadata: item.metadata,
        created_at: item.created_at,
    }
}
