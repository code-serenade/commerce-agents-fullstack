use repo::table::platform_connections::{PlatformConnection, PlatformConnectionStatus};
use serde_json::{Value, json};

#[derive(Debug, Clone, Default)]
pub(super) struct PlatformExecutionParams {
    pub platform: Option<String>,
    pub product_id: Option<String>,
    pub sku_id: Option<String>,
    pub new_price: Option<f64>,
    pub quantity: Option<i64>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PlatformExecutionReadiness<'a> {
    Blocked {
        adapter_status: &'a str,
        message: &'a str,
    },
    Unsupported {
        message: &'a str,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PlatformExecutionActionSpec {
    pub skill_name: &'static str,
    pub display: &'static str,
    pub description: &'static str,
    pub adapter_message: &'static str,
    pub required_params: &'static [&'static str],
    pub optional_params: &'static [&'static str],
    pub idempotency_key_fields: &'static [&'static str],
}

const PLATFORM_EXECUTION_ACTION_SPECS: &[PlatformExecutionActionSpec] = &[
    PlatformExecutionActionSpec {
        skill_name: "platform_update_price",
        display: "更新商品价格",
        description: "审批后更新平台商品或 SKU 价格。",
        adapter_message: "Platform execution adapter is not implemented yet for price update.",
        required_params: &["platform", "product_id", "new_price"],
        optional_params: &["sku_id"],
        idempotency_key_fields: &["platform", "product_id", "sku_id", "new_price"],
    },
    PlatformExecutionActionSpec {
        skill_name: "platform_update_inventory",
        display: "更新商品库存",
        description: "审批后更新平台商品或 SKU 库存。",
        adapter_message: "Platform execution adapter is not implemented yet for inventory update.",
        required_params: &["platform", "product_id", "quantity"],
        optional_params: &["sku_id"],
        idempotency_key_fields: &["platform", "product_id", "sku_id", "quantity"],
    },
    PlatformExecutionActionSpec {
        skill_name: "platform_sync_products",
        display: "同步平台商品",
        description: "审批后从平台同步商品资料。",
        adapter_message: "Platform execution adapter is not implemented yet for product sync.",
        required_params: &["platform"],
        optional_params: &["page", "page_size"],
        idempotency_key_fields: &["platform", "page", "page_size"],
    },
];

pub(super) fn platform_execution_action_specs() -> &'static [PlatformExecutionActionSpec] {
    PLATFORM_EXECUTION_ACTION_SPECS
}

pub(super) fn platform_execution_action_spec(
    skill_name: &str,
) -> Option<&'static PlatformExecutionActionSpec> {
    PLATFORM_EXECUTION_ACTION_SPECS
        .iter()
        .find(|spec| spec.skill_name == skill_name)
}

pub(super) fn platform_execution_params(args: &Value) -> PlatformExecutionParams {
    PlatformExecutionParams {
        platform: value_string(
            args,
            &[
                "platform",
                "/platform",
                "/normalized_inputs/platform",
                "/normalizedInputs/platform",
                "/output/normalized_inputs/platform",
                "/output/normalizedInputs/platform",
            ],
        )
        .map(|value| value.to_ascii_lowercase()),
        product_id: value_string(
            args,
            &[
                "product_id",
                "productId",
                "/product_id",
                "/productId",
                "/normalized_inputs/product_id",
                "/normalizedInputs/productId",
                "/output/normalized_inputs/product_id",
                "/output/normalizedInputs/productId",
            ],
        ),
        sku_id: value_string(
            args,
            &[
                "sku_id",
                "skuId",
                "/sku_id",
                "/skuId",
                "/normalized_inputs/sku_id",
                "/normalizedInputs/skuId",
                "/output/normalized_inputs/sku_id",
                "/output/normalizedInputs/skuId",
            ],
        ),
        new_price: value_f64(
            args,
            &[
                "new_price",
                "newPrice",
                "/new_price",
                "/newPrice",
                "/normalized_inputs/new_price",
                "/normalizedInputs/newPrice",
                "/output/normalized_inputs/new_price",
                "/output/normalizedInputs/newPrice",
            ],
        ),
        quantity: value_i64(
            args,
            &[
                "quantity",
                "/quantity",
                "/normalized_inputs/quantity",
                "/normalizedInputs/quantity",
                "/output/normalized_inputs/quantity",
                "/output/normalizedInputs/quantity",
            ],
        ),
        page: value_i64(
            args,
            &[
                "page",
                "/page",
                "/normalized_inputs/page",
                "/normalizedInputs/page",
                "/output/normalized_inputs/page",
                "/output/normalizedInputs/page",
            ],
        ),
        page_size: value_i64(
            args,
            &[
                "page_size",
                "pageSize",
                "/page_size",
                "/pageSize",
                "/normalized_inputs/page_size",
                "/normalizedInputs/pageSize",
                "/output/normalized_inputs/page_size",
                "/output/normalizedInputs/pageSize",
            ],
        ),
    }
}

