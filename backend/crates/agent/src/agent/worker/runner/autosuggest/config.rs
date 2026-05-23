use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use tracing::warn;

const AUTOSUGGEST_ENV: &str = "CM_AGENT_WORKER_AUTOSUGGEST_POLICY";
const DEFAULT_AUTOSUGGEST_DIR: &str = "config/json/agent/worker-autosuggest";
const LEGACY_AUTOSUGGEST_FILE: &str = "config/json/agent/worker-autosuggest-policy.json";
const EMBEDDED_LEGACY_AUTOSUGGEST: &str =
    include_str!("../../../../../../../config/json/agent/worker-autosuggest-policy.json");

#[derive(Debug, Clone, Default, Deserialize)]
pub(in crate::agent::worker::runner) struct WorkerAutosuggestPolicy {
    #[serde(default)]
    pub(super) role_routes: Vec<RoleAutosuggestRoute>,
    #[serde(default)]
    pub(super) chat: ChatAutosuggestPolicy,
    #[serde(default)]
    pub(super) learning: LearningAutosuggestPolicy,
    #[serde(default)]
    #[allow(dead_code)]
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
    pub(super) save_memory_input: serde_json::Value,
    #[serde(default)]
    pub(super) status_query_input: serde_json::Value,
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
    pub(super) input: serde_json::Value,
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
    pub(super) fallback: bool,
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
    #[serde(default)]
    pub(super) requires_any_skill_ids: Vec<String>,
    #[serde(default)]
    pub(super) requires_no_skill_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AutosuggestIndex {
    #[serde(default)]
    role_routes: Vec<RoleAutosuggestRoute>,
    #[serde(default)]
    files: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AutosuggestFragment {
    #[serde(default)]
    chat: Option<ChatAutosuggestPolicy>,
    #[serde(default)]
    learning: Option<LearningAutosuggestPolicy>,
    #[serde(default)]
    search: Option<SearchAutosuggestPolicy>,
    #[serde(default)]
    rules: Vec<AutosuggestRule>,
}

impl Default for ChatAutosuggestPolicy {
    fn default() -> Self {
        Self {
            save_memory_skill_id: save_memory_skill_id(),
            status_query_skill_id: status_query_skill_id(),
            save_memory_input: serde_json::Value::Object(serde_json::Map::new()),
            status_query_input: serde_json::Value::Object(serde_json::Map::new()),
            save_memory_keywords: Vec::new(),
            status_query_keywords: Vec::new(),
        }
    }
}

impl Default for LearningAutosuggestPolicy {
    fn default() -> Self {
        Self {
            skill_id: learning_skill_id(),
            input: serde_json::Value::Object(serde_json::Map::new()),
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

pub(in crate::agent::worker::runner) fn load_autosuggest_policy() -> WorkerAutosuggestPolicy {
    if let Ok(path) = std::env::var(AUTOSUGGEST_ENV)
        && !path.trim().is_empty()
    {
        match load_autosuggest_path(&resolve_policy_path(&path)) {
            Ok(policy) => return policy,
            Err(error) => warn!(
                path,
                error, "failed to load configured worker autosuggest policy"
            ),
        }
    }

    let directory_path = resolve_policy_path(DEFAULT_AUTOSUGGEST_DIR);
    match load_autosuggest_directory(&directory_path) {
        Ok(policy) => return policy,
        Err(error) => warn!(
            path = %directory_path.display(),
            error,
            "failed to load worker autosuggest directory; trying legacy file"
        ),
    }

    let legacy_path = resolve_policy_path(LEGACY_AUTOSUGGEST_FILE);
    match load_legacy_file(&legacy_path) {
        Ok(policy) => policy,
        Err(error) => {
            warn!(
                path = %legacy_path.display(),
                error,
                "failed to load legacy worker autosuggest policy; using embedded policy"
            );
            serde_json::from_str(EMBEDDED_LEGACY_AUTOSUGGEST)
                .expect("embedded worker autosuggest policy must be valid")
        }
    }
}

fn resolve_policy_path(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.exists() || candidate.is_absolute() {
        return candidate.to_path_buf();
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(candidate)
}

fn load_autosuggest_path(path: &Path) -> Result<WorkerAutosuggestPolicy, String> {
    if path.is_dir() {
        load_autosuggest_directory(path)
    } else {
        load_legacy_file(path)
    }
}

fn load_autosuggest_directory(directory: &Path) -> Result<WorkerAutosuggestPolicy, String> {
    let index = parse_json_file::<AutosuggestIndex>(
        "worker autosuggest index",
        &directory.join("index.json"),
    )?;
    let mut policy = WorkerAutosuggestPolicy {
        role_routes: index.role_routes,
        ..WorkerAutosuggestPolicy::default()
    };

    for file in index.files {
        let fragment_path = directory.join(&file);
        let fragment =
            parse_json_file::<AutosuggestFragment>("worker autosuggest fragment", &fragment_path)?;
        merge_fragment(&mut policy, fragment);
    }

    Ok(policy)
}

fn merge_fragment(policy: &mut WorkerAutosuggestPolicy, fragment: AutosuggestFragment) {
    if let Some(chat) = fragment.chat {
        policy.chat = chat;
    }
    if let Some(learning) = fragment.learning {
        policy.learning = learning;
    }
    if let Some(search) = fragment.search {
        policy.search = search;
    }
    policy.rules.extend(fragment.rules);
}

fn load_legacy_file(path: &Path) -> Result<WorkerAutosuggestPolicy, String> {
    parse_json_file("worker autosuggest policy", path)
}

fn parse_json_file<T>(name: &str, path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let content = fs::read_to_string(path).map_err(|error| format!("read: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("invalid {name}: {error}"))
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
