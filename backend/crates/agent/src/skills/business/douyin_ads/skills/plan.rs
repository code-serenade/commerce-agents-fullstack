use serde_json::{Value, json};

use super::super::helpers::{currency, empty_string_as_null, has_any, int, num, round2, string};

pub(super) fn plan_structure(input: &Value) -> Value {
    let goal = string(input, "goal", "直播间成交与商品转化");
    let budget = num(input, "budget", 10_000.0).max(0.0);
    let platform = string(input, "platform", "电商广告平台");
    let product_name = string(input, "product_name", "未指定商品");
    let product_ref = string(input, "product_ref", "");
    let stage = string(input, "stage", "冷启动");
    let days = int(input, "days", 7).max(1);
    let target_roas = num(input, "target_roas", 2.5).max(0.1);
    let daily_budget = budget / days as f64;

    json!({
        "status": "success",
        "has_data": true,
        "adapter_status": "manual_input",
        "data_readiness": if has_any(input, &["budget", "goal", "product_name", "product_ref", "target_roas"]) { "user_provided" } else { "calculated_from_defaults" },
        "summary": "广告投放结构已生成",
        "数据来源": "用户显式输入 + 新产品 Rust Agent 广告投放 deterministic skill；未直连广告后台或平台实时数据",
        "平台": platform,
        "商品": product_name,
        "商品引用": empty_string_as_null(&product_ref),
        "投放目标": goal,
        "阶段": stage,
        "周期天数": days,
        "总预算": currency(budget),
        "日均预算": currency(daily_budget),
        "目标ROAS": round2(target_roas),
        "计划结构": [
            {
                "层级": "冷启动赛马",
                "预算占比": "30%",
                "建议预算": currency(budget * 0.30),
                "动作": ["拆5-8个小计划", "每计划绑定1个素材方向和1个人群假设", "先看点击率、进房率和首单成本"],
                "保留规则": format!("24小时内ROAS达到目标{}的70%以上或点击率明显优于均值", round2(target_roas))
            },
            {
                "层级": "优胜计划放量",
                "预算占比": "45%",
                "建议预算": currency(budget * 0.45),
                "动作": ["预算每次上调不超过30%", "保留同素材同人群稳定计划", "复制计划测试相邻出价"],
                "放量规则": "连续2个观察窗口转化成本稳定且退款/差评风险可控"
            },
            {
                "层级": "稳定收割",
                "预算占比": "20%",
                "建议预算": currency(budget * 0.20),
                "动作": ["沉淀高意向人群再营销", "直播间承接款和利润款分计划", "控制频次避免素材疲劳"],
                "止损规则": format!("ROAS连续2个观察窗口低于{}则降预算或暂停", round2(target_roas * 0.7))
            },
            {
                "层级": "复盘留样",
                "预算占比": "5%",
                "建议预算": currency(budget * 0.05),
                "动作": ["保留少量探索计划", "记录素材-人群-出价组合", "输出下一轮测试假设"],
                "复盘方式": "按计划、素材、人群包、直播间承接分别复盘"
            }
        ],
        "关键KPI": {
            "目标GMV": currency(budget * target_roas),
            "计划级ROAS止损线": round2(target_roas * 0.70),
            "素材疲劳检查": "CTR下降超过20%或频次持续升高时更换素材",
            "直播承接检查": "点击正常但成交弱时优先排查进房、停留、商品点击和逼单话术"
        },
        "风险提示": [
            "本结果不是广告后台读取结果，只能作为计划结构和预算规则建议",
            "放量前必须补齐消耗、GMV、点击、进房、商品点击、下单和付款指标",
            "平台规则、审核政策和行业基准需要通过 search/platform adapter 单独取得真实来源"
        ]
    })
}
