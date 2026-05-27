use serde_json::Value;

use crate::skills::{
    SearchAdapterConfig, Skill, SkillContext, SkillError, SkillOutcome, accounting, coordination,
    creative, data, design, douyin_ads, engineering, learning, material, ops, search, service, web,
};

pub(super) async fn run_known_skill(
    id: &str,
    input: Value,
    context: SkillContext,
    search_config: &SearchAdapterConfig,
) -> Result<SkillOutcome, SkillError> {
    match custom_category_from_id(id) {
        "ops" => return ops::execute(id, input, &context).await,
        "accounting" => return accounting::execute(id, input, &context).await,
        "data" | "query" => return data::execute(id, input, &context).await,
        "douyin" => return douyin_ads::execute(id, input, &context).await,
        _ => {}
    }

    match id {
        "product_opportunity_score" => ops::execute(id, input, &context).await,
        "product_market_fit_diagnosis" => ops::execute(id, input, &context).await,
        "experiment_mvp_plan" => ops::execute(id, input, &context).await,
        "experiment_review_report" => ops::execute(id, input, &context).await,
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
        "learning_teaching_explain" => {
            learning::LearningTeachingExplain::new()
                .run(input, context)
                .await
        }
        "material_file_understanding" => {
            material::MaterialFileUnderstanding::new()
                .run(input, context)
                .await
        }
        "material_table_profiler" => {
            material::MaterialTableProfiler::new()
                .run(input, context)
                .await
        }
        "material_image_diagnosis" => {
            material::MaterialImageDiagnosis::new()
                .run(input, context)
                .await
        }
        "material_evidence_extractor" => {
            material::MaterialEvidenceExtractor::new()
                .run(input, context)
                .await
        }
        "material_multi_file_compare" => {
            material::MaterialMultiFileCompare::new()
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
