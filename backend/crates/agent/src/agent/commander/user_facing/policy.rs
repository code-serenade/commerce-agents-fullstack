use std::{collections::BTreeMap, fs, path::Path, sync::OnceLock};

use serde::Deserialize;

const USER_FACING_NORMALIZATION_POLICY: &str =
    include_str!("../../../../../../config/json/agent/user-facing-normalization-policy.json");

#[derive(Debug, Clone, Deserialize)]
pub(super) struct UserFacingNormalizationPolicy {
    pub(super) no_user_facing_content_message: Vec<String>,
    #[serde(default)]
    pub(super) material_gap_markers: Vec<String>,
    #[serde(default)]
    pub(super) material_gap_business_markers: Vec<String>,
    pub(super) live_search_default_success_message: String,
    #[serde(default)]
    pub(super) live_search_degraded_lower_markers: Vec<String>,
    #[serde(default)]
    pub(super) live_search_degraded_markers: Vec<String>,
    #[serde(default)]
    pub(super) live_search_success_markers: Vec<String>,
    #[serde(default)]
    pub(super) live_search_required_evidence_markers: Vec<String>,
    #[serde(default)]
    pub(super) unverified_live_search_success_line_markers: Vec<String>,
    #[serde(default)]
    pub(super) unverified_live_search_success_phrases: Vec<String>,
    #[serde(default)]
    pub(super) no_source_claim_markers: Vec<String>,
    #[serde(default)]
    pub(super) internal_context_markers: Vec<String>,
    #[serde(default)]
    pub(super) internal_skill_name_replacements: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) business_phrase_replacements: Vec<[String; 2]>,
    #[serde(default)]
    pub(super) unverified_live_search_availability_phrases: Vec<String>,
    #[serde(default)]
    pub(super) memory_context_leak_markers: Vec<String>,
    #[serde(default)]
    pub(super) internal_noise_lower_markers: Vec<String>,
    #[serde(default)]
    pub(super) internal_noise_markers: Vec<String>,
    #[serde(default)]
    pub(super) prompt_instruction_leak_markers: Vec<String>,
    #[serde(default)]
    pub(super) answer_structure_rules: Vec<UserFacingAnswerStructureRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct UserFacingAnswerStructureRule {
    #[serde(default)]
    pub(super) text_any: Vec<String>,
    #[serde(default)]
    pub(super) sections: Vec<UserFacingAnswerStructureSection>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct UserFacingAnswerStructureSection {
    pub(super) heading: String,
    #[serde(default)]
    pub(super) aliases: Vec<String>,
}

static USER_FACING_POLICY: OnceLock<UserFacingNormalizationPolicy> = OnceLock::new();

pub(super) fn user_facing_policy() -> &'static UserFacingNormalizationPolicy {
    USER_FACING_POLICY.get_or_init(load_user_facing_policy)
}

fn load_user_facing_policy() -> UserFacingNormalizationPolicy {
    load_user_facing_policy_from_path(Path::new(
        "config/json/agent/user-facing-normalization-policy.json",
    ))
    .unwrap_or_else(|| {
        serde_json::from_str(USER_FACING_NORMALIZATION_POLICY).unwrap_or_else(|err| {
            tracing::warn!(
                error = %err,
                "failed to parse embedded user-facing normalization policy"
            );
            UserFacingNormalizationPolicy::fallback()
        })
    })
}

fn load_user_facing_policy_from_path(path: &Path) -> Option<UserFacingNormalizationPolicy> {
    let content = fs::read_to_string(path).ok()?;
    match serde_json::from_str(&content) {
        Ok(policy) => Some(policy),
        Err(err) => {
            tracing::warn!(
                path = %path.display(),
                error = %err,
                "failed to load user-facing normalization policy; using embedded policy"
            );
            None
        }
    }
}

impl UserFacingNormalizationPolicy {
    fn fallback() -> Self {
        Self {
            no_user_facing_content_message: vec![
                "## 当前状态".to_string(),
                String::new(),
                "这次没有拿到可直接展示的完整内容。".to_string(),
                String::new(),
                "## 下一步".to_string(),
                String::new(),
                "- 可以补充更明确的问题、商品、活动或关键材料。".to_string(),
            ],
            material_gap_markers: Vec::new(),
            material_gap_business_markers: Vec::new(),
            live_search_default_success_message: "本轮已通过外部搜索取得实时来源。".to_string(),
            live_search_degraded_lower_markers: Vec::new(),
            live_search_degraded_markers: Vec::new(),
            live_search_success_markers: Vec::new(),
            live_search_required_evidence_markers: Vec::new(),
            unverified_live_search_success_line_markers: Vec::new(),
            unverified_live_search_success_phrases: Vec::new(),
            no_source_claim_markers: Vec::new(),
            internal_context_markers: Vec::new(),
            internal_skill_name_replacements: BTreeMap::new(),
            business_phrase_replacements: Vec::new(),
            unverified_live_search_availability_phrases: Vec::new(),
            memory_context_leak_markers: Vec::new(),
            internal_noise_lower_markers: Vec::new(),
            internal_noise_markers: Vec::new(),
            prompt_instruction_leak_markers: Vec::new(),
            answer_structure_rules: Vec::new(),
        }
    }
}
