use std::{env, fs, path::Path, time::Duration};

use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::llm::{ChatContentPart, ChatMessage, LlmInput},
};
use serde::Deserialize;
use storage::{BucketKind, DownloadObjectRequest, ObjectStorage, StorageConfig};
use utils::image::{normalize_image_mime, vision_image_data_url};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let (source_label, bytes) = match parse_s3_path(&args.path) {
        Some((bucket, key)) => {
            let config_path = args.config.as_deref().unwrap_or("config/services.toml");
            let settings = load_settings(config_path);
            let storage = ObjectStorage::new(settings.s3).unwrap_or_else(|err| {
                eprintln!("failed to initialize storage from '{config_path}': {err}");
                std::process::exit(1);
            });
            let bucket_kind = bucket_kind_from_name(&bucket);
            let bytes = storage
                .download_object(DownloadObjectRequest {
                    bucket: bucket_kind,
                    key: key.clone(),
                })
                .await
                .unwrap_or_else(|err| {
                    eprintln!("failed to download s3 object '{}/{}': {err}", bucket, key);
                    std::process::exit(1);
                });
            (format!("{bucket}/{key}"), bytes.to_vec())
        }
        None => {
            let bytes = fs::read(&args.path).unwrap_or_else(|err| {
                eprintln!("failed to read image file '{}': {err}", args.path);
                std::process::exit(1);
            });
            (args.path.clone(), bytes)
        }
    };
    let mime = args
        .mime
        .or_else(|| mime_from_path(&source_label))
        .unwrap_or_else(|| {
            eprintln!("cannot infer image mime from '{source_label}'; pass --mime");
            std::process::exit(1);
        });
    let Some(normalized_mime) = normalize_image_mime(&mime) else {
        eprintln!("unsupported image mime: {mime}");
        std::process::exit(1);
    };
    let data_url = vision_image_data_url(normalized_mime, &bytes).unwrap_or_else(|err| {
        eprintln!("failed to build vision data url: {err}");
        std::process::exit(1);
    });

    println!("input_path={source_label}");
    println!("input_mime={normalized_mime}");
    println!("input_bytes={}", bytes.len());
    println!("data_url_bytes={}", data_url.len());
    println!("data_url_prefix={}", preview(&data_url, 80));

    if args.ask_llm {
        let config_path = args.config.as_deref().unwrap_or("config/services.toml");
        let settings = load_settings(config_path);
        let llm = select_vision_llm(&settings);
        println!("llm_name={}", llm.name);
        println!("llm_base_url={}", llm.base_url);
        println!("llm_model={}", llm.model);
        let answer = tokio::time::timeout(
            Duration::from_secs(args.timeout_secs),
            ask_vision_llm(&llm, &data_url),
        )
        .await
        .unwrap_or_else(|_| {
            eprintln!(
                "vision llm request timed out after {} seconds",
                args.timeout_secs
            );
            std::process::exit(1);
        })
        .unwrap_or_else(|err| {
            eprintln!("vision llm request failed: {err}");
            std::process::exit(1);
        });
        println!("llm_answer={answer}");
    }

    if args.print {
        println!("{data_url}");
    }
}

struct Args {
    path: String,
    mime: Option<String>,
    config: Option<String>,
    ask_llm: bool,
    timeout_secs: u64,
    print: bool,
}

