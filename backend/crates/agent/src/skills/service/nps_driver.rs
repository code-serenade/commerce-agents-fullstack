use super::*;

pub fn service_nps_driver_analysis_spec() -> SkillSpec {
    SkillSpec::new(
        "service_nps_driver_analysis",
        "NPS驱动因素分析",
        "4类投诉来源根因诊断，计算各类别NPS影响系数，识别最大满意度驱动因素",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "promoter_count",
        "推荐者数量（评分9-10）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "detractor_count",
        "贬低者数量（评分0-6）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "passive_count",
        "被动者数量（评分7-8）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "complaint_distribution",
        "投诉分布：{物流配送: 120, 商品质量: 85}",
        object_schema(),
    ))
    .with_input(SkillInputField::optional(
        "total_tickets",
        "总工单数",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音",
        string_schema(),
    ))
    .with_tag("deterministic")
}

pub struct ServiceNpsDriverAnalysis {
    spec: SkillSpec,
}

impl ServiceNpsDriverAnalysis {
    pub fn new() -> Self {
        Self {
            spec: service_nps_driver_analysis_spec(),
        }
    }
}

impl Default for ServiceNpsDriverAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceNpsDriverAnalysis {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let mut promoter = optional_i64_param(&params, "promoter_count")?.unwrap_or(0);
        let mut detractor = optional_i64_param(&params, "detractor_count")?.unwrap_or(0);
        let mut passive = optional_i64_param(&params, "passive_count")?.unwrap_or(0);
        let has_user_nps_counts = params.get("promoter_count").is_some()
            || params.get("detractor_count").is_some()
            || params.get("passive_count").is_some();
        if promoter + detractor + passive == 0 {
            promoter = 45;
            passive = 35;
            detractor = 20;
        }

        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let mut dist = parse_string_i64_map(params.get("complaint_distribution"))?;
        let has_user_complaint_distribution = !dist.is_empty();
        if dist.is_empty() {
            dist.extend([
                ("物流配送".to_owned(), 38),
                ("商品质量".to_owned(), 25),
                ("售后退款".to_owned(), 18),
                ("客服响应".to_owned(), 12),
                ("商品描述".to_owned(), 7),
            ]);
        }

        let total_tickets = optional_i64_param(&params, "total_tickets")?
            .unwrap_or_else(|| dist.values().sum::<i64>())
            .max(1);
        let total_respondents = promoter + detractor + passive;
        let promoter_pct = promoter as f64 / total_respondents as f64;
        let detractor_pct = detractor as f64 / total_respondents as f64;
        let nps_score = round1((promoter_pct - detractor_pct) * 100.0);

        let mut rows = dist.into_iter().map(|(category, count)| {
            let rate = count as f64 / total_tickets as f64;
            let coeff = nps_coefficient(&category);
            let impact = round2(rate * coeff * 100.0);
            json!({
                "问题类别": category,
                "投诉量": count,
                "投诉率": format!("{:.1}%", rate * 100.0),
                "NPS拖累": format!("{impact:+.2}分"),
                "处理优先级": if impact.abs() > 3.0 { "P0" } else if impact.abs() > 1.5 { "P1" } else { "P2" },
                "_impact_sort": impact
            })
        }).collect::<Vec<_>>();
        rows.sort_by(|a, b| {
            a.get("_impact_sort")
                .and_then(Value::as_f64)
                .partial_cmp(&b.get("_impact_sort").and_then(Value::as_f64))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let max_uplift = round1(
            rows.iter()
                .filter_map(|row| row.get("_impact_sort").and_then(Value::as_f64))
                .sum::<f64>()
                .abs(),
        );
        for row in &mut rows {
            if let Some(map) = row.as_object_mut() {
                map.remove("_impact_sort");
            }
        }
        let industry_avg = match platform.as_str() {
            "淘宝" => 45,
            "京东" => 52,
            "拼多多" => 30,
            "抖音" => 40,
            _ => 40,
        };
        let nps_gap = round1(nps_score - industry_avg as f64);
        let max_pain = rows
            .first()
            .and_then(|row| row.get("问题类别"))
            .cloned()
            .unwrap_or_else(|| json!("无数据"));
        let data_readiness = if has_user_nps_counts && has_user_complaint_distribution {
            "user_provided"
        } else {
            "calculated_from_defaults"
        };

        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": data_readiness,
                "summary": "NPS驱动因素分析已完成",
                "平台": platform,
                "NPS分数": format!("{nps_score:+.1}"),
                "NPS构成": {
                    "推荐者(9-10分)": format!("{:.0}%", promoter_pct * 100.0),
                    "被动者(7-8分)": format!("{:.0}%", passive as f64 / total_respondents as f64 * 100.0),
                    "贬低者(0-6分)": format!("{:.0}%", detractor_pct * 100.0),
                    "受访总数": total_respondents
                },
                "行业基准": {
                    "行业平均NPS": industry_avg,
                    "差距": format!("{nps_gap:+.1}分"),
                    "位置": if nps_gap >= 0.0 { "优于行业" } else { "低于行业" }
                },
                "驱动因素分析": rows,
                "理论最大NPS提升": format!("+{max_uplift}分（解决全部投诉问题）"),
                "最大痛点": max_pain,
                "总工单数": total_tickets
            }),
            "NPS驱动因素分析已完成",
        )
    }
}
