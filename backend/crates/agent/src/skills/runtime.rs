use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::skills::{
    SearchAdapterConfig, Skill, SkillCategory, SkillContext, SkillError, SkillOutcome, SkillSpec,
    accounting, coordination, creative, data, design, engineering, ops, search, service,
    staged_specs, web,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SkillId(pub String);

impl SkillId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SkillId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SkillId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillRequest {
    pub request_id: String,
    pub skill_id: SkillId,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub context: SkillContext,
}

impl SkillRequest {
    pub fn new(request_id: impl Into<String>, skill_id: impl Into<SkillId>, input: Value) -> Self {
        Self {
            request_id: request_id.into(),
            skill_id: skill_id.into(),
            input,
            context: SkillContext::default(),
        }
    }

    pub fn with_context(mut self, context: SkillContext) -> Self {
        self.context = context;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillExecutionStatus {
    Success,
    Deferred,
    PendingApproval,
    NotFound,
    ValidationError,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillRiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillExecutionMode {
    Native,
    Deferred,
    Adapter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillAdapterKind {
    Search,
    Llm,
    Db,
    Platform,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    pub request_id: String,
    pub skill_id: SkillId,
    pub status: SkillExecutionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SkillExecutionResult {
    fn from_outcome(
        request: &SkillRequest,
        spec: &SkillSpec,
        outcome: SkillOutcome,
        mode: SkillExecutionMode,
    ) -> Self {
        let mut metadata = outcome.metadata;
        add_runtime_metadata(&mut metadata, spec, mode);
        let status = status_from_output(&outcome.output).unwrap_or(SkillExecutionStatus::Success);
        Self {
            request_id: request.request_id.clone(),
            skill_id: request.skill_id.clone(),
            status,
            output: Some(outcome.output),
            summary: outcome.summary,
            metadata,
            error: None,
        }
    }

    fn error(
        request: &SkillRequest,
        status: SkillExecutionStatus,
        message: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            skill_id: request.skill_id.clone(),
            status,
            output: None,
            summary: None,
            metadata: Map::new(),
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SkillRuntimeError {
    #[error("duplicate skill id '{0}'")]
    DuplicateSkillId(String),
}

#[derive(Clone, Debug)]
pub struct SkillRegistry {
    specs: BTreeMap<SkillId, SkillSpec>,
}

impl SkillRegistry {
    pub fn staged() -> Result<Self, SkillRuntimeError> {
        Self::from_specs(staged_specs())
    }

    pub fn from_specs(specs: Vec<SkillSpec>) -> Result<Self, SkillRuntimeError> {
        let mut by_id = BTreeMap::new();
        for spec in specs {
            let id = SkillId::new(spec.id.clone());
            if by_id.insert(id.clone(), spec).is_some() {
                return Err(SkillRuntimeError::DuplicateSkillId(id.0));
            }
        }
        Ok(Self { specs: by_id })
    }

    pub fn get(&self, id: impl Into<SkillId>) -> Option<&SkillSpec> {
        self.specs.get(&id.into())
    }

    pub fn all(&self) -> Vec<&SkillSpec> {
        self.specs.values().collect()
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    pub fn by_category(&self, category: &SkillCategory) -> Vec<&SkillSpec> {
        self.specs
            .values()
            .filter(|spec| &spec.category == category)
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&SkillSpec> {
        self.specs
            .values()
            .filter(|spec| spec.tags.iter().any(|value| value == tag))
            .collect()
    }

    pub fn by_execution_mode(&self, mode: SkillExecutionMode) -> Vec<&SkillSpec> {
        self.specs
            .values()
            .filter(|spec| execution_mode(spec) == mode)
            .collect()
    }
}

#[async_trait]
pub trait SkillExecutor: Send + Sync {
    async fn execute(&self, request: SkillRequest) -> SkillExecutionResult;
}

#[derive(Clone, Debug)]
pub struct LocalSkillExecutor {
    registry: SkillRegistry,
    search: SearchAdapterConfig,
}

impl LocalSkillExecutor {
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            registry,
            search: SearchAdapterConfig::default(),
        }
    }

    pub fn new_with_search(registry: SkillRegistry, search: SearchAdapterConfig) -> Self {
        Self { registry, search }
    }

    pub fn staged() -> Result<Self, SkillRuntimeError> {
        Ok(Self::new(SkillRegistry::staged()?))
    }

    pub fn staged_with_search(search: SearchAdapterConfig) -> Result<Self, SkillRuntimeError> {
        Ok(Self::new_with_search(SkillRegistry::staged()?, search))
    }

    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }
}

#[async_trait]
impl SkillExecutor for LocalSkillExecutor {
    async fn execute(&self, request: SkillRequest) -> SkillExecutionResult {
        if let Err(message) = validate_request_envelope(&request) {
            return SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::ValidationError,
                message,
            );
        }

        let Some(spec) = self.registry.get(request.skill_id.clone()) else {
            return SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::NotFound,
                "skill id not found",
            );
        };

        if let Err(error) = validate_request(spec, &request.input) {
            return SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::ValidationError,
                error.to_string(),
            );
        }

        let mode = execution_mode(spec);
        let outcome = run_known_skill(
            spec.id.as_str(),
            request.input.clone(),
            request.context.clone(),
            &self.search,
        )
        .await;
        match outcome {
            Ok(outcome) => SkillExecutionResult::from_outcome(&request, spec, outcome, mode),
            Err(error) => SkillExecutionResult::error(
                &request,
                SkillExecutionStatus::Failed,
                error.to_string(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillExecutorSet {
    local: LocalSkillExecutor,
}

impl SkillExecutorSet {
    pub fn staged() -> Result<Self, SkillRuntimeError> {
        Ok(Self {
            local: LocalSkillExecutor::staged()?,
        })
    }

    pub fn staged_with_search(search: SearchAdapterConfig) -> Result<Self, SkillRuntimeError> {
        Ok(Self {
            local: LocalSkillExecutor::staged_with_search(search)?,
        })
    }

    pub fn registry(&self) -> &SkillRegistry {
        self.local.registry()
    }
}

#[async_trait]
impl SkillExecutor for SkillExecutorSet {
    async fn execute(&self, request: SkillRequest) -> SkillExecutionResult {
        self.local.execute(request).await
    }
}

fn validate_request_envelope(request: &SkillRequest) -> Result<(), &'static str> {
    if request.request_id.trim().is_empty() {
        return Err("request_id is required");
    }
    if request.skill_id.as_str().trim().is_empty() {
        return Err("skill_id is required");
    }
    Ok(())
}

fn validate_request(spec: &SkillSpec, input: &Value) -> Result<(), SkillError> {
    let object = input
        .as_object()
        .ok_or_else(|| SkillError::invalid_parameter("$", "object"))?;
    for field in &spec.inputs {
        if field.required && !object.contains_key(&field.name) {
            return Err(SkillError::missing_parameter(field.name.clone()));
        }
    }
    Ok(())
}

fn execution_mode(spec: &SkillSpec) -> SkillExecutionMode {
    if custom_category(spec) == Some("search") || spec.tags.iter().any(|tag| tag == "adapter") {
        SkillExecutionMode::Adapter
    } else if spec.tags.iter().any(|tag| tag == "deferred") {
        SkillExecutionMode::Deferred
    } else if adapter_kinds(spec).is_empty() {
        SkillExecutionMode::Native
    } else {
        SkillExecutionMode::Adapter
    }
}

fn adapter_kinds(spec: &SkillSpec) -> BTreeSet<SkillAdapterKind> {
    let mut kinds = BTreeSet::new();
    let category = custom_category(spec);
    if category == Some("search") {
        kinds.insert(SkillAdapterKind::Search);
        kinds.insert(SkillAdapterKind::Llm);
    }
    if spec.tags.iter().any(|tag| tag == "content_engine") {
        kinds.insert(SkillAdapterKind::Llm);
    }
    if spec
        .tags
        .iter()
        .any(|tag| tag == "design_engine" || tag == "analysis_engine")
    {
        kinds.insert(SkillAdapterKind::Llm);
    }
    if category == Some("data") && spec.tags.iter().any(|tag| tag == "deferred") {
        kinds.insert(SkillAdapterKind::Db);
    }
    if spec.tags.iter().any(|tag| tag == "pending_approval") || spec.id.starts_with("platform_") {
        kinds.insert(SkillAdapterKind::Platform);
    }
    kinds
}

fn risk_level(spec: &SkillSpec) -> SkillRiskLevel {
    if spec.tags.iter().any(|tag| tag == "pending_approval") || spec.id.starts_with("platform_") {
        SkillRiskLevel::High
    } else if spec.tags.iter().any(|tag| tag == "deferred")
        || custom_category(spec) == Some("search")
    {
        SkillRiskLevel::Medium
    } else {
        SkillRiskLevel::Low
    }
}

fn add_runtime_metadata(
    metadata: &mut Map<String, Value>,
    spec: &SkillSpec,
    mode: SkillExecutionMode,
) {
    metadata.insert("execution_mode".to_string(), json!(mode));
    metadata.insert("risk_level".to_string(), json!(risk_level(spec)));
    metadata.insert(
        "approval_required".to_string(),
        json!(risk_level(spec) == SkillRiskLevel::High),
    );
    let adapters = adapter_kinds(spec).into_iter().collect::<Vec<_>>();
    if !adapters.is_empty() {
        metadata.insert("adapter_requirements".to_string(), json!(adapters));
    }
}

fn status_from_output(output: &Value) -> Option<SkillExecutionStatus> {
    let status = output.get("status")?.as_str()?;
    match status {
        "deferred" => Some(SkillExecutionStatus::Deferred),
        "pending_approval" => Some(SkillExecutionStatus::PendingApproval),
        _ => None,
    }
}

fn custom_category(spec: &SkillSpec) -> Option<&str> {
    match &spec.category {
        SkillCategory::Custom(value) => Some(value.as_str()),
        _ => None,
    }
}

async fn run_known_skill(
    id: &str,
    input: Value,
    context: SkillContext,
    search_config: &SearchAdapterConfig,
) -> Result<SkillOutcome, SkillError> {
    match custom_category_from_id(id) {
        "ops" => return ops::execute(id, input, &context).await,
        "accounting" => return accounting::execute(id, input, &context).await,
        "data" | "query" => return data::execute(id, input, &context).await,
        _ => {}
    }

    match id {
        "search_trends" => {
            search::SearchTrends::new()
                .run_with_adapter(input, context, search_config)
                .await
        }
        "search_competitor" => {
            search::SearchCompetitor::new()
                .run_with_adapter(input, context, search_config)
                .await
        }
        "search_market_info" => {
            search::SearchMarketInfo::new()
                .run_with_adapter(input, context, search_config)
                .await
        }
        "search_product_reviews" => {
            search::SearchProductReviews::new()
                .run_with_adapter(input, context, search_config)
                .await
        }
        "search_platform_policy" => {
            search::SearchPlatformPolicy::new()
                .run_with_adapter(input, context, search_config)
                .await
        }
        "search_industry_benchmarks" => {
            search::SearchIndustryBenchmarks::new()
                .run_with_adapter(input, context, search_config)
                .await
        }
        "service_ticket_handler" => {
            service::ServiceTicketHandler::new()
                .run(input, context)
                .await
        }
        "service_faq_playbook" => service::ServiceFaqPlaybook::new().run(input, context).await,
        "service_escalation_flow" => {
            service::ServiceEscalationFlow::new()
                .run(input, context)
                .await
        }
        "service_dsr_improvement" => {
            service::ServiceDsrImprovement::new()
                .run(input, context)
                .await
        }
        "service_return_handler" => {
            service::ServiceReturnHandler::new()
                .run(input, context)
                .await
        }
        "service_query_product" => {
            service::ServiceQueryProduct::new()
                .run(input, context)
                .await
        }
        "service_nps_analyzer" => service::ServiceNpsAnalyzer::new().run(input, context).await,
        "service_sentiment_analyzer" => {
            service::ServiceSentimentAnalyzer::new()
                .run(input, context)
                .await
        }
        "service_nps_driver_analysis" => {
            service::ServiceNpsDriverAnalysis::new()
                .run(input, context)
                .await
        }
        "creative_video_script" => {
            creative::CreativeVideoScript::new()
                .run(input, context)
                .await
        }
        "creative_seeding_copy" => {
            creative::CreativeSeedingCopy::new()
                .run(input, context)
                .await
        }
        "creative_ip_branding" => {
            creative::CreativeIpBranding::new()
                .run(input, context)
                .await
        }
        "creative_content_calendar" => {
            creative::CreativeContentCalendar::new()
                .run(input, context)
                .await
        }
        "creative_trend_catch" => {
            creative::CreativeTrendCatch::new()
                .run(input, context)
                .await
        }
        "creative_live_script" => {
            creative::CreativeLiveScript::new()
                .run(input, context)
                .await
        }
        "creative_product_article" => {
            creative::CreativeProductArticle::new()
                .run(input, context)
                .await
        }
        "creative_title_ctr_scorer" => {
            creative::CreativeTitleCtrScorer::new()
                .run(input, context)
                .await
        }
        "web_seo_optimize" => web::WebSeoOptimize::new().run(input, context).await,
        "web_title_generator" => web::WebTitleGenerator::new().run(input, context).await,
        "web_page_conversion" => web::WebPageConversion::new().run(input, context).await,
        "web_store_design" => web::WebStoreDesign::new().run(input, context).await,
        "web_keyword_research" => web::WebKeywordResearch::new().run(input, context).await,
        "web_product_description" => web::WebProductDescription::new().run(input, context).await,
        "web_title_seo_scorer" => web::WebTitleSeoScorer::new().run(input, context).await,
        "design_main_image" => design::DesignMainImage::new().run(input, context).await,
        "design_detail_page" => design::DesignDetailPage::new().run(input, context).await,
        "design_color_scheme" => design::DesignColorScheme::new().run(input, context).await,
        "design_material_spec" => design::DesignMaterialSpec::new().run(input, context).await,
        "design_campaign_poster" => {
            design::DesignCampaignPoster::new()
                .run(input, context)
                .await
        }
        "engineering_arch_review" => {
            engineering::EngineeringArchReview::new()
                .run(input, context)
                .await
        }
        "engineering_bug_analysis" => {
            engineering::EngineeringBugAnalysis::new()
                .run(input, context)
                .await
        }
        "engineering_perf_optimize" => {
            engineering::EngineeringPerfOptimize::new()
                .run(input, context)
                .await
        }
        "engineering_tech_selection" => {
            engineering::EngineeringTechSelection::new()
                .run(input, context)
                .await
        }
        "engineering_system_check" => {
            engineering::EngineeringSystemCheck::new()
                .run(input, context)
                .await
        }
        "engineering_sla_monitor" => {
            engineering::EngineeringSlaMonitor::new()
                .run(input, context)
                .await
        }
        "coordination_agent_handoff" => {
            coordination::CoordinationAgentHandoff::new()
                .run(input, context)
                .await
        }
        "coordination_task_orchestration" => {
            coordination::CoordinationTaskOrchestration::new()
                .run(input, context)
                .await
        }
        "coordination_status_query" => {
            coordination::CoordinationStatusQuery::new()
                .run(input, context)
                .await
        }
        "coordination_create_campaign" => {
            coordination::CoordinationCreateCampaign::new()
                .run(input, context)
                .await
        }
        "coordination_create_task" => {
            coordination::CoordinationCreateTask::new()
                .run(input, context)
                .await
        }
        "coordination_save_memory" => {
            coordination::CoordinationSaveMemory::new()
                .run(input, context)
                .await
        }
        "platform_update_price" => {
            coordination::PlatformUpdatePrice::new()
                .run(input, context)
                .await
        }
        "platform_update_inventory" => {
            coordination::PlatformUpdateInventory::new()
                .run(input, context)
                .await
        }
        "platform_sync_products" => {
            coordination::PlatformSyncProducts::new()
                .run(input, context)
                .await
        }
        "coordination_skill_chain_planner" => {
            coordination::CoordinationSkillChainPlanner::new()
                .run(input, context)
                .await
        }
        "coordination_external_agent_call" => {
            coordination::CoordinationExternalAgentCall::new()
                .run(input, context)
                .await
        }
        _ => Err(SkillError::execution(format!(
            "skill executor missing: {id}"
        ))),
    }
}

fn custom_category_from_id(id: &str) -> &str {
    id.split_once('_').map(|(prefix, _)| prefix).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_registry_loads_all_specs() {
        let registry = SkillRegistry::staged().expect("registry should load");
        assert_eq!(registry.len(), 93);
        assert!(registry.get("ops_pricing_strategy").is_some());
        assert!(registry.get("missing_skill").is_none());
    }

    #[test]
    fn registry_rejects_duplicate_ids() {
        let specs = vec![
            SkillSpec::new("dup", "A", "one"),
            SkillSpec::new("dup", "B", "two"),
        ];
        assert!(matches!(
            SkillRegistry::from_specs(specs),
            Err(SkillRuntimeError::DuplicateSkillId(id)) if id == "dup"
        ));
    }

    #[test]
    fn registry_filters_by_category_tag_and_mode() {
        let registry = SkillRegistry::staged().expect("registry should load");
        assert_eq!(
            registry
                .by_category(&SkillCategory::Custom("ops".to_string()))
                .len(),
            11
        );
        assert!(!registry.by_tag("deferred").is_empty());
        assert!(
            !registry
                .by_execution_mode(SkillExecutionMode::Deferred)
                .is_empty()
        );
    }

    #[tokio::test]
    async fn executor_runs_native_ops_skill() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let result = executor
            .execute(SkillRequest::new(
                "req-1",
                "ops_pricing_strategy",
                json!({"cost": 10.0, "competitor_prices": [18.0, 20.0]}),
            ))
            .await;

        assert_eq!(result.status, SkillExecutionStatus::Success);
        assert_eq!(
            result.metadata.get("execution_mode"),
            Some(&json!(SkillExecutionMode::Native))
        );
        assert!(result.output.is_some());
    }

    #[tokio::test]
    async fn executor_runs_representative_native_categories() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let cases = [
            (
                "accounting_cost_calc",
                json!({"purchase_cost": 20.0, "selling_price": 49.0}),
            ),
            ("data_funnel_analysis", json!({})),
            (
                "data_dashboard",
                json!({"gmv": 180000.0, "previous_gmv": 150000.0, "orders": 1800, "visitors": 36000}),
            ),
            ("service_dsr_improvement", json!({})),
            (
                "creative_title_ctr_scorer",
                json!({"title": "春夏新款轻便通勤保温杯"}),
            ),
            (
                "web_title_seo_scorer",
                json!({"title": "春夏新款轻便通勤保温杯", "target_keywords": ["保温杯"]}),
            ),
            ("engineering_sla_monitor", json!({})),
            (
                "coordination_agent_handoff",
                json!({"task_description": "分析店铺转化率下滑并给出运营动作"}),
            ),
        ];

        for (index, (skill_id, input)) in cases.into_iter().enumerate() {
            let result = executor
                .execute(SkillRequest::new(
                    format!("req-native-{index}"),
                    skill_id,
                    input,
                ))
                .await;
            assert_eq!(
                result.status,
                SkillExecutionStatus::Success,
                "{skill_id} failed: {:?}",
                result.error
            );
            assert_eq!(
                result.metadata.get("execution_mode"),
                Some(&json!(SkillExecutionMode::Native)),
                "{skill_id} used unexpected execution mode"
            );
        }
    }

    #[tokio::test]
    async fn executor_runs_native_creative_content_skills() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let cases = [
            (
                "creative_video_script",
                json!({"product_name": "冷萃咖啡", "video_type": "种草", "duration": 30, "platform": "抖音"}),
            ),
            (
                "creative_seeding_copy",
                json!({"product_name": "冷萃咖啡", "target_audience": "通勤白领"}),
            ),
            (
                "creative_ip_branding",
                json!({"brand_name": "示例咖啡品牌", "category": "咖啡饮品"}),
            ),
            (
                "creative_content_calendar",
                json!({"period": "周", "weekly_posts": 5}),
            ),
            (
                "creative_trend_catch",
                json!({"category": "咖啡饮品", "platform": "抖音"}),
            ),
            (
                "creative_live_script",
                json!({"duration_hours": 2.0, "product_count": 4, "platform": "抖音"}),
            ),
            (
                "creative_product_article",
                json!({"product_name": "冷萃咖啡", "article_type": "测评软文"}),
            ),
        ];

        for (index, (skill_id, input)) in cases.into_iter().enumerate() {
            let result = executor
                .execute(SkillRequest::new(
                    format!("req-creative-native-{index}"),
                    skill_id,
                    input,
                ))
                .await;
            assert_eq!(
                result.status,
                SkillExecutionStatus::Success,
                "{skill_id} failed: {:?}",
                result.error
            );
            assert_eq!(
                result.metadata.get("execution_mode"),
                Some(&json!(SkillExecutionMode::Native)),
                "{skill_id} used unexpected execution mode"
            );
            assert!(result.output.is_some(), "{skill_id} returned no output");
        }
    }

    #[tokio::test]
    async fn executor_runs_native_design_and_web_growth_skills() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let cases = [
            (
                "design_main_image",
                json!({"product_name": "冷萃咖啡", "platform": "抖音", "style": "年轻活力"}),
            ),
            (
                "design_detail_page",
                json!({"product_name": "冷萃咖啡", "platform": "淘宝"}),
            ),
            (
                "design_color_scheme",
                json!({"brand_tone": "高端奢华", "category": "咖啡饮品"}),
            ),
            (
                "design_material_spec",
                json!({"usage": "信息流广告", "platforms": ["抖音", "小红书"]}),
            ),
            (
                "design_campaign_poster",
                json!({"campaign_name": "618大促", "discount_info": "满300减50"}),
            ),
            (
                "web_seo_optimize",
                json!({"target_keywords": ["冷萃咖啡", "低糖"], "platform": "淘宝"}),
            ),
            (
                "web_title_generator",
                json!({"product_name": "冷萃咖啡", "keywords": ["冷萃咖啡", "低糖"]}),
            ),
            ("web_page_conversion", json!({"page_type": "商品详情"})),
            (
                "web_store_design",
                json!({"store_type": "旗舰店", "category": "咖啡饮品"}),
            ),
            (
                "web_keyword_research",
                json!({"seed_keyword": "冷萃咖啡", "category": "咖啡饮品"}),
            ),
            (
                "web_product_description",
                json!({"product_name": "冷萃咖啡", "style": "营销型"}),
            ),
        ];

        for (index, (skill_id, input)) in cases.into_iter().enumerate() {
            let result = executor
                .execute(SkillRequest::new(
                    format!("req-design-web-native-{index}"),
                    skill_id,
                    input,
                ))
                .await;
            assert_eq!(
                result.status,
                SkillExecutionStatus::Success,
                "{skill_id} failed: {:?}",
                result.error
            );
            assert_eq!(
                result.metadata.get("execution_mode"),
                Some(&json!(SkillExecutionMode::Native)),
                "{skill_id} used unexpected execution mode"
            );
            assert!(result.output.is_some(), "{skill_id} returned no output");
        }
    }

    #[tokio::test]
    async fn executor_runs_native_service_support_skills() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let cases = [
            (
                "service_ticket_handler",
                json!({
                    "issue_type": "物流延迟",
                    "customer_message": "客户很生气，说快递三天没动，准备投诉平台",
                    "customer_emotion": "威胁投诉"
                }),
            ),
            (
                "service_faq_playbook",
                json!({
                    "questions": ["什么时候发货？", "支持退货退款吗？怎么操作？"],
                    "platform": "淘宝"
                }),
            ),
            (
                "service_escalation_flow",
                json!({
                    "issue_description": "客户威胁投诉平台，已经联系3次还没解决退款",
                    "customer_emotion": "威胁投诉",
                    "previous_contacts": 3
                }),
            ),
            (
                "service_dsr_improvement",
                json!({
                    "description_score": 4.5,
                    "service_score": 4.3,
                    "logistics_score": 4.7
                }),
            ),
        ];

        for (index, (skill_id, input)) in cases.into_iter().enumerate() {
            let result = executor
                .execute(SkillRequest::new(
                    format!("req-service-native-{index}"),
                    skill_id,
                    input,
                ))
                .await;
            assert_eq!(
                result.status,
                SkillExecutionStatus::Success,
                "{skill_id} failed: {:?}",
                result.error
            );
            assert_eq!(
                result.metadata.get("execution_mode"),
                Some(&json!(SkillExecutionMode::Native)),
                "{skill_id} used unexpected execution mode"
            );
            assert!(result.output.is_some(), "{skill_id} returned no output");
        }
    }

    #[tokio::test]
    async fn executor_runs_service_query_product_with_context() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let context = SkillContext::new()
            .with_metadata("request.context.product_ids", json!("product-1"))
            .with_metadata("request.context.refs", json!("product:product-1"))
            .with_metadata(
                "request.context.selected_summary",
                json!("商品：冷萃咖啡。规格：12瓶装。卖点：低糖、即饮、适合办公室囤货。"),
            )
            .with_metadata(
                "request.context.sources_json",
                json!(
                    json!([
                        {
                            "source_type": "product",
                            "id": "product-1",
                            "product_id": "product-1",
                            "title": "冷萃咖啡"
                        }
                    ])
                    .to_string()
                ),
            );
        let result = executor
            .execute(
                SkillRequest::new(
                    "req-service-query-product-context",
                    "service_query_product",
                    json!({"product_name": "冷萃咖啡"}),
                )
                .with_context(context),
            )
            .await;

        assert_eq!(result.status, SkillExecutionStatus::Success);
        assert_eq!(
            result.metadata.get("execution_mode"),
            Some(&json!(SkillExecutionMode::Native))
        );
        let output = result.output.as_ref().expect("skill output");
        assert_eq!(
            output.get("adapter_status"),
            Some(&json!("context_metadata"))
        );
        assert_eq!(
            output.get("data_readiness"),
            Some(&json!("explicit_product_context"))
        );
    }

    #[tokio::test]
    async fn executor_validates_required_input() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let result = executor
            .execute(SkillRequest::new(
                "req-2",
                "ops_pricing_strategy",
                json!({}),
            ))
            .await;

        assert_eq!(result.status, SkillExecutionStatus::ValidationError);
        assert!(result.error.unwrap_or_default().contains("cost"));
    }

    #[tokio::test]
    async fn executor_validates_request_envelope() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let missing_request_id = executor
            .execute(SkillRequest::new(
                "",
                "ops_pricing_strategy",
                json!({"cost": 10.0}),
            ))
            .await;
        assert_eq!(
            missing_request_id.status,
            SkillExecutionStatus::ValidationError
        );
        assert!(
            missing_request_id
                .error
                .unwrap_or_default()
                .contains("request_id")
        );

        let missing_skill_id = executor
            .execute(SkillRequest::new("req-empty-skill", "", json!({})))
            .await;
        assert_eq!(
            missing_skill_id.status,
            SkillExecutionStatus::ValidationError
        );
        assert!(
            missing_skill_id
                .error
                .unwrap_or_default()
                .contains("skill_id")
        );
    }

    #[tokio::test]
    async fn executor_returns_deferred_for_search() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let result = executor
            .execute(SkillRequest::new(
                "req-3",
                "search_trends",
                json!({"query": "咖啡"}),
            ))
            .await;

        assert_eq!(result.status, SkillExecutionStatus::Deferred);
        let output = result.output.as_ref().expect("search output");
        assert_eq!(output.get("status"), Some(&json!("deferred")));
        assert_eq!(
            output.get("adapter_status"),
            Some(&json!("adapter_required"))
        );
        assert_eq!(output.pointer("/adapter/kind"), Some(&json!("search")));
        assert_eq!(output.pointer("/adapter/configured"), Some(&json!(false)));
        assert_eq!(output.get("sources"), Some(&json!([])));
        assert_eq!(output.pointer("/data/items"), Some(&json!([])));
        assert_eq!(output.get("confidence"), Some(&json!(0.0)));
        assert_eq!(output.get("degraded"), Some(&json!(true)));
        assert!(output.get("_search_engine").is_none());
        assert!(
            output
                .pointer("/query/planned")
                .and_then(Value::as_str)
                .is_some_and(|query| query.contains("咖啡"))
        );
        assert_eq!(
            result.metadata.get("risk_level"),
            Some(&json!(SkillRiskLevel::Medium))
        );
        assert_eq!(
            result.metadata.get("execution_mode"),
            Some(&json!(SkillExecutionMode::Adapter))
        );
        assert!(result.metadata.contains_key("adapter_requirements"));
    }

    #[tokio::test]
    async fn executor_returns_structured_deferred_for_search_skills() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let cases = [
            (
                "search_trends",
                json!({"query": "咖啡", "platform": "抖音", "days": 14}),
                "general",
                14,
            ),
            (
                "search_competitor",
                json!({"query": "冷萃咖啡", "focus": "price"}),
                "general",
                60,
            ),
            (
                "search_market_info",
                json!({"query": "咖啡饮品", "topic": "news"}),
                "news",
                7,
            ),
            (
                "search_product_reviews",
                json!({"query": "冷萃咖啡", "focus": "negative"}),
                "general",
                90,
            ),
            (
                "search_platform_policy",
                json!({"platform": "抖音", "policy_type": "content", "days": 21}),
                "news",
                21,
            ),
            (
                "search_industry_benchmarks",
                json!({"industry": "咖啡饮品", "metric": "roi"}),
                "general",
                180,
            ),
        ];

        for (index, (skill_id, input, expected_topic, expected_days)) in
            cases.into_iter().enumerate()
        {
            let result = executor
                .execute(SkillRequest::new(
                    format!("req-search-contract-{index}"),
                    skill_id,
                    input,
                ))
                .await;

            assert_eq!(
                result.status,
                SkillExecutionStatus::Deferred,
                "{skill_id} should remain deferred until search adapter is configured"
            );
            assert_eq!(
                result.metadata.get("execution_mode"),
                Some(&json!(SkillExecutionMode::Adapter)),
                "{skill_id} should be adapter mode"
            );
            assert_eq!(
                result.metadata.get("adapter_requirements"),
                Some(&json!([SkillAdapterKind::Search, SkillAdapterKind::Llm])),
                "{skill_id} adapter requirements changed"
            );
            assert_eq!(
                result.metadata.get("adapter_status"),
                Some(&json!("adapter_required")),
                "{skill_id} adapter status missing"
            );

            let output = result.output.as_ref().expect("search output");
            assert_eq!(
                output.get("adapter_status"),
                Some(&json!("adapter_required")),
                "{skill_id} output adapter status missing"
            );
            assert_eq!(
                output.pointer("/query/topic"),
                Some(&json!(expected_topic)),
                "{skill_id} topic mismatch"
            );
            assert_eq!(
                output.pointer("/query/days"),
                Some(&json!(expected_days)),
                "{skill_id} days mismatch"
            );
            assert_eq!(
                output.pointer("/query/max_results"),
                Some(&json!(6)),
                "{skill_id} max_results mismatch"
            );
            assert_eq!(
                output.pointer("/adapter/provider"),
                Some(&Value::Null),
                "{skill_id} must not claim a provider"
            );
            assert_eq!(
                output.pointer("/adapter/configured"),
                Some(&json!(false)),
                "{skill_id} must report unconfigured adapter"
            );
            assert_eq!(
                output.get("sources"),
                Some(&json!([])),
                "{skill_id} must not fake sources"
            );
            assert_eq!(
                output.pointer("/data/items"),
                Some(&json!([])),
                "{skill_id} must not fake search items"
            );
            assert_eq!(
                output.pointer("/provenance/source_count"),
                Some(&json!(0)),
                "{skill_id} must not fake provenance"
            );
            assert_eq!(
                output.get("confidence"),
                Some(&json!(0.0)),
                "{skill_id} confidence must be zero without sources"
            );
            assert_eq!(
                output.get("degraded"),
                Some(&json!(true)),
                "{skill_id} must be degraded"
            );
            assert!(
                output
                    .get("degraded_reason")
                    .and_then(Value::as_str)
                    .is_some_and(|reason| reason.contains("Search adapter is not configured")),
                "{skill_id} missing degraded reason"
            );
            assert!(
                output.get("_search_engine").is_none(),
                "{skill_id} must not claim a search engine"
            );
            assert!(
                output
                    .pointer("/query/planned")
                    .and_then(Value::as_str)
                    .is_some_and(|query| !query.trim().is_empty()),
                "{skill_id} planned query missing"
            );
        }
    }

    #[tokio::test]
    async fn executor_reports_degraded_when_configured_search_provider_fails() {
        let executor = SkillExecutorSet::staged_with_search(SearchAdapterConfig {
            enabled: true,
            provider: Some("mock-search".to_string()),
            base_url: Some("http://127.0.0.1:9".to_string()),
            search_endpoint: Some("/search".to_string()),
            ..SearchAdapterConfig::default()
        })
        .expect("executor should load");
        let result = executor
            .execute(SkillRequest::new(
                "req-search-provider-fails",
                "search_trends",
                json!({"query": "咖啡"}),
            ))
            .await;

        assert_eq!(result.status, SkillExecutionStatus::Deferred);
        assert_eq!(
            result.metadata.get("execution_mode"),
            Some(&json!(SkillExecutionMode::Adapter))
        );
        assert_eq!(
            result.metadata.get("adapter_status"),
            Some(&json!("degraded"))
        );
        let output = result.output.as_ref().expect("search output");
        assert_eq!(output.get("status"), Some(&json!("deferred")));
        assert_eq!(output.get("adapter_status"), Some(&json!("degraded")));
        assert_eq!(output.pointer("/adapter/configured"), Some(&json!(true)));
        assert_eq!(
            output.pointer("/adapter/provider"),
            Some(&json!("mock-search"))
        );
        assert_eq!(output.get("sources"), Some(&json!([])));
        assert_eq!(output.pointer("/data/items"), Some(&json!([])));
        assert_eq!(output.get("degraded"), Some(&json!(true)));
        assert!(
            output
                .get("degraded_reason")
                .and_then(Value::as_str)
                .is_some_and(|reason| reason.contains("provider request failed")
                    || reason.contains("Connection refused")
                    || reason.contains("error sending request")),
            "unexpected degraded reason: {:?}",
            output.get("degraded_reason")
        );
    }

    #[tokio::test]
    async fn executor_blocks_platform_write_as_pending_approval() {
        let executor = SkillExecutorSet::staged().expect("executor should load");
        let result = executor
            .execute(SkillRequest::new(
                "req-4",
                "platform_update_price",
                json!({"platform": "taobao", "product_id": "p1", "new_price": 99.0}),
            ))
            .await;

        assert_eq!(result.status, SkillExecutionStatus::PendingApproval);
        assert_eq!(result.metadata.get("approval_required"), Some(&json!(true)));
        assert_eq!(
            result.metadata.get("risk_level"),
            Some(&json!(SkillRiskLevel::High))
        );
    }
}
