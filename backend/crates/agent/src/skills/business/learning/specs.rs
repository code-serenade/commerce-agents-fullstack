use serde_json::json;

use crate::skills::{SkillCategory, SkillInputField, SkillPriority, SkillSpec};

fn category() -> SkillCategory {
    SkillCategory::Custom("learning".to_owned())
}

pub fn specs() -> Vec<SkillSpec> {
    vec![learning_teaching_explain_spec()]
}

pub fn learning_teaching_explain_spec() -> SkillSpec {
    SkillSpec::new(
        "learning_teaching_explain",
        "教学解释",
        "按教学模式解释电商概念、指标、策略或操作方法，输出30秒结论、为什么、怎么做、\
         常见误区和自检题。",
    )
    .with_category(category())
    .with_priority(SkillPriority::High)
    .with_input(SkillInputField::required(
        "topic",
        "需要教学解释的主题，例如 ROI和ROAS的区别、DSR是什么、怎么理解转化率",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "learning_level",
        "教学层级：beginner / undergraduate / practitioner / expert",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "scenario",
        "业务场景，例如 电商投放复盘 / 商品详情页优化 / 客服售后分析",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_ref",
        "新产品商品引用，如 product:{uuid}；只有本轮显式选择商品时传入",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "product_name",
        "商品名称；只有本轮显式选择商品时传入",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/抖音/京东/拼多多/小红书等",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "include_self_check",
        "是否包含自检题，默认 true",
        json!({"type": "boolean"}),
    ))
    .with_tag("deterministic")
    .with_tag("teaching")
}
