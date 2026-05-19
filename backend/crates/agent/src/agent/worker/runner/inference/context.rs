use super::text::compact_task_text;
use crate::agent::worker::{Task, runner::policy::worker_policies};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::agent::worker::runner) struct SelectedProductContext {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) platform: Option<String>,
    pub(super) category: Option<String>,
    pub(super) brand: Option<String>,
    pub(super) description: Option<String>,
    pub(super) selling_points: Vec<String>,
}

pub(super) fn selected_products_from_task(task: &Task) -> Vec<SelectedProductContext> {
    let Some(summary) = task
        .context
        .metadata
        .get("context.selected_summary")
        .or_else(|| {
            task.context
                .metadata
                .get("commerce.selected_product_context")
        })
        .map(String::as_str)
    else {
        return Vec::new();
    };

    let mut products = Vec::new();
    let mut pending_selling_points: Vec<(String, String)> = Vec::new();
    for line in summary.lines() {
        if let Some(product) = parse_product_summary_line(line) {
            products.push(product);
            continue;
        }
        if let Some(point) = parse_knowledge_summary_line(line) {
            pending_selling_points.push(point);
        }
    }

    for (product_id, point) in pending_selling_points {
        if let Some(product) = products.iter_mut().find(|product| product.id == product_id)
            && !product.selling_points.contains(&point)
        {
            product.selling_points.push(point);
        }
    }

    products
}

pub(in crate::agent::worker::runner) fn first_selected_product(
    task: &Task,
) -> Option<SelectedProductContext> {
    selected_products_from_task(task).into_iter().next()
}

pub(in crate::agent::worker::runner) fn selected_product_ref(task: &Task) -> Option<String> {
    first_selected_product(task).map(|product| product.ref_key())
}

pub(in crate::agent::worker::runner) fn selected_product_refs(task: &Task) -> Vec<String> {
    selected_products_from_task(task)
        .into_iter()
        .map(|product| product.ref_key())
        .collect()
}

pub(in crate::agent::worker::runner) fn selected_product_name_or_task(task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| compact_task_text(&task.description))
}

pub(in crate::agent::worker::runner) fn selected_product_brand_name_or_task(task: &Task) -> String {
    first_selected_product(task)
        .and_then(|product| {
            product
                .brand
                .filter(|brand| !brand.is_empty())
                .or(Some(product.name))
        })
        .unwrap_or_else(|| compact_task_text(&task.description))
}

pub(in crate::agent::worker::runner) fn selected_product_platform_or_infer(
    task: &Task,
    text: &str,
) -> String {
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| super::metrics::infer_platform(text))
}

pub(in crate::agent::worker::runner) fn selected_product_category_or_infer(
    task: &Task,
    text: &str,
) -> String {
    first_selected_product(task)
        .and_then(|product| product.category)
        .filter(|category| !category.is_empty())
        .unwrap_or_else(|| super::domain::infer_category(text))
}

pub(in crate::agent::worker::runner) fn selected_product_features_or_infer(
    task: &Task,
    text: &str,
) -> Vec<String> {
    let mut features = first_selected_product(task)
        .map(|product| {
            let mut values = product.selling_points;
            if let Some(description) = product.description.filter(|value| !value.is_empty()) {
                values.push(description);
            }
            values
        })
        .unwrap_or_default();
    if features.is_empty() {
        features = infer_feature_list(text);
    }
    features
}

pub(in crate::agent::worker::runner) fn selected_product_count_or_default(
    task: &Task,
    default: i64,
) -> i64 {
    let count = selected_products_from_task(task).len();
    if count == 0 { default } else { count as i64 }
}

pub(in crate::agent::worker::runner) fn selected_product_article_platform_or_default(
    task: &Task,
    text: &str,
) -> String {
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| infer_article_platform(text))
}

pub(in crate::agent::worker::runner) fn title_candidate_or_task(text: &str, task: &Task) -> String {
    extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description))
}

