use std::env;

use agent::api::AgentLlmCfg;
use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::llm::{ChatMessage, LlmInput},
};
use serde::Deserialize;
use toolcraft_config::load_settings;

const DEFAULT_CONFIG_PATH: &str = "config/services.toml";

#[derive(Debug, Deserialize)]
struct ProbeSettings {
    #[serde(default)]
    llm: Vec<AgentLlmCfg>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let config_path = option_value(&args, "--config")
        .or_else(|| env::var("CONFIG_PATH").ok())
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let requested_llm = option_value(&args, "--llm");
    let settings: ProbeSettings = load_settings(&config_path)?;
    let targets = selected_llms(&settings.llm, requested_llm.as_deref())?;

    println!("config_path={config_path}");
    println!(
        "llm_names=[{}]",
        settings
            .llm
            .iter()
            .map(|llm| llm.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );

    for llm in targets {
        println!(
            "probe_start name={} model={} base_url={}",
            llm.name, llm.model, llm.base_url
        );
        match probe_llm(llm).await {
            Ok(content) => println!("probe_ok name={} content={}", llm.name, content.trim()),
            Err(err) => println!("probe_err name={} error={}", llm.name, err),
        }
    }

    Ok(())
}

async fn probe_llm(llm: &AgentLlmCfg) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ChatCompletionsLlm::new(&llm.base_url, &llm.model, llm.api_key.as_deref())?
        .with_max_tokens(Some(64))
        .with_temperature(Some(0.0))
        .with_reasoning_effort(llm.reasoning_effort.as_deref());
    if let Some(endpoint) = &llm.chat_completions_endpoint {
        client = client.with_chat_completions_endpoint(endpoint.clone());
    }
    let output = client
        .chat_once(LlmInput {
            messages: vec![ChatMessage::user("只回复 ok")],
        })
        .await?;
    Ok(output.get_content().to_string())
}

fn selected_llms<'a>(
    llms: &'a [AgentLlmCfg],
    requested: Option<&str>,
) -> Result<Vec<&'a AgentLlmCfg>, Box<dyn std::error::Error>> {
    if llms.is_empty() {
        return Err("no [[llm]] entries configured".into());
    }
    let Some(requested) = requested.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(llms.iter().collect());
    };
    let llm = llms
        .iter()
        .find(|llm| llm.name == requested)
        .ok_or_else(|| format!("llm '{requested}' not found"))?;
    Ok(vec![llm])
}

fn option_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find_map(|items| {
        if items[0] == key {
            Some(items[1].clone())
        } else {
            None
        }
    })
}
