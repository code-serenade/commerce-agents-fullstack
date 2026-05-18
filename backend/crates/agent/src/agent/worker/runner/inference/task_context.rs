fn extract_title_candidate(text: &str) -> Option<String> {
    for quote in ['"', '\'', '“', '”', '「', '」'] {
        let mut parts = text.split(quote);
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

fn extract_keyword_candidates(text: &str) -> Vec<String> {
    ["SEO", "关键词", "标题", "转化", "新品", "直播", "抖音"]
        .iter()
        .filter(|keyword| text.contains(**keyword))
        .map(|keyword| (*keyword).to_string())
        .take(4)
        .collect()
}

fn compact_task_text(value: &str) -> String {
    let trimmed = value.trim();
    const MAX_CHARS: usize = 80;
    if trimmed.chars().count() <= MAX_CHARS {
        return trimmed.to_string();
    }
    let mut output = trimmed.chars().take(MAX_CHARS).collect::<String>();
    output.push_str("...");
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedProductContext {
    id: String,
    name: String,
    platform: Option<String>,
    category: Option<String>,
    brand: Option<String>,
    description: Option<String>,
    selling_points: Vec<String>,
}

fn selected_products_from_task(task: &Task) -> Vec<SelectedProductContext> {
    let Some(summary) = task
        .context
        .metadata
        .get("context.selected_summary")
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

fn first_selected_product(task: &Task) -> Option<SelectedProductContext> {
    selected_products_from_task(task).into_iter().next()
}

fn selected_product_ref(task: &Task) -> Option<String> {
    first_selected_product(task).map(|product| product.ref_key())
}

fn selected_product_refs(task: &Task) -> Vec<String> {
    selected_products_from_task(task)
        .into_iter()
        .map(|product| product.ref_key())
        .collect()
}

fn selected_product_name_or_task(task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| compact_task_text(&task.description))
}

fn selected_product_brand_name_or_task(task: &Task) -> String {
    first_selected_product(task)
        .and_then(|product| {
            product
                .brand
                .filter(|brand| !brand.is_empty())
                .or(Some(product.name))
        })
        .unwrap_or_else(|| compact_task_text(&task.description))
}

fn selected_product_platform_or_infer(task: &Task, text: &str) -> String {
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| infer_platform(text).to_string())
}

fn selected_product_category_or_infer(task: &Task, text: &str) -> String {
    first_selected_product(task)
        .and_then(|product| product.category)
        .filter(|category| !category.is_empty())
        .unwrap_or_else(|| infer_category(text).to_string())
}

fn selected_product_features_or_infer(task: &Task, text: &str) -> Vec<String> {
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

fn selected_product_count_or_default(task: &Task, default: i64) -> i64 {
    let count = selected_products_from_task(task).len();
    if count == 0 { default } else { count as i64 }
}

fn selected_product_article_platform_or_default(task: &Task, text: &str) -> String {
    first_selected_product(task)
        .and_then(|product| product.platform)
        .filter(|platform| !platform.is_empty())
        .unwrap_or_else(|| {
            if text.contains("小红书") {
                "小红书".to_string()
            } else {
                "微信公众号".to_string()
            }
        })
}

impl SelectedProductContext {
    fn ref_key(&self) -> String {
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

