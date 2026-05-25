use std::{collections::BTreeMap, sync::Arc};

use agent::api::{
    AgentId, AgentManager, AgentManagerConfig, AgentRequest, Cognition, CognitionInput,
    CognitionResult, RoleProfile, SessionEvent, SessionId, UserId,
};
use async_trait::async_trait;
use serde_json::{Value, json};

struct RoleRouteCommander {
    runtime_role: &'static str,
}

#[async_trait]
impl Cognition for RoleRouteCommander {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "RouteTask",
                "route": {
                    "target_agent_id": format!("worker.{}", self.runtime_role),
                    "task_summary": "native skill coverage",
                    "goal": "execute native skills for coverage",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "test selected role owns these skills",
                    "expected_output": "native skill results"
                },
                "clarification": {
                    "question": ""
                }
            },
            "confidence": 1.0,
            "rationale": {
                "primary": "coverage test route",
                "evidence": [],
                "alternatives_considered": []
            }
        }))
    }
}

struct SkillBatchWorker {
    runtime_role: String,
}

#[async_trait]
impl Cognition for SkillBatchWorker {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        let requests = skill_requests_for_role(&self.runtime_role);
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "skill_requests": requests,
            "role_output": {
                "summary": format!("{} native skill batch requested", self.runtime_role),
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": [],
                "open_questions": []
            }
        }))
    }
}

#[tokio::test]
async fn data_role_executes_native_data_skills_with_manual_inputs() {
    let events = run_skill_batch("data", "分析 GMV、漏斗、归因、LTV 和预测数据").await;

    assert_skill_status(&events, "data_dashboard", "success");
    assert_skill_status(&events, "data_funnel_analysis", "success");
    assert_skill_status(&events, "data_ab_test_analyzer", "success");
    assert_skill_status(&events, "data_attribution_analysis", "success");
    assert_skill_status(&events, "query_store_metrics", "success");
    assert_no_failed_event(&events);
}

#[tokio::test]
async fn ops_role_executes_native_optimization_skills_with_manual_inputs() {
    let events = run_skill_batch("ops", "做库存优化、智能定价、广告疲劳和 ABC-XYZ 分类").await;

    assert_skill_status(&events, "ops_smart_pricing", "success");
    assert_skill_status(&events, "ops_inventory_optimizer", "success");
    assert_skill_status(&events, "ops_ad_fatigue_detector", "success");
    assert_skill_status(&events, "ops_abc_xyz_classifier", "success");
    assert_no_failed_event(&events);
}

#[tokio::test]
async fn creative_design_web_service_and_coordination_native_skills_execute() {
    for (role, input, expected) in [
        (
            "creative",
            "生成标题评分和种草内容",
            vec!["creative_title_ctr_scorer", "creative_seeding_copy"],
        ),
        (
            "design",
            "生成主图、详情页、配色和素材规范",
            vec![
                "design_main_image",
                "design_detail_page",
                "design_color_scheme",
                "design_material_spec",
            ],
        ),
        (
            "web",
            "生成 SEO、标题、关键词和详情页文案",
            vec![
                "web_seo_optimize",
                "web_title_generator",
                "web_keyword_research",
                "web_product_description",
            ],
        ),
        (
            "service",
            "处理客服工单、FAQ、情绪和满意度",
            vec![
                "service_ticket_handler",
                "service_faq_playbook",
                "service_sentiment_analyzer",
                "service_nps_analyzer",
            ],
        ),
        (
            "engineering",
            "查询 agent 状态并做技术健康检查",
            vec!["coordination_status_query"],
        ),
    ] {
        let events = run_skill_batch(role, input).await;
        for skill_id in expected {
            assert_skill_status(&events, skill_id, "success");
        }
        assert_no_failed_event(&events);
    }
}

