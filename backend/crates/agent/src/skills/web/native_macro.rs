use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    normalize::{normalize_web_output, web_native_output, web_summary},
    specs::{
        web_keyword_research_spec, web_product_description_spec, web_seo_optimize_spec,
        web_store_design_spec, web_title_generator_spec,
    },
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

macro_rules! native_web_skill {
    ($type_name:ident, $spec_fn:ident, required [$($required:literal),*], defaults {$($key:literal => $value:expr),* $(,)?}) => {
        pub struct $type_name {
            spec: SkillSpec,
        }

        impl $type_name {
            pub fn new() -> Self {
                Self { spec: $spec_fn() }
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #[async_trait]
        impl Skill for $type_name {
            fn spec(&self) -> &SkillSpec {
                &self.spec
            }

            async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
                object_params(&params)?;
                $(let _ = string_param(&params, $required)?;)*
                let mut normalized = Map::new();
                $(normalized.insert($key.to_owned(), json!($value(&params)?));)*
                let mut output = web_native_output(&self.spec.id, &normalized);
                normalize_web_output(&self.spec.id, &params, &mut output);
                Ok(SkillOutcome::new(output).with_summary(web_summary(&self.spec.id)))
            }
        }
    };
}

native_web_skill!(
    WebSeoOptimize,
    web_seo_optimize_spec,
    required [],
    defaults {
        "target_keywords" => |p: &Value| Ok(optional_string_vec_param(p, "target_keywords")?.unwrap_or_else(|| vec!["电商".to_owned()])),
        "current_rank" => |p: &Value| Ok(optional_i64_param(p, "current_rank")?.unwrap_or(0)),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
    }
);
native_web_skill!(
    WebTitleGenerator,
    web_title_generator_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "keywords" => |p: &Value| Ok(optional_string_vec_param(p, "keywords")?.unwrap_or_default()),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "营销型".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
    }
);
native_web_skill!(
    WebStoreDesign,
    web_store_design_spec,
    required ["store_type"],
    defaults {
        "store_type" => |p: &Value| string_param(p, "store_type").map(Some),
        "category" => |p: &Value| Ok(optional_string_param(p, "category")?.unwrap_or_else(|| "通用".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "brand_tone" => |p: &Value| Ok(optional_string_param(p, "brand_tone")?.unwrap_or_else(|| "专业".to_owned())),
    }
);
native_web_skill!(
    WebKeywordResearch,
    web_keyword_research_spec,
    required [],
    defaults {
        "seed_keyword" => |p: &Value| optional_string_param(p, "seed_keyword"),
        "category" => |p: &Value| Ok(optional_string_param(p, "category")?.unwrap_or_else(|| "通用".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
    }
);
native_web_skill!(
    WebProductDescription,
    web_product_description_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "营销型".to_owned())),
    }
);
