fn build_trends_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_else(|| "电商".to_owned());
    let days = optional_i64_param(params, "days")?.unwrap_or(30);
    let planned = format!("{query} {platform} 热销 趋势 爆款 2026 最近{days}天");
    Ok(SearchPlan::new(query, planned, "general", days))
}

fn build_competitor_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_default();
    let focus = optional_string_param(params, "focus")?.unwrap_or_default();
    let focus_text = match focus.as_str() {
        "price" => "价格 定价 价位 售价",
        "promotion" => "促销 活动 优惠 折扣 限时",
        "reviews" => "用户评价 口碑 差评 真实体验",
        "features" => "卖点 功能特点 优势 核心参数",
        _ => "价格 卖点 评价 优惠 竞品对比",
    };
    let planned = format!("{query} {platform} {focus_text} 2026");
    Ok(SearchPlan::new(query, planned, "general", 60))
}

fn build_market_info_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let days = optional_i64_param(params, "days")?.unwrap_or(7);
    let topic = optional_string_param(params, "topic")?.unwrap_or_else(|| "news".to_owned());
    let recency = if days <= 7 {
        "最新".to_owned()
    } else {
        format!("{days}天内")
    };
    let planned = format!("{query} {recency} 行业动态 市场分析 2026");
    Ok(SearchPlan::new(query, planned, topic, days))
}

fn build_reviews_query(
    params: &Value,
    query: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_default();
    let focus = optional_string_param(params, "focus")?.unwrap_or_else(|| "overall".to_owned());
    let focus_text = match focus.as_str() {
        "positive" => "好评 亮点 优点 推荐 满意 值得买",
        "negative" => "差评 缺点 投诉 退货 踩雷 槽点",
        _ => "用户评价 口碑 真实反馈 购买体验 评测",
    };
    let planned = format!("{query} {platform} {focus_text} 2026");
    Ok(SearchPlan::new(query, planned, "general", 90))
}

fn build_policy_query(
    params: &Value,
    platform: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let policy_type =
        optional_string_param(params, "policy_type")?.unwrap_or_else(|| "general".to_owned());
    let days = optional_i64_param(params, "days")?.unwrap_or(30);
    let type_text = match policy_type.as_str() {
        "algorithm" => "搜索算法 流量规则 排名机制 权重调整",
        "fee" => "佣金 费率 扣点 服务费 收费标准",
        "promotion" => "大促规则 活动报名 双11 618 商家政策",
        "content" => "内容规范 违禁词 审核标准 发布规定 违规处罚",
        "material" => "素材尺寸 广告位规格 上传格式 文件大小 安全区 审核要求",
        _ => "规则变化 政策更新 新规 公告",
    };
    let planned = format!("{platform} {type_text} 最新 2026");
    Ok(SearchPlan::new(platform, planned, "news", days))
}

fn build_benchmark_query(
    params: &Value,
    industry: &str,
) -> Result<SearchPlan, crate::skills::SkillError> {
    let platform = optional_string_param(params, "platform")?.unwrap_or_else(|| "电商".to_owned());
    let metric = optional_string_param(params, "metric")?.unwrap_or_else(|| "general".to_owned());
    let metric_text = match metric.as_str() {
        "conversion" => "转化率 成交转化 CVR 行业均值 基准",
        "aov" => "客单价 平均订单金额 消费水平 均值",
        "nps" => "NPS 净推荐值 客户满意度 行业标准 平均分",
        "dsr" => "DSR 店铺评分 描述相符 服务态度 物流服务 行业基准",
        "refund" => "退款率 退货率 售后率 行业水平 均值",
        "roi" => "广告ROI 投产比 ROAS 营销效率 基准",
        _ => "行业数据 KPI基准 平均水平 运营指标 数据报告",
    };
    let planned = format!("{industry} {platform} {metric_text} 2025 2026");
    Ok(SearchPlan::new(industry, planned, "general", 180))
}
