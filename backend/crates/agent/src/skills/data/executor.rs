pub struct DataSkill {
    spec: SkillSpec,
}

impl DataSkill {
    pub fn new(skill_id: &str) -> Result<Self, SkillError> {
        let spec = specs()
            .into_iter()
            .find(|spec| spec.id == skill_id)
            .ok_or_else(|| SkillError::invalid_parameter("skill_id", "known data skill id"))?;
        Ok(Self { spec })
    }
}

#[async_trait]
impl Skill for DataSkill {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        execute(&self.spec.id, params, &context).await
    }
}

pub async fn execute(skill_name: &str, input: Value, _context: &SkillContext) -> SkillResult {
    ensure_object(&input)?;
    let mut output = match skill_name {
        "query_store_metrics" => query_store_metrics(&input),
        "query_product_metrics" => query_product_metrics(&input),
        "query_campaign_metrics" => query_campaign_metrics(&input),
        "data_funnel_analysis" => funnel_analysis(&input),
        "data_anomaly_diagnosis" => anomaly_diagnosis(&input),
        "data_trend_forecast" => trend_forecast(&input),
        "data_dashboard" => data_dashboard(&input),
        "data_customer_segmentation" => customer_segmentation(&input),
        "data_competitor_analysis" => competitor_analysis(&input)?,
        "data_multi_period_trend" => multi_period_trend(&input),
        "data_channel_roi" => channel_roi(&input),
        "data_ltv_calculator" => ltv_calculator(&input),
        "data_cohort_analysis" => cohort_analysis(&input),
        "data_ab_test_analyzer" => ab_test_analyzer(&input)?,
        "data_attribution_analysis" => attribution_analysis(&input),
        "data_refund_decomposition" => refund_decomposition(&input),
        "data_seasonal_decompose" => seasonal_decompose(&input),
        "data_price_elasticity" => price_elasticity(&input),
        "data_demand_forecast" => demand_forecast(&input),
        "data_comprehensive_diagnosis" => comprehensive_diagnosis(&input),
        other => return Err(SkillError::invalid_parameter(other, "known data skill")),
    };
    normalize_data_output(skill_name, &input, &mut output);
    Ok(SkillOutcome::new(output).with_summary("data skill executed"))
}