async fn run_skill_batch(runtime_role: &'static str, input: &'static str) -> Vec<SessionEvent> {
    let manager = AgentManager::new(AgentManagerConfig::new_role_aware(
        Arc::new(move || Ok(Box::new(RoleRouteCommander { runtime_role }))),
        Arc::new(|role: &RoleProfile| {
            Ok(Box::new(SkillBatchWorker {
                runtime_role: role.runtime_role.clone(),
            }))
        }),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "collab".to_string());
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "single".to_string(),
    );
    metadata.insert(
        "strategy.primary_role".to_string(),
        runtime_role.to_string(),
    );
    metadata.insert(
        "strategy.selected_roles".to_string(),
        runtime_role.to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId(format!("skill-coverage-user-{runtime_role}"))),
            workspace_id: None,
            agent_id: AgentId(format!("skill-coverage-agent-{runtime_role}")),
            session_id: SessionId(format!("skill-coverage-session-{runtime_role}")),
            input: input.to_string(),
            metadata,
        })
        .expect("stream should start");

    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        let terminal = matches!(
            event,
            SessionEvent::Finished { .. } | SessionEvent::Failed { .. }
        );
        events.push(event);
        if terminal {
            break;
        }
    }
    events
}

fn assert_skill_status(events: &[SessionEvent], skill_id: &str, expected_status: &str) {
    assert!(
        events.iter().any(|event| matches!(
            event,
            SessionEvent::SkillResult {
                skill_id: actual_skill_id,
                status,
                execution_mode,
                ..
            } if actual_skill_id == skill_id
                && status == expected_status
                && execution_mode.as_deref() == Some("native")
        )),
        "expected {skill_id} to finish with {expected_status}; events: {events:#?}"
    );
}

fn assert_no_failed_event(events: &[SessionEvent]) {
    assert!(
        !events.iter().any(|event| {
            matches!(event, SessionEvent::Failed { .. })
                || matches!(
                    event,
                    SessionEvent::SkillResult {
                        status,
                        ..
                    } if status == "failed"
                )
        }),
        "stream should not fail: {events:#?}"
    );
}

