fn normalize_creative_output(skill_id: &str, input: &Value, output: &mut Value) {
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
        json!(creative_data_readiness(skill_id, input)),
    );
    map.insert("summary".to_owned(), json!(creative_summary(skill_id)));
}

fn creative_data_readiness(skill_id: &str, input: &Value) -> &'static str {
    match skill_id {
        "creative_video_script"
            if !has_any_input(input, &["product_name", "product_ref", "product_id"]) =>
        {
            "calculated_from_defaults"
        }
        "creative_seeding_copy"
            if !has_any_input(
                input,
                &[
                    "product_name",
                    "product_ref",
                    "selling_points",
                    "product_id",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "creative_content_calendar"
            if !has_any_input(input, &["platforms", "content_types", "weekly_posts"]) =>
        {
            "calculated_from_defaults"
        }
        "creative_live_script"
            if !has_any_input(
                input,
                &[
                    "duration_hours",
                    "product_count",
                    "product_refs",
                    "product_ids",
                ],
            ) =>
        {
            "calculated_from_defaults"
        }
        "creative_product_article"
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

fn creative_summary(skill_id: &str) -> &'static str {
    match skill_id {
        "creative_video_script" => "短视频脚本已生成",
        "creative_seeding_copy" => "种草文案已生成",
        "creative_ip_branding" => "IP定位方案已生成",
        "creative_content_calendar" => "内容排期已生成",
        "creative_trend_catch" => "内容趋势方向已生成",
        "creative_live_script" => "直播话术脚本已生成",
        "creative_product_article" => "产品软文草稿已生成",
        "creative_title_ctr_scorer" => "标题CTR预测评分已完成",
        _ => "创意技能已返回输入摘要",
    }
}
