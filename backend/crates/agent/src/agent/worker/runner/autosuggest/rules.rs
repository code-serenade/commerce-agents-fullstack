use super::config::{AutosuggestRule, AutosuggestRuleMode, WorkerAutosuggestPolicy};
use crate::agent::worker::{
    Task,
    runner::{
        inference::{
            commerce, context, data, domain, douyin_ads, inventory, memory_task, metrics, numbers,
            search,
        },
        types::ParsedSkillRequest,
    },
};

pub(super) fn configured_rule_requests(
    policy: &WorkerAutosuggestPolicy,
    role_runtime: &str,
    text: &str,
    task: &Task,
    route_mode: &str,
) -> Vec<ParsedSkillRequest> {
    let mut requests = Vec::new();
    let mut fallbacks = Vec::new();
    for rule in &policy.rules {
        if !rule_matches(rule, role_runtime, text, &requests) {
            continue;
        }

        let request = ParsedSkillRequest {
            skill_id: rule.skill_id.clone(),
            input: render_template_value(&rule.input, text, task),
            reason: rule
                .reason
                .clone()
                .or_else(|| Some(format!("autosuggest:{}", rule.id))),
        };

        if rule.fallback {
            fallbacks.push(request);
            continue;
        }

        requests.push(request);

        if route_mode != "requests" || rule.mode == AutosuggestRuleMode::First {
            break;
        }
    }
    if requests.is_empty() {
        requests.extend(fallbacks);
    }
    requests
}

pub(super) fn contains_any(text: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| contains_keyword(text, needle))
}

fn rule_matches(
    rule: &AutosuggestRule,
    role_runtime: &str,
    text: &str,
    current_requests: &[ParsedSkillRequest],
) -> bool {
    (rule.roles.is_empty() || rule.roles.iter().any(|role| role == role_runtime))
        && (rule.r#match.any_keywords.is_empty() || contains_any(text, &rule.r#match.any_keywords))
        && rule
            .r#match
            .all_keywords
            .iter()
            .all(|keyword| contains_keyword(text, keyword))
        && !contains_any(text, &rule.r#match.none_keywords)
        && (rule.r#match.requires_any_skill_ids.is_empty()
            || current_requests.iter().any(|request| {
                rule.r#match
                    .requires_any_skill_ids
                    .iter()
                    .any(|skill_id| skill_id == &request.skill_id)
            }))
        && !current_requests.iter().any(|request| {
            rule.r#match
                .requires_no_skill_ids
                .iter()
                .any(|skill_id| skill_id == &request.skill_id)
        })
}

pub(super) fn render_template_value(
    value: &serde_json::Value,
    text: &str,
    task: &Task,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(template) => render_template_scalar(template, text, task),
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|item| render_template_value(item, text, task))
                .collect(),
        ),
        serde_json::Value::Object(object) => serde_json::Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), render_template_value(value, text, task)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn render_template_scalar(template: &str, text: &str, task: &Task) -> serde_json::Value {
    if let Some(value) = render_exact_template_value(template, text, task) {
        return value;
    }

    let rendered = render_template_string(template, text, task);
    if template.trim().starts_with("{{")
        && template.trim().ends_with("}}")
        && let Ok(value) = rendered.parse::<f64>()
        && value.is_finite()
    {
        return serde_json::json!(value);
    }
    serde_json::Value::String(rendered)
}

