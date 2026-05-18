use std::{fs, path::Path, sync::OnceLock};

use serde::Deserialize;
use tracing::warn;

const INFERENCE_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-inference-policy.json");
const OUTPUT_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-output-policy.json");
const OUTPUT_NORMALIZATION_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-output-normalization-policy.json");

#[derive(Debug)]
pub(super) struct WorkerPolicySet {
    pub(super) autosuggest: super::autosuggest::config::WorkerAutosuggestPolicy,
    #[allow(dead_code)]
    pub(super) inference: serde_json::Value,
    #[allow(dead_code)]
    pub(super) output: serde_json::Value,
    #[allow(dead_code)]
    pub(super) output_normalization: serde_json::Value,
}

static POLICIES: OnceLock<WorkerPolicySet> = OnceLock::new();

pub(super) fn worker_policies() -> &'static WorkerPolicySet {
    POLICIES.get_or_init(load_worker_policies)
}

fn load_worker_policies() -> WorkerPolicySet {
    WorkerPolicySet {
        autosuggest: super::autosuggest::load_autosuggest_policy(),
        inference: load_policy(
            "worker inference policy",
            "config/json/agent/worker-inference-policy.json",
            "CM_AGENT_WORKER_INFERENCE_POLICY",
            INFERENCE_POLICY,
        ),
        output: load_policy(
            "worker output policy",
            "config/json/agent/worker-output-policy.json",
            "CM_AGENT_WORKER_OUTPUT_POLICY",
            OUTPUT_POLICY,
        ),
        output_normalization: load_policy(
            "worker output normalization policy",
            "config/json/agent/worker-output-normalization-policy.json",
            "CM_AGENT_WORKER_OUTPUT_NORMALIZATION_POLICY",
            OUTPUT_NORMALIZATION_POLICY,
        ),
    }
}

fn load_policy<T>(name: &str, relative_path: &str, env_key: &str, embedded: &str) -> T
where
    T: for<'de> Deserialize<'de>,
{
    if let Ok(path) = std::env::var(env_key)
        && !path.trim().is_empty()
    {
        match parse_policy_file(name, Path::new(&path)) {
            Ok(policy) => return policy,
            Err(error) => warn!(
                policy = name,
                path, error, "failed to load configured policy"
            ),
        }
    }

    let workspace_path = workspace_root().join(relative_path);
    match parse_policy_file(name, &workspace_path) {
        Ok(policy) => policy,
        Err(error) => {
            warn!(
                policy = name,
                path = %workspace_path.display(),
                error,
                "failed to load workspace policy; using embedded policy"
            );
            serde_json::from_str(embedded).unwrap_or_else(|error| {
                panic!("embedded {name} is invalid: {error}");
            })
        }
    }
}

fn parse_policy_file<T>(name: &str, path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let content = fs::read_to_string(path).map_err(|error| format!("read: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("invalid {name}: {error}"))
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}
