use serde_json::{Value, json};

#[allow(dead_code)]
pub(super) fn orchestration_template(goal: &str) -> Vec<Value> {
    let launch = goal.contains("新品") || goal.contains("上新");
    let promo = goal.contains("大促") || goal.contains("活动") || goal.contains("618");
    let rows = if launch {
        vec![
            ("竞品分析与市场调研", "data", "2天", "无", "竞品报告"),
            (
                "定价策略与成本核算",
                "accounting",
                "1天",
                "任务1",
                "定价方案",
            ),
            ("主图与详情页设计", "design", "3天", "任务1", "设计稿"),
            ("商品文案与SEO标题", "web", "1天", "任务3", "优化文案"),
            ("种草内容与脚本", "creative", "2天", "任务3", "内容素材"),
            ("客服话术准备", "service", "1天", "任务4", "话术库"),
            ("上线推广与监控", "ops", "持续", "任务4,5,6", "上线运营"),
        ]
    } else if promo {
        vec![
            ("历史数据分析与目标设定", "data", "1天", "无", "数据报告"),
            ("活动策划与预算分配", "ops", "2天", "任务1", "活动方案"),
            (
                "财务预算与盈亏测算",
                "accounting",
                "1天",
                "任务2",
                "预算报告",
            ),
            ("活动海报与素材设计", "design", "2天", "任务2", "视觉素材"),
            ("直播脚本与种草内容", "creative", "1天", "任务2", "内容脚本"),
            ("客服应急话术与预案", "service", "1天", "任务2", "应急手册"),
            (
                "技术压测与系统保障",
                "engineering",
                "1天",
                "任务2",
                "压测报告",
            ),
        ]
    } else {
        vec![
            ("现状分析与数据调研", "data", "1天", "无", "分析报告"),
            ("策略与方案制定", "ops", "1天", "任务1", "执行方案"),
            ("财务可行性评估", "accounting", "1天", "任务2", "财务测算"),
            ("视觉设计", "design", "2天", "任务2", "设计稿"),
            ("内容与文案制作", "creative", "2天", "任务2", "内容素材"),
            ("执行落地与效果监控", "ops", "持续", "任务3,4,5", "执行报告"),
        ]
    };
    rows.into_iter()
        .enumerate()
        .map(|(idx, (task, agent, duration, dep, output))| {
            json!({
                "序号": idx + 1,
                "任务": task,
                "Agent": agent,
                "耗时": duration,
                "依赖": dep,
                "目标产出": output
            })
        })
        .collect()
}

