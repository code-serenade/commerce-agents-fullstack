use async_trait::async_trait;
use serde_json::{Value, json};

use super::{normalize::normalize_creative_output, specs::creative_content_calendar_spec};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param,
};

pub struct CreativeContentCalendar {
    spec: SkillSpec,
}

impl CreativeContentCalendar {
    pub fn new() -> Self {
        Self {
            spec: creative_content_calendar_spec(),
        }
    }
}

impl Default for CreativeContentCalendar {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CreativeContentCalendar {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let period = optional_string_param(&params, "period")?.unwrap_or_else(|| "月".to_owned());
        let platforms = optional_string_vec_param(&params, "platforms")?
            .unwrap_or_else(|| vec!["抖音".to_owned(), "小红书".to_owned()]);
        let content_types =
            optional_string_vec_param(&params, "content_types")?.unwrap_or_else(|| {
                vec![
                    "种草测评".to_owned(),
                    "干货教程".to_owned(),
                    "生活日常".to_owned(),
                    "互动话题".to_owned(),
                ]
            });
        let weekly_posts = optional_i64_param(&params, "weekly_posts")?
            .unwrap_or(5)
            .max(1);
        let schedule = build_content_schedule(&platforms, &content_types, weekly_posts);
        let mut output = json!({
            "能力": "内容排期",
            "周期": period,
            "发布平台": platforms,
            "内容类型": content_types,
            "每周发布数量": weekly_posts,
            "排期": schedule,
            "选题原则": ["首条给痛点或结果", "中段做测评和对比", "末段承接直播/私域/客服问题", "每周复盘标题CTR和完播率"],
            "复盘时要看的数据": ["发布时间", "平台", "选题", "内容形式", "曝光", "点击", "完播", "加购", "评论问题"]
        });
        normalize_creative_output("creative_content_calendar", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("内容排期已生成"))
    }
}

fn build_content_schedule(
    platforms: &[String],
    content_types: &[String],
    weekly_posts: i64,
) -> Vec<Value> {
    let days = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
    let count = weekly_posts.clamp(1, 14) as usize;
    let default_platform = "抖音".to_owned();
    let default_type = "种草测评".to_owned();
    (0..count)
        .map(|index| {
            let platform = if platforms.is_empty() {
                &default_platform
            } else {
                &platforms[index % platforms.len()]
            };
            let content_type = if content_types.is_empty() {
                &default_type
            } else {
                &content_types[index % content_types.len()]
            };
            json!({
                "时间": days[index % days.len()],
                "平台": platform,
                "内容类型": content_type,
                "选题": format!("{}方向：围绕用户痛点、卖点证据和购买顾虑展开", content_type),
                "目标": if index % 3 == 0 { "拉新曝光" } else if index % 3 == 1 { "建立信任" } else { "促进转化" },
                "指标": ["曝光", "点击", "互动", "加购"]
            })
        })
        .collect()
}