fn render_exact_template_value(
    template: &str,
    text: &str,
    task: &Task,
) -> Option<serde_json::Value> {
    match template.trim() {
        "{{price_points}}" => Some(serde_json::Value::Array(
            commerce::infer_price_quantity_points(text),
        )),
        "{{competitor_prices}}" => Some(serde_json::json!(commerce::infer_competitor_prices(text))),
        "{{pricing_strategy_competitor_prices}}" => {
            let cost = commerce::infer_cost(text, 1.0);
            Some(serde_json::json!(
                commerce::infer_pricing_strategy_competitor_prices(text, cost)
            ))
        }
        "{{douyin_ads_budget}}" => Some(optional_number(douyin_ads::infer_budget(text))),
        "{{douyin_ads_target_roas}}" => Some(optional_number(douyin_ads::infer_target_roas(text))),
        "{{douyin_ads_spend}}" => Some(optional_number(douyin_ads::infer_spend(text))),
        "{{douyin_ads_gmv}}" => Some(optional_number(douyin_ads::infer_gmv(text))),
        "{{douyin_ads_current_roas}}" => {
            Some(optional_number(douyin_ads::infer_current_roas(text)))
        }
        "{{douyin_ads_impressions}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text,
            "impressions",
        ))),
        "{{douyin_ads_clicks}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text, "clicks",
        ))),
        "{{douyin_ads_live_room_entries}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text,
            "live_room_entries",
        ))),
        "{{douyin_ads_product_clicks}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text,
            "product_clicks",
        ))),
        "{{douyin_ads_orders}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text, "orders",
        ))),
        "{{douyin_ads_payments}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text, "payments",
        ))),
        "{{douyin_ads_ad_spend}}" => Some(optional_number(douyin_ads::infer_live_metric(
            text, "ad_spend",
        ))),
        "{{douyin_ads_live_gmv}}" => {
            Some(optional_number(douyin_ads::infer_live_metric(text, "gmv")))
        }
        "{{douyin_ads_goal}}" => Some(serde_json::json!(douyin_ads::infer_goal(text, task))),
        "{{douyin_ads_days}}" => Some(serde_json::json!(douyin_ads::infer_days(text))),
        "{{inventory_skus}}" => Some(serde_json::json!(inventory::infer_inventory_skus(text))),
        "{{inventory_ordering_cost}}" => {
            Some(optional_number(inventory::infer_ordering_cost(text)))
        }
        "{{inventory_holding_rate}}" => Some(optional_number(inventory::infer_holding_rate(text))),
        "{{inventory_service_level}}" => {
            Some(optional_string(inventory::infer_service_level(text)))
        }
        "{{ad_fatigue_channels}}" => Some(serde_json::json!(inventory::infer_ad_fatigue_channels(
            text
        ))),
        "{{ad_fatigue_total_budget}}" => Some(optional_number(inventory::infer_total_budget(text))),
        "{{abc_xyz_skus}}" => Some(serde_json::json!(inventory::infer_abc_xyz_skus(text))),
        "{{abc_xyz_top_n}}" => Some(optional_integer(inventory::infer_top_n(text))),
        "{{data_values}}" => Some(serde_json::json!(data::infer_values(text))),
        "{{channel_roi_rows}}" => Some(serde_json::json!(data::infer_channel_roi_rows(text))),
        "{{refund_items}}" => Some(serde_json::json!(data::infer_refund_items(text))),
        "{{multi_periods}}" => Some(serde_json::json!(data::infer_multi_periods(text))),
        "{{data_ltv_cac}}" => Some(optional_number(data::infer_metric_number(text, "cac"))),
        "{{data_ltv_avg_order_value}}" => Some(optional_number(data::infer_metric_number(
            text,
            "avg_order_value",
        ))),
        "{{data_ltv_purchase_frequency}}" => Some(optional_number(data::infer_metric_number(
            text,
            "purchase_frequency",
        ))),
        "{{data_ltv_gross_margin}}" => Some(optional_number(data::infer_metric_number(
            text,
            "gross_margin",
        ))),
        "{{data_ltv_monthly_retention}}" => Some(optional_number(data::infer_metric_number(
            text,
            "monthly_retention",
        ))),
        "{{data_new_customers}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "new_customers",
        ))),
        "{{data_total_customers}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "total_customers",
        ))),
        "{{data_avg_order_value}}" => Some(optional_number(data::infer_metric_number(
            text,
            "avg_order_value",
        ))),
        "{{data_avg_orders_per_customer}}" => Some(optional_number(data::infer_metric_number(
            text,
            "avg_orders_per_customer",
        ))),
        "{{data_recency_days}}" => Some(optional_number(data::infer_metric_number(
            text,
            "recency_days",
        ))),
        "{{data_target_gmv}}" => Some(optional_number(data::infer_metric_number(
            text,
            "target_gmv",
        ))),
        "{{data_previous_gmv}}" => Some(optional_number(data::infer_metric_number(
            text,
            "previous_gmv",
        ))),
        "{{data_gmv}}" => Some(optional_number(data::infer_metric_number(text, "gmv"))),
        "{{data_previous_orders}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "previous_orders",
        ))),
        "{{data_orders}}" => Some(optional_integer(data::infer_metric_integer(text, "orders"))),
        "{{data_previous_visitors}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "previous_visitors",
        ))),
        "{{data_visitors}}" => Some(optional_integer(data::infer_metric_integer(
            text, "visitors",
        ))),
        "{{data_clicks}}" => Some(optional_integer(data::infer_metric_integer(text, "clicks"))),
        "{{data_ad_spend}}" => Some(optional_number(data::infer_metric_number(text, "ad_spend"))),
        "{{data_refund_amount}}" => Some(optional_number(data::infer_metric_number(
            text,
            "refund_amount",
        ))),
        "{{data_gross_profit}}" => Some(optional_number(data::infer_metric_number(
            text,
            "gross_profit",
        ))),
        "{{data_impressions}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "impressions",
        ))),
        "{{data_add_to_cart}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "add_to_cart",
        ))),
        "{{data_payments}}" => Some(optional_integer(data::infer_metric_integer(
            text, "payments",
        ))),
        "{{data_inventory}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "inventory",
        ))),
        "{{data_stockout_count}}" => Some(optional_integer(data::infer_metric_integer(
            text,
            "stockout_count",
        ))),
        "{{data_budget}}" => Some(optional_number(data::infer_metric_number(text, "budget"))),
        "{{data_spend}}" => Some(optional_number(data::infer_metric_number(text, "spend"))),
        "{{data_coupon_cost}}" => Some(optional_number(data::infer_metric_number(
            text,
            "coupon_cost",
        ))),
        "{{data_product_id}}" => Some(optional_string(data::infer_labeled_token(
            text,
            "product_id",
        ))),
        "{{data_sku_id}}" => Some(optional_string(data::infer_labeled_token(text, "sku_id"))),
        "{{data_campaign_id}}" => Some(optional_string(data::infer_labeled_token(
            text,
            "campaign_id",
        ))),
        "{{search_max_results}}" => Some(optional_integer(search::max_results(text))),
        "{{search_bypass_cache}}" => Some(serde_json::json!(search::bypass_cache(text))),
        "{{selected_product_refs}}" => {
            Some(serde_json::json!(context::selected_product_refs(task)))
        }
        "{{selected_product_features}}" => Some(serde_json::json!(
            context::selected_product_features_or_infer(task, text)
        )),
        "{{keyword_candidates}}" => Some(serde_json::json!(context::keyword_candidates(text))),
        "{{data_our_price}}" => Some(optional_number(Some(numbers::infer_percent_number(
            text, 99.0,
        )))),
        "{{ab_control_visitors}}" => Some(optional_integer(data::infer_ab_integer(
            text,
            "control_visitors",
        ))),
        "{{ab_control_conversions}}" => Some(optional_integer(data::infer_ab_integer(
            text,
            "control_conversions",
        ))),
        "{{ab_test_visitors}}" => Some(optional_integer(data::infer_ab_integer(
            text,
            "test_visitors",
        ))),
        "{{ab_test_conversions}}" => Some(optional_integer(data::infer_ab_integer(
            text,
            "test_conversions",
        ))),
        _ => None,
    }
}

