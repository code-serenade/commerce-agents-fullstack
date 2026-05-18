use std::{fs, path::Path, sync::OnceLock};

use serde::Deserialize;
use tracing::warn;

const AUTOSUGGEST_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-autosuggest-policy.json");
const INFERENCE_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-inference-policy.json");
const OUTPUT_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-output-policy.json");
const OUTPUT_NORMALIZATION_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-output-normalization-policy.json");

#[derive(Debug)]
pub(super) struct WorkerPolicySet {
    pub(super) autosuggest: WorkerAutosuggestPolicy,
    #[allow(dead_code)]
    pub(super) inference: serde_json::Value,
    #[allow(dead_code)]
    pub(super) output: serde_json::Value,
    #[allow(dead_code)]
    pub(super) output_normalization: serde_json::Value,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct WorkerAutosuggestPolicy {
    #[serde(default)]
    pub(super) role_routes: Vec<RoleAutosuggestRoute>,
    #[serde(default)]
    pub(super) chat: ChatAutosuggestPolicy,
    #[serde(default)]
    pub(super) learning: LearningAutosuggestPolicy,
    #[serde(default)]
    pub(super) search: SearchAutosuggestPolicy,
    #[serde(default)]
    pub(super) rules: Vec<AutosuggestRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RoleAutosuggestRoute {
    pub(super) role: String,
    pub(super) handler: String,
    #[serde(default = "single_mode")]
    pub(super) mode: String,
    #[serde(default)]
    pub(super) learning_first: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct ChatAutosuggestPolicy {
    #[serde(default = "save_memory_skill_id")]
    pub(super) save_memory_skill_id: String,
    #[serde(default = "status_query_skill_id")]
    pub(super) status_query_skill_id: String,
    #[serde(default)]
    pub(super) save_memory_keywords: Vec<String>,
    #[serde(default)]
    pub(super) status_query_keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LearningAutosuggestPolicy {
    #[serde(default = "learning_skill_id")]
    pub(super) skill_id: String,
    #[serde(default)]
    pub(super) explicit_keywords: Vec<String>,
    #[serde(default)]
    pub(super) question_keywords: Vec<String>,
    #[serde(default)]
    pub(super) explanation_keywords: Vec<String>,
    #[serde(default)]
    pub(super) self_check_disabled_keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SearchAutosuggestPolicy {
    #[serde(default = "competitor_skill_id")]
    pub(super) competitor_skill_id: String,
    #[serde(default = "trends_skill_id")]
    pub(super) trends_skill_id: String,
    #[serde(default = "market_info_skill_id")]
    pub(super) market_info_skill_id: String,
    #[serde(default)]
    pub(super) competitor_keywords: Vec<String>,
    #[serde(default)]
    pub(super) trend_keywords: Vec<String>,
    #[serde(default)]
    pub(super) market_info_keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct AutosuggestRule {
    pub(super) id: String,
    #[serde(default)]
    pub(super) roles: Vec<String>,
    #[serde(default)]
    pub(super) mode: AutosuggestRuleMode,
    #[serde(default)]
    pub(super) r#match: AutosuggestRuleMatch,
    pub(super) skill_id: String,
    #[serde(default)]
    pub(super) input: serde_json::Value,
    #[serde(default)]
    pub(super) reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum AutosuggestRuleMode {
    #[default]
    Append,
    First,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct AutosuggestRuleMatch {
    #[serde(default)]
    pub(super) any_keywords: Vec<String>,
    #[serde(default)]
    pub(super) all_keywords: Vec<String>,
    #[serde(default)]
    pub(super) none_keywords: Vec<String>,
}

impl Default for ChatAutosuggestPolicy {
    fn default() -> Self {
        Self {
            save_memory_skill_id: save_memory_skill_id(),
            status_query_skill_id: status_query_skill_id(),
            save_memory_keywords: Vec::new(),
            status_query_keywords: Vec::new(),
        }
    }
}

impl Default for LearningAutosuggestPolicy {
    fn default() -> Self {
        Self {
            skill_id: learning_skill_id(),
            explicit_keywords: Vec::new(),
            question_keywords: Vec::new(),
            explanation_keywords: Vec::new(),
            self_check_disabled_keywords: Vec::new(),
        }
    }
}

impl Default for SearchAutosuggestPolicy {
    fn default() -> Self {
        Self {
            competitor_skill_id: competitor_skill_id(),
            trends_skill_id: trends_skill_id(),
            market_info_skill_id: market_info_skill_id(),
            competitor_keywords: Vec::new(),
            trend_keywords: Vec::new(),
            market_info_keywords: Vec::new(),
        }
    }
}

static POLICIES: OnceLock<WorkerPolicySet> = OnceLock::new();

pub(super) fn worker_policies() -> &'static WorkerPolicySet {
    POLICIES.get_or_init(load_worker_policies)
}

fn load_worker_policies() -> WorkerPolicySet {
    WorkerPolicySet {
        autosuggest: load_policy(
            "worker autosuggest policy",
            "config/json/agent/worker-autosuggest-policy.json",
            "CM_AGENT_WORKER_AUTOSUGGEST_POLICY",
            AUTOSUGGEST_POLICY,
        ),
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

fn single_mode() -> String {
    "single".to_string()
}

fn save_memory_skill_id() -> String {
    "coordination_save_memory".to_string()
}

fn status_query_skill_id() -> String {
    "coordination_status_query".to_string()
}

fn learning_skill_id() -> String {
    "learning_teaching_explain".to_string()
}

fn competitor_skill_id() -> String {
    "search_competitor".to_string()
}

fn trends_skill_id() -> String {
    "search_trends".to_string()
}

fn market_info_skill_id() -> String {
    "search_market_info".to_string()
}
