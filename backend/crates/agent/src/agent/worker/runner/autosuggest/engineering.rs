fn autosuggest_engineering_requests(text: &str, task: &Task) -> Vec<ParsedSkillRequest> {
    if let Some(primary) = autosuggest_engineering_native(text) {
        if is_engineering_external_knowledge_search_chain_intent(text, &primary.skill_id) {
            return vec![
                primary,
                autosuggest_engineering_knowledge_search(text, task),
            ];
        }
        return vec![primary];
    }

    if let Some(request) = autosuggest_search(text, task) {
        return vec![request];
    }

    Vec::new()
}

fn autosuggest_engineering(text: &str, task: &Task) -> Option<ParsedSkillRequest> {
    autosuggest_engineering_requests(text, task)
        .into_iter()
        .next()
}

fn autosuggest_engineering_native(text: &str) -> Option<ParsedSkillRequest> {
    if has_platform_health_check_intent(text) {
        let test_live = contains_any(text, &["实际", "真实", "连通", "连通性", "测试", "live"]);
        return Some(autosuggest_request(
            "engineering_system_check",
            serde_json::json!({
                "platform": infer_platform_health_check_platform(text),
                "test_live": test_live,
            }),
            if test_live {
                "根据平台接口连通性意图自动补偿平台健康检查 skill，实际检测需审批"
            } else {
                "根据平台连接配置检查意图自动补偿平台健康检查 skill"
            },
        ));
    }

    if contains_any(
        text,
        &[
            "状态",
            "status",
            "进度",
            "session",
            "会话",
            "workspace",
            "工作区",
            "run",
            "任务节点",
            "系统健康",
            "health",
            "agent运行",
        ],
    ) {
        return Some(autosuggest_request(
            "coordination_status_query",
            serde_json::json!({
                "query_type": infer_status_query_type_from_text(text),
            }),
            "根据运行状态/系统健康意图自动补偿协调状态查询 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "architecture",
            "架构",
            "重构",
            "扩展性",
            "系统设计",
            "系统架构",
        ],
    ) {
        return Some(autosuggest_request(
            "engineering_arch_review",
            serde_json::json!({
                "system_type": infer_system_type(text),
                "daily_orders": infer_integer_or_default(text, 1000),
                "pain_points": infer_pain_points(text),
            }),
            "根据架构评估意图自动补偿架构评估 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "bug",
            "故障",
            "报错",
            "异常报错",
            "排查",
            "白屏",
            "错误日志",
        ],
    ) {
        return Some(autosuggest_request(
            "engineering_bug_analysis",
            serde_json::json!({
                "error_type": infer_bug_type(text),
                "context": compact_task_text(text),
                "frequency": infer_bug_frequency(text),
            }),
            "根据故障排查意图自动补偿Bug排查 skill",
        ));
    }

    if contains_any(
        text,
        &[
            "performance",
            "性能",
            "慢",
            "卡",
            "延迟",
            "qps",
            "吞吐",
            "优化",
        ],
    ) && !has_tech_selection_intent(text)
    {
        let numbers = numbers_in_text(text);
        return Some(autosuggest_request(
            "engineering_perf_optimize",
            serde_json::json!({
                "bottleneck": infer_perf_bottleneck(text),
                "current_qps": numbers.first().copied().unwrap_or(500.0),
                "target_qps": numbers.get(1).copied().unwrap_or(2_000.0),
                "tech_stack": infer_stack_hint(text),
            }),
            "根据性能优化意图自动补偿性能优化 skill",
        ));
    }

    if has_tech_selection_intent(text) {
        return Some(autosuggest_request(
            "engineering_tech_selection",
            serde_json::json!({
                "domain": infer_tech_domain(text),
                "current_stack": infer_stack_hint(text),
                "constraints": infer_pain_points(text),
            }),
            "根据技术选型意图自动补偿技术选型 skill",
        ));
    }

    Some(autosuggest_request(
        "engineering_sla_monitor",
        serde_json::json!({}),
        "根据工程岗位任务自动补偿 SLA 监控 skill",
    ))
}

fn has_tech_selection_intent(text: &str) -> bool {
    contains_any(
        text,
        &[
            "tech selection",
            "技术选型",
            "框架选择",
            "数据库选型",
            "选型",
        ],
    )
}

fn is_engineering_external_knowledge_search_chain_intent(
    text: &str,
    native_skill_id: &str,
) -> bool {
    match native_skill_id {
        "engineering_bug_analysis" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "已知解决方案",
                        "解决方案",
                        "最佳实践",
                        "公开资料",
                        "公开来源",
                        "社区案例",
                        "案例",
                    ],
                )
        }
        "engineering_tech_selection" => {
            has_external_search_intent(text)
                || contains_any(
                    text,
                    &[
                        "benchmark",
                        "Benchmark",
                        "基准测试",
                        "性能对比",
                        "最新对比",
                        "技术对比",
                        "公开资料",
                        "公开来源",
                    ],
                )
        }
        "engineering_perf_optimize" | "engineering_arch_review" => {
            has_external_search_intent(text)
                && contains_any(
                    text,
                    &[
                        "benchmark",
                        "Benchmark",
                        "基准",
                        "最佳实践",
                        "公开资料",
                        "公开来源",
                        "性能资料",
                        "架构案例",
                        "案例",
                    ],
                )
        }
        _ => false,
    }
}

fn autosuggest_engineering_knowledge_search(text: &str, task: &Task) -> ParsedSkillRequest {
    autosuggest_request(
        "search_market_info",
        serde_json::Value::Object(engineering_knowledge_search_input(text, task)),
        "根据工程任务中的外部资料/benchmark/最佳实践需求自动追加外部市场资料搜索 skill",
    )
}

fn engineering_knowledge_search_input(
    text: &str,
    task: &Task,
) -> serde_json::Map<String, serde_json::Value> {
    let mut input = search_market_info_input(text, task);
    input.insert(
        "query".to_string(),
        serde_json::json!(engineering_knowledge_search_query(text)),
    );
    input.insert("topic".to_string(), serde_json::json!("general"));
    input.insert(
        "days".to_string(),
        serde_json::json!(infer_engineering_search_days(text, 90)),
    );
    input.insert("_needs_fresh_data".to_string(), serde_json::json!(true));
    input
}

fn engineering_knowledge_search_query(text: &str) -> String {
    if contains_any(
        text,
        &[
            "bug",
            "故障",
            "报错",
            "异常报错",
            "排查",
            "白屏",
            "错误日志",
        ],
    ) {
        return format!(
            "{} {} 解决方案 排查 最佳实践 2026",
            infer_bug_type(text),
            infer_stack_hint(text)
        );
    }

    if has_tech_selection_intent(text) {
        return format!(
            "{} 技术选型 对比 性能 benchmark 2026",
            infer_tech_domain(text)
        );
    }

    if contains_any(
        text,
        &[
            "performance",
            "性能",
            "慢",
            "卡",
            "延迟",
            "qps",
            "吞吐",
            "优化",
        ],
    ) {
        return format!(
            "{} {} 性能优化 benchmark 最佳实践 2026",
            infer_stack_hint(text),
            infer_perf_bottleneck(text)
        );
    }

    format!(
        "{} {} 架构 benchmark 最佳实践 2026",
        infer_system_type(text),
        infer_stack_hint(text)
    )
}

fn infer_engineering_search_days(text: &str, default: i64) -> i64 {
    infer_explicit_period_days(text).unwrap_or(default)
}
