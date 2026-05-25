use std::sync::OnceLock;

use agent::api::SearchAdapterConfig;

use crate::error::{Error, Result};

static EXTERNAL_SEARCH_CONFIG: OnceLock<SearchAdapterConfig> = OnceLock::new();

pub fn init_external_search(config: SearchAdapterConfig) -> Result<()> {
    EXTERNAL_SEARCH_CONFIG
        .set(config)
        .map_err(|_| Error::Custom("External search config already initialized".to_string()))
}

pub fn get_external_search_config() -> SearchAdapterConfig {
    EXTERNAL_SEARCH_CONFIG.get().cloned().unwrap_or_default()
}
