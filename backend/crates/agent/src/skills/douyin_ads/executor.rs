use serde_json::Value;

use super::skills::{audience_package, budget_pacing, live_funnel_diagnosis, plan_structure};
use crate::skills::{SkillContext, SkillError, SkillOutcome};

pub async fn execute(
    skill_name: &str,
    input: Value,
    _context: &SkillContext,
) -> Result<SkillOutcome, SkillError> {
    let output = match skill_name {
        "douyin_ads_plan_structure" => plan_structure(&input),
        "douyin_ads_audience_package" => audience_package(&input),
        "douyin_ads_budget_pacing" => budget_pacing(&input),
        "douyin_ads_live_funnel_diagnosis" => live_funnel_diagnosis(&input),
        other => {
            return Err(SkillError::execution(format!(
                "unknown douyin ads skill: {other}"
            )));
        }
    };
    Ok(SkillOutcome::new(output))
}
