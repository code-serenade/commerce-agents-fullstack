use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use tracing::warn;

const INFERENCE_ENV: &str = "CM_AGENT_WORKER_INFERENCE_POLICY";
const DEFAULT_INFERENCE_FILE: &str = "config/json/agent/worker-inference-policy.json";
const EMBEDDED_INFERENCE_POLICY: &str =
    include_str!("../../../../../../../config/json/agent/worker-inference-policy.json");

#[derive(Debug, Clone, Default, Deserialize)]
pub(in crate::agent::worker::runner) struct WorkerInferencePolicy {
    #[serde(default)]
    rules: HashMap<String, Vec<InferenceRule>>,
}

#[derive(Debug, Clone, Deserialize)]
struct InferenceRule {
    label: String,
    #[serde(default)]
    keywords: Vec<String>,
}

impl WorkerInferencePolicy {
    pub(super) fn match_label(&self, text: &str, rule_set: &str) -> Option<&str> {
        let lower = text.to_ascii_lowercase();
        self.rules.get(rule_set)?.iter().find_map(|rule| {
            rule.keywords
                .iter()
                .any(|keyword| {
                    text.contains(keyword) || lower.contains(&keyword.to_ascii_lowercase())
                })
                .then_some(rule.label.as_str())
        })
    }
}

pub(in crate::agent::worker::runner) fn load_inference_policy() -> WorkerInferencePolicy {
    if let Ok(path) = std::env::var(INFERENCE_ENV)
        && !path.trim().is_empty()
    {
        match parse_inference_file(Path::new(&path)) {
            Ok(policy) => return policy,
            Err(error) => warn!(
                path,
                error, "failed to load configured worker inference policy"
            ),
        }
    }

    let workspace_path = workspace_root().join(DEFAULT_INFERENCE_FILE);
    match parse_inference_file(&workspace_path) {
        Ok(policy) => policy,
        Err(error) => {
            warn!(
                path = %workspace_path.display(),
                error,
                "failed to load workspace inference policy; using embedded policy"
            );
            serde_json::from_str(EMBEDDED_INFERENCE_POLICY)
                .expect("embedded worker inference policy must be valid JSON")
        }
    }
}

fn parse_inference_file(path: &Path) -> Result<WorkerInferencePolicy, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("read: {error}"))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid worker inference policy: {error}"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}
