use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use super::validation::validate_task_graph_rules;

const DEFAULT_TASK_GRAPH_RULES_PATH: &str = "config/json/agent/task-graph-rules.json";
const TASK_GRAPH_RULES_ENV: &str = "CM_AGENT_TASK_GRAPH_RULES";

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct TaskGraphRuleSet {
    #[serde(default = "default_task_graph_schema_version")]
    pub(super) schema_version: u32,
    #[serde(default)]
    pub(super) defaults: TaskGraphDefaults,
    #[serde(default)]
    pub(super) role_aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) role_defaults: HashMap<String, TaskGraphRoleDefault>,
    #[serde(default)]
    pub(super) rules: Vec<TaskGraphRule>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct TaskGraphRule {
    pub(super) id: String,
    #[serde(default)]
    pub(super) description: String,
    #[serde(default)]
    pub(super) priority: i32,
    #[serde(default)]
    pub(super) match_any: Vec<String>,
    #[serde(default)]
    pub(super) match_all: Vec<String>,
    #[serde(default)]
    pub(super) match_all_any: Vec<Vec<String>>,
    #[serde(default)]
    pub(super) r#match: TaskGraphRuleMatch,
    #[serde(default)]
    pub(super) role_order: Vec<String>,
    #[serde(default)]
    pub(super) nodes: Vec<TaskGraphRuleNode>,
    #[serde(default)]
    pub(super) conditional_nodes: Vec<TaskGraphConditionalNode>,
    #[serde(default)]
    pub(super) dynamic_roles: TaskGraphDynamicRoles,
    #[serde(default)]
    pub(super) final_node: Option<TaskGraphFinalNode>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct TaskGraphRuleNode {
    pub(super) id: String,
    pub(super) role: String,
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) objective: String,
    #[serde(default)]
    pub(super) depends_on: Vec<String>,
    #[serde(default)]
    pub(super) max_attempts: Option<u8>,
    #[serde(default)]
    pub(super) acceptance: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct TaskGraphDefaults {
    #[serde(default)]
    pub(super) max_attempts: Option<u8>,
    #[serde(default)]
    pub(super) node_acceptance: Vec<String>,
    #[serde(default)]
    pub(super) final_role: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct TaskGraphRoleDefault {
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) objective: String,
    #[serde(default)]
    pub(super) acceptance: Vec<String>,
    #[serde(default)]
    pub(super) max_attempts: Option<u8>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct TaskGraphRuleMatch {
    #[serde(default)]
    pub(super) any_keywords: Vec<String>,
    #[serde(default)]
    pub(super) all_keywords: Vec<String>,
    #[serde(default)]
    pub(super) all_keyword_groups: Vec<Vec<String>>,
    #[serde(default)]
    pub(super) none_keywords: Vec<String>,
    #[serde(default)]
    pub(super) route_primary_any: Vec<String>,
    #[serde(default)]
    pub(super) route_support_any: Vec<String>,
    #[serde(default)]
    pub(super) route_score_min: Option<f32>,
    #[serde(default)]
    pub(super) route_min_non_chat_roles: Option<usize>,
    #[serde(default)]
    pub(super) allow_without_route: bool,
    #[serde(default)]
    pub(super) explicit_team_request: bool,
    #[serde(default)]
    pub(super) max_chars: Option<usize>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct TaskGraphConditionalNode {
    pub(super) id: String,
    pub(super) role: String,
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) objective: String,
    #[serde(default)]
    pub(super) when: TaskGraphRuleMatch,
    #[serde(default)]
    pub(super) depends_on: Vec<String>,
    #[serde(default)]
    pub(super) max_attempts: Option<u8>,
    #[serde(default)]
    pub(super) acceptance: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct TaskGraphDynamicRoles {
    #[serde(default)]
    pub(super) from_explicit_request: ExplicitRequestRoles,
    #[serde(default)]
    pub(super) from_route: RouteRoles,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct ExplicitRequestRoles {
    #[serde(default)]
    pub(super) enabled: bool,
    #[serde(default)]
    pub(super) trigger_keywords: Vec<String>,
    #[serde(default)]
    pub(super) include_final_role_when_keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct RouteRoles {
    #[serde(default)]
    pub(super) enabled: bool,
    #[serde(default)]
    pub(super) include_primary: bool,
    #[serde(default)]
    pub(super) include_support: RouteSupportMode,
    #[serde(default)]
    pub(super) exclude_roles: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum RouteSupportMode {
    #[default]
    Never,
    Always,
    WhenRequestedOrPrimaryData,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct TaskGraphFinalNode {
    #[serde(default)]
    pub(super) enabled: bool,
    pub(super) id: String,
    pub(super) role: String,
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) objective: String,
    #[serde(default)]
    pub(super) depends_on: FinalNodeDependsOn,
    #[serde(default)]
    pub(super) max_attempts: Option<u8>,
    #[serde(default)]
    pub(super) acceptance: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(untagged)]
pub(super) enum FinalNodeDependsOn {
    Special(String),
    List(Vec<String>),
    #[default]
    Empty,
}

impl TaskGraphRuleSet {
    pub(super) fn default_role_order(&self) -> Vec<String> {
        vec![
            "data".to_string(),
            "creative".to_string(),
            "design".to_string(),
            "service".to_string(),
            "engineering".to_string(),
            "douyin_ads".to_string(),
            "accounting".to_string(),
            "web".to_string(),
            "ops".to_string(),
        ]
    }
}

fn default_task_graph_schema_version() -> u32 {
    1
}

pub(super) fn configured_task_graph_rules_path() -> Option<PathBuf> {
    let path = env::var(TASK_GRAPH_RULES_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_TASK_GRAPH_RULES_PATH.to_string());
    resolve_task_graph_rules_path(&path)
}

fn resolve_task_graph_rules_path(path: &str) -> Option<PathBuf> {
    let candidate = Path::new(path);
    if candidate.exists() {
        return Some(candidate.to_path_buf());
    }
    if path != DEFAULT_TASK_GRAPH_RULES_PATH {
        return None;
    }

    let repo_config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(DEFAULT_TASK_GRAPH_RULES_PATH);
    repo_config_path.exists().then_some(repo_config_path)
}

pub(super) fn load_task_graph_rules(path: impl AsRef<Path>) -> Result<TaskGraphRuleSet, String> {
    let content = fs::read_to_string(path.as_ref()).map_err(|error| error.to_string())?;
    parse_task_graph_rules(&content)
}

fn parse_task_graph_rules(content: &str) -> Result<TaskGraphRuleSet, String> {
    let rules = serde_json::from_str::<TaskGraphRuleSet>(content)
        .map_err(|error| format!("invalid task graph rules json: {error}"))?;
    validate_task_graph_rules(&rules)?;
    Ok(rules)
}
