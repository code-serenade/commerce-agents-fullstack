use super::{
    metrics::extract_metric_value,
    numbers::{infer_explicit_period_days, numbers_in_text},
};
use crate::agent::worker::runner::policy::worker_policies;

pub(in crate::agent::worker::runner) fn infer_avg_daily_demand(text: &str) -> f64 {
    extract_metric_value(text, &aliases("avg_daily_demand"))
        .unwrap_or(number_default("defaults/avg_daily_demand", 1.0))
        .max(number_default("rules/avg_daily_demand/result_min", 0.0))
}

pub(in crate::agent::worker::runner) fn infer_lead_time_days(text: &str, default: i64) -> i64 {
    extract_metric_value(text, &aliases("lead_time_days"))
        .map(|value| value.round().max(1.0) as i64)
        .or_else(|| infer_explicit_period_days(text))
        .unwrap_or(default.max(integer_default("rules/lead_time_days/min", 1)))
}

pub(in crate::agent::worker::runner) fn infer_ordering_cost(text: &str) -> Option<f64> {
    extract_metric_value(text, &aliases("ordering_cost"))
}

pub(in crate::agent::worker::runner) fn infer_holding_rate(text: &str) -> Option<f64> {
    extract_metric_value(text, &aliases("holding_rate"))
}

pub(in crate::agent::worker::runner) fn infer_total_budget(text: &str) -> Option<f64> {
    extract_metric_value(text, &aliases("total_budget"))
}

pub(in crate::agent::worker::runner) fn infer_service_level(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    service_levels().into_iter().find_map(|(value, keywords)| {
        keywords
            .iter()
            .any(|keyword| text.contains(keyword) || lower.contains(&keyword.to_ascii_lowercase()))
            .then_some(value)
    })
}