pub(in crate::agent::worker::runner) fn keyword_candidates(text: &str) -> Vec<String> {
    let config = context_section();
    let max_items = config
        .pointer("/keyword_candidates/max_items")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(4);

    string_list(config, "/keyword_candidates/keywords")
        .into_iter()
        .filter(|keyword| contains_keyword(text, keyword))
        .take(max_items)
        .collect()
}

impl SelectedProductContext {
    pub(in crate::agent::worker::runner) fn name(&self) -> &str {
        &self.name
    }

    pub(in crate::agent::worker::runner) fn platform(&self) -> Option<&str> {
        self.platform.as_deref()
    }

    pub(in crate::agent::worker::runner) fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub(in crate::agent::worker::runner) fn ref_key(&self) -> String {
        if self.id.starts_with("product:") {
            self.id.clone()
        } else {
            format!("product:{}", self.id)
        }
    }
}

fn parse_product_summary_line(line: &str) -> Option<SelectedProductContext> {
    let line = line.trim();
    let rest = line.strip_prefix("Product source product:")?;
    let mut parts = rest.splitn(2, '|');
    let id = parts.next()?.trim();
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let name = fields.get("name")?.trim();
    if id.is_empty() || name.is_empty() {
        return None;
    }
    Some(SelectedProductContext {
        id: id.to_string(),
        name: name.to_string(),
        platform: non_empty_field(&fields, "platform"),
        category: non_empty_field(&fields, "category"),
        brand: non_empty_field(&fields, "brand"),
        description: non_empty_field(&fields, "description"),
        selling_points: Vec::new(),
    })
}

fn extract_title_candidate(text: &str) -> Option<String> {
    for delimiter in string_list(context_section(), "/title_candidate/quote_delimiters") {
        let mut parts = text.split(&delimiter);
        let _ = parts.next();
        if let Some(candidate) = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(compact_task_text(candidate));
        }
    }
    None
}

fn infer_feature_list(text: &str) -> Vec<String> {
    let mut features = Vec::new();
    for keyword in string_list(context_section(), "/feature_inference/keywords") {
        if contains_keyword(text, &keyword) {
            features.push(keyword);
        }
    }
    if features.is_empty() {
        features.push(compact_task_text(text));
    }
    features
}

fn parse_knowledge_summary_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let rest = line.strip_prefix("Knowledge source product_knowledge:")?;
    let mut parts = rest.splitn(2, '|');
    let _knowledge_id = parts.next()?;
    let fields = parse_pipe_fields(parts.next().unwrap_or_default());
    let product_id = fields.get("product")?.trim();
    let content = fields.get("content")?.trim();
    if product_id.is_empty() || content.is_empty() {
        return None;
    }
    Some((product_id.to_string(), content.to_string()))
}

fn parse_pipe_fields(value: &str) -> std::collections::BTreeMap<String, String> {
    value
        .split('|')
        .filter_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn non_empty_field(
    fields: &std::collections::BTreeMap<String, String>,
    key: &str,
) -> Option<String> {
    fields
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn infer_article_platform(text: &str) -> String {
    let config = context_section();
    if let Some(platform) = config
        .pointer("/article_platform/keyword_platforms")
        .and_then(serde_json::Value::as_array)
        .and_then(|rules| {
            rules.iter().find_map(|rule| {
                let platform = rule.get("platform")?.as_str()?;
                let keywords = rule.get("keywords")?.as_array()?;
                keywords
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .any(|keyword| contains_keyword(text, keyword))
                    .then_some(platform)
            })
        })
    {
        return platform.to_string();
    }

    config
        .pointer("/article_platform/default")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn context_section() -> &'static serde_json::Value {
    worker_policies()
        .inference
        .section("context")
        .unwrap_or(&serde_json::Value::Null)
}

fn string_list(config: &serde_json::Value, pointer: &str) -> Vec<String> {
    config
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn contains_keyword(text: &str, keyword: &str) -> bool {
    text.contains(keyword)
        || text
            .to_ascii_lowercase()
            .contains(&keyword.to_ascii_lowercase())
}
