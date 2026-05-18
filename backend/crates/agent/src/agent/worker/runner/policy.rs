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
    pub(super) inference: super::inference::config::WorkerInferencePolicy,
    pub(super) output: WorkerOutputPolicy,
    pub(super) output_normalization: OutputNormalizationPolicy,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WorkerOutputPolicy {
    fallback_role_name: String,
    fallback_finding: String,
    fallback_recommendation: String,
    fallback_risk: String,
    #[serde(default)]
    category_boundary: OutputCategoryBoundaryPolicy,
    #[serde(default)]
    roles: std::collections::BTreeMap<String, RoleOutputPolicy>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct OutputCategoryBoundaryPolicy {
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
    #[serde(default)]
    deferred_skills: Vec<String>,
    default_skill_boundary_label: Option<String>,
    #[serde(default)]
    skill_boundary_labels: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RoleOutputPolicy {
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

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OutputNormalizationPolicy {
    #[serde(default)]
    category_level_task_keywords: Vec<String>,
    #[serde(default)]
    specific_product_identity_keywords: Vec<String>,
    #[serde(default)]
    unselected_product_boundary_keywords: Vec<String>,
    #[serde(default)]
    runtime_noise_keywords: Vec<String>,
    #[serde(default)]
    refusal_summary_keywords: Vec<String>,
    #[serde(default)]
    category_boundary_explanation_keywords: Vec<String>,
    #[serde(default)]
    role_skill_runtime_gap_keywords: Vec<String>,
    #[serde(default)]
    runtime_data_gap_keywords: Vec<String>,
    category_runtime_summary_replacement: String,
    default_summary_template: String,
    category_product_summary_template: String,
    default_evidence: String,
    selected_product_evidence_template: String,
    unselected_product_evidence: String,
    upstream_evidence_template: String,
    #[serde(default)]
    runtime_evidence_keep_markers: Vec<String>,
    category_skill_boundary_template: String,
    unsupported_benchmark: UnsupportedBenchmarkPolicy,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct UnsupportedBenchmarkPolicy {
    summary_replacement: String,
    risk: String,
    #[serde(default)]
    allowed_source_markers: Vec<String>,
    #[serde(default)]
    assertive_comparison_terms: Vec<String>,
    #[serde(default)]
    benchmark_terms: Vec<String>,
    #[serde(default)]
    benchmark_number_terms: Vec<String>,
    #[serde(default)]
    benchmark_request_terms: Vec<String>,
    #[serde(default)]
    assertive_benchmark_terms: Vec<String>,
}

fn one() -> usize {
    1
}

impl WorkerOutputPolicy {
    pub(super) fn role_display_name(&self, role: &str) -> &str {
        self.role(role)
            .map(|role| role.display_name.as_str())
            .unwrap_or(&self.fallback_role_name)
    }

    pub(super) fn default_role_findings(&self, role: &str, task: &str) -> Vec<String> {
        let values = self
            .role(role)
            .map(|role| role.default_findings.as_slice())
            .unwrap_or(&[]);
        if values.is_empty() {
            vec![render(&self.fallback_finding, task)]
        } else {
            values.iter().map(|value| render(value, task)).collect()
        }
    }

    pub(super) fn required_findings_for_role(&self, role: &str) -> usize {
        self.role(role)
            .map(|role| role.required_findings)
            .unwrap_or(1)
    }

    pub(super) fn required_recommendations_for_role(&self, role: &str) -> usize {
        self.role(role)
            .map(|role| role.required_recommendations)
            .unwrap_or(1)
    }

    pub(super) fn default_role_finding(&self, role: &str, index: usize, task: &str) -> String {
        let values = self
            .role(role)
            .map(|role| role.default_findings.as_slice())
            .unwrap_or(&[]);
        render(
            indexed_or_fallback(values, index, &self.fallback_finding),
            task,
        )
    }

    pub(super) fn default_role_recommendations(&self, role: &str, task: &str) -> Vec<String> {
        let values = self
            .role(role)
            .map(|role| role.default_recommendations.as_slice())
            .unwrap_or(&[]);
        if values.is_empty() {
            vec![render(&self.fallback_recommendation, task)]
        } else {
            values.iter().map(|value| render(value, task)).collect()
        }
    }

    pub(super) fn default_role_recommendation(
        &self,
        role: &str,
        index: usize,
        task: &str,
    ) -> String {
        let values = self
            .role(role)
            .map(|role| role.default_recommendations.as_slice())
            .unwrap_or(&[]);
        render(
            indexed_or_fallback(values, index, &self.fallback_recommendation),
            task,
        )
    }

    pub(super) fn role_requires_risk(&self, role: &str) -> bool {
        self.role(role)
            .is_some_and(|role| !role.default_risks.is_empty())
    }

    pub(super) fn default_role_risks(&self, role: &str) -> Vec<String> {
        let values = self
            .role(role)
            .map(|role| role.default_risks.as_slice())
            .unwrap_or(&[]);
        if values.is_empty() {
            vec![self.fallback_risk.clone()]
        } else {
            values.to_vec()
        }
    }

    pub(super) fn normalize_role_specific_findings(
        &self,
        role: &str,
        findings: &mut Vec<String>,
        task: &str,
    ) {
        apply_rules(
            findings,
            task,
            self.role(role)
                .map(|role| role.finding_rules.as_slice())
                .unwrap_or(&[]),
        );
    }

    pub(super) fn normalize_role_specific_recommendations(
        &self,
        role: &str,
        recommendations: &mut Vec<String>,
        task: &str,
    ) {
        apply_rules(
            recommendations,
            task,
            self.role(role)
                .map(|role| role.recommendation_rules.as_slice())
                .unwrap_or(&[]),
        );
    }

    pub(super) fn normalize_role_specific_risks(&self, role: &str, risks: &mut Vec<String>) {
        apply_rules(
            risks,
            "",
            self.role(role)
                .map(|role| role.risk_rules.as_slice())
                .unwrap_or(&[]),
        );
    }

    pub(super) fn category_boundary(&self) -> &OutputCategoryBoundaryPolicy {
        &self.category_boundary
    }

    pub(super) fn is_category_deferred_skill(&self, skill: &str) -> bool {
        self.category_boundary
            .deferred_skills
            .iter()
            .any(|candidate| candidate == skill)
    }

    pub(super) fn is_category_non_blocking_skill(&self, skill: &str) -> bool {
        self.category_boundary
            .non_blocking_skills
            .iter()
            .any(|candidate| candidate == skill)
    }

    pub(super) fn category_skill_boundary_label(&self, skill: &str) -> String {
        self.category_boundary
            .skill_boundary_labels
            .get(skill)
            .cloned()
            .or_else(|| self.category_boundary.default_skill_boundary_label.clone())
            .unwrap_or_else(|| "该能力本轮只作为可见边界记录".to_string())
    }

    pub(super) fn role(&self, role: &str) -> Option<&RoleOutputPolicy> {
        self.roles.get(role)
    }
}

impl OutputCategoryBoundaryPolicy {
    pub(super) fn evidence(&self) -> &[String] {
        &self.evidence
    }

    pub(super) fn risks(&self) -> &[String] {
        &self.risks
    }

    pub(super) fn removed_product_identity_evidence(&self) -> &[String] {
        &self.removed_product_identity_evidence
    }

    pub(super) fn removed_product_identity_recommendations(&self) -> &[String] {
        &self.removed_product_identity_recommendations
    }

    pub(super) fn removed_product_identity_risks(&self) -> &[String] {
        &self.removed_product_identity_risks
    }
}

impl RoleOutputPolicy {
    pub(super) fn category_findings(&self) -> &[String] {
        &self.category_findings
    }

    pub(super) fn category_recommendations(&self) -> &[String] {
        &self.category_recommendations
    }
}

impl OutputNormalizationPolicy {
    pub(super) fn category_level_task_keywords(&self) -> &[String] {
        &self.category_level_task_keywords
    }

    pub(super) fn specific_product_identity_keywords(&self) -> &[String] {
        &self.specific_product_identity_keywords
    }

    pub(super) fn unselected_product_boundary_keywords(&self) -> &[String] {
        &self.unselected_product_boundary_keywords
    }

    pub(super) fn runtime_noise_keywords(&self) -> &[String] {
        &self.runtime_noise_keywords
    }

    pub(super) fn refusal_summary_keywords(&self) -> &[String] {
        &self.refusal_summary_keywords
    }

    pub(super) fn category_boundary_explanation_keywords(&self) -> &[String] {
        &self.category_boundary_explanation_keywords
    }

    pub(super) fn role_skill_runtime_gap_keywords(&self) -> &[String] {
        &self.role_skill_runtime_gap_keywords
    }

    pub(super) fn runtime_data_gap_keywords(&self) -> &[String] {
        &self.runtime_data_gap_keywords
    }

    pub(super) fn category_runtime_summary_replacement(&self) -> &str {
        &self.category_runtime_summary_replacement
    }

    pub(super) fn default_summary(&self, role: &str, task: &str) -> String {
        render_role_task_template(&self.default_summary_template, role, task)
    }

    pub(super) fn category_product_summary(&self, role: &str, task: &str) -> String {
        render_role_task_template(&self.category_product_summary_template, role, task)
    }

    pub(super) fn default_evidence(&self) -> &str {
        &self.default_evidence
    }

    pub(super) fn selected_product_evidence(
        &self,
        product_name: &str,
        product_ref: &str,
    ) -> String {
        self.selected_product_evidence_template
            .replace("{{product_name}}", product_name)
            .replace("{{product_ref}}", product_ref)
    }

    pub(super) fn unselected_product_evidence(&self) -> &str {
        &self.unselected_product_evidence
    }

    pub(super) fn upstream_evidence(&self, node_id: &str, role: &str) -> String {
        self.upstream_evidence_template
            .replace("{{node_id}}", node_id)
            .replace("{{role}}", role)
    }

    pub(super) fn runtime_evidence_keep_markers(&self) -> &[String] {
        &self.runtime_evidence_keep_markers
    }

    pub(super) fn category_skill_boundary_output(&self, label: &str) -> String {
        self.category_skill_boundary_template
            .replace("{{label}}", label)
    }

    pub(super) fn unsupported_benchmark(&self) -> &UnsupportedBenchmarkPolicy {
        &self.unsupported_benchmark
    }
}

impl UnsupportedBenchmarkPolicy {
    pub(super) fn summary_replacement(&self) -> &str {
        &self.summary_replacement
    }

    pub(super) fn risk(&self) -> &str {
        &self.risk
    }

    pub(super) fn allowed_source_markers(&self) -> &[String] {
        &self.allowed_source_markers
    }

    pub(super) fn assertive_comparison_terms(&self) -> &[String] {
        &self.assertive_comparison_terms
    }

    pub(super) fn benchmark_terms(&self) -> &[String] {
        &self.benchmark_terms
    }

    pub(super) fn benchmark_number_terms(&self) -> &[String] {
        &self.benchmark_number_terms
    }

    pub(super) fn benchmark_request_terms(&self) -> &[String] {
        &self.benchmark_request_terms
    }

    pub(super) fn assertive_benchmark_terms(&self) -> &[String] {
        &self.assertive_benchmark_terms
    }
}

fn apply_rules(items: &mut Vec<String>, task: &str, rules: &[OutputRule]) {
    for rule in rules {
        if !contains_any_in_items(items, &rule.absent_any) {
            super::util::push_unique(items, render(&rule.message, task));
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

fn render_role_task_template(template: &str, role: &str, task: &str) -> String {
    template.replace("{{role}}", role).replace("{{task}}", task)
}

fn contains_any_in_items(items: &[String], needles: &[String]) -> bool {
    if needles.is_empty() {
        return false;
    }
    items
        .iter()
        .any(|item| needles.iter().any(|needle| item.contains(needle)))
}

static POLICIES: OnceLock<WorkerPolicySet> = OnceLock::new();

pub(super) fn worker_policies() -> &'static WorkerPolicySet {
    POLICIES.get_or_init(load_worker_policies)
}

fn load_worker_policies() -> WorkerPolicySet {
    WorkerPolicySet {
        autosuggest: super::autosuggest::load_autosuggest_policy(),
        inference: super::inference::load_inference_policy(),
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
