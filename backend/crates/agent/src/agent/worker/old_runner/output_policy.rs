use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Deserialize;

const DEFAULT_POLICY_PATH: &str = "config/json/agent/worker-output-policy.json";
const POLICY_ENV: &str = "CM_AGENT_WORKER_OUTPUT_POLICY";
const EMBEDDED_POLICY: &str =
    include_str!("../../../../../../config/json/agent/worker-output-policy.json");

#[derive(Debug, Clone, Deserialize)]
struct WorkerOutputPolicy {
    fallback_role_name: String,
    fallback_finding: String,
    fallback_recommendation: String,
    fallback_risk: String,
    #[serde(default)]
    category_boundary: CategoryBoundaryPolicy,
    #[serde(default)]
    roles: BTreeMap<String, RoleOutputPolicy>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct CategoryBoundaryPolicy {
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    risks: Vec<String>,
    #[serde(default)]
    removed_product_identity_evidence: Vec<String>,
    #[serde(default)]
    removed_product_identity_recommendations: Vec<String>,
    #[serde(default)]
    removed_product_identity_risks: Vec<String>,
    #[serde(default)]
    non_blocking_skills: Vec<String>,
    default_skill_boundary_label: Option<String>,
    #[serde(default)]
    skill_boundary_labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RoleOutputPolicy {
    display_name: String,
    #[serde(default = "one")]
    required_findings: usize,
    #[serde(default = "one")]
    required_recommendations: usize,
    #[serde(default)]
    default_findings: Vec<String>,
    #[serde(default)]
    default_recommendations: Vec<String>,
    #[serde(default)]
    default_risks: Vec<String>,
    #[serde(default)]
    finding_rules: Vec<OutputRule>,
    #[serde(default)]
    recommendation_rules: Vec<OutputRule>,
    #[serde(default)]
    risk_rules: Vec<OutputRule>,
    #[serde(default)]
    category_findings: Vec<String>,
    #[serde(default)]
    category_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OutputRule {
    #[serde(default)]
    absent_any: Vec<String>,
    message: String,
}

fn one() -> usize {
    1
}

static POLICY: OnceLock<WorkerOutputPolicy> = OnceLock::new();

pub fn role_display_name(role: &str) -> String {
    policy()
        .role(role)
        .map(|role| role.display_name.clone())
        .unwrap_or_else(|| policy().fallback_role_name.clone())
}

pub fn default_role_findings(role: &str, task: &str) -> Vec<String> {
    let values = policy()
        .role(role)
        .map(|role| role.default_findings.as_slice())
        .unwrap_or(&[]);
    if values.is_empty() {
        vec![render(&policy().fallback_finding, task)]
    } else {
        values.iter().map(|value| render(value, task)).collect()
    }
}

pub fn required_findings_for_role(role: &str) -> usize {
    policy()
        .role(role)
        .map(|role| role.required_findings)
        .unwrap_or(1)
}

pub fn required_recommendations_for_role(role: &str) -> usize {
    policy()
        .role(role)
        .map(|role| role.required_recommendations)
        .unwrap_or(1)
}

pub fn normalize_role_specific_findings(role: &str, findings: &mut Vec<String>, task: &str) {
    apply_rules(
        findings,
        task,
        policy()
            .role(role)
            .map(|role| role.finding_rules.as_slice())
            .unwrap_or(&[]),
    );
}

pub fn normalize_role_specific_recommendations(
    role: &str,
    recommendations: &mut Vec<String>,
    task: &str,
) {
    apply_rules(
        recommendations,
        task,
        policy()
            .role(role)
            .map(|role| role.recommendation_rules.as_slice())
            .unwrap_or(&[]),
    );
}

pub fn normalize_role_specific_risks(role: &str, risks: &mut Vec<String>) {
    apply_rules(
        risks,
        "",
        policy()
            .role(role)
            .map(|role| role.risk_rules.as_slice())
            .unwrap_or(&[]),
    );
}

pub fn default_role_finding(role: &str, index: usize, task: &str) -> String {
    let values = policy()
        .role(role)
        .map(|role| role.default_findings.as_slice())
        .unwrap_or(&[]);
    render(
        indexed_or_fallback(values, index, &policy().fallback_finding),
        task,
    )
}

pub fn default_role_recommendations(role: &str, task: &str) -> Vec<String> {
    let values = policy()
        .role(role)
        .map(|role| role.default_recommendations.as_slice())
        .unwrap_or(&[]);
    if values.is_empty() {
        vec![render(&policy().fallback_recommendation, task)]
    } else {
        values.iter().map(|value| render(value, task)).collect()
    }
}

pub fn default_role_recommendation(role: &str, index: usize, task: &str) -> String {
    let values = policy()
        .role(role)
        .map(|role| role.default_recommendations.as_slice())
        .unwrap_or(&[]);
    render(
        indexed_or_fallback(values, index, &policy().fallback_recommendation),
        task,
    )
}

pub fn role_requires_risk(role: &str) -> bool {
    policy()
        .role(role)
        .is_some_and(|role| !role.default_risks.is_empty())
}

pub fn default_role_risks(role: &str) -> Vec<String> {
    let values = policy()
        .role(role)
        .map(|role| role.default_risks.as_slice())
        .unwrap_or(&[]);
    if values.is_empty() {
        vec![policy().fallback_risk.clone()]
    } else {
        values.to_vec()
    }
}

pub fn apply_category_team_output_shape(
    role: &str,
    output: &mut crate::core::protocol::RoleWorkOutput,
) {
    let Some(role_policy) = policy().role(role) else {
        return;
    };
    for finding in &role_policy.category_findings {
        push_unique(&mut output.findings, finding.clone());
    }
    for recommendation in &role_policy.category_recommendations {
        push_unique(&mut output.recommendations, recommendation.clone());
    }
}

pub fn apply_category_boundary_output(
    output: &mut crate::core::protocol::RoleWorkOutput,
    removed_product_specific_request: bool,
) {
    let category_boundary = &policy().category_boundary;
    for evidence in &category_boundary.evidence {
        push_unique(&mut output.evidence, evidence.clone());
    }
    for risk in &category_boundary.risks {
        push_unique(&mut output.risks, risk.clone());
    }
    if !removed_product_specific_request {
        return;
    }
    for evidence in &category_boundary.removed_product_identity_evidence {
        push_unique(&mut output.evidence, evidence.clone());
    }
    for recommendation in &category_boundary.removed_product_identity_recommendations {
        push_unique(&mut output.recommendations, recommendation.clone());
    }
    for risk in &category_boundary.removed_product_identity_risks {
        push_unique(&mut output.risks, risk.clone());
    }
}

pub fn is_category_non_blocking_skill(skill: &str) -> bool {
    policy()
        .category_boundary
        .non_blocking_skills
        .iter()
        .any(|candidate| candidate == skill)
}

pub fn category_skill_boundary_label(skill: &str) -> String {
    policy()
        .category_boundary
        .skill_boundary_labels
        .get(skill)
        .cloned()
        .or_else(|| {
            policy()
                .category_boundary
                .default_skill_boundary_label
                .clone()
        })
        .unwrap_or_else(|| "该能力本轮只作为可见边界记录".to_string())
}

fn apply_rules(items: &mut Vec<String>, task: &str, rules: &[OutputRule]) {
    for rule in rules {
        if !contains_any_in_items(items, &rule.absent_any) {
            push_unique(items, render(&rule.message, task));
        }
    }
}

fn indexed_or_fallback<'a>(values: &'a [String], index: usize, fallback: &'a str) -> &'a str {
    values
        .get(index)
        .or_else(|| values.last())
        .map(String::as_str)
        .unwrap_or(fallback)
}

fn render(template: &str, task: &str) -> String {
    template.replace("{{task}}", task)
}

fn contains_any_in_items(items: &[String], needles: &[String]) -> bool {
    if needles.is_empty() {
        return false;
    }
    items
        .iter()
        .any(|item| needles.iter().any(|needle| item.contains(needle)))
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if value.trim().is_empty() || items.iter().any(|item| item == &value) {
        return;
    }
    items.push(value);
}

fn policy() -> &'static WorkerOutputPolicy {
    POLICY.get_or_init(load_policy)
}

impl WorkerOutputPolicy {
    fn role(&self, role: &str) -> Option<&RoleOutputPolicy> {
        self.roles.get(role)
    }
}

fn load_policy() -> WorkerOutputPolicy {
    env::var(POLICY_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| Some(workspace_root().join(DEFAULT_POLICY_PATH)))
        .and_then(|path| load_policy_from_path(&path).ok())
        .unwrap_or_else(|| parse_policy(EMBEDDED_POLICY).expect("embedded worker output policy"))
}

fn load_policy_from_path(path: &Path) -> Result<WorkerOutputPolicy, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    parse_policy(&content)
}

fn parse_policy(content: &str) -> Result<WorkerOutputPolicy, String> {
    serde_json::from_str(content).map_err(|error| format!("invalid worker output policy: {error}"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_policy_parses() {
        let policy = parse_policy(EMBEDDED_POLICY).expect("policy should parse");
        assert_eq!(policy.role("data").unwrap().required_findings, 2);
        assert_eq!(policy.role("ops").unwrap().required_recommendations, 3);
    }

    #[test]
    fn renders_task_placeholders() {
        let finding = default_role_finding("data", 0, "分析转化率");
        assert!(finding.contains("分析转化率"));
    }

    #[test]
    fn applies_missing_rule_once() {
        let mut findings = vec![];
        normalize_role_specific_findings("engineering", &mut findings, "检查接口");
        normalize_role_specific_findings("engineering", &mut findings, "检查接口");
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("埋点"));
    }
}
