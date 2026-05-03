use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::llm::{LlmInput, LlmOutput},
};

use crate::{
    error::{Error, Result},
    settings::LlmCfg,
};

pub type LlmClient = ChatCompletionsLlm;

struct LlmRegistry {
    default_name: String,
    clients: HashMap<String, Arc<LlmClient>>,
}

static LLM_CLIENTS: OnceLock<LlmRegistry> = OnceLock::new();

pub fn init_llm_clients(configs: Vec<LlmCfg>) -> Result<()> {
    if configs.is_empty() {
        return Ok(());
    }

    let default_name = configs
        .first()
        .map(|config| config.name.clone())
        .expect("configs is not empty");
    let mut clients = HashMap::with_capacity(configs.len());

    for config in configs {
        if clients.contains_key(&config.name) {
            return Err(Error::Custom(format!(
                "duplicate llm client name: {}",
                config.name
            )));
        }

        let mut client =
            ChatCompletionsLlm::new(&config.base_url, &config.model, config.api_key.as_deref())
                .map_err(|err| {
                    Error::Custom(format!("init llm client '{}' failed: {err}", config.name))
                })?
                .with_max_tokens(config.max_tokens)
                .with_temperature(config.temperature)
                .with_reasoning_effort(config.reasoning_effort.as_deref());

        if let Some(endpoint) = config.chat_completions_endpoint {
            client = client.with_chat_completions_endpoint(endpoint);
        }

        clients.insert(config.name, Arc::new(client));
    }

    LLM_CLIENTS
        .set(LlmRegistry {
            default_name,
            clients,
        })
        .map_err(|_| Error::Custom("LLM clients already initialized".to_owned()))?;

    Ok(())
}

#[allow(dead_code)]
pub fn get_llm_client(name: Option<&str>) -> Result<Arc<LlmClient>> {
    let registry = LLM_CLIENTS
        .get()
        .ok_or_else(|| Error::Custom("LLM clients not initialized".to_owned()))?;
    let name = name.unwrap_or(&registry.default_name);

    registry
        .clients
        .get(name)
        .cloned()
        .ok_or_else(|| Error::Custom(format!("LLM client '{name}' not found")))
}

#[allow(dead_code)]
pub fn list_llm_client_names() -> Result<Vec<String>> {
    let registry = LLM_CLIENTS
        .get()
        .ok_or_else(|| Error::Custom("LLM clients not initialized".to_owned()))?;
    let mut names: Vec<String> = registry.clients.keys().cloned().collect();
    names.sort();
    Ok(names)
}

#[allow(dead_code)]
pub async fn chat_once(name: Option<&str>, input: LlmInput) -> Result<LlmOutput> {
    get_llm_client(name)?
        .chat_once(input)
        .await
        .map_err(|err| Error::Custom(format!("LLM chat_once failed: {err}")))
}
