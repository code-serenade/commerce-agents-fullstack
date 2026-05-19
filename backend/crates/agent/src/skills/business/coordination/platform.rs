use async_trait::async_trait;
use serde_json::{Value, json};

use super::specs::{
    coordination_external_agent_call_spec, platform_sync_products_spec,
    platform_update_inventory_spec, platform_update_price_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_f64_param,
    optional_i64_param, optional_string_param, string_param,
};

macro_rules! deferred_coordination_skill {
    ($type_name:ident, $spec_fn:ident, $status:literal, required [$($required:literal),*], defaults {$($key:literal => $value:expr),* $(,)?}) => {
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
                Ok(SkillOutcome::new(json!({
                    "status": $status,
                    "approval_required": $status == "pending_approval",
                    "deferred_reason": "协调/平台动作尚未接入Rust runtime；本实现不执行数据库、网络或平台写动作",
                    "skill": self.spec.id,
                    "normalized_inputs": normalized,
                    "no_side_effects": true
                })).with_summary("协调技能骨架已返回"))
            }
        }
    };
}

deferred_coordination_skill!(
    PlatformUpdatePrice,
    platform_update_price_spec,
    "pending_approval",
    required ["platform", "product_id"],
    defaults {
        "platform" => |p: &Value| string_param(p, "platform").map(Some),
        "product_id" => |p: &Value| string_param(p, "product_id").map(Some),
        "new_price" => |p: &Value| optional_f64_param(p, "new_price"),
        "sku_id" => |p: &Value| optional_string_param(p, "sku_id"),
    }
);
deferred_coordination_skill!(
    PlatformUpdateInventory,
    platform_update_inventory_spec,
    "pending_approval",
    required ["platform", "product_id"],
    defaults {
        "platform" => |p: &Value| string_param(p, "platform").map(Some),
        "product_id" => |p: &Value| string_param(p, "product_id").map(Some),
        "quantity" => |p: &Value| optional_i64_param(p, "quantity"),
        "sku_id" => |p: &Value| optional_string_param(p, "sku_id"),
    }
);
deferred_coordination_skill!(
    PlatformSyncProducts,
    platform_sync_products_spec,
    "pending_approval",
    required ["platform"],
    defaults {
        "platform" => |p: &Value| string_param(p, "platform").map(Some),
        "page" => |p: &Value| Ok(optional_i64_param(p, "page")?.unwrap_or(1)),
        "page_size" => |p: &Value| Ok(optional_i64_param(p, "page_size")?.unwrap_or(20)),
    }
);
deferred_coordination_skill!(
    CoordinationExternalAgentCall,
    coordination_external_agent_call_spec,
    "pending_approval",
    required ["endpoint", "description"],
    defaults {
        "endpoint" => |p: &Value| string_param(p, "endpoint").map(Some),
        "method" => |p: &Value| Ok(optional_string_param(p, "method")?.unwrap_or_else(|| "GET".to_owned())),
        "payload" => |p: &Value| Ok(p.get("payload").cloned().unwrap_or_else(|| json!({}))),
        "description" => |p: &Value| string_param(p, "description").map(Some),
        "extract_key" => |p: &Value| optional_string_param(p, "extract_key"),
    }
);