pub(super) fn skill_chain_template(
    goal: &str,
    role: &str,
    action: &str,
) -> Vec<(&'static str, &'static str)> {
    let goal_lower = goal.to_lowercase();
    if goal_lower.contains("nps驱动")
        || goal.contains("投诉根因")
        || goal.contains("满意度根因")
        || goal.contains("贬低者")
    {
        return vec![
            ("query_store_metrics", "加载退款、评价和客户满意度相关数据"),
            ("service_nps_driver_analysis", "量化投诉类别对NPS的拖累"),
            ("service_sentiment_analyzer", "识别高危负面评价和情绪强度"),
            ("service_dsr_improvement", "生成DSR与售后改善动作"),
        ];
    }
    if goal_lower.contains("nps") || goal.contains("满意度") {
        return vec![
            ("query_store_metrics", "加载服务、退款和评价基线"),
            ("service_nps_analyzer", "计算NPS/CSAT基线"),
            ("service_nps_driver_analysis", "识别满意度驱动因素"),
            ("service_dsr_improvement", "生成服务改善动作"),
        ];
    }
    if goal_lower.contains("sla")
        || goal_lower.contains("p99")
        || goal_lower.contains("p90")
        || goal.contains("错误预算")
    {
        return vec![
            ("engineering_sla_monitor", "计算延迟分位数和错误预算"),
            ("engineering_perf_optimize", "基于SLA问题制定性能优化方案"),
            (
                "engineering_system_check",
                "检查平台/接口状态并进入审批确认",
            ),
        ];
    }
    if goal_lower.contains("ctr")
        || goal.contains("点击率")
        || goal.contains("标题吸引力")
        || goal.contains("情感触发")
    {
        return vec![
            ("creative_title_ctr_scorer", "评分标题吸引力和情感触发词"),
            ("creative_seeding_copy", "基于评分结果优化种草文案"),
            ("web_title_seo_scorer", "同步检查标题SEO质量"),
        ];
    }
    if goal_lower.contains("abc")
        || goal_lower.contains("xyz")
        || goal.contains("SKU分类")
        || goal.contains("库存矩阵")
    {
        return vec![
            ("query_store_metrics", "加载SKU销量、库存和波动数据"),
            ("ops_abc_xyz_classifier", "生成ABC-XYZ双维分类矩阵"),
            (
                "ops_inventory_optimizer",
                "按分类结果制定安全库存和补货策略",
            ),
        ];
    }
    if goal.contains("广告疲劳")
        || goal.contains("投放疲劳")
        || goal.contains("素材更新")
        || goal.contains("创意刷新")
    {
        return vec![
            ("query_store_metrics", "加载广告花费、点击率、ROI和UV数据"),
            ("ops_ad_fatigue_detector", "识别疲劳渠道和CTR衰减"),
            ("data_channel_roi", "对比渠道ROI，确认加码或收缩方向"),
            ("coordination_create_task", "创建素材更新或预算调整任务"),
        ];
    }
    if goal_lower.contains("rfm") || goal.contains("客群分层") || goal.contains("客户分层")
    {
        return vec![
            ("query_store_metrics", "加载客户交易和复购数据"),
            ("data_customer_segmentation", "生成RFM分层和高价值客群标签"),
            ("data_ltv_calculator", "计算不同客群LTV/CAC健康度"),
            (
                "coordination_save_memory",
                "保存高价值客群特征供后续对话复用",
            ),
        ];
    }
    if goal_lower.contains("seo") || goal.contains("关键词") || role == "web" {
        return vec![
            ("query_store_metrics", "加载当前搜索流量和转化基线"),
            ("web_keyword_research", "建立关键词矩阵"),
            ("web_title_seo_scorer", "评分当前标题"),
            ("web_seo_optimize", "输出优化方案"),
        ];
    }
    if goal.contains("设计") || goal.contains("主图") || role == "design" {
        return vec![
            ("design_material_spec", "确定素材规格"),
            ("design_main_image", "生成主图方案"),
            ("design_detail_page", "规划详情页"),
        ];
    }
    if goal.contains("技术") || goal_lower.contains("bug") || role == "engineering" {
        return vec![
            ("engineering_bug_analysis", "定位技术问题"),
            ("engineering_perf_optimize", "给出优化方案"),
            (
                "engineering_system_check",
                "检查平台/接口状态并进入审批确认",
            ),
        ];
    }
    if goal.contains("现金流") || goal.contains("预算差异") || goal.contains("预实差异")
    {
        return vec![
            ("query_store_metrics", "加载收入、支出和预算执行数据"),
            ("accounting_cash_flow_forecast", "预测未来现金流压力"),
            ("accounting_budget_vs_actual", "定位预算与实际差异"),
            ("accounting_financial_narrative", "生成财务健康诊断"),
        ];
    }
    if goal.contains("财务")
        || goal.contains("利润")
        || goal.contains("成本")
        || goal.contains("亏损")
    {
        return vec![
            ("query_store_metrics", "加载GMV、费用和利润相关数据"),
            ("accounting_pl_statement", "计算P&L利润表"),
            ("accounting_financial_narrative", "生成财务健康度诊断"),
        ];
    }
    if goal.contains("预测")
        || goal.contains("未来")
        || goal.contains("下月")
        || goal.contains("下季")
    {
        return vec![
            ("query_store_metrics", "加载历史时序数据"),
            ("data_demand_forecast", "预测订单或GMV需求"),
            ("data_seasonal_decompose", "分离季节性噪音和真实趋势"),
            ("data_cohort_analysis", "检查留存队列变化"),
        ];
    }
    if goal.contains("库存")
        || goal.contains("补货")
        || goal.contains("滞销")
        || goal.contains("积压")
    {
        return vec![
            ("query_store_metrics", "加载销量、库存和周转数据"),
            ("ops_inventory_optimizer", "计算补货、安全库存和滞销风险"),
            ("data_demand_forecast", "预测后续需求"),
            ("ops_smart_pricing", "如库存积压，制定清仓或利润定价方案"),
        ];
    }
    if goal.contains("促销")
        || goal.contains("大促")
        || goal.contains("618")
        || goal.contains("双11")
    {
        return vec![
            ("query_store_metrics", "加载当前GMV基线和广告ROI"),
            ("ops_promo_planning", "制定促销活动方案"),
            ("creative_seeding_copy", "生成活动配套种草文案"),
            ("coordination_create_campaign", "将活动沉淀为营销活动草稿"),
        ];
    }
    if goal.contains("新品")
        || goal.contains("上新")
        || goal.contains("选品")
        || goal.contains("商品")
    {
        return vec![
            ("ops_listing_copy", "生成商品标题、卖点和上架文案"),
            ("design_main_image", "生成主图设计方案"),
            ("creative_video_script", "生成首批种草视频脚本"),
            ("web_seo_optimize", "补齐搜索关键词和详情页SEO方向"),
        ];
    }
    if role == "data" || goal.contains("数据") || goal.contains("指标") || goal.contains("异常")
    {
        return vec![
            ("query_store_metrics", "加载多维度经营指标"),
            ("data_anomaly_diagnosis", "检测异常指标并定位根因"),
            ("data_multi_period_trend", "比较多周期趋势和拐点"),
        ];
    }
    if role == "accounting" {
        return vec![
            ("query_store_metrics", "加载财务分析所需经营指标"),
            ("accounting_pl_statement", "计算P&L利润表"),
            ("accounting_financial_narrative", "输出财务健康诊断"),
        ];
    }
    if role == "service" {
        return vec![
            ("query_store_metrics", "加载退款率、评价和客服相关指标"),
            ("service_sentiment_analyzer", "分析评价和投诉情绪强度"),
            ("service_faq_playbook", "生成标准化FAQ和应答口径"),
            ("service_dsr_improvement", "输出DSR改善方案"),
        ];
    }
    if role == "creative" {
        return vec![
            ("query_store_metrics", "了解商品和平台表现基线"),
            ("creative_video_script", "生成短视频脚本"),
            ("creative_seeding_copy", "生成配套种草文案"),
        ];
    }
    if action == "create" {
        return vec![
            ("query_store_metrics", "加载真实业务数据作为输入"),
            ("creative_video_script", "生成内容资产"),
            ("coordination_create_task", "创建后续工作区任务"),
        ];
    }
    if action == "plan" {
        return vec![
            ("query_store_metrics", "了解当前业绩基准"),
            ("ops_assortment_planning", "制定品类和选品规划"),
            ("ops_execution_plan", "生成30天可落地执行计划"),
            ("coordination_create_task", "把关键动作保存为工作区任务"),
        ];
    }
    vec![
        ("query_store_metrics", "加载真实业务数据作为分析基础"),
        ("ops_execution_plan", "生成可落地执行方案"),
        ("coordination_create_task", "创建后续工作区任务"),
    ]
}
