use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{
        infer_learning_level, infer_scenario, next_practice, normalized_optional_string,
        scenario_example, self_check_question,
    },
    profiles::topic_profile,
    specs::learning_teaching_explain_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_bool_param,
    optional_string_param, string_param,
};

pub struct LearningTeachingExplain {
    spec: SkillSpec,
}

impl LearningTeachingExplain {
    pub fn new() -> Self {
        Self {
            spec: learning_teaching_explain_spec(),
        }
    }
}

impl Default for LearningTeachingExplain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for LearningTeachingExplain {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let topic = string_param(&params, "topic")?;
        let learning_level = optional_string_param(&params, "learning_level")?
            .unwrap_or_else(|| infer_learning_level(&topic).to_owned());
        let scenario = optional_string_param(&params, "scenario")?
            .unwrap_or_else(|| infer_scenario(&topic).to_owned());
        let product_ref =
            normalized_optional_string(optional_string_param(&params, "product_ref")?);
        let product_name =
            normalized_optional_string(optional_string_param(&params, "product_name")?);
        let platform = normalized_optional_string(optional_string_param(&params, "platform")?);
        let include_self_check =
            optional_bool_param(&params, "include_self_check")?.unwrap_or(true);

        let product_bound = product_ref.is_some() || product_name.is_some();
        let profile = topic_profile(&topic);
        let product_context = if product_bound {
            json!({
                "mode": "explicit_selected_product",
                "product_ref": product_ref,
                "product_name": product_name,
                "platform": platform,
                "note": "仅使用本轮显式选择的商品上下文；未读取历史商品或预设样例商品。"
            })
        } else {
            json!({
                "mode": "unselected",
                "note": "本轮未显式选择商品，以下为通用教学解释；不会注入历史商品、预设商品或演示商品。"
            })
        };

        let mut output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": if product_bound { "explicit_product_context" } else { "user_provided" },
            "summary": format!("教学解释已生成：{topic}"),
            "skill": self.spec.id,
            "教学主题": topic,
            "教学层级": learning_level,
            "业务场景": scenario,
            "商品上下文": product_context,
            "30秒结论": profile.conclusion,
            "术语白话解释": profile.terms,
            "为什么": profile.why,
            "怎么做": profile.how,
            "场景示例": scenario_example(&scenario, product_bound),
            "常见误区": profile.pitfalls,
            "下一步练习": next_practice(&scenario),
            "治理说明": [
                "本 skill 是新产品 Rust Agent native 教学能力，不调用外部平台、不读取实时店铺数据、不执行高风险动作。",
                "如果需要结合具体商品，必须由本轮显式选品或显式 contextRefs 提供商品上下文。",
                "如果用户要求最新规则、公开来源或实时数据，应另行触发 search_* adapter，不能把教学解释伪造成实时搜索。"
            ]
        });
        if include_self_check {
            output["自检问题"] = self_check_question(&topic, &profile);
        }

        Ok(SkillOutcome::new(output).with_summary(format!("教学解释已生成：{topic}")))
    }
}
