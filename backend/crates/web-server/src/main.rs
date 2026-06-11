mod clients;
mod dto;
mod error;
mod handlers;
mod logging;
mod notifications;
mod routes;
mod settings;
mod statics;

use std::sync::Arc;

use clients::auth_client::fetch_verify_jwt;
use settings::Settings;
use toolcraft_axum_kit::http_server;

use crate::{
    logging::init_tracing_to_file,
    statics::{
        agent_manager::init_agent_manager, db_manager::init_db,
        external_search::init_external_search, request_client::init_request_client,
        storage_manager::init_storage,
    },
};

#[tokio::main]
async fn main() {
    init_tracing_to_file();
    let settings = Settings::load("config/services.toml").unwrap();
    let internal_base_url = settings
        .jwt_verify
        .url
        .trim_end_matches("/jwt_verify_config")
        .to_owned();
    init_request_client(
        internal_base_url,
        settings.jwt_verify.header.clone(),
        settings.jwt_verify.token.clone(),
    )
    .expect("Request client initialization failed");
    init_db(settings.db.clone())
        .await
        .expect("DatabaseManager initialization failed");
    init_storage(settings.s3.clone()).expect("Storage initialization failed");
    tracing::info!(
        usage_default = ?settings.agent_llm_usage.default,
        usage_fast = ?settings.agent_llm_usage.fast,
        usage_thinking = ?settings.agent_llm_usage.thinking,
        usage_vision = ?settings.agent_llm_usage.vision,
        llm_count = settings.llm.len(),
        llms = ?settings
            .llm
            .iter()
            .map(|llm| {
                format!(
                    "{}:{} thinking={:?} reasoning_effort={:?} endpoint={:?}",
                    llm.name,
                    llm.model,
                    llm.thinking,
                    llm.reasoning_effort,
                    llm.chat_completions_endpoint
                )
            })
            .collect::<Vec<_>>(),
        search_enabled = settings.external_search.enabled,
        search_base_url = ?settings.external_search.base_url,
        search_endpoint = ?settings.external_search.search_endpoint,
        worker_cognition_timeout_sec = settings
            .agent_runtime
            .worker_cognition_timeout_sec
            .unwrap_or(45),
        memory_path = ?settings.agent_runtime.memory_path,
        "agent runtime config loaded"
    );
    init_agent_manager(agent::api::AgentRuntimeCfg {
        llm: settings.llm.clone(),
        llm_usage: settings.agent_llm_usage.clone(),
        search: settings.external_search.clone(),
        worker_cognition_timeout_sec: settings
            .agent_runtime
            .worker_cognition_timeout_sec
            .unwrap_or(45),
        memory_path: settings.agent_runtime.memory_path.clone(),
    })
    .expect("Agent manager initialization failed");
    init_external_search(settings.external_search.clone())
        .expect("External search config initialization failed");
    let jwt = Arc::new(
        fetch_verify_jwt()
            .await
            .expect("VerifyJwt initialization failed"),
    );

    let router = routes::create_routes(jwt);
    let http_task = http_server::start(settings.http.port, router);

    let _ = tokio::join!(http_task);
}