pub(in crate::agent::worker::runner) fn infer_inventory_skus(text: &str) -> Vec<serde_json::Value> {
    split_segments(text, "inventory_skus")
        .filter_map(parse_inventory_sku_segment)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_ad_fatigue_channels(
    text: &str,
) -> Vec<serde_json::Value> {
    let platform_labels = string_list("platforms", "/ad_fatigue_labels");
    let mut rows = Vec::new();
    let mut seen = Vec::<String>::new();
    for label in &platform_labels {
        let Some(segment) = segment_after_label(text, label, &platform_labels) else {
            continue;
        };
        let name = canonical_ad_channel_name(label);
        if seen.iter().any(|value| value == &name) {
            continue;
        }
        let ctr_baseline =
            extract_metric_value(&segment, &nested_aliases("ad_fatigue/ctr_baseline"));
        let ctr_recent = extract_metric_value(&segment, &nested_aliases("ad_fatigue/ctr_recent"));
        let impressions_per_user =
            extract_metric_value(&segment, &nested_aliases("ad_fatigue/impressions_per_user"));
        let budget = extract_metric_value(&segment, &nested_aliases("ad_fatigue/budget"));
        let roas = extract_metric_value(&segment, &nested_aliases("ad_fatigue/roas"));

        if [ctr_baseline, ctr_recent, impressions_per_user, budget, roas]
            .iter()
            .all(Option::is_none)
        {
            continue;
        }

        let mut row = serde_json::Map::new();
        row.insert("name".to_string(), serde_json::json!(name));
        insert_optional_number(&mut row, "ctr_baseline", ctr_baseline);
        insert_optional_number(&mut row, "ctr_recent", ctr_recent);
        insert_optional_number(&mut row, "impressions_per_user", impressions_per_user);
        insert_optional_number(&mut row, "budget", budget);
        insert_optional_number(&mut row, "roas", roas);
        seen.push(name);
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

pub(in crate::agent::worker::runner) fn infer_abc_xyz_skus(text: &str) -> Vec<serde_json::Value> {
    split_segments(text, "abc_xyz_skus")
        .filter_map(parse_abc_xyz_sku_segment)
        .collect()
}

pub(in crate::agent::worker::runner) fn infer_top_n(text: &str) -> Option<i64> {
    extract_metric_value(text, &aliases("top_n")).map(|value| {
        value.round().clamp(
            number_default("rules/top_n/min", 1.0),
            number_default("rules/top_n/max", 100.0),
        ) as i64
    })
}

fn parse_inventory_sku_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if !contains_any(
        trimmed,
        &parser_aliases(
            "inventory_skus",
            "/required_text_alias_group",
            "inventory_sku_required_text",
        ),
    ) {
        return None;
    }
    let monthly_sales = extract_last_metric_value(
        trimmed,
        &parser_aliases(
            "inventory_skus",
            "/field_alias_groups/monthly_sales",
            "inventory_sku_monthly_sales",
        ),
    )?;
    let cost = extract_last_metric_value(
        trimmed,
        &parser_aliases(
            "inventory_skus",
            "/field_alias_groups/cost",
            "inventory_sku_cost",
        ),
    )?;
    let stock_qty = extract_last_metric_value(
        trimmed,
        &parser_aliases(
            "inventory_skus",
            "/field_alias_groups/stock_qty",
            "inventory_sku_stock_qty",
        ),
    )?;
    Some(serde_json::json!({
        "name": infer_inventory_sku_name(trimmed),
        "monthly_sales": monthly_sales,
        "cost": cost,
        "stock_qty": stock_qty,
    }))
}

fn parse_abc_xyz_sku_segment(segment: &str) -> Option<serde_json::Value> {
    let trimmed = segment.trim();
    if !contains_any(
        trimmed,
        &parser_aliases(
            "abc_xyz_skus",
            "/required_text_alias_group",
            "inventory_sku_required_text",
        ),
    ) {
        return None;
    }
    let gmv = extract_last_metric_value(
        trimmed,
        &parser_aliases("abc_xyz_skus", "/gmv_alias_group", "abc_xyz_gmv"),
    )?;
    let history = infer_abc_xyz_sales_history(trimmed);
    let mut row = serde_json::Map::new();
    row.insert(
        "sku_id".to_string(),
        serde_json::json!(infer_abc_xyz_sku_id(trimmed)),
    );
    row.insert("gmv".to_string(), serde_json::json!(gmv));
    if !history.is_empty() {
        row.insert("sales_history".to_string(), serde_json::json!(history));
    }
    Some(serde_json::Value::Object(row))
}

fn infer_inventory_sku_name(segment: &str) -> String {
    let trimmed = normalize_sku_prefix(segment);
    clean_sku_label(
        trimmed,
        &aliases("inventory_sku_name_markers"),
        &string_default("defaults/sku_label_fallback", "SKU"),
    )
}

fn infer_abc_xyz_sku_id(segment: &str) -> String {
    let trimmed = normalize_sku_prefix(segment);
    clean_sku_label(
        trimmed,
        &aliases("abc_xyz_sku_id_markers"),
        &string_default("defaults/sku_label_fallback", "SKU"),
    )
}

fn normalize_sku_prefix(segment: &str) -> &str {
    let trim_prefix_chars = sku_trim_prefix_chars();
    let trimmed = segment
        .trim()
        .trim_start_matches(|ch| trim_prefix_chars.contains(&ch))
        .trim();
    let lower = trimmed.to_ascii_lowercase();
    let sku_prefix_marker = string_default("rules/sku_label_cleanup/sku_prefix_marker", "sku");
    if !bool_default("rules/sku_label_cleanup/prefer_last_sku_prefix", true) {
        return trimmed;
    }
    lower
        .rfind(&sku_prefix_marker.to_ascii_lowercase())
        .map(|index| trimmed[index ..].trim())
        .unwrap_or(trimmed)
}

fn clean_sku_label<T: AsRef<str>>(segment: &str, markers: &[T], fallback: &str) -> String {
    let lower = segment.to_ascii_lowercase();
    let end = markers
        .iter()
        .filter_map(|marker| lower.find(&marker.as_ref().to_ascii_lowercase()))
        .min()
        .unwrap_or(segment.len());
    let trim_delimiters = sku_trim_delimiters();
    let raw = segment[.. end]
        .trim()
        .trim_matches(|ch| trim_delimiters.contains(&ch))
        .trim();
    let raw = raw
        .rfind(|ch| trim_delimiters.contains(&ch))
        .map(|index| raw[index + 1 ..].trim())
        .unwrap_or(raw);
    let lower_raw = raw.to_ascii_lowercase();
    let sku_prefix_marker = string_default("rules/sku_label_cleanup/sku_prefix_marker", "sku");
    let raw = if bool_default("rules/sku_label_cleanup/prefer_last_sku_prefix", true) {
        lower_raw
            .rfind(&sku_prefix_marker.to_ascii_lowercase())
            .map(|index| raw[index ..].trim())
            .unwrap_or(raw)
    } else {
        raw
    };
    if raw.is_empty() {
        fallback.to_string()
    } else {
        raw.to_string()
    }
}

fn infer_abc_xyz_sales_history(segment: &str) -> Vec<f64> {
    let Some(history_segment) = segment_after_abc_history_label(
        segment,
        &parser_aliases(
            "abc_xyz_skus",
            "/history_alias_group",
            "abc_xyz_sales_history",
        ),
    ) else {
        return Vec::new();
    };
    numbers_in_text(&history_segment)
}

fn segment_after_abc_history_label<T: AsRef<str>>(text: &str, labels: &[T]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let mut best: Option<(usize, &str)> = None;
    for label in labels {
        let label = label.as_ref();
        let label_lower = label.to_ascii_lowercase();
        if let Some(index) = lower.find(&label_lower)
            && best
                .map(|(best_index, _)| index < best_index)
                .unwrap_or(true)
        {
            best = Some((index, label));
        }
    }
    let (index, label) = best?;
    let start = index + label.len();
    let tail = text[start ..].trim();
    let split_on = parser_string_list("abc_xyz_skus", "/history_split_on");
    let end = find_first_delimiter(tail, &split_on).unwrap_or(tail.len());
    Some(tail[.. end].trim().to_string())
}

fn extract_last_metric_value<T: AsRef<str>>(text: &str, aliases: &[T]) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    for alias in aliases {
        let alias = alias.as_ref();
        let alias_lower = alias.to_ascii_lowercase();
        let index = lower
            .rmatch_indices(&alias_lower)
            .next()
            .map(|(index, _)| index);
        let Some(index) = index else {
            continue;
        };
        let start = index + alias_lower.len();
        if let Some(value) = super::metrics::parse_metric_number_after_alias(&text[start ..]) {
            return Some(value);
        }
    }
    None
}

fn segment_after_label<T: AsRef<str>>(text: &str, label: &str, labels: &[T]) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();
    let index = lower.find(&label_lower)?;
    let start = index + label.len();
    let tail = &text[start ..];
    let tail_lower = tail.to_ascii_lowercase();
    let next = labels
        .iter()
        .map(AsRef::as_ref)
        .filter(|other| other.to_ascii_lowercase() != label_lower)
        .filter_map(|other| tail_lower.find(&other.to_ascii_lowercase()))
        .min()
        .unwrap_or(tail.len());
    Some(tail[.. next].to_string())
}