pub(super) fn platform_execution_readiness_status<'a>(
    skill_name: &'a str,
    params: &'a PlatformExecutionParams,
    connection: Option<&PlatformConnection>,
) -> PlatformExecutionReadiness<'a> {
    if !is_platform_execution_skill(skill_name) {
        return PlatformExecutionReadiness::Unsupported {
            message: "Tool approval is not a platform execution action.",
        };
    }

    if params.platform.is_none() {
        return PlatformExecutionReadiness::Blocked {
            adapter_status: "validation_error",
            message: "Missing platform in tool approval arguments.",
        };
    }

    let Some(validation_error) = platform_execution_validation_error(skill_name, params) else {
        return PlatformExecutionReadiness::Unsupported {
            message: "Platform execution action is not supported by the current service adapter.",
        };
    };

    if let Some(message) = validation_error {
        return PlatformExecutionReadiness::Blocked {
            adapter_status: "validation_error",
            message,
        };
    }

    let Some(connection) = connection else {
        return PlatformExecutionReadiness::Blocked {
            adapter_status: "connection_missing",
            message: "No enabled platform connection was found for this user and platform.",
        };
    };

    if !connection.enabled || connection.status == PlatformConnectionStatus::Disabled {
        return PlatformExecutionReadiness::Blocked {
            adapter_status: "connection_disabled",
            message: "Platform connection exists but is disabled.",
        };
    }

    PlatformExecutionReadiness::Blocked {
        adapter_status: "adapter_not_implemented",
        message: platform_adapter_not_implemented_message(skill_name),
    }
}

pub(super) fn platform_execution_validation_error(
    skill_name: &str,
    params: &PlatformExecutionParams,
) -> Option<Option<&'static str>> {
    let spec = platform_execution_action_spec(skill_name)?;
    let validation_error = match spec.skill_name {
        "platform_update_price" => {
            if params.product_id.is_none() || params.new_price.is_none() {
                Some("Missing product_id or new_price for platform price update.")
            } else {
                None
            }
        }
        "platform_update_inventory" => {
            if params.product_id.is_none() || params.quantity.is_none() {
                Some("Missing product_id or quantity for platform inventory update.")
            } else {
                None
            }
        }
        "platform_sync_products" => None,
        _ => return None,
    };
    Some(validation_error)
}

pub(super) fn platform_adapter_not_implemented_message(skill_name: &str) -> &'static str {
    platform_execution_action_spec(skill_name)
        .map(|spec| spec.adapter_message)
        .unwrap_or("Platform execution adapter is not implemented yet for this action.")
}

pub(super) fn is_platform_execution_skill(skill_name: &str) -> bool {
    skill_name.starts_with("platform_")
}

pub(super) fn supported_platform_action(skill_name: &str) -> bool {
    platform_execution_action_spec(skill_name).is_some()
}

pub(super) fn platform_execution_details(params: &PlatformExecutionParams) -> Value {
    json!({
        "productId": params.product_id,
        "skuId": params.sku_id,
        "newPrice": params.new_price,
        "quantity": params.quantity,
        "page": params.page,
        "pageSize": params.page_size,
    })
}

fn value_string(value: &Value, paths: &[&str]) -> Option<String> {
    value_at_paths(value, paths).and_then(|value| match value {
        Value::String(text) => normalize_string(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

fn value_f64(value: &Value, paths: &[&str]) -> Option<f64> {
    value_at_paths(value, paths).and_then(|value| match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    })
}

fn value_i64(value: &Value, paths: &[&str]) -> Option<i64> {
    value_at_paths(value, paths).and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn value_at_paths<'a>(value: &'a Value, paths: &[&str]) -> Option<&'a Value> {
    for path in paths {
        let current = if path.starts_with('/') {
            value.pointer(path)
        } else {
            value.get(*path)
        };
        if let Some(current) = current
            && !current.is_null()
        {
            return Some(current);
        }
    }
    None
}

fn normalize_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_execution_action_registry_contains_governed_actions() {
        let specs = platform_execution_action_specs();

        assert_eq!(specs.len(), 3);
        assert!(supported_platform_action("platform_update_price"));
        assert!(supported_platform_action("platform_update_inventory"));
        assert!(supported_platform_action("platform_sync_products"));
        assert!(!supported_platform_action("platform_publish_campaign"));
    }

    #[test]
    fn platform_execution_action_registry_drives_adapter_messages() {
        assert_eq!(
            platform_adapter_not_implemented_message("platform_update_price"),
            "Platform execution adapter is not implemented yet for price update."
        );
        assert_eq!(
            platform_adapter_not_implemented_message("platform_publish_campaign"),
            "Platform execution adapter is not implemented yet for this action."
        );
    }

    #[test]
    fn platform_execution_action_registry_drives_validation() {
        let missing_price = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            ..Default::default()
        };
        assert_eq!(
            platform_execution_validation_error("platform_update_price", &missing_price),
            Some(Some(
                "Missing product_id or new_price for platform price update."
            ))
        );

        let unknown = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            ..Default::default()
        };
        assert_eq!(
            platform_execution_validation_error("platform_publish_campaign", &unknown),
            None
        );
    }
}
