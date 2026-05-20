use std::{collections::BTreeMap, env};

use agent::api::{
    AgentId, AgentLlmCfg, AgentLlmUsageCfg, AgentRequest, AgentRuntimeCfg, SearchAdapterConfig,
    SessionEvent, SessionId, UserId, build_agent_manager,
};
use serde::Deserialize;
use serde_json::json;
use storage::{BucketKind, ObjectStorage, PresignDownloadRequest, SignedDownload, StorageConfig};
use toolcraft_config::load_settings;

const DEFAULT_CONFIG_PATH: &str = "config/services.toml";
const DEFAULT_OBJECT_KEY: &str =
    "chat/dab296dfcf45336647f6862d3f434a53/c1ef76a8-7c46-4d03-b28a-8c3b5042cc65";
const DEFAULT_INPUT: &str = "图片的内容是什么？请直接回答，不要启动多岗位任务。";

#[derive(Debug, Deserialize)]
struct ProbeSettings {
    s3: StorageConfig,
    #[serde(default)]
    llm: Vec<AgentLlmCfg>,
    #[serde(default)]
    default_llm: Option<String>,
    #[serde(default)]
    agent_llm_usage: AgentLlmUsageCfg,
    #[serde(default)]
    external_search: SearchAdapterConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let config_path = option_value(&args, "--config")
        .or_else(|| env::var("CONFIG_PATH").ok())
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let object_path = option_value(&args, "--key")
        .or_else(|| env::var("PRIVATE_OBJECT_KEY").ok())
        .unwrap_or_else(|| DEFAULT_OBJECT_KEY.to_string());
    let input = trailing_args(&args).join(" ");
    let input = if input.trim().is_empty() {
        DEFAULT_INPUT.to_string()
    } else {
        input
    };

    let settings: ProbeSettings = load_settings(&config_path)?;
    if settings.llm.is_empty() {
        return Err("no [[llm]] entries configured".into());
    }

    let (bucket, object_key) = parse_object_path(&object_path, &settings.s3)?;
    let storage = ObjectStorage::new(settings.s3.clone())?;
    let signed = storage.presign_download(PresignDownloadRequest {
        bucket,
        key: object_key.clone(),
        expires_secs: Some(600),
    })?;

    if args.iter().any(|arg| arg == "--print-url") {
        println!("{}", signed.download_url);
        return Ok(());
    }

    let images_json = serde_json::to_string(&vec![json!({
        "url": signed.download_url,
        "detail": "high",
        "source_ref": format!("private:{object_key}"),
    })])?;

    let mut metadata = BTreeMap::new();
    metadata.insert("attachments.image_count".to_string(), "1".to_string());
    metadata.insert("attachments.images_json".to_string(), images_json);
    metadata.insert("attachments.image_source".to_string(), "private_object_key_probe".to_string());
    metadata.insert("strategy.id".to_string(), "quick".to_string());
    metadata.insert("strategy.response_mode".to_string(), "quick".to_string());
    metadata.insert("strategy.collaboration_mode".to_string(), "auto".to_string());

    println!("config_path={config_path}");
    println!("bucket={}", signed.bucket);
    println!("object_key={}", signed.key);
    println!(
        "llm_names=[{}]",
        settings
            .llm
            .iter()
            .map(|llm| llm.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("default_llm={:?}", settings.default_llm);
    println!("llm_usage={:?}", settings.agent_llm_usage);
    println!("input={input}");
    println!();

    let manager = build_agent_manager(AgentRuntimeCfg {
        llm: settings.llm,
        default_llm: settings.default_llm,
        llm_usage: settings.agent_llm_usage,
        search: settings.external_search,
    })?;

    let mut events = manager.run_stream(AgentRequest {
        user_id: Some(UserId("private-image-probe-user".to_string())),
        workspace_id: None,
        agent_id: AgentId("private-image-probe-agent".to_string()),
        session_id: SessionId("private-image-probe-session".to_string()),
        input,
        metadata,
    })?;

    while let Some(event) = events.recv().await {
        match event {
            SessionEvent::RouteDecided {
                route_mode,
                primary_runtime_role,
                support_runtime_roles,
                model_name,
                ..
            } => {
                println!(
                    "event=route_decided route_mode={} primary={:?} support={:?} model={:?}",
                    route_mode, primary_runtime_role, support_runtime_roles, model_name
                );
            }
            SessionEvent::Output { content, .. } => {
                println!("event=output");
                println!("{content}");
            }
            SessionEvent::Failed { reason, .. } => {
                println!("event=failed reason={reason}");
                break;
            }
            SessionEvent::Finished { .. } => {
                println!("event=finished");
                break;
            }
            SessionEvent::WorkerStarted { worker_id, .. } => {
                println!("event=worker_started worker={}", worker_id.0);
            }
            SessionEvent::TaskGraphPlanned { graph, .. } => {
                println!("event=task_graph_planned nodes={}", graph.nodes.len());
            }
            _ => {}
        }
    }

    Ok(())
}

fn parse_object_path(
    raw: &str,
    config: &StorageConfig,
) -> Result<(BucketKind, String), Box<dyn std::error::Error>> {
    let value = raw.trim().trim_start_matches('/');
    if value.is_empty() {
        return Err("object key cannot be empty".into());
    }

    let private_prefix = format!("{}/", config.private_bucket.trim_matches('/'));
    if let Some(key) = value.strip_prefix(&private_prefix)
        && !key.trim().is_empty()
    {
        return Ok((BucketKind::Private, key.to_string()));
    }

    let public_prefix = format!("{}/", config.public_bucket.trim_matches('/'));
    if let Some(key) = value.strip_prefix(&public_prefix)
        && !key.trim().is_empty()
    {
        return Ok((BucketKind::Public, key.to_string()));
    }

    Ok((BucketKind::Private, value.to_string()))
}

#[allow(dead_code)]
fn _signed_download_debug(signed: &SignedDownload) -> String {
    format!(
        "method={} bucket={} key={} expires_secs={:?}",
        signed.method, signed.bucket, signed.key, signed.expires_secs
    )
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

fn trailing_args(args: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" | "--key" => index += 2,
            "--print-url" => index += 1,
            _ => {
                output.push(args[index].clone());
                index += 1;
            }
        }
    }
    output
}
