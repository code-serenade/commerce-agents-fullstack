use async_trait::async_trait;
use serde_json::{Value, json};

use super::specs::engineering_system_check_spec;
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_bool_param,
    optional_string_param,
};

pub struct EngineeringSystemCheck {
    spec: SkillSpec,
}

impl EngineeringSystemCheck {
    pub fn new() -> Self {
        Self {
            spec: engineering_system_check_spec(),
        }
    }
}

impl Default for EngineeringSystemCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for EngineeringSystemCheck {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let test_live = optional_bool_param(&params, "test_live")?.unwrap_or(false);
        let platform = optional_string_param(&params, "platform")?;
        let status = if test_live {
            "pending_approval"
        } else {
            "deferred"
        };
        Ok(SkillOutcome::new(json!({
            "status": status,
            "approval_required": test_live,
            "deferred_reason": if test_live { "实际调用平台API健康检查属于外部平台动作，需要显式审批和适配器接入" } else { "平台连接配置读取尚未接入Rust skill runtime" },
            "skill": self.spec.id,
            "platform": platform,
            "planned_checks": ["凭证完整性", "连接配置", "最近同步状态", "错误码分布"],
            "no_side_effects": true
        })).with_summary("平台接口健康检查骨架已返回"))
    }
}
