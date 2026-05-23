use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use tracing::warn;

const INFERENCE_ENV: &str = "CM_AGENT_WORKER_INFERENCE_POLICY";
const DEFAULT_INFERENCE_DIR: &str = "config/json/agent/worker-inference";
const DEFAULT_INFERENCE_FILE: &str = "config/json/agent/worker-inference-policy.json";
const EMBEDDED_INFERENCE_POLICY: &str =
    include_str!("../../../../../../../config/json/agent/worker-inference-policy.json");

#[derive(Debug, Clone, Default, Deserialize)]
pub(in crate::agent::worker::runner) struct WorkerInferencePolicy {
    #[serde(default)]
    rules: HashMap<String, Vec<InferenceRule>>,
    #[serde(flatten)]
    sections: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct InferenceRule {
    label: String,
    #[serde(default)]
    keywords: Vec<String>,
}

impl WorkerInferencePolicy {
    pub(super) fn match_label(&self, text: &str, rule_set: &str) -> Option<&str> {
        let lower = text.to_ascii_lowercase();
        self.rules.get(rule_set)?.iter().find_map(|rule| {
            rule.keywords
                .iter()
                .any(|keyword| {
                    text.contains(keyword) || lower.contains(&keyword.to_ascii_lowercase())
                })
                .then_some(rule.label.as_str())
        })
    }

    pub(super) fn section(&self, name: &str) -> Option<&serde_json::Value> {
        self.sections.get(name)
    }

    pub(super) fn string_list(&self, section: &str, pointer: &str) -> Vec<String> {
        self.section(section)
            .and_then(|value| value.pointer(pointer))
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(super) fn string_list_or(
        &self,
        section: &str,
        pointer: &str,
        fallback: &[&str],
    ) -> Vec<String> {
        let values = self.string_list(section, pointer);
        if values.is_empty() {
            fallback.iter().map(|value| (*value).to_string()).collect()
        } else {
            values
        }
    }

    pub(super) fn string_or(&self, section: &str, pointer: &str, fallback: &str) -> String {
        self.section(section)
            .and_then(|value| value.pointer(pointer))
            .and_then(serde_json::Value::as_str)
            .unwrap_or(fallback)
            .to_string()
    }

    pub(super) fn f64_or(&self, section: &str, pointer: &str, fallback: f64) -> f64 {
        self.section(section)
            .and_then(|value| value.pointer(pointer))
            .and_then(serde_json::Value::as_f64)
            .filter(|value| value.is_finite())
            .unwrap_or(fallback)
    }

    pub(super) fn i64_or(&self, section: &str, pointer: &str, fallback: i64) -> i64 {
        self.section(section)
            .and_then(|value| value.pointer(pointer))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(fallback)
    }
}

pub(in crate::agent::worker::runner) fn load_inference_policy() -> WorkerInferencePolicy {
    if let Ok(path) = std::env::var(INFERENCE_ENV)
        && !path.trim().is_empty()
    {
        match load_inference_path(&resolve_policy_path(&path)) {
            Ok(policy) => return policy,
            Err(error) => warn!(
                path,
                error, "failed to load configured worker inference policy"
            ),
        }
    }

    let directory_path = resolve_policy_path(DEFAULT_INFERENCE_DIR);
    match load_inference_directory(&directory_path) {
        Ok(policy) => policy,
        Err(error) => {
            warn!(
                path = %directory_path.display(),
                error,
                "failed to load worker inference directory; trying legacy file"
            );

            let legacy_path = resolve_policy_path(DEFAULT_INFERENCE_FILE);
            match parse_inference_file(&legacy_path) {
                Ok(policy) => policy,
                Err(error) => {
                    warn!(
                        path = %legacy_path.display(),
                        error,
                        "failed to load legacy worker inference policy; using embedded policy"
                    );
                    serde_json::from_str(EMBEDDED_INFERENCE_POLICY)
                        .expect("embedded worker inference policy must be valid JSON")
                }
            }
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

fn load_inference_path(path: &Path) -> Result<WorkerInferencePolicy, String> {
    if path.is_dir() {
        load_inference_directory(path)
    } else {
        parse_inference_file(path)
    }
}

#[derive(Debug, Deserialize)]
struct InferenceIndex {
    #[serde(default)]
    files: Vec<String>,
}

fn load_inference_directory(directory: &Path) -> Result<WorkerInferencePolicy, String> {
    let index =
        parse_json_file::<InferenceIndex>("worker inference index", &directory.join("index.json"))?;
    let mut policy = WorkerInferencePolicy::default();
    let mut loaded = HashSet::new();

    for file in index.files {
        if !loaded.insert(file.clone()) {
            continue;
        }
        let fragment_path = directory.join(&file);
        let fragment =
            parse_json_file::<serde_json::Value>("worker inference fragment", &fragment_path)?;
        merge_policy_fragment(&mut policy, fragment)?;
    }

    Ok(policy)
}

fn merge_policy_fragment(
    policy: &mut WorkerInferencePolicy,
    fragment: serde_json::Value,
) -> Result<(), String> {
    let serde_json::Value::Object(fragment) = fragment else {
        return Err("worker inference fragment must be a JSON object".to_string());
    };

    for (key, value) in fragment {
        if key == "rules" {
            merge_rules(policy, value)?;
        } else {
            merge_section(&mut policy.sections, key, value);
        }
    }
    Ok(())
}

fn merge_rules(policy: &mut WorkerInferencePolicy, value: serde_json::Value) -> Result<(), String> {
    let serde_json::Value::Object(rules) = value else {
        merge_section(&mut policy.sections, "rules".to_string(), value);
        return Ok(());
    };

    let mut non_label_rules = serde_json::Map::new();
    for (key, value) in rules {
        if is_label_rule_list(&value) {
            let mut rules = serde_json::from_value::<Vec<InferenceRule>>(value)
                .map_err(|error| format!("invalid inference label rules for {key}: {error}"))?;
            policy.rules.entry(key).or_default().append(&mut rules);
        } else {
            non_label_rules.insert(key, value);
        }
    }

    if !non_label_rules.is_empty() {
        merge_section(
            &mut policy.sections,
            "rules".to_string(),
            serde_json::Value::Object(non_label_rules),
        );
    }

    Ok(())
}

fn is_label_rule_list(value: &serde_json::Value) -> bool {
    value.as_array().is_some_and(|rules| {
        rules.iter().all(|rule| {
            rule.as_object().is_some_and(|rule| {
                rule.get("label")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
                    && rule
                        .get("keywords")
                        .and_then(serde_json::Value::as_array)
                        .is_some()
            })
        })
    })
}

fn merge_section(
    sections: &mut HashMap<String, serde_json::Value>,
    key: String,
    value: serde_json::Value,
) {
    match (sections.get_mut(&key), value) {
        (Some(serde_json::Value::Object(existing)), serde_json::Value::Object(next)) => {
            for (nested_key, nested_value) in next {
                existing.insert(nested_key, nested_value);
            }
        }
        (_, value) => {
            sections.insert(key, value);
        }
    }
}

fn parse_inference_file(path: &Path) -> Result<WorkerInferencePolicy, String> {
    parse_json_file("worker inference policy", path)
}

fn parse_json_file<T>(name: &str, path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let content = fs::read_to_string(path).map_err(|error| format!("read: {error}"))?;
    serde_json::from_str(&content).map_err(|error| format!("invalid {name}: {error}"))
}
