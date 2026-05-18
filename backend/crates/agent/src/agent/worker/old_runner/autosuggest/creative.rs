fn autosuggest_creative_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_creative_native(text, task) {
        if is_creative_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_creative_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_creative_title_ctr(text, task)]
}

fn autosuggest_creative(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_creative_requests(text, task).into_iter().next()
}

fn autosuggest_creative_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(text, &["live", "直播"]) {
        let product_count = selected_product_count_or_default(task, 6);
        return Some(autosuggest_request(
            "creative_live_script",
            serde_json::json!({
                "duration_hours": infer_live_duration_hours(text),
                "product_count": product_count,
                "product_refs": selected_product_refs(task),
                "live_type": if contains_any(text, &["new", "新品"]) { "新品首发" } else { "日常带货" },
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据直播/内容任务自动补偿直播脚本 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "video",
            "short video",
            "短视频",
            "视频脚本",
            "拍摄脚本",
            "开箱",
            "测评",
        ],
    ) {
        return Some(autosuggest_request(
            "creative_video_script",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "video_type": infer_video_type(text),
                "duration": infer_video_duration_seconds(text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据短视频/脚本意图自动补偿短视频脚本 skill",
        ));
    }

    if contains_any(text, &["seeding", "种草", "笔记", "小红书", "分享文案"]) {
        return Some(autosuggest_request(
            "creative_seeding_copy",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "selling_points": selected_product_features_or_infer(task, text),
                "target_audience": infer_audience(text),
                "tone": if contains_any(text, &["测评", "专业"]) { "专业测评" } else { "闺蜜分享" },
            }),
            "根据种草/笔记意图自动补偿种草文案 skill",
        ));
    }

    if contains_any(text, &["ip", "IP", "人设", "定位", "账号定位", "品牌定位"]) {
        return Some(autosuggest_request(
            "creative_ip_branding",
            serde_json::json!({
                "brand_name": selected_product_brand_name_or_task(task),
                "category": selected_product_category_or_infer(task, text),
                "target_audience": infer_audience(text),
                "differentiator": "专业+有温度",
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据 IP/品牌定位意图自动补偿 IP 定位 skill",
        ));
    }

    if contains_any(
        text,
        &["calendar", "schedule", "排期", "日历", "内容计划", "选题"],
    ) {
        return Some(autosuggest_request(
            "creative_content_calendar",
            serde_json::json!({
                "period": if contains_any(text, &["月", "monthly", "month"]) { "月" } else { "周" },
                "platforms": [infer_platform(text), "小红书"],
                "weekly_posts": numbers_in_text(text).first().copied().unwrap_or(5.0).round().clamp(1.0, 14.0) as i64,
            }),
            "根据内容排期/选题意图自动补偿内容排期 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "article",
            "soft article",
            "软文",
            "长文",
            "推广文章",
            "公众号",
        ],
    ) {
        return Some(autosuggest_request(
            "creative_product_article",
            serde_json::json!({
                "product_name": selected_product_name_or_task(task),
                "product_ref": selected_product_ref(task),
                "article_type": if contains_any(text, &["对比"]) { "对比评测" } else if contains_any(text, &["攻略"]) { "使用攻略" } else { "测评软文" },
                "platform": selected_product_article_platform_or_default(task, text),
            }),
            "根据软文/推广文章意图自动补偿产品软文 skill",
        ));
    }

    if contains_any(text, &["trend", "hot", "趋势", "热点", "爆款", "流行"]) {
        return Some(autosuggest_request(
            "creative_trend_catch",
            serde_json::json!({
                "category": selected_product_category_or_infer(task, text),
                "platform": selected_product_platform_or_infer(task, text),
            }),
            "根据趋势/热点意图自动补偿趋势捕捉 skill",
        ));
    }

    None
}

fn autosuggest_creative_title_ctr(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "creative_title_ctr_scorer",
        serde_json::json!({
            "title": extract_title_candidate(text).unwrap_or_else(|| compact_task_text(&task.description)),
            "platform": infer_platform(text),
        }),
        "根据创意岗位任务自动补偿标题 CTR 评分 skill",
    )
}

fn is_creative_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    let has_fresh_or_external = has_external_search_intent(text)
        || contains_any(
            text,
            &[
                "当前",
                "近期",
                "今年",
                "今日",
                "今天",
                "热搜",
                "榜单",
                "站外",
                "公开来源",
                "公开资料",
            ],
        );

    match native_skill_id {
        "creative_video_script" | "creative_seeding_copy" => {
            (has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "爆款", "热门", "热点", "趋势", "风格", "标签", "话题", "内容",
                    ],
                ))
                || contains_any(
                    text,
                    &[
                        "最新爆款",
                        "实时爆款",
                        "热门风格",
                        "爆款话题",
                        "热搜",
                        "榜单",
                    ],
                )
        }
        "creative_content_calendar" => {
            has_fresh_or_external
                || contains_any(
                    text,
                    &[
                        "营销节点",
                        "内容节点",
                        "热门话题",
                        "热点话题",
                        "节日热点",
                        "节日营销",
                    ],
                )
        }
        "creative_trend_catch" => {
            has_fresh_or_external
                || contains_any(text, &["最新趋势", "实时趋势", "热搜", "榜单", "热点话题"])
        }
        "creative_live_script" => {
            (has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "直播",
                        "直播间",
                        "带货",
                        "场控",
                        "话术",
                        "玩法",
                        "互动",
                        "转化",
                        "爆款",
                        "热门",
                        "节奏",
                        "脚本",
                    ],
                ))
                || contains_any(
                    text,
                    &[
                        "热门直播间",
                        "爆款直播",
                        "直播间玩法",
                        "直播间脚本",
                        "实时直播",
                        "最新直播",
                    ],
                )
        }
        "creative_product_article" => {
            (has_fresh_or_external
                && contains_any(
                    text,
                    &[
                        "软文",
                        "长文",
                        "公众号",
                        "推广文章",
                        "文章",
                        "内容",
                        "风格",
                        "话题",
                        "爆款",
                        "热门",
                        "种草",
                        "测评",
                        "攻略",
                        "对比",
                        "公开资料",
                    ],
                ))
                || contains_any(
                    text,
                    &[
                        "热门软文",
                        "爆款软文",
                        "爆款文章",
                        "热门文章",
                        "最新软文",
                        "最新文章",
                        "公众号爆款",
                    ],
                )
        }
        _ => false,
    }
}

