use std::{collections::HashMap, env, fs, path::Path};

const DEFAULT_EVALUATION_POLICY_PATH: &str = "config/json/agent/task-graph/evaluation-policy.json";
const EVALUATION_POLICY_ENV: &str = "CM_AGENT_TASK_GRAPH_EVALUATION_POLICY";

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct EvaluationPolicy {
    #[serde(default = "default_min_summary_chars")]
    pub(super) min_summary_chars: usize,
    #[serde(default)]
    pub(super) placeholder_keywords: Vec<String>,
    #[serde(default)]
    pub(super) unsupported_evidence_keywords: Vec<String>,
    #[serde(default)]
    pub(super) refusal_keywords: Vec<String>,
    #[serde(default)]
    pub(super) blocking_open_question_keywords: Vec<String>,
    #[serde(default)]
    pub(super) downgradable_prefixes: Vec<String>,
    #[serde(default)]
    pub(super) boundary_gap_reasons: Vec<String>,
    #[serde(default)]
    pub(super) reason_localizations: HashMap<String, String>,
    #[serde(default)]
    pub(super) prefix_localizations: Vec<PrefixLocalization>,
    #[serde(default)]
    pub(super) roles: HashMap<String, RoleEvaluationPolicy>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct RoleEvaluationPolicy {
    #[serde(default)]
    pub(super) min_findings: Option<usize>,
    #[serde(default)]
    pub(super) min_recommendations: Option<usize>,
    #[serde(default)]
    pub(super) require_risks: bool,
    #[serde(default)]
    pub(super) action_oriented: bool,
    #[serde(default)]
    pub(super) risk_sensitive: bool,
    #[serde(default)]
    pub(super) specific_content_keywords: Vec<String>,
    #[serde(default)]
    pub(super) specific_content_source: SpecificContentSource,
    #[serde(default)]
    pub(super) reasons: RoleEvaluationReasons,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SpecificContentSource {
    Findings,
    #[default]
    Recommendations,
    All,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct RoleEvaluationReasons {
    #[serde(default)]
    pub(super) min_findings: String,
    #[serde(default)]
    pub(super) min_recommendations: String,
    #[serde(default)]
    pub(super) require_risks: String,
    #[serde(default)]
    pub(super) specific_content: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct PrefixLocalization {
    pub(super) prefix: String,
    pub(super) message: String,
}

pub(super) fn load_evaluation_policy() -> EvaluationPolicy {
    configured_policy_path(DEFAULT_EVALUATION_POLICY_PATH, EVALUATION_POLICY_ENV)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str::<EvaluationPolicy>(&content).ok())
        .unwrap_or_default()
}

fn configured_policy_path(default_path: &str, env_key: &str) -> Option<std::path::PathBuf> {
    let path = env::var(env_key)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| default_path.to_string());
    let candidate = Path::new(&path);
    if candidate.exists() {
        return Some(candidate.to_path_buf());
    }
    if path != default_path {
        return None;
    }
    let repo_config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(default_path);
    repo_config_path.exists().then_some(repo_config_path)
}

impl Default for EvaluationPolicy {
    fn default() -> Self {
        Self {
            min_summary_chars: default_min_summary_chars(),
            placeholder_keywords: vec![
                "todo".to_string(),
                "tbd".to_string(),
                "placeholder".to_string(),
                "无法处理".to_string(),
                "不知道".to_string(),
                "作为ai".to_string(),
                "as an ai".to_string(),
            ],
            unsupported_evidence_keywords: vec![
                "已查询".to_string(),
                "已检索".to_string(),
                "查询到".to_string(),
                "搜索结果显示".to_string(),
                "数据库显示".to_string(),
                "调用工具".to_string(),
                "tool result".to_string(),
            ],
            refusal_keywords: vec![
                "不能回答".to_string(),
                "无法回答".to_string(),
                "无法完成任务".to_string(),
                "无法继续".to_string(),
                "无法推进".to_string(),
                "我不能".to_string(),
                "i cannot".to_string(),
                "can't help".to_string(),
            ],
            blocking_open_question_keywords: vec![
                "无法继续".to_string(),
                "无法判断".to_string(),
                "无法完成任务".to_string(),
                "阻止".to_string(),
                "必须提供".to_string(),
                "必须补充".to_string(),
            ],
            downgradable_prefixes: vec![
                "quality completeness:".to_string(),
                "quality actionability:".to_string(),
                "quality risk awareness:".to_string(),
            ],
            boundary_gap_reasons: vec![
                "worker reported failed status".to_string(),
                "open questions remain".to_string(),
                "role output has open questions".to_string(),
                "content looks like placeholder or failure text".to_string(),
                "structured role output is missing".to_string(),
                "content is empty".to_string(),
                "quality refusal detection: refusal without explicit open question".to_string(),
                "role output claims external tool execution".to_string(),
            ],
            reason_localizations: default_reason_localizations(),
            prefix_localizations: default_prefix_localizations(),
            roles: default_role_evaluation_policies(),
        }
    }
}

fn default_min_summary_chars() -> usize {
    8
}

fn default_reason_localizations() -> HashMap<String, String> {
    [
        (
            "data role should provide at least two findings",
            "数据岗位需要补齐至少两个可复核发现。",
        ),
        (
            "data role should mark data or interpretation risks",
            "数据岗位需要说明数据口径或解释风险。",
        ),
        (
            "service role output lacks service-specific content",
            "服务岗位需要补充客服承接、售后边界或升级 SOP。",
        ),
        (
            "service role should provide scripts, SOP, or escalation steps",
            "服务岗位需要给出话术、SOP 或升级步骤。",
        ),
        (
            "engineering role output lacks technical content",
            "工程岗位需要补充埋点、接口、稳定性或回滚边界。",
        ),
        (
            "engineering role should provide technical risks or rollback boundaries",
            "工程岗位需要说明技术风险或回滚边界。",
        ),
        (
            "structured role output is missing",
            "岗位没有返回结构化产物。",
        ),
        ("content is empty", "岗位产物为空。"),
        ("open questions remain", "仍有阻塞性待确认问题。"),
        ("worker reported failed status", "worker 报告执行失败。"),
        (
            "selected product context is not allowed for this task",
            "本轮明确要求不引用预设商品，岗位不能把历史商品或默认商品当成本轮商品。",
        ),
        (
            "content looks like placeholder or failure text",
            "岗位产物像占位或失败文本。",
        ),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect()
}

fn default_prefix_localizations() -> Vec<PrefixLocalization> {
    vec![
        PrefixLocalization {
            prefix: "quality completeness: summary is too thin".to_string(),
            message: "岗位摘要过薄，需要补充具体判断。".to_string(),
        },
        PrefixLocalization {
            prefix: "quality completeness: no findings or recommendations".to_string(),
            message: "岗位缺少发现或行动建议。".to_string(),
        },
        PrefixLocalization {
            prefix: "quality actionability:".to_string(),
            message: "岗位缺少可执行建议。".to_string(),
        },
        PrefixLocalization {
            prefix: "quality risk awareness:".to_string(),
            message: "岗位缺少风险边界。".to_string(),
        },
        PrefixLocalization {
            prefix: "upstream evidence: missing citation for input".to_string(),
            message: "协作岗位没有引用上游岗位产物，需要补齐交接证据。".to_string(),
        },
        PrefixLocalization {
            prefix: "upstream evidence: dependent role must cite upstream inputs".to_string(),
            message: "协作岗位需要引用上游输入。".to_string(),
        },
    ]
}

fn default_role_evaluation_policies() -> HashMap<String, RoleEvaluationPolicy> {
    [
        (
            "data",
            RoleEvaluationPolicy {
                min_findings: Some(2),
                require_risks: true,
                risk_sensitive: true,
                reasons: RoleEvaluationReasons {
                    min_findings: "data role should provide at least two findings".to_string(),
                    require_risks: "data role should mark data or interpretation risks".to_string(),
                    ..RoleEvaluationReasons::default()
                },
                ..RoleEvaluationPolicy::default()
            },
        ),
        (
            "ops",
            RoleEvaluationPolicy {
                min_recommendations: Some(3),
                action_oriented: true,
                risk_sensitive: true,
                reasons: RoleEvaluationReasons {
                    min_recommendations: "ops role should provide at least three recommendations"
                        .to_string(),
                    ..RoleEvaluationReasons::default()
                },
                ..RoleEvaluationPolicy::default()
            },
        ),
    ]
    .into_iter()
    .map(|(role, policy)| (role.to_string(), policy))
    .collect()
}
