use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Deserialize;

const DEFAULT_POLICY_PATH: &str = "config/json/agent/worker-output-normalization-policy.json";
const POLICY_ENV: &str = "CM_AGENT_WORKER_OUTPUT_NORMALIZATION_POLICY";
const EMBEDDED_POLICY: &str = include_str!(
    "../../../../../../../config/json/agent/worker-output-normalization-policy.json"
);

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

static POLICY: OnceLock<OutputNormalizationPolicy> = OnceLock::new();

pub(super) fn policy() -> &'static OutputNormalizationPolicy {
    POLICY.get_or_init(load_policy)
}

pub(super) fn contains_any(value: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

pub(super) fn contains_any_case_insensitive_ascii(value: &str, needles: &[String]) -> bool {
    let lower = value.to_ascii_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
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

fn load_policy() -> OutputNormalizationPolicy {
    env::var(POLICY_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| Some(workspace_root().join(DEFAULT_POLICY_PATH)))
        .and_then(|path| load_policy_from_path(&path).ok())
        .unwrap_or_else(|| {
            parse_policy(EMBEDDED_POLICY).expect("embedded worker output normalization policy")
        })
}

fn load_policy_from_path(path: &Path) -> Result<OutputNormalizationPolicy, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    parse_policy(&content)
}

fn parse_policy(content: &str) -> Result<OutputNormalizationPolicy, String> {
    serde_json::from_str(content)
        .map_err(|error| format!("invalid worker output normalization policy: {error}"))
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
        assert!(
            policy
                .category_level_task_keywords()
                .iter()
                .any(|keyword| keyword == "类目")
        );
        assert_eq!(
            policy.unsupported_benchmark().summary_replacement(),
            "本岗位已按可用数据形成判断；没有真实外部来源时，不输出行业区间、行业均值或行业百分位，只保留可复核指标、业务边界和待补基准。"
        );
    }
}
