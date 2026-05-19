use serde_json::{Map, Value, json};

use super::{
    common::{
        AttributionChannel, attribution_models_to_show, empty_string_as_null, i, json_ok, n,
        normal_confidence, normalize, normalized_rate, one_hot, pct, percent,
        position_based_weights, required_i, round2, round4, s, signed_percent,
    },
    common_spec::deferred,
    output_contract::data_summary,
};
use crate::skills::SkillError;

pub(super) fn attribution_analysis(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let channels = input
        .get("channels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if channels.is_empty() {
        return attribution_deferred(
            "data_attribution_analysis",
            "需要多渠道GMV、UV、转化率和广告花费数据",
            input,
        );
    }

    let mut rows = channels
        .iter()
        .filter_map(|channel| {
            let name = s(channel, "name", "");
            let gmv = n(channel, "gmv", 0.0).max(0.0);
            let uv = n(channel, "uv", 0.0).max(0.0);
            let conversion_rate =
                normalized_rate(n(channel, "conversion_rate", n(channel, "conv", 0.0)));
            let ad_spend = n(channel, "ad_spend", 0.0).max(0.0);
            ((gmv > 0.0 || uv > 0.0) && !name.trim().is_empty()).then_some(AttributionChannel {
                name,
                gmv,
                uv,
                conversion_rate,
                ad_spend,
            })
        })
        .collect::<Vec<_>>();

    if rows.len() < 2 {
        return attribution_deferred(
            "data_attribution_analysis",
            "归因分析至少需要2个有效渠道",
            input,
        );
    }
    rows.sort_by(|a, b| b.gmv.total_cmp(&a.gmv));

    let total_gmv = rows.iter().map(|channel| channel.gmv).sum::<f64>();
    if total_gmv <= 0.0 {
        return attribution_deferred(
            "data_attribution_analysis",
            "总GMV为0，无法计算渠道归因",
            input,
        );
    }

    let n_channels = rows.len();
    let last_touch_weights = one_hot(n_channels, 0);
    let linear_weights = normalize(vec![1.0; n_channels]);
    let decay_signals = rows
        .iter()
        .map(|channel| {
            if channel.ad_spend > 0.0 {
                channel.ad_spend
            } else {
                channel.conversion_rate * channel.uv
            }
        })
        .collect::<Vec<_>>();
    let time_decay_weights = if decay_signals.iter().all(|value| value.abs() < 1e-9) {
        linear_weights.clone()
    } else {
        normalize(decay_signals)
    };
    let position_weights = position_based_weights(n_channels);
    let requested_model = s(input, "attribution_model", "all");
    let selected_models = attribution_models_to_show(&requested_model);
    let model_map = [
        ("last_touch", "末次归因", last_touch_weights),
        ("linear", "线性归因", linear_weights),
        ("time_decay", "时间衰减归因", time_decay_weights.clone()),
        ("position", "位置归因(U形)", position_weights),
    ];

    let channel_details = rows
        .iter()
        .enumerate()
        .map(|(idx, channel)| {
            let actual_share = channel.gmv / total_gmv;
            let mut attributed = Map::new();
            for (key, display, weights) in &model_map {
                if !selected_models.iter().any(|model| model == key) {
                    continue;
                }
                let weight = weights[idx];
                let attributed_gmv = total_gmv * weight;
                let vs_actual = pct(weight - actual_share, actual_share.max(0.001));
                attributed.insert(
                    (*display).to_owned(),
                    json!({
                        "归因GMV": round2(attributed_gmv),
                        "归因权重": percent(weight * 100.0, 1),
                        "vs实际占比": signed_percent(vs_actual, 1),
                    }),
                );
            }
            json!({
                "渠道": channel.name,
                "实际GMV": round2(channel.gmv),
                "实际GMV占比": percent(actual_share * 100.0, 1),
                "UV": round2(channel.uv),
                "转化率": percent(channel.conversion_rate * 100.0, 2),
                "广告花费": round2(channel.ad_spend),
                "归因GMV对比": Value::Object(attributed),
            })
        })
        .collect::<Vec<_>>();

    let mut insights = Vec::new();
    for (idx, channel) in rows.iter().enumerate() {
        let actual_share = channel.gmv / total_gmv;
        let contribution = time_decay_weights[idx];
        let gap = contribution - actual_share;
        if gap > 0.10 {
            insights.push(format!(
                "{}：归因权重比实际GMV占比高{}，可能被低估，建议增加预算或资源",
                channel.name,
                percent(gap * 100.0, 1)
            ));
        } else if gap < -0.10 {
            insights.push(format!(
                "{}：归因权重比实际GMV占比低{}，可能被高估，需复核预算效率",
                channel.name,
                percent(gap.abs() * 100.0, 1)
            ));
        }
    }
    if insights.is_empty() {
        insights.push("各渠道贡献相对均衡，暂无明显高估/低估".to_owned());
    }

    let best_idx = time_decay_weights
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": "user_provided",
        "adapter_status": "manual_input",
        "summary": format!("渠道归因分析完成，{}归因贡献最高", rows[best_idx].name),
        "数据来源": "用户提供渠道数据",
        "商品引用": empty_string_as_null(product_ref),
        "分析周期": format!("{}天", i(input, "days", 30).max(1)),
        "归因模型": if requested_model == "all" { "all" } else { requested_model.as_str() },
        "分析说明": "末次归因=成交最高渠道；线性=等权分配；时间衰减=以广告花费或转化信号代理近期触点；位置归因=首末各40%中间20%",
        "总GMV": round2(total_gmv),
        "渠道数": rows.len(),
        "各渠道归因明细": channel_details,
        "关键洞察": insights,
        "预算再分配建议": format!("时间衰减模型下，{}归因贡献最高，建议优先保障该渠道预算，同时复核低归因高花费渠道。", rows[best_idx].name),
    })
}

