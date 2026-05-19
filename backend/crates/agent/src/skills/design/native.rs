use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    output::{design_native_output, design_summary, normalize_design_output},
    specs::{
        design_campaign_poster_spec, design_color_scheme_spec, design_detail_page_spec,
        design_main_image_spec,
    },
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, string_param,
};

macro_rules! native_design_skill {
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
                let mut output = design_native_output(&self.spec.id, &normalized);
                normalize_design_output(&self.spec.id, &params, &mut output);
                Ok(SkillOutcome::new(output).with_summary(design_summary(&self.spec.id)))
            }
        }
    };
}

native_design_skill!(
    DesignMainImage,
    design_main_image_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "简约现代".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "extra_requirements" => |p: &Value| optional_string_param(p, "extra_requirements"),
    }
);

native_design_skill!(
    DesignDetailPage,
    design_detail_page_spec,
    required [],
    defaults {
        "product_name" => |p: &Value| optional_string_param(p, "product_name"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "简约现代".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "extra_requirements" => |p: &Value| optional_string_param(p, "extra_requirements"),
    }
);

native_design_skill!(
    DesignColorScheme,
    design_color_scheme_spec,
    required ["brand_tone"],
    defaults {
        "brand_tone" => |p: &Value| string_param(p, "brand_tone").map(Some),
        "category" => |p: &Value| optional_string_param(p, "category"),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "extra_requirements" => |p: &Value| optional_string_param(p, "extra_requirements"),
    }
);

native_design_skill!(
    DesignCampaignPoster,
    design_campaign_poster_spec,
    required ["campaign_name"],
    defaults {
        "campaign_name" => |p: &Value| string_param(p, "campaign_name").map(Some),
        "discount_info" => |p: &Value| Ok(optional_string_param(p, "discount_info")?.unwrap_or_else(|| "全场5折起".to_owned())),
        "campaign_date" => |p: &Value| optional_string_param(p, "campaign_date"),
        "style" => |p: &Value| Ok(optional_string_param(p, "style")?.unwrap_or_else(|| "热闹促销".to_owned())),
        "platform" => |p: &Value| Ok(optional_string_param(p, "platform")?.unwrap_or_else(|| "淘宝".to_owned())),
        "product_id" => |p: &Value| optional_i64_param(p, "product_id"),
        "product_ref" => |p: &Value| optional_string_param(p, "product_ref"),
    }
);
