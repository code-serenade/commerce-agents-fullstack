use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use super::validation::validate_task_graph_rules;

const DEFAULT_TASK_GRAPH_RULES_PATH: &str = "config/json/agent/task-graph/index.json";
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
struct TaskGraphIndex {
    #[serde(default = "default_task_graph_schema_version")]
    schema_version: u32,
    #[serde(default)]
    defaults_file: Option<String>,
    #[serde(default)]
    role_aliases_file: Option<String>,
    #[serde(default)]
    role_defaults_file: Option<String>,
    #[serde(default)]
    files: Vec<String>,
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
    #[serde(default)]
    pub(super) role_order: Vec<String>,
    #[serde(default)]
    pub(super) explicit_team_keywords: Vec<String>,
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
        self.defaults.role_order.clone()
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
        return Some(normalize_task_graph_path(candidate));
    }

    None
}

fn normalize_task_graph_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("index.json")
    } else {
        path.to_path_buf()
    }
}

pub(super) fn load_task_graph_rules(path: impl AsRef<Path>) -> Result<TaskGraphRuleSet, String> {
    let path = normalize_task_graph_path(path.as_ref());
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "index.json")
    {
        return load_task_graph_directory(&path);
    }

    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    parse_task_graph_rules(&content)
}

fn load_task_graph_directory(index_path: &Path) -> Result<TaskGraphRuleSet, String> {
    let index_content = fs::read_to_string(index_path).map_err(|error| error.to_string())?;
    let index = serde_json::from_str::<TaskGraphIndex>(&index_content)
        .map_err(|error| format!("invalid task graph index json: {error}"))?;
    if index.schema_version > 2 {
        return Err(format!(
            "unsupported task graph index schema_version {}",
            index.schema_version
        ));
    }

    let directory = index_path
        .parent()
        .ok_or_else(|| "task graph index must have a parent directory".to_string())?;
    let mut rules = TaskGraphRuleSet {
        schema_version: index.schema_version,
        defaults: load_optional_fragment(directory, index.defaults_file.as_deref())?
            .unwrap_or_default(),
        role_aliases: load_optional_fragment(directory, index.role_aliases_file.as_deref())?
            .unwrap_or_default(),
        role_defaults: load_optional_fragment(directory, index.role_defaults_file.as_deref())?
            .unwrap_or_default(),
        rules: Vec::new(),
    };

    for file in index.files {
        let rule_path = directory.join(&file);
        let content = fs::read_to_string(&rule_path)
            .map_err(|error| format!("read {}: {error}", rule_path.display()))?;
        let rule = serde_json::from_str::<TaskGraphRule>(&content)
            .map_err(|error| format!("invalid task graph rule {}: {error}", rule_path.display()))?;
        rules.rules.push(rule);
    }

    validate_task_graph_rules(&rules)?;
    Ok(rules)
}

fn load_optional_fragment<T>(directory: &Path, file: Option<&str>) -> Result<Option<T>, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let Some(file) = file else {
        return Ok(None);
    };
    let path = directory.join(file);
    let content =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let fragment = serde_json::from_str::<T>(&content)
        .map_err(|error| format!("invalid task graph fragment {}: {error}", path.display()))?;
    Ok(Some(fragment))
}

fn parse_task_graph_rules(content: &str) -> Result<TaskGraphRuleSet, String> {
    let rules = serde_json::from_str::<TaskGraphRuleSet>(content)
        .map_err(|error| format!("invalid task graph rules json: {error}"))?;
    validate_task_graph_rules(&rules)?;
    Ok(rules)
}