fn autosuggest_creative_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> ParsedSkillRequest {
    match native_skill_id {
        "creative_content_calendar" => autosuggest_request(
            "search_market_info",
            serde_json::Value::Object(creative_calendar_market_search_input(text, task)),
            "根据内容排期中的营销节点/热门话题需求自动追加外部市场搜索 skill",
        ),
        _ => autosuggest_request(
            "search_trends",
            serde_json::Value::Object(creative_trend_search_input(text, task, native_skill_id)),
            "根据创意内容中的实时爆款/趋势需求自动追加外部趋势搜索 skill",
        ),
    }
}

fn creative_trend_search_input(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_trends_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(creative_trend_search_query(text, task, native_skill_id)),
    );
    input.insert(
        "platform".to_string(),
        serde_json::json!(selected_product_platform_or_infer(task, text)),
    );
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_creative_search_days(text, 14)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn creative_calendar_market_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(format!(
            "电商营销节日 热门话题 内容节点 {} 2026",
            selected_product_platform_or_infer(task, text)
        )),
    );
    input.insert("topic".to_string(), serde_json::json!("news"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_creative_search_days(text, 30)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn creative_trend_search_query(text: &str, task: &Task, native_skill_id: &str) -> String {
    let platform = selected_product_platform_or_infer(task, text);
    let subject = infer_creative_search_subject(text, task);

    match native_skill_id {
        "creative_video_script" => {
            format!(
                "{platform} {subject} 爆款视频 {} 热门内容 2026",
                infer_video_type(text)
            )
        }
        "creative_seeding_copy" => {
            format!("{platform} {subject} 种草文案 热门风格 爆款话题 2026")
        }
        "creative_trend_catch" => {
            let category = selected_product_category_or_infer(task, text);
            format!("{category} {platform} 热门内容趋势 爆款 2026")
        }
        "creative_live_script" => {
            format!("{platform} {subject} 直播带货 直播间玩法 场控话术 转化节奏 2026")
        }
        "creative_product_article" => {
            format!("{platform} {subject} 产品软文 爆款文章 推广长文 内容风格 2026")
        }
        _ => format!("{subject} {platform} 内容趋势 热点 爆款 2026"),
    }
}

fn infer_creative_search_subject(text: &str, task: &Task) -> String {
    if let Some(product) = first_selected_product(task) {
        return product.name;
    }

    for prefix in ["请帮", "帮", "给", "为"] {
        let Some((_, rest)) = text.split_once(prefix) else {
            continue;
        };
        let rest = rest.trim_start();
        let stop = [
            "写", "做", "生成", "排", "分析", "设计", "出", "，", ",", "。", "；", ";", " ",
        ]
        .iter()
        .filter_map(|token| rest.find(token))
        .min();
        let Some(stop) = stop else {
            continue;
        };
        let candidate = rest[.. stop].trim();
        let char_count = candidate.chars().count();
        if (2 ..= 20).contains(&char_count) {
            return candidate.to_string();
        }
    }

    if let Some(subject) = extract_creative_object_subject(text) {
        return subject;
    }

    infer_search_query_for_task(text, task)
}

fn extract_creative_object_subject(text: &str) -> Option<String> {
    let action_index = ["写", "生成", "制作", "做", "出"]
        .iter()
        .filter_map(|token| text.find(token).map(|index| index + token.len()))
        .min()?;
    let rest = text[action_index ..].trim_start_matches(['一', '套', '篇', '个', '条', '份']);
    let stop = [
        "直播",
        "短视频",
        "视频",
        "脚本",
        "种草",
        "笔记",
        "软文",
        "长文",
        "推广文章",
        "公众号",
        "话术",
        "文案",
        "，",
        ",",
        "。",
        "；",
        ";",
    ]
    .iter()
    .filter_map(|token| rest.find(token))
    .min()?;
    let candidate = rest[.. stop].trim();
    let char_count = candidate.chars().count();
    if (2 ..= 20).contains(&char_count) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn infer_creative_search_days(text: &str, default: i64) -> i64 {
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

fn infer_live_duration_hours(text: &str) -> f64 {
    let Some(value) = numbers_in_text(text).first().copied() else {
        return 3.0;
    };

    if contains_any(text, &["分钟", "min", "minute", "minutes"]) {
        return (value / 60.0).clamp(0.5, 12.0);
    }

    if contains_any(text, &["小时", "个小时", "直播时长", "hour", "hours"]) {
        return value.clamp(0.5, 12.0);
    }

    3.0
}
