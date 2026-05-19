use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{conversion_actions, conversion_findings},
    normalize::normalize_web_output,
    specs::web_page_conversion_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, optional_f64_param, string_param,
};

pub struct WebPageConversion {
    spec: SkillSpec,
}

impl WebPageConversion {
    pub fn new() -> Self {
        Self {
            spec: web_page_conversion_spec(),
        }
    }
}

impl Default for WebPageConversion {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for WebPageConversion {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        let page_type = string_param(&params, "page_type")?;
        let conv = optional_f64_param(&params, "current_conversion")?.unwrap_or(3.0);
        let bounce = optional_f64_param(&params, "bounce_rate")?.unwrap_or(55.0);
        let stay = optional_f64_param(&params, "avg_stay_time")?.unwrap_or(45.0);
        let benchmark = match page_type.as_str() {
            "首页" => 5.0,
            "商品详情" => 4.0,
            "购物车" => 65.0,
            "结算" => 80.0,
            "列表页" => 8.0,
            _ => 5.0,
        };
        let gap = conv - benchmark;
        let mut output = json!({
            "has_data": true,
            "执行模式": "native_conversion_audit",
            "数据来源": "用户输入 + 页面转化基准规则",
            "页面类型": page_type,
            "当前数据": {"转化率": format!("{conv}%"), "跳出率": format!("{bounce}%"), "停留时间": format!("{stay}秒")},
            "行业基准转化率": format!("{benchmark}%"),
            "转化差距": format!("{gap:+.2}%"),
            "初步判断": if conv >= benchmark { "当前转化率达到或超过基准，优先做AB测试微调" } else { "当前转化率低于基准，优先检查首屏信息、信任背书、CTA和价格呈现" },
            "瓶颈诊断": conversion_findings(conv, bounce, stay, benchmark),
            "优化动作": conversion_actions(&page_type, conv, bounce, stay, benchmark),
            "AB测试清单": [
                {"实验": "首屏卖点顺序", "变量": "价格利益点 vs 使用场景", "指标": "点击/加购"},
                {"实验": "CTA文案", "变量": "立即购买 vs 领取优惠", "指标": "按钮点击率"},
                {"实验": "信任背书位置", "变量": "首屏展示 vs 参数后展示", "指标": "停留和转化"}
            ],
            "后续数据需求": ["近30天PV/UV", "点击热区", "加购率", "支付转化率", "渠道来源"]
        });
        normalize_web_output("web_page_conversion", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("页面转化优化方案已生成"))
    }
}
