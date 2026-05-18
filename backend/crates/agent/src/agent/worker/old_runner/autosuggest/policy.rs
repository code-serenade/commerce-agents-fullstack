use std::{
    env as autosuggest_env, fs as autosuggest_fs,
    path::{Path as AutosuggestPath, PathBuf as AutosuggestPathBuf},
    sync::OnceLock as AutosuggestOnceLock,
};

const DEFAULT_POLICY_PATH: &str = "config/json/agent/worker-autosuggest-policy.json";
const POLICY_ENV: &str = "CM_AGENT_WORKER_AUTOSUGGEST_POLICY";
const EMBEDDED_POLICY: &str =
    include_str!("../../../../../../../config/json/agent/worker-autosuggest-policy.json");

#[derive(Debug, Clone, serde::Deserialize)]
struct WorkerAutosuggestPolicy {
    #[serde(default)]
    role_routes: Vec<RoleAutosuggestRoute>,
    #[serde(default)]
    chat: ChatAutosuggestPolicy,
    #[serde(default)]
    learning: LearningAutosuggestPolicy,
    #[serde(default)]
    search: SearchAutosuggestPolicy,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RoleAutosuggestRoute {
    role: String,
    handler: String,
    #[serde(default = "single_mode")]
    mode: String,
    #[serde(default)]
    learning_first: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ChatAutosuggestPolicy {
    #[serde(default = "save_memory_skill_id")]
    save_memory_skill_id: String,
    #[serde(default = "status_query_skill_id")]
    status_query_skill_id: String,
    #[serde(default)]
    save_memory_keywords: Vec<String>,
    #[serde(default)]
    status_query_keywords: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct LearningAutosuggestPolicy {
    #[serde(default = "learning_skill_id")]
    skill_id: String,
    #[serde(default)]
    explicit_keywords: Vec<String>,
    #[serde(default)]
    question_keywords: Vec<String>,
    #[serde(default)]
    explanation_keywords: Vec<String>,
    #[serde(default)]
    self_check_disabled_keywords: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SearchAutosuggestPolicy {
    #[serde(default = "competitor_skill_id")]
    competitor_skill_id: String,
    #[serde(default = "trends_skill_id")]
    trends_skill_id: String,
    #[serde(default = "market_info_skill_id")]
    market_info_skill_id: String,
    #[serde(default)]
    competitor_keywords: Vec<String>,
    #[serde(default)]
    trend_keywords: Vec<String>,
    #[serde(default)]
    market_info_keywords: Vec<String>,
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

static AUTOSUGGEST_POLICY: AutosuggestOnceLock<WorkerAutosuggestPolicy> =
    AutosuggestOnceLock::new();

fn configured_role_requests(role: &str, text: &str, task: &Task) -> Option<Vec<ParsedSkillRequest>> {
    let route = policy().role_routes.iter().find(|route| route.role == role)?;
    if route.learning_first
        && let Some(request) = autosuggest_learning(text, task)
    {
        return Some(vec![request]);
    }
    match route.mode.as_str() {
        "requests" => Some(autosuggest_requests_for_handler(
            route.handler.as_str(),
            text,
            task,
        )),
        _ => Some(
            autosuggest_for_handler(route.handler.as_str(), text, task)
                .into_iter()
                .collect(),
        ),
    }
}

fn chat_save_memory_skill_id() -> &'static str {
    policy().chat.save_memory_skill_id.as_str()
}

fn chat_status_query_skill_id() -> &'static str {
    policy().chat.status_query_skill_id.as_str()
}

fn has_chat_save_memory_intent(text: &str) -> bool {
    contains_any_config(text, &policy().chat.save_memory_keywords)
}

fn has_chat_status_query_intent(text: &str) -> bool {
    contains_any_config(text, &policy().chat.status_query_keywords)
}

fn learning_skill_id_from_policy() -> &'static str {
    policy().learning.skill_id.as_str()
}

fn has_learning_teaching_intent_from_policy(text: &str) -> bool {
    contains_any_config(text, &policy().learning.explicit_keywords)
        || (contains_any_config(text, &policy().learning.question_keywords)
            && contains_any_config(text, &policy().learning.explanation_keywords))
}

fn is_learning_self_check_disabled(text: &str) -> bool {
    contains_any_config(text, &policy().learning.self_check_disabled_keywords)
}

fn search_competitor_skill_id() -> &'static str {
    policy().search.competitor_skill_id.as_str()
}

fn search_trends_skill_id() -> &'static str {
    policy().search.trends_skill_id.as_str()
}

