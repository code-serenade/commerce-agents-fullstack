#[allow(dead_code)]
pub(super) fn agent_matches(task: &str) -> Vec<(String, String, i64)> {
    const AGENTS: &[(&str, &str, &[&str])] = &[
        (
            "ops",
            "运营专家",
            &[
                "运营", "促销", "活动", "推广", "投放", "选品", "定价", "库存",
            ],
        ),
        (
            "data",
            "数据分析师",
            &[
                "数据",
                "分析",
                "报表",
                "漏斗",
                "转化率",
                "趋势",
                "异常",
                "指标",
            ],
        ),
        (
            "service",
            "客服专家",
            &[
                "客服", "工单", "投诉", "退款", "退货", "售后", "DSR", "评价",
            ],
        ),
        (
            "design",
            "设计师",
            &["设计", "主图", "详情页", "配色", "海报", "视觉", "素材"],
        ),
        (
            "accounting",
            "财务分析师",
            &["财务", "成本", "利润", "预算", "ROI", "税务", "合规"],
        ),
        (
            "engineering",
            "技术工程师",
            &["技术", "架构", "bug", "性能", "部署", "开发", "接口"],
        ),
        (
            "web",
            "SEO专家",
            &["SEO", "网站", "店铺装修", "关键词", "标题", "转化", "建站"],
        ),
        (
            "creative",
            "内容创作者",
            &["视频", "文案", "种草", "直播", "IP", "内容", "创意"],
        ),
    ];

    AGENTS
        .iter()
        .copied()
        .map(|(agent, name, keywords)| {
            let score = keywords
                .iter()
                .filter(|keyword| task.contains(**keyword))
                .count() as i64;
            (agent.to_owned(), name.to_owned(), score)
        })
        .filter(|(_, _, score)| *score > 0)
        .collect()
}
