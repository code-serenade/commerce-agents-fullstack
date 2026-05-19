fn normalize_web_output(skill_id: &str, input: &Value, output: &mut Value) {
    let Some(map) = output.as_object_mut() else {
        return;
    };
    if map.contains_key("status") {
        return;
    }
    map.insert("status".to_owned(), json!("success"));
    map.insert("has_data".to_owned(), json!(true));
    map.insert("adapter_status".to_owned(), json!("manual_input"));
    map.insert(
        "data_readiness".to_owned(),
        json!(web_data_readiness(skill_id, input)),
    );
    map.insert("summary".to_owned(), json!(web_summary(skill_id)));
}

fn web_data_readiness(skill_id: &str, input: &Value) -> &'static str {
    match skill_id {
        "web_seo_optimize"
            if !has_any_input(
                input,
                &[
                    "target_keywords",
                    "current_rank",
                    "product_ref",
                    "product_id",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "web_title_generator"
            if !has_any_input(
                input,
                &["product_name", "product_ref", "keywords", "product_id"],
            ) =>
        {
            "calculated_from_defaults"
        }
        "web_page_conversion"
            if !has_any_input(
                input,
                &["current_conversion", "bounce_rate", "avg_stay_time"],
            ) =>
        {
            "calculated_from_defaults"
        }
        "web_keyword_research"
            if !has_any_input(input, &["seed_keyword", "product_ref", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        "web_product_description"
            if !has_any_input(input, &["product_name", "product_ref", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        _ => "user_provided",
    }
}

fn has_any_input(input: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        input.get(*key).is_some_and(|value| match value {
            Value::Null => false,
            Value::String(value) => !value.trim().is_empty(),
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
            _ => true,
        })
    })
}

fn web_native_output(skill_id: &str, input: &Map<String, Value>) -> Value {
    match skill_id {
        "web_seo_optimize" => seo_optimize(input),
        "web_title_generator" => title_generator(input),
        "web_store_design" => store_design(input),
        "web_keyword_research" => keyword_research(input),
        "web_product_description" => product_description(input),
        _ => json!({"has_data": false, "error": format!("unknown web skill: {skill_id}")}),
    }
}

fn web_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "web_seo_optimize" => "SEO优化方案已生成",
        "web_title_generator" => "标题候选已生成",
        "web_title_seo_scorer" => "标题SEO评分已完成",
        "web_page_conversion" => "页面转化优化方案已生成",
        "web_store_design" => "店铺装修方案已生成",
        "web_keyword_research" => "关键词矩阵已生成",
        "web_product_description" => "详情页文案已生成",
        _ => "Web增长方案已生成",
    }
}