fn search_market_info_skill_id() -> &'static str {
    policy().search.market_info_skill_id.as_str()
}

fn has_search_competitor_intent(text: &str) -> bool {
    contains_any_config(text, &policy().search.competitor_keywords)
}

fn has_search_trend_intent(text: &str) -> bool {
    contains_any_config(text, &policy().search.trend_keywords)
}

fn has_search_market_info_intent(text: &str) -> bool {
    contains_any_config(text, &policy().search.market_info_keywords)
}

fn autosuggest_requests_for_handler(
    handler: &str,
    text: &str,
    task: &Task,
) -> Vec<ParsedSkillRequest> {
    match handler {
        "ops" => autosuggest_ops_requests(text, task),
        "service" => autosuggest_service_requests(text, task),
        "data" => autosuggest_data_requests(text, task),
        "creative" => autosuggest_creative_requests(text, task),
        "web" => autosuggest_web_requests(text, task),
        "accounting" => autosuggest_accounting_requests(text, task),
        "design" => autosuggest_design_requests(text, task),
        "douyin_ads" => autosuggest_douyin_ads_requests(text, task),
        "engineering" => autosuggest_engineering_requests(text, task),
        _ => autosuggest_for_handler(handler, text, task)
            .into_iter()
            .collect(),
    }
}

fn autosuggest_for_handler(
    handler: &str,
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    match handler {
        "chat" => autosuggest_chat(text, task),
        "ops" => autosuggest_ops(text, task),
        "data" => autosuggest_data(text, task),
        "accounting" => autosuggest_accounting(text, task),
        "service" => autosuggest_service(text, task),
        "creative" => autosuggest_creative(text, task),
        "web" => autosuggest_web(text, task),
        "design" => autosuggest_design(text, task),
        "douyin_ads" => autosuggest_douyin_ads(text, task),
        "engineering" => autosuggest_engineering(text, task),
        _ => None,
    }
}

fn contains_any_config(text: &str, needles: &[String]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}

fn policy() -> &'static WorkerAutosuggestPolicy {
    AUTOSUGGEST_POLICY.get_or_init(load_policy)
}

fn load_policy() -> WorkerAutosuggestPolicy {
    autosuggest_env::var(POLICY_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(AutosuggestPathBuf::from)
        .or_else(|| Some(workspace_root().join(DEFAULT_POLICY_PATH)))
        .and_then(|path| load_policy_from_path(&path).ok())
        .unwrap_or_else(|| parse_policy(EMBEDDED_POLICY).expect("embedded worker autosuggest policy"))
}

fn load_policy_from_path(path: &AutosuggestPath) -> Result<WorkerAutosuggestPolicy, String> {
    let content = autosuggest_fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    parse_policy(&content)
}

fn parse_policy(content: &str) -> Result<WorkerAutosuggestPolicy, String> {
    serde_json::from_str(content).map_err(|error| format!("invalid worker autosuggest policy: {error}"))
}

fn workspace_root() -> AutosuggestPathBuf {
    AutosuggestPathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(AutosuggestPath::parent)
        .map(AutosuggestPath::to_path_buf)
        .unwrap_or_else(|| AutosuggestPathBuf::from(env!("CARGO_MANIFEST_DIR")))
}
