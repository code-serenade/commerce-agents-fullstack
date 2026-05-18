fn autosuggest_douyin_ads_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_douyin_ads_native(text, task) {
        if is_douyin_ads_search_chain_intent(text, &primary.skill_id) {
            let search = autosuggest_douyin_ads_search(text, task, &primary.skill_id);
            return vec![primary, search];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    vec![autosuggest_request(
        "douyin_ads_plan_structure",
        serde_json::Value::Object(infer_douyin_ads_plan_structure_input(text, task)),
        "根据千川投放岗位任务自动补偿计划结构 skill",
    )]
}

fn autosuggest_douyin_ads(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_douyin_ads_requests(text, task)
        .into_iter()
        .next()
}

fn autosuggest_douyin_ads_native(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    if contains_any(
        text,
        &[
            "直播间",
            "进房",
            "场观",
            "商品点击",
            "下单",
            "付款",
            "漏斗",
            "GPM",
            "gpm",
            "live room",
            "funnel",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_live_funnel_diagnosis",
            serde_json::Value::Object(infer_douyin_ads_live_funnel_input(text, task)),
            "根据直播间投放漏斗/进房/付款转化意图自动补偿千川直播漏斗诊断 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "人群包",
            "定向",
            "人群",
            "相似人群",
            "达人粉丝",
            "再营销",
            "DMP",
            "dmp",
            "audience",
            "targeting",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_audience_package",
            serde_json::Value::Object(infer_douyin_ads_audience_input(text, task)),
            "根据千川人群包/定向意图自动补偿千川人群包设计 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "计划结构",
            "投放计划",
            "冷启动",
            "赛马",
            "短视频引流",
            "campaign",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_plan_structure",
            serde_json::Value::Object(infer_douyin_ads_plan_structure_input(text, task)),
            "根据千川投放/计划结构/冷启动意图自动补偿千川计划结构 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "预算", "消耗", "花费", "节奏", "放量", "稳投", "收缩", "止损", "复盘", "投产", "ROI",
            "roi", "ROAS", "roas", "pacing", "scale",
        ],
    ) {
        return Some(autosuggest_request(
            "douyin_ads_budget_pacing",
            serde_json::Value::Object(infer_douyin_ads_budget_pacing_input(text, task)),
            "根据千川预算节奏/ROI/ROAS意图自动补偿千川预算节奏 skill",
        ));
    }

    if contains_any(text, &["千川", "巨量千川", "投流", "投放"]) {
        return Some(autosuggest_request(
            "douyin_ads_plan_structure",
            serde_json::Value::Object(infer_douyin_ads_plan_structure_input(text, task)),
            "根据千川投放任务自动补偿千川计划结构 skill",
        ));
    }

    None
}

fn is_douyin_ads_search_chain_intent(text: &str, native_skill_id: &str) -> bool {
    match native_skill_id {
        "douyin_ads_plan_structure" => {
            has_external_search_intent(text)
                || contains_any(text, &["最新投放规则", "千川规则", "平台规则", "流量规则"])
        }
        "douyin_ads_audience_package" => {
            has_external_search_intent(text)
                || contains_any(text, &["最新人群趋势", "人群趋势", "兴趣趋势"])
        }
        "douyin_ads_budget_pacing" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &["行业ROI", "行业 ROI", "行业ROAS", "行业 ROAS", "基准"],
                )
        }
        "douyin_ads_live_funnel_diagnosis" => {
            has_external_search_intent(text)
                || contains_any(text, &["直播间基准", "直播转化基准", "GPM基准", "gpm基准"])
        }
        _ => has_external_search_intent(text),
    }
}

fn autosuggest_douyin_ads_search(
    text: &str,
    task: &Task,
    native_skill_id: &str,
) -> ParsedSkillRequest {
    match native_skill_id {
        "douyin_ads_plan_structure"
            if is_platform_policy_search_intent(text)
                || contains_any(text, &["平台规则", "千川规则", "流量规则", "审核规则"]) =>
        {
            let mut input = search_platform_policy_input(text, task);
            input.insert(
                "platform".to_string(),
                serde_json::json!(selected_product_platform_or_infer(task, text)),
            );
            input.insert("policy_type".to_string(), serde_json::json!("algorithm"));
            autosuggest_request(
                "search_platform_policy",
                serde_json::Value::Object(input),
                "根据千川平台规则/流量规则诉求自动追加平台规则搜索 skill",
            )
        }
        "douyin_ads_budget_pacing" | "douyin_ads_live_funnel_diagnosis" => {
            let mut input = search_industry_benchmarks_input(text, task);
            input.insert("metric".to_string(), serde_json::json!("roi"));
            input.insert(
                "platform".to_string(),
                serde_json::json!(selected_product_platform_or_infer(task, text)),
            );
            autosuggest_request(
                "search_industry_benchmarks",
                serde_json::Value::Object(input),
                "根据千川ROI/直播转化基准诉求自动追加行业基准搜索 skill",
            )
        }
        "douyin_ads_audience_package" => {
            let mut input = search_trends_input(text, task);
            input.insert(
                "query".to_string(),
                serde_json::json!(format!(
                    "{} 抖音 千川 人群 兴趣 趋势",
                    selected_product_category_or_infer(task, text)
                )),
            );
            input.insert("platform".to_string(), serde_json::json!("抖音"));
            autosuggest_request(
                "search_trends",
                serde_json::Value::Object(input),
                "根据千川人群/兴趣趋势诉求自动追加趋势搜索 skill",
            )
        }
        _ => {
            let mut input = search_market_info_input(text, task);
            input.insert(
                "query".to_string(),
                serde_json::json!(format!(
                    "{} 抖音 千川 投放 策略",
                    selected_product_category_or_infer(task, text)
                )),
            );
            autosuggest_request(
                "search_market_info",
                serde_json::Value::Object(input),
                "根据千川投放外部资料诉求自动追加市场资料搜索 skill",
            )
        }
    }
}
