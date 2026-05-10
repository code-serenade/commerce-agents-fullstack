use std::sync::OnceLock;

use service::dto::external_search::ExternalSearchConfig;

use crate::error::{Error, Result};

static EXTERNAL_SEARCH_CONFIG: OnceLock<ExternalSearchConfig> = OnceLock::new();

pub fn init_external_search(config: ExternalSearchConfig) -> Result<()> {
    EXTERNAL_SEARCH_CONFIG
        .set(config)
        .map_err(|_| Error::Custom("External search config already initialized".to_string()))
}

pub fn get_external_search_config() -> ExternalSearchConfig {
    EXTERNAL_SEARCH_CONFIG.get().cloned().unwrap_or_default()
}
