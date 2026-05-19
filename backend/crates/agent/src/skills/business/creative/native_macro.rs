use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    normalize::normalize_creative_output,
    outcomes::native_creative_outcome,
    specs::{
        creative_ip_branding_spec, creative_live_script_spec, creative_product_article_spec,
        creative_seeding_copy_spec, creative_trend_catch_spec, creative_video_script_spec,
    },
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_f64_param,
    optional_i64_param, optional_string_param, optional_string_vec_param, string_param,
};

macro_rules! native_creative_skill {
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
                let mut normalized = serde_json::Map::new();
                $(normalized.insert($key.to_owned(), json!($value(&params)?));)*
                let mut outcome = native_creative_outcome(&self.spec.id, normalized);
                normalize_creative_output(&self.spec.id, &params, &mut outcome.output);
                Ok(outcome)
            }
        }
    };
}

native_creative_skill!(
    CreativeVideoScript,
    creative_video_script_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "video_type" => |p: &Value| Ok(optional_string_param(p, "video_type")?.unwrap_or_else(|| "种草".to_owned())),
        "duration" => |p: &Value| Ok(optional_i64_param(p, "duration")?.unwrap_or(30)),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "抖音".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
    }
);
native_creative_skill!(
    CreativeSeedingCopy,
    creative_seeding_copy_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "selling_points" => |p: &Value| Ok(optional_string_vec_param(p, "selling_points")?.unwrap_or_default()),
        "target_audience" => |p: &Value| Ok(optional_string_param(p, "target_audience")?.unwrap_or_else(|| "年轻女性".to_owned())),
        "tone" => |p: &Value| Ok(optional_string_param(p, "tone")?.unwrap_or_else(|| "闺蜜分享".to_owned())),
    }
);
native_creative_skill!(
    CreativeIpBranding,
    creative_ip_branding_spec,
    required ["brand_name", "category"],
    defaults {
        "brand_name" => |p: &Value| string_param(p, "brand_name").map(Some),
        "category" => |p: &Value| string_param(p, "category").map(Some),
        "target_audience" => |p: &Value| Ok(optional_string_param(p, "target_audience")?.unwrap_or_else(|| "18-35岁年轻消费者".to_owned())),
        "differentiator" => |p: &Value| Ok(optional_string_param(p, "differentiator")?.unwrap_or_else(|| "专业+有温度".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "小红书/抖音".to_owned())),
    }
);
native_creative_skill!(
    CreativeTrendCatch,
    creative_trend_catch_spec,
    required ["category"],
    defaults {
        "category" => |p: &Value| string_param(p, "category").map(Some),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "抖音".to_owned())),
    }
);
native_creative_skill!(
    CreativeLiveScript,
    creative_live_script_spec,
    required [],
    defaults {
        "duration_hours" => |p: &Value| Ok(optional_f64_param(p, "duration_hours")?.unwrap_or(3.0)),
        "product_count" => |p: &Value| Ok(optional_i64_param(p, "product_count")?.unwrap_or(6)),
        "product_refs" => |p: &Value| Ok(optional_string_vec_param(p, "product_refs")?.unwrap_or_default()),
        "live_type" => |p: &Value| Ok(optional_string_param(p, "live_type")?.unwrap_or_else(|| "日常带货".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "抖音".to_owned())),
    }
);
native_creative_skill!(
    CreativeProductArticle,
    creative_product_article_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "article_type" => |p: &Value| Ok(optional_string_param(p, "article_type")?.unwrap_or_else(|| "测评软文".to_owned())),
        "word_count" => |p: &Value| Ok(optional_i64_param(p, "word_count")?.unwrap_or(800)),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "微信公众号".to_owned())),
    }
);
