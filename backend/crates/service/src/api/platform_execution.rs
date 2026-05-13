use repo::table::platform_connections::{PlatformConnection, PlatformConnectionStatus};
use serde::Serialize;
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlatformExecutionAdapterRequest {
    pub skill_name: String,
    pub platform: String,
    pub adapter_status: String,
    pub execution_mode: String,
    pub idempotency_key: String,
    pub idempotency_key_fields: Vec<String>,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlatformExecutionAdapterOutcome {
    pub status: String,
    pub adapter_status: String,
    pub message: String,
    pub adapter_request: Option<PlatformExecutionAdapterRequest>,
    pub provider_result: Option<Value>,
    pub retryable: bool,
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

pub(super) fn platform_execution_adapter_request(
    skill_name: &str,
    params: &PlatformExecutionParams,
    adapter_status: &str,
) -> Option<PlatformExecutionAdapterRequest> {
    let spec = platform_execution_action_spec(skill_name)?;
    let platform = params.platform.clone()?;
    let args = platform_execution_args(params);
    Some(PlatformExecutionAdapterRequest {
        skill_name: spec.skill_name.to_string(),
        platform,
        adapter_status: adapter_status.to_string(),
        execution_mode: "adapter".to_string(),
        idempotency_key: platform_execution_idempotency_key(spec, params),
        idempotency_key_fields: spec
            .idempotency_key_fields
            .iter()
            .map(|field| field.to_string())
            .collect(),
        args,
    })
}

pub(super) fn platform_execution_adapter_outcome(
    skill_name: &str,
    params: &PlatformExecutionParams,
    adapter_status: &str,
    message: &str,
    retryable: bool,
) -> PlatformExecutionAdapterOutcome {
    PlatformExecutionAdapterOutcome {
        status: "failed".to_string(),
        adapter_status: adapter_status.to_string(),
        message: message.to_string(),
        adapter_request: platform_execution_adapter_request(skill_name, params, adapter_status),
        provider_result: None,
        retryable,
    }
}

pub(super) fn platform_execution_unimplemented_outcome(
    skill_name: &str,
    params: &PlatformExecutionParams,
    message: &str,
) -> PlatformExecutionAdapterOutcome {
    platform_execution_adapter_outcome(
        skill_name,
        params,
        "adapter_not_implemented",
        message,
        false,
    )
}

pub(super) fn platform_execution_dispatch_outcome(
    skill_name: &str,
    params: &PlatformExecutionParams,
    connection: Option<&PlatformConnection>,
) -> PlatformExecutionAdapterOutcome {
    match platform_execution_readiness_status(skill_name, params, connection) {
        PlatformExecutionReadiness::Blocked {
            adapter_status,
            message,
        } => {
            if adapter_status == "adapter_not_implemented" {
                platform_execution_unimplemented_outcome(skill_name, params, message)
            } else {
                platform_execution_adapter_outcome(
                    skill_name,
                    params,
                    adapter_status,
                    message,
                    false,
                )
            }
        }
        PlatformExecutionReadiness::Unsupported { message } => platform_execution_adapter_outcome(
            skill_name,
            params,
            "unsupported_action",
            message,
            false,
        ),
    }
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

pub(super) fn platform_execution_args(params: &PlatformExecutionParams) -> Value {
    json!({
        "platform": params.platform,
        "product_id": params.product_id,
        "sku_id": params.sku_id,
        "new_price": params.new_price,
        "quantity": params.quantity,
        "page": params.page,
        "page_size": params.page_size,
    })
}

fn platform_execution_idempotency_key(
    spec: &PlatformExecutionActionSpec,
    params: &PlatformExecutionParams,
) -> String {
    let parts = spec
        .idempotency_key_fields
        .iter()
        .map(|field| format!("{field}={}", idempotency_value(field, params)))
        .collect::<Vec<_>>()
        .join("|");
    format!("{}:{parts}", spec.skill_name)
}

fn idempotency_value(field: &str, params: &PlatformExecutionParams) -> String {
    match field {
        "platform" => params.platform.clone(),
        "product_id" => params.product_id.clone(),
        "sku_id" => params.sku_id.clone(),
        "new_price" => params.new_price.map(|value| value.to_string()),
        "quantity" => params.quantity.map(|value| value.to_string()),
        "page" => params.page.map(|value| value.to_string()),
        "page_size" => params.page_size.map(|value| value.to_string()),
        _ => None,
    }
    .unwrap_or_else(|| "null".to_string())
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
    use repo::table::platform_connections::PlatformSyncStatus;

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

    #[test]
    fn platform_execution_adapter_request_uses_registry_idempotency_fields() {
        let params = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            sku_id: Some("sku-9".to_string()),
            new_price: Some(129.5),
            ..Default::default()
        };

        let request = platform_execution_adapter_request(
            "platform_update_price",
            &params,
            "adapter_not_implemented",
        )
        .expect("supported platform action should build adapter request");

        assert_eq!(request.skill_name, "platform_update_price");
        assert_eq!(request.platform, "douyin");
        assert_eq!(request.adapter_status, "adapter_not_implemented");
        assert_eq!(
            request.idempotency_key,
            "platform_update_price:platform=douyin|product_id=prod-1|sku_id=sku-9|new_price=129.5"
        );
        assert_eq!(
            request.idempotency_key_fields,
            vec!["platform", "product_id", "sku_id", "new_price"]
        );
        assert_eq!(
            request.args.get("product_id").and_then(Value::as_str),
            Some("prod-1")
        );
    }

    #[test]
    fn platform_execution_adapter_request_requires_supported_action_and_platform() {
        let params = PlatformExecutionParams {
            product_id: Some("prod-1".to_string()),
            ..Default::default()
        };

        assert!(
            platform_execution_adapter_request(
                "platform_update_price",
                &params,
                "validation_error"
            )
            .is_none()
        );
        assert!(
            platform_execution_adapter_request(
                "platform_publish_campaign",
                &PlatformExecutionParams {
                    platform: Some("douyin".to_string()),
                    ..Default::default()
                },
                "unsupported_action"
            )
            .is_none()
        );
    }

    #[test]
    fn platform_execution_unimplemented_outcome_is_not_retryable() {
        let params = PlatformExecutionParams {
            platform: Some("jd".to_string()),
            product_id: Some("prod-1".to_string()),
            quantity: Some(12),
            ..Default::default()
        };

        let outcome = platform_execution_unimplemented_outcome(
            "platform_update_inventory",
            &params,
            "Platform execution adapter is not implemented yet for inventory update.",
        );

        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.adapter_status, "adapter_not_implemented");
        assert!(!outcome.retryable);
        assert!(outcome.provider_result.is_none());
        assert_eq!(
            outcome
                .adapter_request
                .as_ref()
                .map(|request| request.idempotency_key.as_str()),
            Some("platform_update_inventory:platform=jd|product_id=prod-1|sku_id=null|quantity=12")
        );
    }

    #[test]
    fn platform_execution_dispatch_outcome_reports_missing_connection() {
        let params = PlatformExecutionParams {
            platform: Some("taobao".to_string()),
            product_id: Some("prod-1".to_string()),
            new_price: Some(99.0),
            ..Default::default()
        };

        let outcome = platform_execution_dispatch_outcome("platform_update_price", &params, None);

        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.adapter_status, "connection_missing");
        assert!(!outcome.retryable);
        assert_eq!(
            outcome
                .adapter_request
                .as_ref()
                .map(|request| request.idempotency_key.as_str()),
            Some(
                "platform_update_price:platform=taobao|product_id=prod-1|sku_id=null|new_price=99"
            )
        );
    }

    #[test]
    fn platform_execution_dispatch_outcome_keeps_unimplemented_boundary() {
        let params = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            sku_id: Some("sku-9".to_string()),
            new_price: Some(129.5),
            ..Default::default()
        };
        let connection = test_platform_connection("douyin");

        let outcome = platform_execution_dispatch_outcome(
            "platform_update_price",
            &params,
            Some(&connection),
        );

        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.adapter_status, "adapter_not_implemented");
        assert_eq!(
            outcome.message,
            "Platform execution adapter is not implemented yet for price update."
        );
        assert!(outcome.provider_result.is_none());
        assert!(!outcome.retryable);
    }

    #[test]
    fn platform_execution_dispatch_outcome_reports_unsupported_action() {
        let params = PlatformExecutionParams {
            platform: Some("douyin".to_string()),
            product_id: Some("prod-1".to_string()),
            ..Default::default()
        };
        let connection = test_platform_connection("douyin");

        let outcome = platform_execution_dispatch_outcome(
            "platform_publish_campaign",
            &params,
            Some(&connection),
        );

        assert_eq!(outcome.status, "failed");
        assert_eq!(outcome.adapter_status, "unsupported_action");
        assert!(outcome.adapter_request.is_none());
        assert!(!outcome.retryable);
    }

    fn test_platform_connection(platform: &str) -> PlatformConnection {
        PlatformConnection {
            id: uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
            user_id: uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            platform: platform.to_string(),
            app_key: Some("ak".to_string()),
            app_secret_enc: Some("secret".to_string()),
            access_token_enc: Some("token".to_string()),
            refresh_token_enc: Some("refresh".to_string()),
            token_expires_at: None,
            shop_id: Some("shop-1".to_string()),
            shop_name: Some("Demo Shop".to_string()),
            enabled: true,
            status: PlatformConnectionStatus::Configured,
            last_tested_at: None,
            test_result: json!({}),
            last_synced_at: None,
            sync_status: PlatformSyncStatus::Idle,
            sync_message: None,
            extra: json!({}),
            created_at: time::OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap(),
            updated_at: time::OffsetDateTime::from_unix_timestamp(1_778_544_000).unwrap(),
        }
    }
}
