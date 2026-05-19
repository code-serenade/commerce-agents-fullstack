use serde_json::Value;

use super::{
    part::{diagnosis_is_user_success, diagnosis_status},
    synthesis::comprehensive_risks,
};
use crate::skills::data::common::value_pointer_number;

pub(super) fn comprehensive_health_score(parts: &[Value]) -> i64 {
    if let Some(score) = parts
        .iter()
        .find(|part| {
            part.get("skill").and_then(Value::as_str) == Some("data_dashboard")
                && diagnosis_is_user_success(part)
        })
        .and_then(|part| part.get("output"))
        .and_then(|output| value_pointer_number(output, "/经营健康度/总分"))
    {
        return score.round().clamp(0.0, 100.0) as i64;
    }

    let user_success = parts
        .iter()
        .filter(|part| diagnosis_is_user_success(part))
        .count() as i64;
    let deferred = parts
        .iter()
        .filter(|part| diagnosis_status(part) == "deferred")
        .count() as i64;
    let risk_count = comprehensive_risks(parts).len() as i64;
    (62 + user_success * 5 - deferred * 3 - risk_count * 2).clamp(35, 92)
}

pub(super) fn diagnosis_grade(score: i64) -> &'static str {
    if score >= 80 {
        "健康"
    } else if score >= 60 {
        "需关注"
    } else {
        "风险"
    }
}
