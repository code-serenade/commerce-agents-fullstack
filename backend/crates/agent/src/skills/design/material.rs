use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    output::{material_spec_for, normalize_design_output},
    specs::design_material_spec_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, optional_i64_param,
    optional_string_vec_param, string_param,
};

pub struct DesignMaterialSpec {
    spec: SkillSpec,
}

impl DesignMaterialSpec {
    pub fn new() -> Self {
        Self {
            spec: design_material_spec_spec(),
        }
    }
}

impl Default for DesignMaterialSpec {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for DesignMaterialSpec {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        let usage = string_param(&params, "usage")?;
        let platforms = optional_string_vec_param(&params, "platforms")?
            .unwrap_or_else(|| vec!["淘宝".to_owned(), "京东".to_owned(), "抖音".to_owned()]);
        let product_id = optional_i64_param(&params, "product_id")?;
        let mut output = json!({
            "has_data": true,
            "执行模式": "native_design_spec",
            "数据来源": "用户输入 + 内置平台素材规格规则",
            "用途": usage,
            "目标平台": platforms,
            "商品ID": product_id,
            "平台规格": platforms.iter().map(|platform| material_spec_for(platform, &usage)).collect::<Vec<_>>(),
            "命名规范": "{platform}_{usage}_{product_or_campaign}_{size}_{version}",
            "交付清单": ["源文件", "平台上传图", "压缩备份", "文案备份", "验收截图"],
            "验收标准": ["画面主体清晰", "关键信息在安全区内", "文案无违规词", "导出文件符合平台体积限制", "移动端首屏可读"]
        });
        normalize_design_output("design_material_spec", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("素材规范已生成"))
    }
}
