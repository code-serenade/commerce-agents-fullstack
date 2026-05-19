use std::{env, fs, path::Path};

use super::{DEFAULT_NODE_ACCEPTANCE, util::contains_string_any};
use crate::core::protocol::{TaskGraph, TaskGraphId, TaskId, TaskNode, TaskNodeId, WorkerId};

const DEFAULT_CHAT_POLICY_PATH: &str = "config/json/agent/task-graph/chat-policy.json";
const CHAT_POLICY_ENV: &str = "CM_AGENT_TASK_GRAPH_CHAT_POLICY";

#[derive(Debug, Clone, serde::Deserialize)]
struct ChatPolicy {
    #[serde(default)]
    direct_answer: DirectAnswerPolicy,
    #[serde(default)]
    fallback_node: ChatNodePolicy,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct DirectAnswerPolicy {
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    short_question_keywords: Vec<String>,
    #[serde(default)]
    arithmetic_keywords: Vec<String>,
    #[serde(default)]
    arithmetic_exclude_keywords: Vec<String>,
    #[serde(default = "default_arithmetic_max_chars")]
    arithmetic_max_chars: usize,
    #[serde(default = "default_short_question_max_chars")]
    short_question_max_chars: usize,
    #[serde(default)]
    node: ChatNodePolicy,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ChatNodePolicy {
    #[serde(default = "default_chat_node_id")]
    id: String,
    #[serde(default = "default_chat_role")]
    role: String,
    #[serde(default = "default_chat_title")]
    title: String,
    #[serde(default = "default_chat_objective")]
    objective: String,
    #[serde(default = "default_chat_max_attempts")]
    max_attempts: u8,
    #[serde(default)]
    acceptance: Vec<String>,
}

impl Default for ChatNodePolicy {
    fn default() -> Self {
        Self {
            id: default_chat_node_id(),
            role: default_chat_role(),
            title: default_chat_title(),
            objective: default_chat_objective(),
            max_attempts: default_chat_max_attempts(),
            acceptance: Vec::new(),
        }
    }
}

pub(super) fn is_direct_answer_task(task: &str) -> bool {
    let trimmed = task.trim();
    if trimmed.is_empty() {
        return false;
    }

    let policy = load_chat_policy();
    if contains_string_any(trimmed, &policy.direct_answer.keywords) {
        return true;
    }

    let has_digit = trimmed.chars().any(|ch| ch.is_ascii_digit());
    let char_count = trimmed.chars().count();
    let arithmetic_request = has_digit
        && contains_string_any(trimmed, &policy.direct_answer.arithmetic_keywords)
        && char_count <= policy.direct_answer.arithmetic_max_chars
        && !contains_string_any(trimmed, &policy.direct_answer.arithmetic_exclude_keywords);
    let short_question =
        contains_string_any(trimmed, &policy.direct_answer.short_question_keywords)
            && char_count <= policy.direct_answer.short_question_max_chars;

    arithmetic_request || short_question
}

pub(super) fn plan_direct_answer_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    let policy = load_chat_policy();
    chat_graph(task_id, task, &policy.direct_answer.node)
}

pub(super) fn plan_fallback_chat_graph(task_id: &TaskId, task: &str) -> TaskGraph {
    let policy = load_chat_policy();
    chat_graph(task_id, task, &policy.fallback_node)
}

fn chat_graph(task_id: &TaskId, task: &str, policy: &ChatNodePolicy) -> TaskGraph {
    TaskGraph {
        graph_id: TaskGraphId(format!("graph-{}", task_id.0)),
        root_task: task.to_string(),
        nodes: vec![chat_node(policy, task)],
    }
}

fn chat_node(policy: &ChatNodePolicy, task: &str) -> TaskNode {
    TaskNode {
        id: TaskNodeId(policy.id.clone()),
        role: policy.role.clone(),
        worker_id: WorkerId(format!("worker.{}", policy.role)),
        title: render_task_template(&policy.title, task),
        objective: render_task_template(&policy.objective, task),
        input_refs: Vec::new(),
        acceptance: if policy.acceptance.is_empty() {
            vec![DEFAULT_NODE_ACCEPTANCE.to_string()]
        } else {
            policy.acceptance.clone()
        },
        max_attempts: policy.max_attempts.max(1),
    }
}

fn load_chat_policy() -> ChatPolicy {
    configured_chat_policy_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str::<ChatPolicy>(&content).ok())
        .unwrap_or_default()
}

