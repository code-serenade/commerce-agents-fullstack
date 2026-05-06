use std::sync::OnceLock;

use storage::{ObjectStorage, StorageConfig};

use crate::error::{Error, Result};

static STORAGE: OnceLock<ObjectStorage> = OnceLock::new();

pub fn init_storage(config: StorageConfig) -> Result<()> {
    let storage = ObjectStorage::new(config)
        .map_err(|err| Error::Custom(format!("storage init failed: {err}")))?;
    STORAGE
        .set(storage)
        .map_err(|_| Error::Custom("Storage already initialized".to_owned()))?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_storage() -> &'static ObjectStorage {
    STORAGE.get().expect("Storage not initialized")
}