impl Args {
    fn parse() -> Self {
        let mut path = None;
        let mut mime = None;
        let mut config = None;
        let mut ask_llm = false;
        let mut timeout_secs = 30;
        let mut print = false;
        let mut iter = env::args().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--mime" => {
                    mime = iter.next();
                    if mime.is_none() {
                        eprintln!("--mime requires a value");
                        usage_and_exit(2);
                    }
                }
                "--config" => {
                    config = iter.next();
                    if config.is_none() {
                        eprintln!("--config requires a value");
                        usage_and_exit(2);
                    }
                }
                "--ask-llm" => ask_llm = true,
                "--timeout-secs" => {
                    let Some(value) = iter.next() else {
                        eprintln!("--timeout-secs requires a value");
                        usage_and_exit(2);
                    };
                    timeout_secs = value.parse::<u64>().unwrap_or_else(|err| {
                        eprintln!("invalid --timeout-secs value '{value}': {err}");
                        std::process::exit(2);
                    });
                }
                "--print" => print = true,
                "-h" | "--help" => usage_and_exit(0),
                value if value.starts_with('-') => {
                    eprintln!("unknown option: {value}");
                    usage_and_exit(2);
                }
                value => {
                    if path.replace(value.to_string()).is_some() {
                        eprintln!("only one image path is supported");
                        usage_and_exit(2);
                    }
                }
            }
        }

        let path = path.unwrap_or_else(|| {
            eprintln!("missing image path");
            usage_and_exit(2);
        });

        Self {
            path,
            mime,
            config,
            ask_llm,
            timeout_secs,
            print,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Settings {
    s3: StorageConfig,
    #[serde(default)]
    agent_llm_usage: AgentLlmUsageCfg,
    #[serde(default)]
    llm: Vec<AgentLlmCfg>,
}

#[derive(Debug, Default, Deserialize)]
struct AgentLlmUsageCfg {
    vision: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentLlmCfg {
    name: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    reasoning_effort: Option<String>,
    chat_completions_endpoint: Option<String>,
}

async fn ask_vision_llm(
    llm: &AgentLlmCfg,
    data_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ChatCompletionsLlm::new(&llm.base_url, &llm.model, llm.api_key.as_deref())?
        .with_max_tokens(llm.max_tokens.or(Some(512)))
        .with_temperature(llm.temperature.or(Some(0.0)))
        .with_reasoning_effort(llm.reasoning_effort.as_deref());
    if let Some(endpoint) = &llm.chat_completions_endpoint {
        client = client.with_chat_completions_endpoint(endpoint.clone());
    }
    let output = client
        .chat_once(LlmInput {
            messages: vec![ChatMessage::user_with_parts(vec![
                ChatContentPart::text("请简短回答：这张图片是什么？"),
                ChatContentPart::image_url_detail(data_url, "high"),
            ])],
        })
        .await?;
    Ok(output.get_content().to_string())
}

fn load_settings(path: &str) -> Settings {
    let raw = fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("failed to read config '{path}': {err}");
        std::process::exit(1);
    });
    toml::from_str(&raw).unwrap_or_else(|err| {
        eprintln!("failed to parse config '{path}': {err}");
        std::process::exit(1);
    })
}

fn select_vision_llm(settings: &Settings) -> AgentLlmCfg {
    let Some(name) = settings.agent_llm_usage.vision.as_deref() else {
        eprintln!("config missing [agent_llm_usage].vision");
        std::process::exit(1);
    };
    settings
        .llm
        .iter()
        .find(|llm| llm.name == name)
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("vision llm '{name}' not found in [[llm]]");
            std::process::exit(1);
        })
}

fn parse_s3_path(value: &str) -> Option<(String, String)> {
    let value = value.trim().trim_start_matches("s3://");
    let (bucket, key) = value.split_once('/')?;
    (!bucket.is_empty() && !key.is_empty()).then(|| (bucket.to_string(), key.to_string()))
}

fn bucket_kind_from_name(bucket: &str) -> BucketKind {
    match bucket {
        "public" => BucketKind::Public,
        "private" => BucketKind::Private,
        value => {
            eprintln!("unsupported bucket '{value}'; expected public or private");
            std::process::exit(1);
        }
    }
}

fn mime_from_path(path: &str) -> Option<String> {
    match Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => Some("image/png".to_string()),
        Some("jpg" | "jpeg") => Some("image/jpeg".to_string()),
        Some("webp") => Some("image/webp".to_string()),
        _ => None,
    }
}

fn preview(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn usage_and_exit(code: i32) -> ! {
    eprintln!(
        "usage: cargo run -p utils --bin vision_image_data_url -- <local-image-path|bucket/key> \
         [--config config/services.toml] [--mime image/jpeg] [--ask-llm] [--timeout-secs 30] \
         [--print]"
    );
    eprintln!(
        "note: S3 and LLM access are only in this dev test binary; utils library code stays pure."
    );
    std::process::exit(code);
}
