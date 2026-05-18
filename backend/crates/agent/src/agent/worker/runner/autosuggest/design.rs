fn autosuggest_design_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if is_design_pure_material_policy_search_intent(text) {
        return vec![autosuggest_design_material_policy_search(text, task)];
    }

    if is_design_pure_visual_search_intent(text) {
        return vec![autosuggest_design_visual_search(
            text,
            task,
            infer_design_visual_search_skill_id(text),
        )];
    }

    let Some(primary) = autosuggest_design_native(text, task) else {
        return Vec::new();
    };

    if is_design_visual_trend_search_chain_intent(text, &primary.skill_id) {
        let search = autosuggest_design_visual_search(text, task, &primary.skill_id);
        return vec![primary, search];
    }

    if is_design_color_trend_search_chain_intent(text, &primary.skill_id) {
        return vec![primary, autosuggest_design_color_trend_search(text, task)];
    }

    if is_design_material_policy_search_chain_intent(text, &primary.skill_id) {
        return vec![
            primary,
            autosuggest_design_material_policy_search(text, task),
        ];
    }

    vec![primary]
}

fn autosuggest_design(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_design_requests(text, task).into_iter().next()
}

fn autosuggest_design_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["material", "asset", "素材", "规格", "尺寸"]) {
        return Some(autosuggest_request(
            "design_material_spec",
            serde_json::json!({
                "usage": infer_design_usage(text),
                "platforms": [selected_product_platform_or_infer(task, text)],
            }),
            "根据素材/规格意图自动补偿素材规范 skill",
        ));
    }

    if contains_any(text, &["详情页", "详情", "长图"]) {
        return Some(autosuggest_request(
            "design_detail_page",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "platform": selected_product_platform_or_infer(task, text),
                "style": infer_design_style(text),
            }),
            "根据详情页设计意图自动补偿详情页设计 skill",
        ));
    }

    if contains_any(text, &["配色", "色彩", "品牌色", "颜色"]) {
        return Some(autosuggest_request(
            "design_color_scheme",
            serde_json::json!({
                "brand_tone": infer_design_style(text),
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
            }),
            "根据配色/品牌色意图自动补偿配色方案 skill",
        ));
    }

    if contains_any(text, &["海报", "banner", "Banner", "活动图", "活动海报"]) {
        return Some(autosuggest_request(
            "design_campaign_poster",
            serde_json::json!({
                "campaign_name": if contains_any(text, &["618"]) { "618大促" } else if contains_any(text, &["双11", "双十一"]) { "双11大促" } else { "营销活动" },
                "discount_info": extract_discount_hint(text),
                "platform": selected_product_platform_or_infer(task, text),
                "product_ref": selected_product_ref(task),
                "style": infer_design_style(text),
            }),
            "根据活动海报意图自动补偿海报设计 skill",
        ));
    }

    Some(autosuggest_request(
        "design_main_image",
        serde_json::json!({
            "product_name": selected_product_name_or_task(task),
            "product_ref": selected_product_ref(task),
            "platform": selected_product_platform_or_infer(task, text),
        }),
        "根据设计岗位任务自动补偿主图设计 skill",
    ))
}

fn is_design_visual_trend_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    matches!(
        native_skill_id,
        "design_main_image" | "design_campaign_poster" | "design_detail_page"
    ) && has_design_visual_trend_search_intent(text)
}

fn is_design_color_trend_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    native_skill_id == "design_color_scheme" && has_design_color_trend_search_intent(text)
}

fn is_design_material_policy_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    native_skill_id == "design_material_spec" && has_design_material_policy_search_intent(text)
}

fn has_design_visual_trend_search_intent(text: &str) -> bool {
    let has_design_signal = contains_any(
        text,
        &[
            "视觉趋势",
            "设计趋势",
            "主图趋势",
            "详情页趋势",
            "海报趋势",
            "爆款主图",
            "爆款海报",
            "高点击率",
            "高转化详情页",
            "竞品视觉",
            "热门视觉",
            "流行视觉",
            "风格趋势",
            "素材趋势",
            "设计参考",
            "视觉参考",
        ],
    );
    let has_fresh_or_evidence = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "最新",
                "近期",
                "当前",
                "今年",
                "2026",
                "公开来源",
                "公开资料",
                "参考",
                "对标",
                "竞品",
                "爆款",
            ],
        );

    has_design_signal && has_fresh_or_evidence
}

fn has_design_color_trend_search_intent(text: &str) -> bool {
    let has_color_signal = contains_any(
        text,
        &[
            "配色趋势",
            "色彩趋势",
            "流行色",
            "年度色",
            "品牌色趋势",
            "竞品配色",
            "热门配色",
            "流行配色",
            "视觉色彩",
            "color trend",
            "palette trend",
        ],
    ) || (contains_any(text, &["配色", "色彩", "品牌色", "颜色"])
        && contains_any(text, &["趋势", "流行", "热门", "竞品", "参考", "对标"]));

    let has_fresh_or_evidence = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "最新",
                "近期",
                "当前",
                "今年",
                "2026",
                "公开来源",
                "公开资料",
                "参考",
                "对标",
                "竞品",
            ],
        );

    has_color_signal && has_fresh_or_evidence
}