fn optional_number(value: Option<f64>) -> serde_json::Value {
    match value {
        Some(value) if value.is_finite() => serde_json::json!(value),
        _ => serde_json::Value::Null,
    }
}

fn optional_integer(value: Option<i64>) -> serde_json::Value {
    match value {
        Some(value) => serde_json::json!(value),
        _ => serde_json::Value::Null,
    }
}

fn optional_string(value: Option<String>) -> serde_json::Value {
    value
        .filter(|value| !value.trim().is_empty())
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null)
}

fn render_template_string(template: &str, text: &str, task: &Task) -> String {
    template
        .replace("{{task_text}}", text.trim())
        .replace("{{task_description}}", task.description.trim())
        .replace("{{task_id}}", task.id.as_str())
        .replace("{{platform}}", &metrics::infer_platform(text))
        .replace(
            "{{store_metrics_platform}}",
            &metrics::infer_store_metrics_platform(text),
        )
        .replace("{{stage}}", &metrics::infer_stage(text))
        .replace("{{metric}}", metrics::infer_metric(text))
        .replace(
            "{{period_days}}",
            &numbers::infer_period_days(text).to_string(),
        )
        .replace(
            "{{integer}}",
            &numbers::infer_integer_or_default(text, 1).to_string(),
        )
        .replace(
            "{{percent}}",
            &numbers::infer_percent_number(text, 0.0).to_string(),
        )
        .replace(
            "{{second_percent}}",
            &numbers::infer_second_percent_number(text, 0.0).to_string(),
        )
        .replace("{{cohort_unit}}", metrics::infer_cohort_unit(text))
        .replace(
            "{{seasonal_period}}",
            &metrics::infer_seasonal_period(text).to_string(),
        )
        .replace("{{memory_key}}", &memory_task::infer_memory_key(text))
        .replace("{{memory_type}}", memory_task::infer_memory_type(text))
        .replace(
            "{{memory_content}}",
            &memory_task::infer_memory_content(text, task),
        )
        .replace("{{task_title}}", &memory_task::infer_task_title(text, task))
        .replace(
            "{{task_owner_role}}",
            memory_task::infer_task_owner_role(text),
        )
        .replace(
            "{{task_priority}}",
            &memory_task::infer_task_priority(text).to_string(),
        )
        .replace(
            "{{task_acceptance}}",
            &memory_task::infer_task_acceptance(text),
        )
        .replace("{{return_reason}}", &domain::infer_return_reason(text))
        .replace(
            "{{service_issue_type}}",
            &domain::infer_service_issue_type(text),
        )
        .replace("{{service_emotion}}", &domain::infer_service_emotion(text))
        .replace("{{design_usage}}", &domain::infer_design_usage(text))
        .replace("{{design_style}}", &domain::infer_design_style(text))
        .replace("{{page_type}}", &domain::infer_page_type(text))
        .replace("{{discount_hint}}", &domain::infer_discount_hint(text))
        .replace("{{video_type}}", &domain::infer_video_type(text))
        .replace(
            "{{video_duration_seconds}}",
            &domain::infer_video_duration_seconds(text).to_string(),
        )
        .replace("{{audience}}", &domain::infer_audience(text))
        .replace("{{category}}", &domain::infer_category(text))
        .replace("{{product_name}}", &domain::infer_product_name(text))
        .replace("{{campaign_name}}", &domain::infer_campaign_name(text))
        .replace("{{campaign_type}}", &domain::infer_campaign_type(text))
        .replace("{{cost}}", &commerce::infer_cost(text, 1.0).to_string())
        .replace(
            "{{current_price}}",
            &commerce::infer_current_price(text, 9.9).to_string(),
        )
        .replace(
            "{{stock_days}}",
            &commerce::infer_stock_days(text, 30.0).to_string(),
        )
        .replace(
            "{{elasticity}}",
            &commerce::infer_elasticity(text, -1.2).to_string(),
        )
        .replace(
            "{{budget}}",
            &commerce::infer_default_budget(text).to_string(),
        )
        .replace("{{system_type}}", &domain::infer_system_type(text))
        .replace("{{bug_type}}", &domain::infer_bug_type(text))
        .replace("{{bug_frequency}}", &domain::infer_bug_frequency(text))
        .replace("{{perf_bottleneck}}", &domain::infer_perf_bottleneck(text))
        .replace("{{stack_hint}}", &domain::infer_stack_hint(text))
        .replace("{{tech_domain}}", &domain::infer_tech_domain(text))
        .replace(
            "{{status_query_type}}",
            &domain::infer_status_query_type(text),
        )
        .replace(
            "{{platform_health_check_platform}}",
            &domain::infer_platform_health_check_platform(text),
        )
        .replace("{{learning_topic}}", &domain::infer_learning_topic(text))
        .replace("{{learning_level}}", &domain::infer_learning_level(text))
        .replace(
            "{{learning_scenario}}",
            &domain::infer_learning_scenario(text),
        )
        .replace(
            "{{selected_product_ref}}",
            context::selected_product_ref(task).as_deref().unwrap_or(""),
        )
        .replace(
            "{{selected_product_name}}",
            &context::selected_product_name_or_task(task),
        )
        .replace(
            "{{selected_product_name_or_query}}",
            &context::selected_product_name_or_task(task),
        )
        .replace(
            "{{selected_product_brand_name}}",
            &context::selected_product_brand_name_or_task(task),
        )
        .replace(
            "{{selected_product_count}}",
            &context::selected_product_count_or_default(task, 6).to_string(),
        )
        .replace(
            "{{selected_product_article_platform}}",
            &context::selected_product_article_platform_or_default(task, text),
        )
        .replace(
            "{{title_candidate}}",
            &context::title_candidate_or_task(text, task),
        )
        .replace(
            "{{douyin_ads_product_name}}",
            &douyin_ads::infer_product_name(text, task),
        )
        .replace(
            "{{douyin_ads_stage}}",
            &douyin_ads::infer_stage_for_ads(text),
        )
        .replace(
            "{{inventory_avg_daily_demand}}",
            &inventory::infer_avg_daily_demand(text).to_string(),
        )
        .replace(
            "{{inventory_lead_time_days}}",
            &inventory::infer_lead_time_days(text, 7).to_string(),
        )
        .replace("{{attribution_model}}", data::infer_attribution_model(text))
        .replace("{{ab_test_name}}", data::infer_ab_test_name(text))
        .replace("{{ab_metric}}", data::infer_ab_metric(text))
        .replace(
            "{{forecast_periods}}",
            &data::infer_forecast_periods(text).to_string(),
        )
        .replace("{{data_days}}", &data::default_days(text).to_string())
        .replace("{{data_top_n}}", &data::infer_top_n(text, 5).to_string())
        .replace("{{diagnosis_focus}}", data::infer_diagnosis_focus(text))
        .replace("{{search_query}}", &search::query_for_task(text, task))
        .replace(
            "{{search_industry}}",
            &search::industry_for_task(text, task),
        )
        .replace(
            "{{search_platform}}",
            &search::platform_for_task(text, task),
        )
        .replace(
            "{{search_bypass_cache}}",
            if search::bypass_cache(text) {
                "true"
            } else {
                "false"
            },
        )
        .replace("{{search_policy_type}}", search::policy_type(text))
        .replace("{{search_review_focus}}", search::review_focus(text))
        .replace(
            "{{search_competitor_focus}}",
            search::competitor_focus(text),
        )
        .replace(
            "{{search_benchmark_metric}}",
            search::benchmark_metric(text),
        )
        .replace("{{search_topic}}", search::topic(text))
        .replace("{{search_days}}", &search::days(text).to_string())
}

fn contains_keyword(text: &str, needle: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains(needle) || lower.contains(&needle.to_ascii_lowercase())
}
