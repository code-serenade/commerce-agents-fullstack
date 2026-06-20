use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    helpers::{compact_text, metadata_refs, metadata_text, normalized_optional_text},
    specs::material_table_profiler_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_i64_param,
    optional_string_param, optional_string_vec_param,
};

const DEFAULT_MAX_ROWS: usize = 80;
const MAX_COLUMNS: usize = 40;
const MAX_CELL_CHARS: usize = 120;

#[derive(Clone, Debug)]
struct ParsedTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    source_format: &'static str,
}

#[derive(Clone, Debug, Default)]
struct ColumnProfile {
    name: String,
    inferred_type: &'static str,
    non_empty_count: usize,
    missing_count: usize,
    numeric_count: usize,
    bool_count: usize,
    currency_count: usize,
    percent_count: usize,
    unique_count: usize,
    examples: Vec<String>,
    min: Option<f64>,
    max: Option<f64>,
    mean: Option<f64>,
    ecommerce_semantics: Vec<String>,
    quality_flags: Vec<String>,
}

pub struct MaterialTableProfiler {
    spec: SkillSpec,
}

impl MaterialTableProfiler {
    pub fn new() -> Self {
        Self {
            spec: material_table_profiler_spec(),
        }
    }
}

impl Default for MaterialTableProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for MaterialTableProfiler {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let table_name = normalized_optional_text(optional_string_param(&params, "table_name")?)
            .or_else(|| {
                metadata_text(
                    &context,
                    &["request.context.source_titles", "context.source_titles"],
                )
            });
        let user_question =
            normalized_optional_text(optional_string_param(&params, "user_question")?)
                .or_else(|| metadata_text(&context, &["task_description"]));
        let max_rows = optional_i64_param(&params, "max_rows")?
            .and_then(|value| usize::try_from(value.clamp(1, 500)).ok())
            .unwrap_or(DEFAULT_MAX_ROWS);
        let mut context_refs = optional_string_vec_param(&params, "context_refs")?
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| normalized_optional_text(Some(value)))
            .collect::<Vec<_>>();
        context_refs.extend(metadata_refs(
            &context,
            &[
                "request.context.refs",
                "context.refs",
                "request.context.combined_refs",
                "context.combined_refs",
            ],
        ));
        dedupe(&mut context_refs);

        let parsed = parse_rows_param(params.get("rows"), max_rows).or_else(|| {
            let table_text = normalized_optional_text(
                optional_string_param(&params, "table_text").ok().flatten(),
            )
            .or_else(|| {
                metadata_text(
                    &context,
                    &[
                        "request.context.combined_summary",
                        "context.combined_summary",
                        "request.workspaceContextPack.summary",
                        "workspaceContextPack.summary",
                    ],
                )
            });
            table_text.and_then(|text| parse_table_text(&text, max_rows))
        });

        let Some(table) = parsed else {
            let summary = format!(
                "未收到可剖析的表格行数据：{}。",
                table_name.as_deref().unwrap_or("未命名表格")
            );
            let output = json!({
                "status": "deferred",
                "has_data": false,
                "adapter_status": "manual_or_context_input",
                "data_readiness": "missing_table",
                "summary": summary,
                "skill": self.spec.id,
                "表格名称": table_name,
                "资料引用": context_refs,
                "字段画像": [],
                "可计算指标": [],
                "数据质量问题": ["缺少 rows 数组、CSV/TSV/Markdown 表格文本或可解析的上下文表格。"],
                "下一步建议": [
                    "先完成 Excel/CSV 文本抽取，或把表格内容传入 table_text。",
                    "如果来自上传文件，应先调用 material_file_understanding 确认文件正文是否可读。"
                ],
                "治理说明": [
                    "本 skill 不读取磁盘文件，只剖析本轮显式传入的表格文本或结构化 rows。",
                    "缺少表格内容时不得生成字段、指标或测算结论。"
                ]
            });
            return Ok(SkillOutcome::new(output).with_summary(summary));
        };

        let row_count = table.rows.len();
        let column_count = table.headers.len();
        let profiles = profile_columns(&table);
        let metric_candidates = metric_candidates(&profiles);
        let quality_issues = quality_issues(&profiles, row_count, column_count);
        let downstream_skills =
            downstream_skill_suggestions(&profiles, user_question.as_deref().unwrap_or_default());
        let summary = format!(
            "已剖析表格：{}，{} 行样本、{} 个字段，识别 {} 个可计算指标候选。",
            table_name.as_deref().unwrap_or("未命名表格"),
            row_count,
            column_count,
            metric_candidates.len()
        );
        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_or_context_input",
            "data_readiness": "table_profiled",
            "summary": summary.clone(),
            "skill": self.spec.id,
            "表格名称": table_name,
            "资料引用": context_refs,
            "来源格式": table.source_format,
            "行数样本": row_count,
            "字段数": column_count,
            "字段列表": table.headers,
            "字段画像": profiles.iter().map(column_profile_json).collect::<Vec<_>>(),
            "可计算指标": metric_candidates,
            "数据质量问题": quality_issues,
            "后续适配 Skill": downstream_skills,
            "问题相关性": relevance_note(user_question.as_deref(), &profiles),
            "下一步建议": next_actions(&profiles),
            "治理说明": [
                "本 skill 只做表格结构剖析和轻量字段质量判断，不替代财务测算、归因建模或完整数据仓库校验。",
                "如果字段画像显示缺失、含义不清或异常值，后续岗位回答必须先标注假设和数据风险。",
                "如果用户要求真实店铺后台或最新外部行业指标，应调用 data_* 或 search_*，不能把样本表格伪造成实时数据。"
            ]
        });

        Ok(SkillOutcome::new(output).with_summary(summary))
    }
}

