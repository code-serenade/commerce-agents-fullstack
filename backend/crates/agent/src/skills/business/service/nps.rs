use super::*;

pub fn service_nps_analyzer_spec() -> SkillSpec {
    SkillSpec::new(
        "service_nps_analyzer",
        "NPS客户满意度分析",
        "计算NPS净推荐值、CSAT满意度评分和客户忠诚度分层",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "total_respondents",
        "调查总人数",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "score_distribution",
        "评分分布，key=评分，value=人数",
        object_schema(),
    ))
    .with_input(SkillInputField::optional(
        "promoters_count",
        "推荐者数量（9-10分）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "passives_count",
        "被动者数量（7-8分）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "detractors_count",
        "批评者数量（0-6分）",
        integer_schema(),
    ))
    .with_input(SkillInputField::optional(
        "top_issues",
        "批评者主要问题列表",
        string_array_schema(),
    ))
    .with_input(SkillInputField::optional(
        "platform",
        "平台：淘宝/京东/拼多多/抖音/综合",
        string_schema(),
    ))
    .with_input(SkillInputField::optional(
        "days",
        "若无调查数据，从外部数据推算的天数",
        integer_schema(),
    ))
    .with_tag("deterministic")
}

pub struct ServiceNpsAnalyzer {
    spec: SkillSpec,
}

impl ServiceNpsAnalyzer {
    pub fn new() -> Self {
        Self {
            spec: service_nps_analyzer_spec(),
        }
    }
}

impl Default for ServiceNpsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceNpsAnalyzer {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let mut promoters = 0_i64;
        let mut passives = 0_i64;
        let mut detractors = 0_i64;
        let mut total = 0_i64;
        let mut data_source = "用户提供";

        if let Some(distribution) = params.get("score_distribution").and_then(Value::as_object) {
            for (score, count) in distribution {
                let score = score.parse::<i64>().map_err(|_| {
                    SkillError::invalid_parameter("score_distribution", "score keys 0-10")
                })?;
                let count = count.as_i64().ok_or_else(|| {
                    SkillError::invalid_parameter("score_distribution", "integer counts")
                })?;
                total += count;
                if score >= 9 {
                    promoters += count;
                } else if score >= 7 {
                    passives += count;
                } else {
                    detractors += count;
                }
            }
        } else if params.get("promoters_count").is_some() {
            promoters = optional_i64_param(&params, "promoters_count")?.unwrap_or(0);
            passives = optional_i64_param(&params, "passives_count")?.unwrap_or(0);
            detractors = optional_i64_param(&params, "detractors_count")?.unwrap_or(0);
            total = optional_i64_param(&params, "total_respondents")?
                .unwrap_or(promoters + passives + detractors);
        } else {
            data_source = "默认样例";
            promoters = 45;
            passives = 35;
            detractors = 20;
            total = 100;
        }

        if total <= 0 {
            total = promoters + passives + detractors;
        }
        if total <= 0 {
            return Err(SkillError::execution(
                "无有效数据，请提供评分分布或各分段人数",
            ));
        }

        let prom_pct = round1(promoters as f64 / total as f64 * 100.0);
        let pass_pct = round1(passives as f64 / total as f64 * 100.0);
        let detr_pct = round1(detractors as f64 / total as f64 * 100.0);
        let nps = round1(prom_pct - detr_pct);
        let csat_pct = round1((promoters + passives) as f64 / total as f64 * 100.0);
        let top_issues = optional_string_vec_param_or_empty(&params, "top_issues")?;
        let data_readiness = if data_source == "默认样例" {
            "calculated_from_defaults"
        } else {
            "user_provided"
        };

        let output = json!({
            "status": "success",
            "has_data": true,
            "adapter_status": "manual_input",
            "data_readiness": data_readiness,
            "summary": "NPS与CSAT分析已完成",
            "数据来源": data_source,
            "调查总人数": total,
            "NPS净推荐值": nps,
            "NPS评级": nps_grade(nps),
            "电商行业基准": "NPS 优秀≥50 | 正常≥0 | 危险<-10",
            "分群详情": {
                "推荐者(9-10分)": format!("{promoters}人（{prom_pct}%）"),
                "被动者(7-8分)": format!("{passives}人（{pass_pct}%）"),
                "批评者(0-6分)": format!("{detractors}人（{detr_pct}%）")
            },
            "CSAT满意度": format!("{csat_pct}%（{}）", csat_grade(csat_pct)),
            "忠诚度洞察": {
                "推荐者价值": format!("贡献约{prom_pct}%的口碑传播和复购"),
                "被动者风险": format!("{pass_pct}%的客户随时可能流向竞品，需主动维系"),
                "批评者负面影响": format!("预估影响约{}人的购买决策", (detractors as f64 * 5.6).round() as i64),
                "转化优先级": "被动者→推荐者成本最低，批评者→中立是当务之急"
            },
            "批评者主要问题": top_issues.iter().take(5).collect::<Vec<_>>(),
            "改善优先级": top_issues.first().map(|issue| format!("重点解决：{issue}（批评者最常提及）"))
        });

        outcome(output, "NPS与CSAT分析已完成")
    }
}