fn has_design_material_policy_search_intent(text: &str) -> bool {
    let has_material_signal =
        contains_any(
            text,
            &[
                "素材规范",
                "素材规格",
                "广告素材规格",
                "广告位规格",
                "尺寸规范",
                "尺寸规则",
                "平台尺寸",
                "信息流尺寸",
                "上传规范",
                "文件大小",
                "安全区",
                "审核规范",
                "格式要求",
                "ad spec",
                "asset spec",
            ],
        ) || (contains_any(text, &["素材", "规格", "尺寸", "广告位", "格式"])
            && contains_any(
                text,
                &["规则", "规范", "要求", "审核", "平台", "官方", "最新"],
            ));

    let has_policy_or_fresh = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "最新",
                "近期",
                "当前",
                "今年",
                "2026",
                "规则",
                "规范",
                "要求",
                "平台",
                "官方",
                "公开来源",
                "公开资料",
            ],
        );

    has_material_signal && has_policy_or_fresh
}

fn is_design_pure_visual_search_intent(text: &str) -> bool {
    has_design_visual_trend_search_intent(text)
        && contains_any(
            text,
            &["查", "查一下", "搜索", "搜", "联网", "公开来源", "公开资料"],
        )
        && !contains_any(
            text,
            &[
                "做",
                "生成",
                "规划",
                "输出",
                "设计方案",
                "设计一版",
                "制作",
                "创建",
                "给方案",
                "出方案",
                "优化",
                "改版",
            ],
        )
}

fn is_design_pure_material_policy_search_intent(text: &str) -> bool {
    has_design_material_policy_search_intent(text)
        && contains_any(
            text,
            &["查", "查一下", "搜索", "搜", "联网", "公开来源", "公开资料"],
        )
        && !contains_any(
            text,
            &[
                "做",
                "生成",
                "规划",
                "输出",
                "整理",
                "设计方案",
                "制作",
                "创建",
                "给方案",
                "出方案",
                "交付",
            ],
        )
}

fn infer_design_visual_search_skill_id(text: &str) -> &'static str {
    if contains_any(text, &["海报", "banner", "Banner", "活动图", "活动海报"]) {
        "design_campaign_poster"
    } else if contains_any(text, &["详情页", "详情", "长图"]) {
        "design_detail_page"
    } else {
        "design_main_image"
    }
}

fn autosuggest_design_color_trend_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_trends",
        serde_json::Value::Object(design_color_trend_search_input(text, task)),
        "根据设计配色任务中的最新色彩趋势/竞品配色参考需求自动追加外部趋势搜索 skill",
    )
}

fn design_color_trend_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(design_color_trend_search_query(text, task)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_design_search_days(text, 90)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn design_color_trend_search_query(text: &str, task: &Task) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = design_visual_search_subject(text, task);
    let style = infer_design_style(text);
    format!("{platform} {subject} {style} 品牌配色 色彩趋势 流行色 竞品配色 2026")
}

fn autosuggest_design_material_policy_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_platform_policy",
        serde_json::Value::Object(design_material_policy_search_input(text, task)),
        "根据设计素材规范任务中的平台尺寸/审核/格式规则需求自动追加平台规则搜索 skill",
    )
}

fn design_material_policy_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_platform_policy_input(text, task);
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert("policy_type".to_string(), serde_json::json!("material"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_design_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn autosuggest_design_visual_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> ParsedSkillRequest {
    autosuggest_request(
        "search_trends",
        serde_json::Value::Object(design_visual_trend_search_input(
            text,
            task,
            native_skill_id,
        )),
        "根据设计任务中的最新视觉趋势/爆款参考需求自动追加外部趋势搜索 skill",
    )
}

fn design_visual_trend_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(design_visual_trend_search_query(
            text,
            task,
            native_skill_id
        )),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_design_search_days(
            text,
            design_visual_trend_default_days(native_skill_id),
        )),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn design_visual_trend_search_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = design_visual_search_subject(text, task);

    match native_skill_id {
        "design_campaign_poster" => {
            format!(
                "{} {platform} {subject} 活动海报 设计趋势 爆款 2026",
                infer_campaign_name(text)
            )
        }
        "design_detail_page" => {
            format!("{platform} {subject} 详情页设计 高转化 视觉趋势 2026")
        }
        _ => format!("{platform} {subject} 主图设计 爆款 高点击率 视觉趋势 2026"),
    }
}

fn design_visual_search_subject(text: &str, task: &Task) -> String {
    first_selected_product(task)
        .map(|product| product.name)
        .unwrap_or_else(|| selected_product_category_or_infer(task, text))
}

fn design_visual_trend_default_days(native_skill_id: &str) -> i64 {
    match native_skill_id {
        "design_campaign_poster" => 30,
        "design_detail_page" => 60,
        _ => 60,
    }
}

fn infer_design_search_days(text: &str, default: i64) -> i64 {
    if !numbers_in_text(text).is_empty()
        && contains_any(
            text,
            &["最近", "近", "过去", "天内", "日内", "周内", "月内"],
        )
    {
        infer_period_days(text)
    } else {
        default
    }
}