fn parse_rows_param(value: Option<&Value>, max_rows: usize) -> Option<ParsedTable> {
    let rows = value?.as_array()?;
    if rows.is_empty() {
        return None;
    }

    if rows.iter().all(Value::is_object) {
        let mut headers = BTreeSet::new();
        for row in rows.iter().take(max_rows) {
            for key in row.as_object()?.keys() {
                headers.insert(key.clone());
            }
        }
        let headers = headers.into_iter().take(MAX_COLUMNS).collect::<Vec<_>>();
        let parsed_rows = rows
            .iter()
            .take(max_rows)
            .map(|row| {
                let object = row.as_object().expect("checked object rows");
                headers
                    .iter()
                    .map(|key| cell_to_string(object.get(key).unwrap_or(&Value::Null)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        return (!headers.is_empty()).then_some(ParsedTable {
            headers,
            rows: parsed_rows,
            source_format: "rows_object",
        });
    }

    let arrays = rows
        .iter()
        .filter_map(Value::as_array)
        .take(max_rows + 1)
        .collect::<Vec<_>>();
    if arrays.is_empty() {
        return None;
    }
    let width = arrays
        .iter()
        .map(|row| row.len())
        .max()
        .unwrap_or(0)
        .min(MAX_COLUMNS);
    if width == 0 {
        return None;
    }
    let first = arrays[0];
    let first_looks_header = first.iter().all(|cell| cell.as_str().is_some())
        && arrays.get(1).is_some_and(|row| {
            row.iter()
                .any(|cell| numeric_value(&cell_to_string(cell)).is_some())
        });
    let headers = if first_looks_header {
        first
            .iter()
            .take(width)
            .enumerate()
            .map(|(index, cell)| normalize_header(&cell_to_string(cell), index))
            .collect::<Vec<_>>()
    } else {
        (0..width)
            .map(|index| format!("字段{}", index + 1))
            .collect()
    };
    let start = usize::from(first_looks_header);
    let parsed_rows = arrays
        .iter()
        .skip(start)
        .take(max_rows)
        .map(|row| normalized_row(row.iter().map(cell_to_string), width))
        .collect::<Vec<_>>();
    Some(ParsedTable {
        headers,
        rows: parsed_rows,
        source_format: "rows_array",
    })
}

fn parse_table_text(text: &str, max_rows: usize) -> Option<ParsedTable> {
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !is_markdown_separator(line))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }

    let delimiter = infer_delimiter(&lines)?;
    let mut rows = lines
        .into_iter()
        .filter(|line| line.contains(delimiter))
        .map(|line| split_line(line, delimiter))
        .filter(|row| row.len() >= 2)
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return None;
    }

    let width = rows
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .min(MAX_COLUMNS);
    if width == 0 {
        return None;
    }
    let first = rows.remove(0);
    let first_looks_header = first
        .iter()
        .all(|cell| numeric_value(cell).is_none() && !cell.trim().is_empty());
    let headers = if first_looks_header {
        first
            .into_iter()
            .take(width)
            .enumerate()
            .map(|(index, cell)| normalize_header(&cell, index))
            .collect::<Vec<_>>()
    } else {
        rows.insert(0, first);
        (0..width)
            .map(|index| format!("字段{}", index + 1))
            .collect()
    };
    let rows = rows
        .into_iter()
        .take(max_rows)
        .map(|row| normalized_row(row.into_iter(), width))
        .collect::<Vec<_>>();

    Some(ParsedTable {
        headers,
        rows,
        source_format: match delimiter {
            ',' => "csv_text",
            '\t' => "tsv_text",
            '|' => "markdown_table",
            _ => "delimited_text",
        },
    })
}

fn profile_columns(table: &ParsedTable) -> Vec<ColumnProfile> {
    let mut profiles = Vec::new();
    for (index, name) in table.headers.iter().enumerate() {
        let mut values = Vec::new();
        for row in &table.rows {
            values.push(row.get(index).cloned().unwrap_or_default());
        }
        profiles.push(profile_column(name, &values));
    }
    profiles
}

fn profile_column(name: &str, values: &[String]) -> ColumnProfile {
    let mut non_empty = Vec::new();
    let mut numeric_values = Vec::new();
    let mut unique = BTreeSet::new();
    let mut bool_count = 0;
    let mut currency_count = 0;
    let mut percent_count = 0;
    let mut examples = Vec::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        non_empty.push(trimmed.to_owned());
        unique.insert(trimmed.to_ascii_lowercase());
        push_unique(&mut examples, compact_text(trimmed, MAX_CELL_CHARS));
        if boolean_value(trimmed).is_some() {
            bool_count += 1;
        }
        if looks_currency(trimmed) {
            currency_count += 1;
        }
        if looks_percent(trimmed) {
            percent_count += 1;
        }
        if let Some(number) = numeric_value(trimmed) {
            numeric_values.push(number);
        }
    }

    let non_empty_count = non_empty.len();
    let missing_count = values.len().saturating_sub(non_empty_count);
    let numeric_count = numeric_values.len();
    let inferred_type = infer_column_type(
        name,
        values.len(),
        numeric_count,
        bool_count,
        currency_count,
        percent_count,
    );
    let min = numeric_values.iter().copied().reduce(f64::min);
    let max = numeric_values.iter().copied().reduce(f64::max);
    let mean = (!numeric_values.is_empty())
        .then(|| numeric_values.iter().sum::<f64>() / numeric_values.len() as f64);
    let ecommerce_semantics = ecommerce_semantics(name);
    let quality_flags =
        column_quality_flags(values.len(), missing_count, &numeric_values, inferred_type);

    ColumnProfile {
        name: name.to_owned(),
        inferred_type,
        non_empty_count,
        missing_count,
        numeric_count,
        bool_count,
        currency_count,
        percent_count,
        unique_count: unique.len(),
        examples: examples.into_iter().take(3).collect(),
        min,
        max,
        mean,
        ecommerce_semantics,
        quality_flags,
    }
}

