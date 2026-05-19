use super::helpers::{field, spec};
use crate::skills::SkillSpec;

pub fn specs() -> Vec<SkillSpec> {
    vec![
        spec(
            "douyin_ads_plan_structure",
            "千川计划结构",
            "根据目标、预算、阶段和商品上下文生成巨量千川计划结构、预算拆分、冷启动赛马和止损规则",
            vec![
                field(
                    "goal",
                    "string",
                    "投放目标，例如直播间成交、短视频引流、新品冷启动",
                    false,
                ),
                field("budget", "number", "投放总预算（元）", false),
                field("platform", "string", "平台，默认抖音", false),
                field(
                    "product_name",
                    "string",
                    "商品名称；仅来自本轮显式输入或显式选品",
                    false,
                ),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
                field("stage", "string", "投放阶段：冷启动/测试/放量/稳定", false),
                field("days", "integer", "投放周期天数，默认7", false),
                field("target_roas", "number", "目标ROAS，默认2.5", false),
            ],
        ),
        spec(
            "douyin_ads_audience_package",
            "千川人群包设计",
            "根据商品、人群、类目和预算生成千川人群包矩阵、排除规则、再营销池和测试顺序",
            vec![
                field("target_audience", "string", "目标人群描述", false),
                field(
                    "product_name",
                    "string",
                    "商品名称；仅来自本轮显式输入或显式选品",
                    false,
                ),
                field("category", "string", "商品类目", false),
                field("budget", "number", "投放预算（元）", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
                field("stage", "string", "投放阶段：冷启动/测试/放量/稳定", false),
            ],
        ),
        spec(
            "douyin_ads_budget_pacing",
            "千川预算节奏",
            "根据预算、消耗、GMV或ROAS计算日预算节奏、放量/收缩判断和ROI止损线",
            vec![
                field("budget", "number", "投放总预算（元）", false),
                field("days", "integer", "投放周期天数，默认7", false),
                field("spend", "number", "已消耗金额（元）", false),
                field("gmv", "number", "已产生GMV（元）", false),
                field("target_roas", "number", "目标ROAS，默认2.5", false),
                field(
                    "current_roas",
                    "number",
                    "当前ROAS；如果缺失则用GMV/消耗计算",
                    false,
                ),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
            ],
        ),
        spec(
            "douyin_ads_live_funnel_diagnosis",
            "直播间投放漏斗诊断",
            "基于曝光、点击、进房、商品点击、下单、付款和GMV诊断直播间千川投放漏斗瓶颈",
            vec![
                field("impressions", "number", "曝光量", false),
                field("clicks", "number", "广告点击数", false),
                field("live_room_entries", "number", "直播间进房数", false),
                field("product_clicks", "number", "商品点击数", false),
                field("orders", "number", "下单数", false),
                field("payments", "number", "付款数", false),
                field("ad_spend", "number", "广告消耗（元）", false),
                field("gmv", "number", "成交GMV（元）", false),
                field(
                    "product_ref",
                    "string",
                    "新产品商品引用 product:{uuid}；仅来自本轮显式选品",
                    false,
                ),
            ],
        ),
    ]
}
