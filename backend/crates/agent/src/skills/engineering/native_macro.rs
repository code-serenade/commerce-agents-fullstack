use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    normalize::{engineering_native_output, engineering_summary, normalize_engineering_output},
    specs::{
        engineering_arch_review_spec, engineering_bug_analysis_spec,
        engineering_perf_optimize_spec, engineering_tech_selection_spec,
    },
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_f64_param,
    optional_i64_param, optional_string_param, optional_string_vec_param, string_param,
};

macro_rules! native_engineering_skill {
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
                let mut output = engineering_native_output(&self.spec.id, &normalized);
                normalize_engineering_output(&self.spec.id, &params, &mut output);
                Ok(SkillOutcome::new(output).with_summary(engineering_summary(&self.spec.id)))
            }
        }
    };
}

native_engineering_skill!(
    EngineeringArchReview,
    engineering_arch_review_spec,
    required ["system_type"],
    defaults {
        "system_type" => |p: &Value| string_param(p, "system_type").map(Some),
        "daily_orders" => |p: &Value| Ok(optional_i64_param(p, "daily_orders")?.unwrap_or(1000)),
        "tech_stack" => |p: &Value| Ok(optional_string_vec_param(p, "tech_stack")?.unwrap_or_default()),
        "pain_points" => |p: &Value| Ok(optional_string_vec_param(p, "pain_points")?.unwrap_or_default()),
        "current_metrics" => |p: &Value| Ok(p.get("current_metrics").cloned().unwrap_or_else(|| json!({}))),
    }
);
native_engineering_skill!(
    EngineeringBugAnalysis,
    engineering_bug_analysis_spec,
    required ["error_type"],
    defaults {
        "error_type" => |p: &Value| string_param(p, "error_type").map(Some),
        "error_message" => |p: &Value| optional_string_param(p, "error_message"),
        "frequency" => |p: &Value| Ok(optional_string_param(p, "frequency")?.unwrap_or_else(|| "偶发".to_owned())),
        "tech_stack" => |p: &Value| optional_string_param(p, "tech_stack"),
        "context" => |p: &Value| optional_string_param(p, "context"),
    }
);
native_engineering_skill!(
    EngineeringPerfOptimize,
    engineering_perf_optimize_spec,
    required ["bottleneck"],
    defaults {
        "bottleneck" => |p: &Value| string_param(p, "bottleneck").map(Some),
        "current_qps" => |p: &Value| Ok(optional_f64_param(p, "current_qps")?.unwrap_or(500.0)),
        "target_qps" => |p: &Value| Ok(optional_f64_param(p, "target_qps")?.unwrap_or(2000.0)),
        "tech_stack" => |p: &Value| optional_string_param(p, "tech_stack"),
        "metrics" => |p: &Value| Ok(p.get("metrics").cloned().unwrap_or_else(|| json!({}))),
    }
);
native_engineering_skill!(
    EngineeringTechSelection,
    engineering_tech_selection_spec,
    required ["domain"],
    defaults {
        "domain" => |p: &Value| string_param(p, "domain").map(Some),
        "requirements" => |p: &Value| Ok(optional_string_vec_param(p, "requirements")?.unwrap_or_default()),
        "constraints" => |p: &Value| Ok(optional_string_vec_param(p, "constraints")?.unwrap_or_default()),
        "team_size" => |p: &Value| Ok(optional_i64_param(p, "team_size")?.unwrap_or(5)),
        "current_stack" => |p: &Value| optional_string_param(p, "current_stack"),
    }
);