fn column_profile_json(profile: &ColumnProfile) -> Value {
    json!({
        "name": profile.name,
        "inferred_type": profile.inferred_type,
        "non_empty_count": profile.non_empty_count,
        "missing_count": profile.missing_count,
        "numeric_count": profile.numeric_count,
        "unique_count": profile.unique_count,
        "examples": profile.examples,
        "numeric_summary": {
            "min": profile.min,
            "max": profile.max,
            "mean": profile.mean
        },
        "ecommerce_semantics": profile.ecommerce_semantics,
        "quality_flags": profile.quality_flags
    })
}

fn metric_candidates(profiles: &[ColumnProfile]) -> Vec<Value> {
    let mut metrics = Vec::new();
    let by_semantic = semantic_index(profiles);
    push_metric_if(
        &mut metrics,
        &by_semantic,
        "gmv",
        &["gmv", "revenue", "sales_amount"],
        "GMV/销售额",
    );
    push_metric_if(&mut metrics, &by_semantic, "orders", &["orders"], "订单量");
    push_metric_if(&mut metrics, &by_semantic, "cost", &["cost"], "成本");
    push_metric_if(
        &mut metrics,
        &by_semantic,
        "price",
        &["price"],
        "售价/客单价",
    );
    push_metric_if(
        &mut metrics,
        &by_semantic,
        "ad_spend",
        &["ad_spend"],
        "广告花费",
    );
    push_metric_if(
        &mut metrics,
        &by_semantic,
        "visitors",
        &["visitors"],
        "访客数",
    );
    push_metric_if(&mut metrics, &by_semantic, "clicks", &["clicks"], "点击数");
    push_metric_if(
        &mut metrics,
        &by_semantic,
        "conversion_rate",
        &["conversion_rate"],
        "转化率",
    );
    if by_semantic.contains_key("gmv") && by_semantic.contains_key("cost") {
        metrics.push(json!({
            "metric": "gross_profit_candidate",
            "label": "毛利候选",
            "formula_hint": "GMV - 成本或 销售额 - 成本",
            "required_fields": [by_semantic["gmv"], by_semantic["cost"]],
            "next_skill": "accounting_profit_calc"
        }));
    }
    if by_semantic.contains_key("gmv") && by_semantic.contains_key("ad_spend") {
        metrics.push(json!({
            "metric": "roas_candidate",
            "label": "ROAS 候选",
            "formula_hint": "GMV / 广告花费",
            "required_fields": [by_semantic["gmv"], by_semantic["ad_spend"]],
            "next_skill": "data_channel_roi"
        }));
    }
    if by_semantic.contains_key("orders") && by_semantic.contains_key("visitors") {
        metrics.push(json!({
            "metric": "order_conversion_candidate",
            "label": "下单转化率候选",
            "formula_hint": "订单量 / 访客数",
            "required_fields": [by_semantic["orders"], by_semantic["visitors"]],
            "next_skill": "data_funnel_analysis"
        }));
    }
    metrics
}

