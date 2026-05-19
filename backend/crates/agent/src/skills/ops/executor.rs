use serde_json::{Value, json};

use super::{
    common::{invalid_input, nums, outcome, strings},
    core_ops::{
        assortment_planning, channel_strategy, execution_plan, inventory_planning, listing_copy,
        pricing_strategy, promo_planning,
    },
    optimization::{abc_xyz_classifier, ad_fatigue_detector, inventory_optimizer, smart_pricing},
};
use crate::skills::{SkillContext, SkillError, SkillOutcome};

pub async fn execute(
    skill_name: &str,
    input: Value,
    _context: &SkillContext,
) -> Result<SkillOutcome, SkillError> {
    let mut value = match skill_name {
        "ops_pricing_strategy" => pricing_strategy(&input),
        "ops_promo_planning" => promo_planning(&input),
        "ops_channel_strategy" => channel_strategy(&input),
        "ops_inventory_planning" => inventory_planning(&input),
        "ops_listing_copy" => listing_copy(&input),
        "ops_assortment_planning" => assortment_planning(&input),
        "ops_execution_plan" => execution_plan(&input),
        "ops_smart_pricing" => smart_pricing(&input),
        "ops_inventory_optimizer" => inventory_optimizer(&input),
        "ops_ad_fatigue_detector" => ad_fatigue_detector(&input),
        "ops_abc_xyz_classifier" => abc_xyz_classifier(&input),
        other => return Err(invalid_input(format!("unknown ops skill: {other}"))),
    };
    normalize_ops_output(skill_name, &input, &mut value);
    Ok(outcome(value))
}

fn normalize_ops_output(skill_name: &str, input: &Value, output: &mut Value) {
    let Some(map) = output.as_object_mut() else {
        return;
    };
    if map.contains_key("status") {
        return;
    }

    let has_data = output_has_data(map);
    if !has_data {
        map.insert("status".to_owned(), json!("deferred"));
        map.insert("has_data".to_owned(), json!(false));
        map.insert("adapter_status".to_owned(), json!("user_input_required"));
        map.insert("data_readiness".to_owned(), json!("missing_user_input"));
        map.insert("summary".to_owned(), json!(ops_summary(skill_name)));
        return;
    }

    map.insert("status".to_owned(), json!("success"));
    map.insert("has_data".to_owned(), json!(true));
    map.insert("adapter_status".to_owned(), json!("manual_input"));
    map.insert(
        "data_readiness".to_owned(),
        json!(ops_data_readiness(skill_name, input)),
    );
    map.insert("summary".to_owned(), json!(ops_summary(skill_name)));
}

fn output_has_data(map: &serde_json::Map<String, Value>) -> bool {
    !matches!(map.get("has_data").and_then(Value::as_bool), Some(false))
        && !map.contains_key("error")
}

fn ops_data_readiness(skill_name: &str, input: &Value) -> &'static str {
    match skill_name {
        "ops_pricing_strategy" | "ops_smart_pricing"
            if nums(input, "competitor_prices").is_empty() =>
        {
            "calculated_from_defaults"
        }
        "ops_listing_copy" if strings(input, "features").is_empty() => "calculated_from_defaults",
        _ => "user_provided",
    }
}

fn ops_summary(skill_name: &str) -> &'static str {
    match skill_name {
        "ops_promo_planning" => "促销策划方案已生成",
        "ops_pricing_strategy" => "定价策略已生成",
        "ops_channel_strategy" => "渠道策略已生成",
        "ops_inventory_planning" => "库存规划已生成",
        "ops_listing_copy" => "上架文案已生成",
        "ops_assortment_planning" => "选品规划已生成",
        "ops_execution_plan" => "运营执行计划已生成",
        "ops_smart_pricing" => "智能定价方案已生成",
        "ops_inventory_optimizer" => "库存优化方案已生成",
        "ops_ad_fatigue_detector" => "广告疲劳检测报告已生成",
        "ops_abc_xyz_classifier" => "ABC-XYZ库存分类报告已生成",
        _ => "运营技能结果已生成",
    }
}