fn attribution_deferred(skill: &str, reason: &str, input: &Value) -> Value {
    let mut output = deferred(skill, reason, input);
    if let Some(map) = output.as_object_mut() {
        map.insert("summary".to_owned(), json!(data_summary(skill)));
    }
    output
}

pub(super) fn ltv_calculator(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let aov = n(input, "avg_order_value", 120.0);
    let freq = n(input, "purchase_frequency", 1.6);
    let margin = n(input, "gross_margin", 35.0) / 100.0;
    let retention = (n(input, "monthly_retention", 35.0) / 100.0).clamp(0.01, 0.95);
    let cac = n(input, "cac", 35.0);
    let ltv = aov * freq * margin / (1.0 - retention);
    json!({
        "数据来源": "用户提供/默认估算",
        "商品引用": empty_string_as_null(product_ref),
        "LTV": round2(ltv),
        "月贡献毛利": round2(aov * freq * margin),
        "CAC": round2(cac),
        "LTV/CAC": round2(ltv / cac.max(0.01)),
        "回收周期(月)": round2(cac / (aov * freq * margin).max(0.01)),
        "健康度": if ltv / cac.max(0.01) >= 3.0 { "健康" } else if ltv / cac.max(0.01) >= 1.5 { "可优化" } else { "风险" }
    })
}

pub(super) fn cohort_analysis(input: &Value) -> Value {
    let product_ref = s(input, "product_ref", "");
    let unit = s(input, "cohort_unit", "week");
    let label = if unit == "month" { "月" } else { "周" };
    let new_customers = i(input, "new_customers", 120).max(1);
    let base = [1.0, 0.35, 0.22, 0.15, 0.10, 0.07];
    let cohorts = (0..4)
        .map(|idx| {
            let acquired = (new_customers as f64 * (0.9_f64).powi(idx)).round() as i64;
            let curve = base
                .iter()
                .take(6 - idx as usize)
                .enumerate()
                .map(|(period, rate)| json!({format!("第{period}{label}"): format!("{:.1}% ({}人)", rate * 100.0, (acquired as f64 * rate).round() as i64)}))
                .collect::<Vec<_>>();
            json!({"队列": format!("第{}{label}队列", idx + 1), "获取新客": acquired, "留存曲线": curve})
        })
        .collect::<Vec<_>>();
    json!({
        "数据来源": "指数留存模型",
        "商品引用": empty_string_as_null(product_ref),
        "队列留存矩阵": cohorts,
        "关键流失节点": [{"节点": format!("第1{label}后"), "流失率": "约65%", "原因": "首次体验不满意或无复购触点"}],
        "关键发现": {"首期留存率": "约35%", "最高价值窗口": format!("首次购买后第1-2{label}")}
    })
}

pub(super) fn ab_test_analyzer(input: &Value) -> Result<Value, SkillError> {
    let product_ref = s(input, "product_ref", "");
    let ctrl_n = required_i(input, "control_visitors")?;
    let ctrl_c = required_i(input, "control_conversions")?;
    let test_n = required_i(input, "test_visitors")?;
    let test_c = required_i(input, "test_conversions")?;
    if ctrl_n <= 0 || test_n <= 0 {
        return Err(SkillError::invalid_parameter(
            "visitors",
            "positive integer",
        ));
    }
    if ctrl_c < 0 || test_c < 0 || ctrl_c > ctrl_n || test_c > test_n {
        return Err(SkillError::invalid_parameter(
            "conversions",
            "0 <= conversions <= visitors",
        ));
    }
    let ctrl_rate = ctrl_c as f64 / ctrl_n as f64;
    let test_rate = test_c as f64 / test_n as f64;
    let pooled = (ctrl_c + test_c) as f64 / (ctrl_n + test_n) as f64;
    let se = (pooled * (1.0 - pooled) * (1.0 / ctrl_n as f64 + 1.0 / test_n as f64)).sqrt();
    let z = if se > 0.0 {
        (test_rate - ctrl_rate) / se
    } else {
        0.0
    };
    let confidence = normal_confidence(z.abs());
    json_ok(json!({
        "测试名称": s(input, "test_name", "A/B测试"),
        "商品引用": empty_string_as_null(product_ref),
        "指标": s(input, "metric_name", "转化率"),
        "对照组转化率": percent(ctrl_rate * 100.0, 2),
        "实验组转化率": percent(test_rate * 100.0, 2),
        "相对提升": signed_percent(pct(test_rate - ctrl_rate, ctrl_rate), 2),
        "Z值": round4(z),
        "置信度": percent(confidence * 100.0, 2),
        "结论": if confidence >= 0.95 { "差异显著，可考虑推全" } else if confidence >= 0.90 { "接近显著，建议扩大样本" } else { "差异不显著，继续观察" }
    }))
}
