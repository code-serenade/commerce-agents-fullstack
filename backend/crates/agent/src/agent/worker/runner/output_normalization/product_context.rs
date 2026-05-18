use crate::agent::worker::Task;

#[derive(Debug, Clone)]
pub(super) struct SelectedProductContext {
    pub(super) id: String,
    pub(super) name: String,
}

impl SelectedProductContext {
    pub(super) fn ref_key(&self) -> String {
        if self.id.starts_with("product:") {
            self.id.clone()
        } else {
            format!("product:{}", self.id)
        }
    }
}

pub(super) fn first_selected_product(task: &Task) -> Option<SelectedProductContext> {
    selected_products_from_task(task).into_iter().next()
}

fn selected_products_from_task(task: &Task) -> Vec<SelectedProductContext> {
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

    summary
        .lines()
        .filter_map(parse_product_summary_line)
        .collect()
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
    })
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
