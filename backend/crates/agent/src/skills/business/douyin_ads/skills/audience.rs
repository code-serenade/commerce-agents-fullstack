use serde_json::{Value, json};

use super::super::helpers::{currency, empty_string_as_null, has_any, num, string};

pub(super) fn audience_package(input: &Value) -> Value {
    let audience = string(input, "target_audience", "目标消费者");
    let product_name = string(input, "product_name", "未指定商品");
    let category = string(input, "category", "电商消费品");
    let budget = num(input, "budget", 8_000.0).max(0.0);
    let product_ref = string(input, "product_ref", "");
    let stage = string(input, "stage", "冷启动");

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": if has_any(input, &["target_audience", "product_name", "category", "budget", "product_ref"]) { "user_provided" } else { "calculated_from_defaults" },
        "summary": "广告人群包设计已生成",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 广告投放 deterministic skill；未读取广告后台人群数据",
        "商品": product_name,
        "商品引用": empty_string_as_null(&product_ref),
        "类目": category,
        "目标人群": audience,
        "阶段": stage,
        "预算": currency(budget),
        "人群包矩阵": [
            {
                "人群包": "核心转化人群",
                "预算占比": "35%",
                "建议预算": currency(budget * 0.35),
                "定向逻辑": format!("{category}强兴趣 + 近期互动/购买意向 + {audience}"),
                "观察指标": ["点击率", "进房率", "商品点击率", "支付ROI"],
                "动作": "优先跑转化目标，保留ROI稳定计划"
            },
            {
                "人群包": "相似扩展人群",
                "预算占比": "25%",
                "建议预算": currency(budget * 0.25),
                "定向逻辑": "基于成交/加购/直播互动种子做相似扩展",
                "观察指标": ["获客成本", "首单转化率", "新客占比"],
                "动作": "冷启动后承接放量，严格控制成本波动"
            },
            {
                "人群包": "内容兴趣人群",
                "预算占比": "25%",
                "建议预算": currency(budget * 0.25),
                "定向逻辑": "短视频互动、达人内容、类目兴趣、场景兴趣组合",
                "观察指标": ["3秒播放", "点击率", "进房成本"],
                "动作": "用于素材赛马和直播间引流，不直接按GMV单点判断"
            },
            {
                "人群包": "再营销人群",
                "预算占比": "15%",
                "建议预算": currency(budget * 0.15),
                "定向逻辑": "近7/15/30天互动、进房、商品点击、加购和未支付用户",
                "观察指标": ["支付转化率", "客单价", "ROI"],
                "动作": "配合限时权益和直播间逼单话术收割"
            }
        ],
        "排除规则": [
            "排除近期已购买但未到复购周期用户，避免浪费预算",
            "排除高频曝光无互动人群，降低素材疲劳",
            "排除投诉/退款高风险标签，避免放大售后风险"
        ],
        "测试顺序": [
            "先跑核心转化人群验证商品与直播间承接",
            "再跑内容兴趣人群验证素材点击和进房",
            "最后扩展相似人群并建立再营销收割池"
        ],
        "风险提示": [
            "本结果没有读取真实DMP人群包规模或覆盖率",
            "实际投放前需要补齐历史成交人群、直播互动人群和素材表现"
        ]
    })
}