fn quality_issues(
    profiles: &[ColumnProfile],
    row_count: usize,
    column_count: usize,
) -> Vec<String> {
    let mut issues = Vec::new();
    if row_count == 0 {
        issues.push("没有可用数据行。".to_owned());
    }
    if column_count == 0 {
        issues.push("没有可用字段。".to_owned());
    }
    if column_count >= MAX_COLUMNS {
        issues.push(format!(
            "字段数达到剖析上限 {MAX_COLUMNS}，可能有字段被截断。"
        ));
    }
    for profile in profiles {
        for flag in &profile.quality_flags {
            push_unique(&mut issues, format!("字段「{}」：{flag}", profile.name));
        }
    }
    if issues.is_empty() {
        issues.push("未发现明显缺失或异常；仍需结合业务场景确认字段含义。".to_owned());
    }
    issues
}

fn downstream_skill_suggestions(profiles: &[ColumnProfile], question: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    let semantics = semantic_index(profiles);
    if question.contains("利润") || question.contains("毛利") || semantics.contains_key("cost")
    {
        push_unique(&mut suggestions, "accounting_profit_calc".to_owned());
    }
    if question.contains("ROI") || question.contains("ROAS") || semantics.contains_key("ad_spend") {
        push_unique(&mut suggestions, "data_channel_roi".to_owned());
    }
    if question.contains("漏斗")
        || semantics.contains_key("visitors")
        || semantics.contains_key("clicks")
    {
        push_unique(&mut suggestions, "data_funnel_analysis".to_owned());
    }
    if question.contains("库存") || semantics.contains_key("inventory") {
        push_unique(&mut suggestions, "ops_inventory_optimizer".to_owned());
    }
    if suggestions.is_empty() {
        suggestions.push("material_file_understanding".to_owned());
        suggestions.push("data_comprehensive_diagnosis".to_owned());
    }
    suggestions
}

fn next_actions(profiles: &[ColumnProfile]) -> Vec<String> {
    let mut actions = Vec::new();
    actions.push(
        "先让用户确认字段含义，尤其是金额是否含税、成本是否含物流/投放、时间范围是日/周/月。"
            .to_owned(),
    );
    if profiles.iter().any(|profile| profile.missing_count > 0) {
        actions.push("对缺失较多字段先补数或在后续测算中标注缺失假设。".to_owned());
    }
    if profiles
        .iter()
        .any(|profile| !profile.ecommerce_semantics.is_empty())
    {
        actions.push(
            "把字段画像交给对应 data/accounting/ops Skill，避免岗位 Agent 重新猜字段含义。"
                .to_owned(),
        );
    }
    actions
}

fn relevance_note(user_question: Option<&str>, profiles: &[ColumnProfile]) -> String {
    let Some(question) = user_question
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return "未提供具体问题，当前只做通用表格结构剖析。".to_owned();
    };
    let matched = profiles
        .iter()
        .filter(|profile| {
            question.contains(&profile.name)
                || profile
                    .ecommerce_semantics
                    .iter()
                    .any(|item| question.contains(item))
        })
        .map(|profile| profile.name.clone())
        .collect::<Vec<_>>();
    if matched.is_empty() {
        "用户问题未明显命中字段名，后续回答应先说明字段映射假设。".to_owned()
    } else {
        format!("用户问题命中字段/语义：{}。", matched.join("、"))
    }
}