fn skill_requests_for_role(runtime_role: &str) -> Value {
    let requests = match runtime_role {
        "data" => vec![
            request(
                "data_dashboard",
                json!({
                    "platform": "抖音",
                    "days": 30,
                    "gmv": 186000,
                    "previous_gmv": 151000,
                    "orders": 1560,
                    "previous_orders": 1300,
                    "visitors": 42000,
                    "previous_visitors": 39000,
                    "clicks": 9200,
                    "ad_spend": 36000,
                    "refund_amount": 5400,
                    "gross_profit": 62000,
                    "target_gmv": 200000
                }),
            ),
            request(
                "data_funnel_analysis",
                json!({
                    "exposure": 100000,
                    "clicks": 13800,
                    "add_to_cart": 2400,
                    "orders": 1080,
                    "paid_orders": 1000,
                    "gmv": 168000
                }),
            ),
            request(
                "data_ab_test_analyzer",
                json!({
                    "control_visitors": 10000,
                    "control_conversions": 510,
                    "test_visitors": 9800,
                    "test_conversions": 650,
                    "ab_test_name": "主图卖点 A/B"
                }),
            ),
            request(
                "data_attribution_analysis",
                json!({
                    "channels": [
                        {"name": "抖音", "gmv": 90000, "cost": 18000},
                        {"name": "小红书", "gmv": 52000, "cost": 9000},
                        {"name": "搜索", "gmv": 36000, "cost": 4200}
                    ]
                }),
            ),
            request(
                "query_store_metrics",
                json!({
                    "platform": "all",
                    "platforms": [
                        {"platform": "douyin", "gmv": 90000, "orders": 820, "visitors": 21000, "clicks": 5000, "ad_spend": 18000},
                        {"platform": "xiaohongshu", "gmv": 52000, "orders": 430, "visitors": 12000, "clicks": 2700, "ad_spend": 9000}
                    ]
                }),
            ),
        ],
        "ops" => vec![
            request(
                "ops_smart_pricing",
                json!({
                    "cost": 68,
                    "current_price": 129,
                    "current_monthly_sales": 1800,
                    "competitor_prices": [119, 139, 149],
                    "platform": "淘宝",
                    "elasticity": -1.7,
                    "stock_days": 132
                }),
            ),
            request(
                "ops_inventory_optimizer",
                json!({
                    "lead_time_days": 12,
                    "service_level": "95%",
                    "skus": [
                        {"name": "标准款", "monthly_sales": 900, "cost": 48, "stock_qty": 520},
                        {"name": "礼盒款", "monthly_sales": 360, "cost": 76, "stock_qty": 1100},
                        {"name": "试用装", "monthly_sales": 1200, "cost": 18, "stock_qty": 300}
                    ]
                }),
            ),
            request(
                "ops_ad_fatigue_detector",
                json!({
                    "platform": "抖音",
                    "channels": [
                        {"name": "信息流A", "ctr_baseline": 4.2, "ctr_recent": 2.7, "impressions_per_user": 8.5, "budget": 12000, "roas": 2.1},
                        {"name": "达人素材B", "ctr_baseline": 3.6, "ctr_recent": 3.4, "impressions_per_user": 3.2, "budget": 6000, "roas": 4.3}
                    ]
                }),
            ),
            request(
                "ops_abc_xyz_classifier",
                json!({
                    "skus": [
                        {"name": "A款", "gmv": 120000, "monthly_sales": [320, 340, 310]},
                        {"name": "B款", "gmv": 52000, "monthly_sales": [120, 90, 150]},
                        {"name": "C款", "gmv": 18000, "monthly_sales": [30, 35, 28]}
                    ]
                }),
            ),
            request(
                "coordination_skill_chain_planner",
                json!({
                    "goal": "提升新品首月转化",
                    "role": "ops",
                    "action": "optimize",
                    "skills_available": ["data_funnel_analysis", "ops_smart_pricing", "creative_title_ctr_scorer"]
                }),
            ),
        ],
        "engineering" => vec![
            request(
                "coordination_status_query",
                json!({
                    "query_type": "agent_status",
                    "request_reason": "coverage check"
                }),
            ),
            request(
                "engineering_system_check",
                json!({
                    "platform": "douyin",
                    "test_live": false
                }),
            ),
        ],
        "creative" => vec![
            request(
                "creative_title_ctr_scorer",
                json!({
                    "title": "春季敏感肌修护精华",
                    "platform": "小红书",
                    "audience": "敏感肌女性"
                }),
            ),
            request(
                "creative_seeding_copy",
                json!({
                    "product_name": "云朵修护精华",
                    "platform": "小红书",
                    "audience": "敏感肌新手",
                    "selling_points": ["舒缓", "轻薄", "妆前可用"]
                }),
            ),
        ],
        "design" => vec![
            request(
                "design_main_image",
                json!({"product_name": "云朵修护精华", "platform": "淘宝", "style": "清爽实验室"}),
            ),
            request(
                "design_detail_page",
                json!({"product_name": "云朵修护精华", "platform": "京东", "extra_requirements": "突出敏感肌"}),
            ),
            request(
                "design_color_scheme",
                json!({"brand_tone": "温和可信", "category": "护肤"}),
            ),
            request(
                "design_material_spec",
                json!({"platforms": ["淘宝", "抖音"], "usage": "详情页"}),
            ),
        ],
        "web" => vec![
            request(
                "web_seo_optimize",
                json!({"target_keywords": ["敏感肌精华", "修护精华"], "current_rank": 18}),
            ),
            request(
                "web_title_generator",
                json!({"product_name": "云朵修护精华", "keywords": ["敏感肌", "舒缓"]}),
            ),
            request(
                "web_keyword_research",
                json!({"seed_keyword": "敏感肌修护精华", "platform": "百度"}),
            ),
            request(
                "web_product_description",
                json!({"product_name": "云朵修护精华", "features": ["舒缓泛红", "清爽不黏"]}),
            ),
        ],
        "service" => vec![
            request(
                "service_ticket_handler",
                json!({
                    "issue_type": "破损补发",
                    "ticket": "客户反馈收到后瓶身破损，希望尽快补发并赔偿。",
                    "order_status": "已签收"
                }),
            ),
            request(
                "service_faq_playbook",
                json!({"product_name": "云朵修护精华", "questions": ["敏感肌能用吗", "多久见效"]}),
            ),
            request(
                "service_sentiment_analyzer",
                json!({"customer_text": "等了好久还没到，太差了！我要投诉；客服后来处理很快，挺满意"}),
            ),
            request(
                "service_nps_analyzer",
                json!({"promoters": 620, "passives": 210, "detractors": 70, "csat": 88}),
            ),
        ],
        _ => Vec::new(),
    };
    Value::Array(requests)
}

fn request(skill_id: &str, input: Value) -> Value {
    json!({
        "skill_id": skill_id,
        "input": input,
        "reason": format!("coverage request for {skill_id}")
    })
}
