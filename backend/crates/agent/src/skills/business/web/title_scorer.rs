use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::{build_seo_suggestions, keyword_position_score, repeated_char_penalty, seo_grade},
    normalize::normalize_web_output,
    specs::web_title_seo_scorer_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillError, SkillOutcome, SkillResult, SkillSpec, object_params,
    optional_string_param, optional_string_vec_param,
};

pub struct WebTitleSeoScorer {
    spec: SkillSpec,
}

impl WebTitleSeoScorer {
    pub fn new() -> Self {
        Self {
            spec: web_title_seo_scorer_spec(),
        }
    }
}

impl Default for WebTitleSeoScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for WebTitleSeoScorer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let title = optional_string_param(&params, "title")?
            .ok_or_else(|| SkillError::missing_parameter("title"))?;
        let keywords = optional_string_vec_param(&params, "target_keywords")?.unwrap_or_default();
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let len = title.chars().count() as i64;
        let (lo, hi) = match platform.as_str() {
            "淘宝" => (28, 34),
            "京东" => (28, 36),
            "拼多多" => (20, 28),
            "抖音" => (15, 22),
            _ => (24, 34),
        };
        let mut deductions = Vec::new();
        let len_score = if (lo..=hi).contains(&len) {
            25
        } else if len < lo {
            let score = (25 - (lo - len) * 2).max(0);
            deductions.push(format!(
                "标题偏短({len}字 < 推荐{lo}字)，损失{}分",
                25 - score
            ));
            score
        } else {
            let score = (25 - (len - hi)).max(5);
            deductions.push(format!(
                "标题偏长({len}字 > 推荐{hi}字)，损失{}分",
                25 - score
            ));
            score
        };

        let primary = keywords.first().cloned().unwrap_or_default();
        let (position_score, pos_bonus) = if primary.is_empty() {
            (15, 0)
        } else {
            let bonus = keyword_position_score(&title, &primary);
            if bonus < 0 {
                deductions.push(format!("主关键词「{primary}」未在标题中出现，扣20分"));
            } else if bonus < 20 {
                deductions.push(format!("主关键词「{primary}」位置靠后，建议移至前30%"));
            }
            (
                (if title.contains(&primary) { 10 } else { 0 } + bonus).clamp(0, 30),
                bonus,
            )
        };

        let (kw_score, coverage_pct, missing) = if keywords.is_empty() {
            (15, 0.0, Vec::new())
        } else {
            let covered = keywords
                .iter()
                .filter(|kw| title.to_lowercase().contains(&kw.to_lowercase()))
                .count();
            let coverage = covered as f64 / keywords.len() as f64;
            let missing = keywords
                .iter()
                .filter(|kw| !title.to_lowercase().contains(&kw.to_lowercase()))
                .cloned()
                .collect::<Vec<_>>();
            if coverage < 0.6 {
                deductions.push(format!(
                    "关键词覆盖率{:.0}%，未包含: {}",
                    coverage * 100.0,
                    missing
                        .iter()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            ((coverage * 25.0).round() as i64, coverage, missing)
        };

        let special = title
            .chars()
            .filter(|c| "！？!?★☆▶◆●■□[]【】《》<>~～".contains(*c))
            .count() as i64;
        let repeat_penalty = repeated_char_penalty(&title);
        let readable_score = (20 - special * 2 - repeat_penalty).max(0);
        if special > 3 {
            deductions.push(format!("特殊符号过多({special}个)，影响可读性"));
        }
        let total = len_score + position_score + kw_score + readable_score;
        let suggestions = build_seo_suggestions(&primary, pos_bonus, &missing, len, lo, hi);

        let mut output = json!({
            "平台": platform,
            "标题": title,
            "综合SEO评分": format!("{total}/100"),
            "总分": total,
            "评级": seo_grade(total),
            "分项评分": {
                "字符长度": {"得分": len_score, "满分": 25, "实际字数": len, "推荐范围": format!("{lo}-{hi}字")},
                "主关键词位置": {"得分": position_score, "满分": 30, "关键词": primary, "位置分": pos_bonus},
                "关键词覆盖": {"得分": kw_score, "满分": 25, "覆盖率": format!("{:.0}%", coverage_pct * 100.0)},
                "可读性": {"得分": readable_score, "满分": 20, "特殊符号数": special}
            },
            "扣分项": if deductions.is_empty() { vec!["无明显扣分项".to_owned()] } else { deductions },
            "优化建议": suggestions
        });
        normalize_web_output("web_title_seo_scorer", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("标题SEO评分已完成"))
    }
}