fn canonical_ad_channel_name(label: &str) -> String {
    let lower = label.to_ascii_lowercase();
    canonical_channels()
        .into_iter()
        .find_map(|(name, aliases)| {
            aliases
                .iter()
                .any(|alias| lower == alias.to_ascii_lowercase())
                .then_some(name)
        })
        .unwrap_or_else(|| string_default("platforms/unknown_channel", "其他"))
}

fn insert_optional_number(
    row: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<f64>,
) {
    if let Some(value) = value {
        row.insert(key.to_string(), serde_json::json!(value));
    }
}

fn contains_any<T: AsRef<str>>(text: &str, needles: &[T]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .map(AsRef::as_ref)
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}

fn aliases(key: &str) -> Vec<String> {
    string_list("aliases", &format!("/{}", key))
}

fn nested_aliases(path: &str) -> Vec<String> {
    string_list("aliases", &format!("/{}", path))
}

fn string_list(section: &str, pointer: &str) -> Vec<String> {
    worker_policies().inference.string_list(section, pointer)
}

fn parser_string_list(parser: &str, pointer: &str) -> Vec<String> {
    string_list("parsers", &format!("/{parser}{pointer}"))
}

fn parser_aliases(parser: &str, pointer: &str, fallback_alias_group: &str) -> Vec<String> {
    worker_policies()
        .inference
        .section("parsers")
        .and_then(|value| value.pointer(&format!("/{parser}{pointer}")))
        .and_then(serde_json::Value::as_str)
        .map(aliases)
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| aliases(fallback_alias_group))
}

