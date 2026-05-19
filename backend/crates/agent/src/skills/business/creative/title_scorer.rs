use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{ctr_grade, score_aida, score_triggers},
    normalize::normalize_creative_output,
    specs::creative_title_ctr_scorer_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillError, SkillOutcome, SkillResult, SkillSpec, object_params,
    optional_string_param,
};

pub struct CreativeTitleCtrScorer {
    spec: SkillSpec,
}

impl CreativeTitleCtrScorer {
    pub fn new() -> Self {
        Self {
            spec: creative_title_ctr_scorer_spec(),
        }
    }
}

impl Default for CreativeTitleCtrScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CreativeTitleCtrScorer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let title = optional_string_param(&params, "title")?
            .ok_or_else(|| SkillError::missing_parameter("title"))?;
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let audience =
            optional_string_param(&params, "target_audience")?.unwrap_or_else(|| "通用".to_owned());
        let aida = score_aida(&title);
        let trigger = score_triggers(&title);
        let title_len = title.chars().count() as i64;
        let (lo, hi) = match platform.as_str() {
            "淘宝" => (25, 32),
            "京东" => (25, 35),
            "拼多多" => (18, 25),
            "抖音" => (12, 20),
            _ => (20, 30),
        };
        let mid = (lo + hi) / 2;
        let struct_score = if (lo ..= hi).contains(&title_len) {
            25
        } else {
            (25 - (title_len - mid).abs()).max(5)
        };
        let aida_score = (aida.weighted * 40.0).round() as i64;
        let trigger_score = (trigger.normalized * 35.0).round() as i64;
        let total = aida_score + trigger_score + struct_score;
        let base_ctr: f64 = match platform.as_str() {
            "淘宝" => 3.2,
            "京东" => 2.8,
            "拼多多" => 4.5,
            "抖音" => 5.0,
            _ => 3.5,
        };
        let predicted_ctr = ((base_ctr + (total as f64 - 50.0) * 0.03) * 100.0).round() / 100.0;
        let mut suggestions = Vec::new();
        if aida.weighted < 0.5 {
            suggestions.push("加强AIDA缺失维度：注意力、兴趣、欲望或行动号召".to_owned());
        }
        if trigger.normalized < 0.3 {
            suggestions.push("增加情感触发词（紧迫感/稀缺性/社交证明）".to_owned());
        }
        if title_len < lo {
            suggestions.push(format!("标题偏短，可补充卖点至{lo}字以上"));
        }
        if suggestions.is_empty() {
            suggestions.push("标题CTR潜力良好，可微调紧迫感词语".to_owned());
        }

        let mut output = json!({
            "平台": platform,
            "标题": title,
            "目标受众": audience,
            "综合CTR评分": format!("{total}/100"),
            "总分": total,
            "评级": ctr_grade(total),
            "预测CTR": format!("{:.2}%（行业基准{}%）", predicted_ctr.clamp(0.5, 15.0), base_ctr),
            "分项评分": {
                "AIDA结构": {"得分": format!("{aida_score}/40"), "命中要素": aida.hits},
                "情感触发词密度": {"得分": format!("{trigger_score}/35"), "命中词": trigger.words},
                "标题结构": {"得分": format!("{struct_score}/25"), "字数": title_len, "推荐范围": format!("{lo}-{hi}字")}
            },
            "优化建议": suggestions
        });
        normalize_creative_output("creative_title_ctr_scorer", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("标题CTR预测评分已完成"))
    }
}
