use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    chain_helpers::{
        normalize_available_skills, normalize_chain_action, normalize_chain_role,
        skill_chain_prerequisite_gaps, skill_chain_prerequisite_notes, skill_chain_step,
    },
    templates::skill_chain_template,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec,
    coordination::specs::coordination_skill_chain_planner_spec, object_params,
    optional_string_param, optional_string_vec_param, string_param,
};

pub struct CoordinationSkillChainPlanner {
    spec: SkillSpec,
}

impl CoordinationSkillChainPlanner {
    pub fn new() -> Self {
        Self {
            spec: coordination_skill_chain_planner_spec(),
        }
    }
}

impl Default for CoordinationSkillChainPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationSkillChainPlanner {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let goal = string_param(&params, "goal")?;
        let role = normalize_chain_role(
            optional_string_param(&params, "role")?.unwrap_or_else(|| "ops".to_owned()),
        );
        let action = normalize_chain_action(
            optional_string_param(&params, "action")?.unwrap_or_else(|| "analyze".to_owned()),
        );
        let available = normalize_available_skills(
            optional_string_vec_param(&params, "skills_available")?.unwrap_or_default(),
        );
        let chain = skill_chain_template(&goal, &role, &action);
        let detailed_steps = chain
            .iter()
            .enumerate()
            .map(|(index, (skill, reason))| skill_chain_step(index + 1, skill, reason, &available))
            .collect::<Vec<_>>();
        let prerequisite_notes = skill_chain_prerequisite_notes(&chain);
        let prerequisite_gaps = skill_chain_prerequisite_gaps(&chain);
        let unavailable = detailed_steps
            .iter()
            .filter(|step| step.get("可用").and_then(Value::as_bool) == Some(false))
            .filter_map(|step| {
                step.get("技能")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        let approval_steps = detailed_steps
            .iter()
            .filter(|step| step.get("执行模式").and_then(Value::as_str) == Some("pending_approval"))
            .filter_map(|step| {
                step.get("技能")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        let adapter_steps = detailed_steps
            .iter()
            .filter(|step| step.get("执行模式").and_then(Value::as_str) == Some("adapter_required"))
            .filter_map(|step| {
                step.get("技能")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();
        let unavailable_count = unavailable.len();
        let has_adapter_steps = !adapter_steps.is_empty();
        let requires_approval = !approval_steps.is_empty();
        let summary = format!(
            "技能链规划已生成：{}步，{}个adapter提示，{}个审批提示",
            detailed_steps.len(),
            adapter_steps.len(),
            approval_steps.len()
        );
        Ok(SkillOutcome::new(json!({
            "status": "success",
            "任务目标": goal,
            "角色": role,
            "动作": action,
            "推荐执行链": detailed_steps,
            "前置依赖说明": prerequisite_notes,
            "前置依赖缺口": prerequisite_gaps,
            "不可用技能": unavailable,
            "审批提示": approval_steps,
            "adapter提示": adapter_steps,
            "可用技能过滤": !available.is_empty(),
            "执行建议": "先加载真实数据或明确缺数据状态，再计算分析，最后生成内容、沉淀任务或进入审批；不得把adapter_required步骤展示成已读取平台实时数据。",
            "总步骤数": chain.len(),
            "规划来源": "Rust deterministic commerce template"
        }))
        .with_summary(summary)
        .with_metadata("skill_chain_step_count", json!(chain.len()))
        .with_metadata("skill_chain_has_adapter_steps", json!(has_adapter_steps))
        .with_metadata("skill_chain_requires_approval", json!(requires_approval))
        .with_metadata("skill_chain_unavailable_count", json!(unavailable_count)))
    }
}
