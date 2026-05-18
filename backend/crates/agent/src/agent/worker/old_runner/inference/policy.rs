const INFERENCE_POLICY_PATH: &str = "config/json/agent/worker-inference-policy.json";
const EMBEDDED_INFERENCE_POLICY: &str =
    include_str!("../../../../../../../config/json/agent/worker-inference-policy.json");

static INFERENCE_POLICY: std::sync::OnceLock<InferencePolicy> = std::sync::OnceLock::new();

#[derive(Debug, serde::Deserialize)]
struct InferencePolicy {
    rules: std::collections::HashMap<String, Vec<PolicyRule>>,
}

#[derive(Debug, serde::Deserialize)]
struct PolicyRule {
    label: String,
    keywords: Vec<String>,
}

impl InferencePolicy {
    fn load() -> Self {
        let json = std::fs::read_to_string(INFERENCE_POLICY_PATH)
            .unwrap_or_else(|_| EMBEDDED_INFERENCE_POLICY.to_owned());
        serde_json::from_str(&json).unwrap_or_else(|_| {
            serde_json::from_str(EMBEDDED_INFERENCE_POLICY)
                .expect("embedded worker inference policy must be valid JSON")
        })
    }

    fn match_label<'a>(&'a self, text: &str, rule_set: &str) -> Option<&'a str> {
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
}

fn inference_policy() -> &'static InferencePolicy {
    INFERENCE_POLICY.get_or_init(InferencePolicy::load)
}

fn policy_match_label(text: &str, rule_set: &str) -> Option<&'static str> {
    inference_policy().match_label(text, rule_set)
}