fn semantic_index(profiles: &[ColumnProfile]) -> BTreeMap<&'static str, String> {
    let mut index = BTreeMap::new();
    for profile in profiles {
        for semantic in &profile.ecommerce_semantics {
            index
                .entry(semantic_key(semantic))
                .or_insert_with(|| profile.name.clone());
        }
    }
    index
}

fn push_metric_if(
    metrics: &mut Vec<Value>,
    by_semantic: &BTreeMap<&'static str, String>,
    metric: &'static str,
    semantics: &[&'static str],
    label: &'static str,
) {
    if let Some(field) = semantics
        .iter()
        .find_map(|semantic| by_semantic.get(semantic))
    {
        metrics.push(json!({
            "metric": metric,
            "label": label,
            "field": field,
            "formula_hint": "直接字段或后续按业务规则汇总",
            "next_skill": "data_comprehensive_diagnosis"
        }));
    }
}

fn semantic_key(value: &str) -> &'static str {
    match value {
        "GMV/销售额" => "gmv",
        "收入" => "revenue",
        "订单量" => "orders",
        "成本" => "cost",
        "售价/价格" => "price",
        "广告花费" => "ad_spend",
        "访客数" => "visitors",
        "点击数" => "clicks",
        "转化率" => "conversion_rate",
        "库存" => "inventory",
        "退款" => "refund",
        _ => "other",
    }
}

fn ecommerce_semantics(name: &str) -> Vec<String> {
    let lower = name.to_ascii_lowercase();
    let mut semantics = Vec::new();
    for (keywords, label) in [
        (
            &["gmv", "销售额", "成交额", "支付金额", "销售金额"][..],
            "GMV/销售额",
        ),
        (&["收入", "营收", "revenue"], "收入"),
        (&["订单", "orders", "销量", "件数"], "订单量"),
        (&["成本", "cost", "进货价"], "成本"),
        (&["售价", "价格", "price", "客单价"], "售价/价格"),
        (
            &["广告费", "投放花费", "ad_spend", "spend", "消耗"],
            "广告花费",
        ),
        (&["访客", "uv", "visitors"], "访客数"),
        (&["点击", "click", "clicks"], "点击数"),
        (&["转化率", "conversion", "cvr"], "转化率"),
        (&["库存", "stock", "inventory"], "库存"),
        (&["退款", "退货", "refund"], "退款"),
    ] {
        if keywords
            .iter()
            .any(|keyword| lower.contains(&keyword.to_ascii_lowercase()) || name.contains(keyword))
        {
            push_unique(&mut semantics, label.to_owned());
        }
    }
    semantics
}

fn column_quality_flags(
    row_count: usize,
    missing_count: usize,
    numeric_values: &[f64],
    inferred_type: &str,
) -> Vec<String> {
    let mut flags = Vec::new();
    if row_count > 0 && missing_count * 2 >= row_count {
        flags.push("缺失比例偏高".to_owned());
    }
    if matches!(inferred_type, "number" | "currency" | "percent") {
        if numeric_values.iter().any(|value| *value < 0.0) {
            flags.push("存在负数，需确认是否为退款、成本冲减或录入错误".to_owned());
        }
        if let (Some(min), Some(max)) = (
            numeric_values.iter().copied().reduce(f64::min),
            numeric_values.iter().copied().reduce(f64::max),
        ) && min > 0.0
            && max / min > 100.0
        {
            flags.push("最大最小差异超过 100 倍，可能存在异常值或混合算法".to_owned());
        }
    }
    flags
}

fn infer_column_type(
    name: &str,
    row_count: usize,
    numeric_count: usize,
    bool_count: usize,
    currency_count: usize,
    percent_count: usize,
) -> &'static str {
    if row_count == 0 {
        return "unknown";
    }
    if percent_count > 0 && percent_count * 2 >= numeric_count.max(1) {
        return "percent";
    }
    if currency_count > 0 && currency_count * 2 >= numeric_count.max(1) {
        return "currency";
    }
    if numeric_count * 2 >= row_count {
        if looks_date_name(name) {
            return "date_or_number";
        }
        return "number";
    }
    if bool_count * 2 >= row_count {
        return "boolean";
    }
    if looks_date_name(name) {
        return "date";
    }
    "text"
}