fn configured_chat_policy_path() -> Option<std::path::PathBuf> {
    let path = env::var(CHAT_POLICY_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_CHAT_POLICY_PATH.to_string());
    let candidate = Path::new(&path);
    if candidate.exists() {
        return Some(candidate.to_path_buf());
    }
    if path != DEFAULT_CHAT_POLICY_PATH {
        return None;
    }
    let repo_config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(DEFAULT_CHAT_POLICY_PATH);
    repo_config_path.exists().then_some(repo_config_path)
}

fn render_task_template(template: &str, task: &str) -> String {
    template.replace("{{task}}", task)
}

impl Default for ChatPolicy {
    fn default() -> Self {
        Self {
            direct_answer: DirectAnswerPolicy {
                keywords: vec![
                    "只输出".to_string(),
                    "只给".to_string(),
                    "只回答".to_string(),
                    "只回复".to_string(),
                    "不要解释".to_string(),
                    "不用解释".to_string(),
                    "无需解释".to_string(),
                    "直接回答".to_string(),
                    "直接给".to_string(),
                    "简短回答".to_string(),
                    "简要回答".to_string(),
                    "一句话".to_string(),
                    "only output".to_string(),
                    "answer only".to_string(),
                    "no explanation".to_string(),
                ],
                short_question_keywords: vec![
                    "是多少".to_string(),
                    "多少".to_string(),
                    "几个".to_string(),
                    "几".to_string(),
                ],
                arithmetic_keywords: vec![
                    "+".to_string(),
                    "-".to_string(),
                    "*".to_string(),
                    "/".to_string(),
                    "×".to_string(),
                    "÷".to_string(),
                    "加".to_string(),
                    "减".to_string(),
                    "乘".to_string(),
                    "除".to_string(),
                    "计算".to_string(),
                    "等于".to_string(),
                    "结果".to_string(),
                ],
                arithmetic_exclude_keywords: vec![
                    "分析".to_string(),
                    "方案".to_string(),
                    "策略".to_string(),
                    "运营".to_string(),
                    "电商".to_string(),
                    "漏斗".to_string(),
                    "转化".to_string(),
                    "点击".to_string(),
                    "曝光".to_string(),
                    "加购".to_string(),
                    "下单".to_string(),
                    "付款".to_string(),
                ],
                arithmetic_max_chars: default_arithmetic_max_chars(),
                short_question_max_chars: default_short_question_max_chars(),
                node: ChatNodePolicy {
                    title: "直接回答".to_string(),
                    objective: "直接回答用户原始问题，必须遵守用户对简短、只输出、不要解释的要求，\
                                不启动多智能体：{{task}}"
                        .to_string(),
                    max_attempts: 1,
                    acceptance: vec!["必须直接回答，不展开多岗位过程。".to_string()],
                    ..ChatNodePolicy::default()
                },
            },
            fallback_node: ChatNodePolicy {
                title: "通用对话".to_string(),
                objective: "以对话助手身份回答用户问题；如果用户意图不清，先澄清，不启动多智能体：\
                            {{task}}"
                    .to_string(),
                max_attempts: 1,
                ..ChatNodePolicy::default()
            },
        }
    }
}

fn default_arithmetic_max_chars() -> usize {
    40
}

fn default_short_question_max_chars() -> usize {
    40
}

fn default_chat_node_id() -> String {
    "chat".to_string()
}

fn default_chat_role() -> String {
    "chat".to_string()
}

fn default_chat_title() -> String {
    "通用对话".to_string()
}

fn default_chat_objective() -> String {
    "以对话助手身份回答用户问题；如果用户意图不清，先澄清，不启动多智能体：{{task}}".to_string()
}

fn default_chat_max_attempts() -> u8 {
    1
}
