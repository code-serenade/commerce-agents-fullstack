fn infer_system_type(text: &str) -> &'static str {
    policy_match_label(text, "engineering.system_type").unwrap_or("电商平台")
}

fn infer_bug_type(text: &str) -> &'static str {
    policy_match_label(text, "engineering.bug_type").unwrap_or("数据异常")
}

fn infer_bug_frequency(text: &str) -> &'static str {
    if contains_any(text, &["必现", "每次"]) {
        "必现"
    } else if contains_any(text, &["频繁", "经常"]) {
        "频繁"
    } else {
        "偶发"
    }
}

fn infer_perf_bottleneck(text: &str) -> &'static str {
    policy_match_label(text, "engineering.perf_bottleneck").unwrap_or("接口")
}

fn infer_stack_hint(text: &str) -> String {
    let mut parts = Vec::new();
    for stack in [
        "Rust", "Python", "FastAPI", "React", "Postgres", "MySQL", "Redis", "Docker", "Nginx",
    ] {
        if text.contains(stack)
            || text
                .to_ascii_lowercase()
                .contains(&stack.to_ascii_lowercase())
        {
            parts.push(stack.to_string());
        }
    }
    if parts.is_empty() {
        "未明确技术栈".to_string()
    } else {
        parts.join("/")
    }
}

fn infer_learning_topic(text: &str) -> String {
    if contains_any(text, &["ROI", "roi", "ROAS", "roas", "投产"]) {
        return "ROI和ROAS的区别".to_string();
    }
    if contains_any(text, &["CTR", "ctr", "点击率"]) {
        return "点击率CTR".to_string();
    }
    if contains_any(text, &["CVR", "cvr", "转化率"]) {
        return "转化率CVR".to_string();
    }
    if contains_any(text, &["DSR", "dsr", "店铺评分", "服务评分"]) {
        return "DSR店铺评分".to_string();
    }
    if contains_any(text, &["NPS", "nps", "净推荐值"]) {
        return "NPS净推荐值".to_string();
    }
    if contains_any(text, &["复购", "复购率"]) {
        return "复购率".to_string();
    }
    if contains_any(text, &["客单价", "AOV", "aov"]) {
        return "客单价AOV".to_string();
    }

    compact_task_text(text)
        .trim_start_matches("请")
        .trim_start_matches("帮我")
        .trim_start_matches("给我")
        .trim()
        .to_string()
}

fn infer_learning_level_from_text(text: &str) -> &'static str {
    if contains_any(text, &["本科", "大学", "undergraduate"]) {
        "undergraduate"
    } else if contains_any(text, &["小白", "新手", "入门", "零基础", "beginner"]) {
        "beginner"
    } else if contains_any(text, &["专家", "深入", "高级", "专业", "expert"]) {
        "expert"
    } else {
        "practitioner"
    }
}

fn infer_learning_scenario(text: &str) -> &'static str {
    if contains_any(
        text,
        &["投放", "广告", "千川", "ROI", "roi", "ROAS", "roas"],
    ) {
        "电商投放复盘"
    } else if contains_any(text, &["客服", "售后", "退款", "DSR", "dsr", "NPS", "nps"]) {
        "客服售后分析"
    } else if contains_any(
        text,
        &["转化", "详情页", "主图", "标题", "CTR", "ctr", "点击率"],
    ) {
        "商品转化优化"
    } else {
        "电商经营学习"
    }
}

fn infer_tech_domain(text: &str) -> &'static str {
    if contains_any(text, &["queue", "mq", "消息队列"]) {
        "消息队列"
    } else if contains_any(text, &["database", "数据库", "mysql", "postgres"]) {
        "数据库"
    } else if contains_any(text, &["cache", "缓存", "redis"]) {
        "缓存"
    } else if contains_any(text, &["search", "搜索", "elasticsearch"]) {
        "搜索引擎"
    } else if contains_any(text, &["monitor", "监控"]) {
        "监控"
    } else {
        "Web框架"
    }
}

fn infer_pain_points(text: &str) -> Vec<String> {
    let mut pains = Vec::new();
    for point in [
        "扩展性不足",
        "性能瓶颈",
        "接口超时",
        "并发压力",
        "运维复杂",
        "数据一致性风险",
    ] {
        if text.contains(point) {
            pains.push(point.to_string());
        }
    }
    if pains.is_empty() {
        pains.push("需要结合业务上下文进一步确认痛点".to_string());
    }
    pains
}

fn infer_status_query_type_from_text(text: &str) -> &'static str {
    if contains_any(text, &["workspace", "工作区"]) {
        "workspace_status"
    } else if contains_any(text, &["session", "会话"]) {
        "session_status"
    } else if contains_any(text, &["task", "任务", "节点", "进度"]) {
        "task_status"
    } else if contains_any(text, &["agent", "worker"]) {
        "agent_status"
    } else {
        "system_health"
    }
}

fn has_platform_health_check_intent(text: &str) -> bool {
    let mentions_platform = contains_any(
        text,
        &[
            "淘宝",
            "天猫",
            "抖音",
            "千川",
            "京东",
            "拼多多",
            "pdd",
            "jd",
            "taobao",
            "tmall",
            "douyin",
            "tiktok",
            "api",
            "API",
            "平台",
        ],
    );
    let mentions_health = contains_any(
        text,
        &[
            "接口健康",
            "健康检查",
            "连接状态",
            "连接配置",
            "连通性",
            "连通",
            "测试连接",
            "API健康",
            "api健康",
            "同步状态",
            "凭证",
            "密钥",
            "授权",
        ],
    );
    mentions_platform && mentions_health
}

fn infer_platform_health_check_platform(text: &str) -> String {
    let platform = infer_search_platform(text);
    if platform == "电商" {
        String::new()
    } else {
        platform.to_string()
    }
}