fn infer_delimiter(lines: &[&str]) -> Option<char> {
    [',', '\t', '|']
        .into_iter()
        .max_by_key(|delimiter| {
            lines
                .iter()
                .take(10)
                .map(|line| line.matches(*delimiter).count())
                .sum::<usize>()
        })
        .filter(|delimiter| {
            lines
                .iter()
                .any(|line| line.matches(*delimiter).count() >= 1)
        })
}

fn split_line(line: &str, delimiter: char) -> Vec<String> {
    line.trim_matches('|')
        .split(delimiter)
        .map(|cell| compact_text(cell.trim().trim_matches('"'), MAX_CELL_CHARS))
        .collect()
}

fn normalized_row(values: impl Iterator<Item = String>, width: usize) -> Vec<String> {
    let mut row = values.take(width).collect::<Vec<_>>();
    while row.len() < width {
        row.push(String::new());
    }
    row
}

fn normalize_header(value: &str, index: usize) -> String {
    let value = value.trim().trim_matches('|').trim();
    if value.is_empty() {
        format!("字段{}", index + 1)
    } else {
        compact_text(value, 40)
    }
}

fn is_markdown_separator(line: &str) -> bool {
    line.trim_matches('|').split('|').all(|cell| {
        cell.trim()
            .chars()
            .all(|char| matches!(char, '-' | ':' | ' '))
    })
}

fn cell_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => compact_text(text, MAX_CELL_CHARS),
        Value::Number(number) => number.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => compact_text(&value.to_string(), MAX_CELL_CHARS),
    }
}

fn numeric_value(value: &str) -> Option<f64> {
    let cleaned = value
        .trim()
        .trim_start_matches(['￥', '$'])
        .trim_end_matches('%')
        .replace(',', "");
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn boolean_value(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "y" | "是" | "已" => Some(true),
        "false" | "no" | "n" | "否" | "未" => Some(false),
        _ => None,
    }
}

fn looks_currency(value: &str) -> bool {
    value.contains('￥') || value.contains('$') || value.contains("元")
}

fn looks_percent(value: &str) -> bool {
    value.contains('%')
}

fn looks_date_name(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("date") || value.contains("日期") || value.contains("时间")
}

fn push_unique(out: &mut Vec<String>, value: String) {
    let value = value.trim();
    if value.is_empty() || out.iter().any(|item| item == value) {
        return;
    }
    out.push(value.to_owned());
}

fn dedupe(items: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    items.retain(|item| seen.insert(item.clone()));
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::MaterialTableProfiler;
    use crate::skills::{Skill, SkillContext};

    #[tokio::test]
    async fn profiles_csv_text_and_metric_candidates() {
        let outcome = MaterialTableProfiler::new()
            .run(
                json!({
                    "table_name": "campaign.csv",
                    "table_text": "商品,GMV,成本,广告费,访客,订单\nA,12000,7000,1200,3000,180\nB,8000,5200,900,2400,120",
                    "user_question": "分析利润和 ROAS"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert_eq!(outcome.output["字段数"], 6);
        assert!(
            outcome.output["可计算指标"]
                .as_array()
                .expect("metrics should be array")
                .iter()
                .any(|item| item["metric"] == "gross_profit_candidate")
        );
        assert!(
            outcome.output["后续适配 Skill"]
                .as_array()
                .expect("skills should be array")
                .iter()
                .any(|item| item == "accounting_profit_calc")
        );
    }

    #[tokio::test]
    async fn profiles_rows_object_input() {
        let outcome = MaterialTableProfiler::new()
            .run(
                json!({
                    "rows": [
                        {"sku": "A", "库存": 20, "销量": 100},
                        {"sku": "B", "库存": 0, "销量": 80}
                    ],
                    "user_question": "看库存风险"
                }),
                SkillContext::new(),
            )
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "success");
        assert!(
            outcome.output["后续适配 Skill"]
                .as_array()
                .expect("skills should be array")
                .iter()
                .any(|item| item == "ops_inventory_optimizer")
        );
    }

    #[tokio::test]
    async fn defers_when_no_table_content_exists() {
        let outcome = MaterialTableProfiler::new()
            .run(json!({"table_name": "empty.xlsx"}), SkillContext::new())
            .await
            .expect("skill should run");

        assert_eq!(outcome.output["status"], "deferred");
        assert_eq!(outcome.output["data_readiness"], "missing_table");
    }
}
