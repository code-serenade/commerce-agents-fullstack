use async_trait::async_trait;
use serde_json::{Value, json};

use super::specs::coordination_task_orchestration_spec;
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params,
    optional_string_vec_param, string_param,
};

pub struct CoordinationTaskOrchestration {
    spec: SkillSpec,
}

impl CoordinationTaskOrchestration {
    pub fn new() -> Self {
        Self {
            spec: coordination_task_orchestration_spec(),
        }
    }
}

impl Default for CoordinationTaskOrchestration {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationTaskOrchestration {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let goal = string_param(&params, "goal")?;
        let requirements = optional_string_vec_param(&params, "requirements")?.unwrap_or_default();
        let tasks = orchestration_template(&goal);
        Ok(SkillOutcome::new(json!({
            "目标": goal,
            "需求": if requirements.is_empty() { vec!["参考标准流程".to_owned()] } else { requirements },
            "任务编排": tasks,
            "总工期估算": "约7天（含并行）",
            "关键路径": "任务1 -> 任务2 -> 并行执行 -> 收尾",
            "风险提示": ["多Agent协作需同步边界和验收标准", "设计/内容/技术任务建议设置缓冲"],
            "编排方式": "Rust deterministic template"
        }))
        .with_summary("任务编排骨架已生成"))
    }
}

fn orchestration_template(goal: &str) -> Vec<Value> {
    let launch = goal.contains("新品") || goal.contains("上新");
    let promo = goal.contains("大促") || goal.contains("活动") || goal.contains("618");
    let rows = if launch {
        vec![
            ("竞品分析与市场调研", "data", "2天", "无", "竞品报告"),
            (
                "定价策略与成本核算",
                "accounting",
                "1天",
                "任务1",
                "定价方案",
            ),
            ("主图与详情页设计", "design", "3天", "任务1", "设计稿"),
            ("商品文案与SEO标题", "web", "1天", "任务3", "优化文案"),
            ("种草内容与脚本", "creative", "2天", "任务3", "内容素材"),
            ("客服话术准备", "service", "1天", "任务4", "话术库"),
            ("上线推广与监控", "ops", "持续", "任务4,5,6", "上线运营"),
        ]
    } else if promo {
        vec![
            ("历史数据分析与目标设定", "data", "1天", "无", "数据报告"),
            ("活动策划与预算分配", "ops", "2天", "任务1", "活动方案"),
            (
                "财务预算与盈亏测算",
                "accounting",
                "1天",
                "任务2",
                "预算报告",
            ),
            ("活动海报与素材设计", "design", "2天", "任务2", "视觉素材"),
            ("直播脚本与种草内容", "creative", "1天", "任务2", "内容脚本"),
            ("客服应急话术与预案", "service", "1天", "任务2", "应急手册"),
            (
                "技术压测与系统保障",
                "engineering",
                "1天",
                "任务2",
                "压测报告",
            ),
        ]
    } else {
        vec![
            ("现状分析与数据调研", "data", "1天", "无", "分析报告"),
            ("策略与方案制定", "ops", "1天", "任务1", "执行方案"),
            ("财务可行性评估", "accounting", "1天", "任务2", "财务测算"),
            ("视觉设计", "design", "2天", "任务2", "设计稿"),
            ("内容与文案制作", "creative", "2天", "任务2", "内容素材"),
            ("执行落地与效果监控", "ops", "持续", "任务3,4,5", "执行报告"),
        ]
    };
    rows.into_iter()
        .enumerate()
        .map(|(idx, (task, agent, duration, dep, output))| {
            json!({
                "序号": idx + 1,
                "任务": task,
                "Agent": agent,
                "耗时": duration,
                "依赖": dep,
                "目标产出": output
            })
        })
        .collect()
}
