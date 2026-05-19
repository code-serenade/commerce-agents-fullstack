use async_trait::async_trait;
use serde_json::{Value, json};

use super::{
    helpers::round1, normalize::normalize_engineering_output, specs::engineering_sla_monitor_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_f64_param,
    optional_i64_param, optional_string_param,
};

pub struct EngineeringSlaMonitor {
    spec: SkillSpec,
}

impl EngineeringSlaMonitor {
    pub fn new() -> Self {
        Self {
            spec: engineering_sla_monitor_spec(),
        }
    }
}

impl Default for EngineeringSlaMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for EngineeringSlaMonitor {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let samples = params
            .get("latency_samples")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
            .unwrap_or_default();

        let (p50, p90, p99) = if samples.is_empty() {
            (
                optional_f64_param(&params, "p50_ms")?.unwrap_or(150.0),
                optional_f64_param(&params, "p90_ms")?.unwrap_or(400.0),
                optional_f64_param(&params, "p99_ms")?.unwrap_or(1800.0),
            )
        } else {
            (
                percentile(samples.clone(), 50.0),
                percentile(samples.clone(), 90.0),
                percentile(samples, 99.0),
            )
        };

        let period_days = optional_i64_param(&params, "period_days")?
            .unwrap_or(30)
            .max(1);
        let system_name =
            optional_string_param(&params, "system_name")?.unwrap_or_else(|| "电商系统".to_owned());
        let total_requests = optional_i64_param(&params, "total_requests")?
            .unwrap_or(0)
            .max(0);
        let error_count = optional_i64_param(&params, "error_count")?
            .unwrap_or(0)
            .max(0);
        let downtime_min = optional_f64_param(&params, "downtime_minutes")?
            .unwrap_or(0.0)
            .max(0.0);
        let slo_avail = optional_f64_param(&params, "slo_availability")?.unwrap_or(99.9);
        let slo_p99 = optional_f64_param(&params, "slo_p99_ms")?.unwrap_or(2000.0);
        let period_minutes = period_days as f64 * 24.0 * 60.0;

        let availability = if downtime_min > 0.0 {
            round4((1.0 - downtime_min / period_minutes) * 100.0)
        } else if total_requests > 0 {
            round4((1.0 - error_count as f64 / total_requests as f64) * 100.0)
        } else {
            99.9
        };

        let budget_total = (1.0 - slo_avail / 100.0) * period_minutes;
        let budget_used_pct = if budget_total > 0.0 {
            round1(downtime_min / budget_total * 100.0)
        } else {
            0.0
        };
        let budget_left_pct = round1(100.0 - budget_used_pct);
        let p99_pass = p99 <= slo_p99;
        let p90_pass = p90 <= 500.0;
        let availability_pass = availability >= slo_avail;
        let violations = [p99_pass, p90_pass, availability_pass]
            .iter()
            .filter(|passed| !**passed)
            .count();
        let peak_p99 = p99 * 2.5;

        let mut output = json!({
            "系统": system_name,
            "统计周期": format!("{period_days}天"),
            "延迟分位数": {
                "P50（中位数）": format!("{p50:.0}ms"),
                "P90（90%用户体验）": format!("{p90:.0}ms"),
                "P99（长尾体验）": format!("{p99:.0}ms"),
                "目标P99": format!("{slo_p99:.0}ms"),
                "P99状态": if p99_pass { "达标".to_owned() } else { format!("超标{}%", round1((p99 - slo_p99) / slo_p99 * 100.0)) }
            },
            "可用性": {
                "实际可用性": format!("{availability:.3}%"),
                "SLO目标": format!("{slo_avail:.3}%"),
                "本期宕机": format!("{downtime_min:.1}分钟"),
                "状态": if availability_pass { "达标" } else { "未达标" }
            },
            "错误预算": {
                "本期可用预算": format!("{budget_total:.1}分钟"),
                "已消耗": format!("{downtime_min:.1}分钟 ({budget_used_pct}%)"),
                "剩余预算": format!("{budget_left_pct}%"),
                "预算状态": budget_state(budget_left_pct)
            },
            "SLO达标情况": [
                {"指标": "P99延迟", "目标": slo_p99, "实际": round1(p99), "状态": if p99_pass { "达标" } else { "未达标" }},
                {"指标": "P90延迟", "目标": 500, "实际": round1(p90), "状态": if p90_pass { "达标" } else { "未达标" }},
                {"指标": "可用性", "目标": slo_avail, "实际": availability, "状态": if availability_pass { "达标" } else { "未达标" }}
            ],
            "SLO违规项数": violations,
            "大促峰值风险": {
                "预估大促P99": format!("{peak_p99:.0}ms（当前×2.5倍流量）"),
                "风险等级": if peak_p99 > slo_p99 * 3.0 { "高风险" } else if peak_p99 > slo_p99 * 1.5 { "中风险" } else { "低风险" }
            }
        });

        if total_requests > 0 {
            output["请求统计"] = json!({
                "总请求数": total_requests,
                "错误数": error_count,
                "错误率": format!("{:.3}%", error_count as f64 / total_requests as f64 * 100.0)
            });
        }

        normalize_engineering_output("engineering_sla_monitor", &params, &mut output);
        Ok(SkillOutcome::new(output).with_summary("SLA监控分析已完成"))
    }
}

fn percentile(mut data: Vec<f64>, pct: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = pct / 100.0 * (data.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(data.len() - 1);
    let frac = idx - lo as f64;
    data[lo] * (1.0 - frac) + data[hi] * frac
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn budget_state(left: f64) -> &'static str {
    if left <= 0.0 {
        "预算耗尽"
    } else if left < 20.0 {
        "预算紧张"
    } else if left < 50.0 {
        "预算警告"
    } else {
        "预算充足"
    }
}
