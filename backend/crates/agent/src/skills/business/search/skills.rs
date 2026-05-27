use async_trait::async_trait;
use serde_json::Value;

use super::{
    config::SearchAdapterConfig,
    output::{
        SearchSourceFilterStats, deferred_search_output, degraded_search_output, live_search_output,
    },
    provider::{
        optional_search_string_param, query_provider, search_bypass_cache, search_max_results,
    },
    queries::{
        build_benchmark_query, build_competitor_query, build_market_info_query, build_policy_query,
        build_reviews_query, build_trends_query,
    },
    specs::{
        search_competitor_spec, search_industry_benchmarks_spec, search_market_info_spec,
        search_platform_policy_spec, search_product_reviews_spec, search_trends_spec,
    },
};
use crate::skills::{Skill, SkillContext, SkillResult, SkillSpec, object_params, string_param};

macro_rules! deferred_search_skill {
    ($type_name:ident, $spec_fn:ident, $required:literal, $builder:expr) => {
        pub struct $type_name {
            spec: SkillSpec,
        }

        impl $type_name {
            pub fn new() -> Self {
                Self { spec: $spec_fn() }
            }

            pub async fn run_with_adapter(
                &self,
                params: Value,
                _context: SkillContext,
                adapter: &SearchAdapterConfig,
            ) -> SkillResult {
                object_params(&params)?;
                let required_value = string_param(&params, $required)?;
                let bypass_cache = search_bypass_cache(&params)?;
                let max_results = search_max_results(&params)?;
                let product_ref = optional_search_string_param(&params, "product_ref")?;
                let product_name = optional_search_string_param(&params, "product_name")?;
                let product_category = optional_search_string_param(&params, "product_category")?;
                let plan = $builder(&params, &required_value)?
                    .with_bypass_cache(bypass_cache)
                    .with_product_context(product_ref, product_name, product_category)
                    .with_adapter_defaults(adapter)?
                    .with_max_results(max_results);
                if !adapter.is_configured() {
                    return Ok(deferred_search_output(
                        &self.spec.id,
                        required_value,
                        plan,
                        adapter,
                    ));
                }

                let payload = match query_provider(adapter, &plan).await {
                    Ok(payload) => payload,
                    Err(reason) => {
                        return Ok(degraded_search_output(
                            &self.spec.id,
                            required_value,
                            plan,
                            adapter,
                            reason,
                            SearchSourceFilterStats::default(),
                        ));
                    }
                };
                Ok(live_search_output(
                    &self.spec.id,
                    required_value,
                    plan,
                    adapter,
                    &payload,
                ))
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #[async_trait]
        impl Skill for $type_name {
            fn spec(&self) -> &SkillSpec {
                &self.spec
            }

            async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
                self.run_with_adapter(params, context, &SearchAdapterConfig::default())
                    .await
            }
        }
    };
}

deferred_search_skill!(
    SearchTrends,
    search_trends_spec,
    "query",
    build_trends_query
);
deferred_search_skill!(
    SearchCompetitor,
    search_competitor_spec,
    "query",
    build_competitor_query
);
deferred_search_skill!(
    SearchMarketInfo,
    search_market_info_spec,
    "query",
    build_market_info_query
);
deferred_search_skill!(
    SearchProductReviews,
    search_product_reviews_spec,
    "query",
    build_reviews_query
);
deferred_search_skill!(
    SearchPlatformPolicy,
    search_platform_policy_spec,
    "platform",
    build_policy_query
);
deferred_search_skill!(
    SearchIndustryBenchmarks,
    search_industry_benchmarks_spec,
    "industry",
    build_benchmark_query
);