fn split_segments<'a>(text: &'a str, parser: &str) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    let split_on = parser_string_list(parser, "/split_on");
    if split_on.is_empty() {
        Box::new(std::iter::once(text))
    } else {
        Box::new(text.split(move |ch| split_on.iter().any(|separator| is_char(separator, ch))))
    }
}

fn find_first_delimiter(text: &str, delimiters: &[String]) -> Option<usize> {
    delimiters
        .iter()
        .filter_map(|delimiter| text.find(delimiter))
        .min()
}

fn is_char(value: &str, ch: char) -> bool {
    let mut chars = value.chars();
    chars.next() == Some(ch) && chars.next().is_none()
}

fn service_levels() -> Vec<(String, Vec<String>)> {
    worker_policies()
        .inference
        .section("service_levels")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let value = item.get("value")?.as_str()?.to_string();
                    let keywords = item
                        .get("keywords")?
                        .as_array()?
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>();
                    Some((value, keywords))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn canonical_channels() -> Vec<(String, Vec<String>)> {
    worker_policies()
        .inference
        .section("platforms")
        .and_then(|value| value.pointer("/canonical_channels"))
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let label = item.get("label")?.as_str()?.to_string();
                    let aliases = item
                        .get("aliases")?
                        .as_array()?
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>();
                    Some((label, aliases))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn sku_trim_delimiters() -> Vec<char> {
    char_list("rules", "/sku_label_cleanup/trim_delimiters")
}

fn sku_trim_prefix_chars() -> Vec<char> {
    char_list("rules", "/sku_label_cleanup/trim_prefix_chars")
}

fn char_list(section: &str, pointer: &str) -> Vec<char> {
    string_list(section, pointer)
        .into_iter()
        .filter_map(|value| {
            let mut chars = value.chars();
            chars.next().filter(|_| chars.next().is_none())
        })
        .collect()
}

fn string_default(pointer: &str, fallback: &str) -> String {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback.to_string();
    };
    worker_policies()
        .inference
        .string_or(section, &format!("/{}", nested), fallback)
}

fn bool_default(pointer: &str, fallback: bool) -> bool {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback;
    };
    worker_policies()
        .inference
        .section(section)
        .and_then(|value| value.pointer(&format!("/{}", nested)))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(fallback)
}

fn number_default(pointer: &str, fallback: f64) -> f64 {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback;
    };
    worker_policies()
        .inference
        .f64_or(section, &format!("/{}", nested), fallback)
}

fn integer_default(pointer: &str, fallback: i64) -> i64 {
    let Some((section, nested)) = pointer.split_once('/') else {
        return fallback;
    };
    worker_policies()
        .inference
        .i64_or(section, &format!("/{}", nested), fallback)
}
