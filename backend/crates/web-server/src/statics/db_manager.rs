use std::sync::OnceLock;

use db_core::{DatabaseConfig, DatabaseManager, DbContext};

use crate::error::{Error, Result};

static DB_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

pub async fn init_db(configs: Vec<DatabaseConfig>) -> Result<()> {
    let manager = DatabaseManager::new(configs)
        .await
        .map_err(|e| Error::Custom(format!("DatabaseManager init failed: {e}")))?;

    DB_MANAGER
        .set(manager)
        .map_err(|_| Error::Custom("DatabaseManager already initialized".to_owned()))?;
    Ok(())
}

fn get_db_manager() -> &'static DatabaseManager {
    DB_MANAGER.get().expect("DatabaseManager not initialized")
}

#[allow(dead_code)]
pub fn get_default_ctx() -> DbContext {
    get_db_manager().default()
}

pub fn get_admin_ctx() -> DbContext {
    get_specific_ctx("app")
}

#[allow(dead_code)]
pub fn get_app_ctx() -> DbContext {
    get_specific_ctx("app")
}

pub fn get_specific_ctx(name: &str) -> DbContext {
    get_db_manager()
        .get(name)
        .unwrap_or_else(|_| panic!("database '{name}' not found"))
}
