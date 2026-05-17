use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
};

use agent::api::{
    AgentId, AgentManager, AgentManagerConfig, AgentRequest, Cognition, CognitionInput,
    CognitionResult, InMemoryMemoryStore, MemoryKind, RoleProfile, SessionEvent, SessionId, UserId,
};
use async_trait::async_trait;
use serde_json::json;

const CONFIGURED_DOUYIN_CONVERSION_INPUT: &str =
    "我是做咖啡运营的，主要营销平台是抖音，现在马上就五一了，请给提升转化率的运营方案";

struct CapturingCommanderCognition {
    available_workers: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl Cognition for CapturingCommanderCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        let count = input
            .context
            .metadata
            .get("available_workers.count")
            .cloned();
        *self.available_workers.lock().expect("lock capture") = count;

        CognitionResult::Success(json!({
            "decision": {
                "kind": "RouteTask",
                "route": {
                    "target_agent_id": "worker.ops",
                    "task_summary": "default roles",
                    "goal": "verify default roles",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "ops is a default role worker",
                    "expected_output": "done"
                },
                "clarification": {
                    "question": ""
                }
            },
            "confidence": 1.0,
            "rationale": {
                "primary": "default role catalog test",
                "evidence": [],
                "alternatives_considered": []
            }
        }))
    }
}

struct RouteWithoutTargetCommanderCognition;

#[async_trait]
impl Cognition for RouteWithoutTargetCommanderCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "RouteTask",
                "route": {
                    "target_agent_id": "",
                    "task_summary": "route without target",
                    "goal": "let role router choose the worker",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "router should choose",
                    "expected_output": "done"
                },
                "clarification": {
                    "question": ""
                }
            },
            "confidence": 1.0,
            "rationale": {
                "primary": "role router fallback test",
                "evidence": [],
                "alternatives_considered": []
            }
        }))
    }
}

struct NoopWorkerCognition;

#[async_trait]
impl Cognition for NoopWorkerCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": test_role_output(&input)
        }))
    }
}

struct CapabilityIntroWorkerCognition;

#[async_trait]
impl Cognition for CapabilityIntroWorkerCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        let task = input.intent.description;
        let summary = if task.contains("能帮我完成什么任务") || task.contains("你能做什么")
        {
            "我可以帮你完成普通问答、需求澄清、方案整理、文案草拟，以及数据、运营、设计、\
             技术等方向的初步分析。复杂任务我会交给更合适的专业角色协作处理。"
        } else {
            "我在，可以直接告诉我你的问题或目标。"
        };

        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": {
                "summary": summary,
                "findings": ["这是普通聊天能力介绍，不需要启动多智能体"],
                "recommendations": ["用户可以继续描述具体任务"],
                "evidence": ["原始任务"],
                "risks": [],
                "open_questions": []
            }
        }))
    }
}

struct ShortAnswerWorkerCognition;

#[async_trait]
impl Cognition for ShortAnswerWorkerCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": {
                "summary": "42",
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": [],
                "open_questions": []
            }
        }))
    }
}

struct StructuredSummaryWorkerCognition;

#[async_trait]
impl Cognition for StructuredSummaryWorkerCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": {
                "summary": "已经接入问答模型。",
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": [],
                "open_questions": []
            }
        }))
    }
}

struct SkillRequestWorkerCognition;

#[async_trait]
impl Cognition for SkillRequestWorkerCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "skill_requests": [
                {
                    "skill_id": "coordination_agent_handoff",
                    "input": {
                        "current_agent": "chat",
                        "task_description": "需要运营角色继续拆解咖啡转化方案",
                        "context": "测试 skill SSE 事件"
                    },
                    "reason": "需要把普通问答交接给更合适的角色"
                }
            ],
            "role_output": {
                "summary": "已请求协同交接技能",
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": [],
                "open_questions": []
            }
        }))
    }
}

struct AutosuggestSkillWorkerCognition;

#[async_trait]
impl Cognition for AutosuggestSkillWorkerCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": {
                "summary": "需要先分析转化漏斗，但本轮模型没有显式输出 skill_requests。",
                "findings": ["曝光到点击存在待验证断点", "点击到加购需要拆分承接"],
                "recommendations": ["先跑漏斗分析", "再判断页面和投放动作"],
                "evidence": ["原始任务"],
                "risks": ["缺少真实口径时只能做初步判断"],
                "open_questions": []
            }
        }))
    }
}

struct CapturingWorkerCognition {
    runtime_role: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl Cognition for CapturingWorkerCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        *self.runtime_role.lock().expect("lock role capture") =
            input.context.metadata.get("role.runtime_role").cloned();

        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": test_role_output(&input)
        }))
    }
}

struct CapturingContextWorkerCognition {
    metadata_value: Arc<Mutex<Option<String>>>,
    fact_value: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl Cognition for CapturingContextWorkerCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        *self.metadata_value.lock().expect("lock metadata capture") = input
            .context
            .metadata
            .get("request.context.selected_summary")
            .cloned()
            .or_else(|| {
                input
                    .context
                    .metadata
                    .get("request.context.summary")
                    .cloned()
            });
        *self.fact_value.lock().expect("lock fact capture") = input
            .context
            .facts
            .iter()
            .find(|fact| fact.source == "commerce.selected_product_context")
            .map(|fact| fact.content.clone());

        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": test_role_output(&input)
        }))
    }
}

struct CapturingConversationHistoryWorkerCognition {
    intent_value: Arc<Mutex<Option<String>>>,
    metadata_value: Arc<Mutex<Option<String>>>,
    fact_value: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl Cognition for CapturingConversationHistoryWorkerCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        *self.intent_value.lock().expect("lock intent capture") =
            Some(input.intent.description.clone());
        *self.metadata_value.lock().expect("lock metadata capture") = input
            .context
            .metadata
            .get("request.conversation.history_summary")
            .cloned();
        *self.fact_value.lock().expect("lock fact capture") = input
            .context
            .facts
            .iter()
            .find(|fact| fact.source == "conversation.history")
            .map(|fact| fact.content.clone());

        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": test_role_output(&input)
        }))
    }
}

struct CapturingWorkerSetCognition {
    runtime_roles: Arc<Mutex<HashSet<String>>>,
}

#[async_trait]
impl Cognition for CapturingWorkerSetCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        if let Some(runtime_role) = input.context.metadata.get("role.runtime_role") {
            self.runtime_roles
                .lock()
                .expect("lock role set capture")
                .insert(runtime_role.clone());
        }

        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": test_role_output(&input)
        }))
    }
}

fn test_role_output(input: &CognitionInput) -> serde_json::Value {
    let role = input
        .context
        .metadata
        .get("role.runtime_role")
        .map(String::as_str)
        .unwrap_or("chat");
    let mut evidence = vec!["原始任务".to_string()];
    if input
        .context
        .metadata
        .get("assignment.input_count")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
        > 0
    {
        evidence.push("worker.assignment.input.data".to_string());
    }

    match role {
        "data" => json!({
            "summary": "数据角色已完成漏斗指标和归因口径诊断。",
            "findings": ["漏斗指标需要拆解", "归因口径需要统一"],
            "recommendations": ["按渠道和页面阶段复核转化断点"],
            "evidence": evidence,
            "risks": ["口径不一致会导致误判"],
            "open_questions": []
        }),
        "creative" => json!({
            "summary": "创意角色已形成直播脚本文案方向。",
            "findings": ["内容需要突出首段钩子"],
            "recommendations": ["输出三版直播脚本开头和标题文案"],
            "evidence": evidence,
            "risks": ["不能编造产品功效或用户反馈"],
            "open_questions": []
        }),
        "ops" => json!({
            "summary": "运营角色已形成可执行运营调整方案。",
            "findings": ["运营目标需要转化链路承接"],
            "recommendations": ["P1 优化首屏卖点", "P2 调整投放人群", "P3 每日复盘转化"],
            "evidence": evidence,
            "risks": ["预算消耗需要止损线"],
            "open_questions": []
        }),
        _ => json!({
            "summary": "测试 worker 已完成当前角色节点。",
            "findings": ["已基于当前任务形成判断"],
            "recommendations": ["按角色边界推进下一步"],
            "evidence": evidence,
            "risks": ["需要确认上下文边界"],
            "open_questions": []
        }),
    }
}

#[tokio::test]
async fn manager_initializes_selected_roles_for_each_session() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session".to_string()),
            input: "你能解释一下这个 agent 是什么吗".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert_eq!(manager.active_session_count(), 0);
}

#[tokio::test]
async fn role_worker_receives_role_context() {
    let runtime_role = Arc::new(Mutex::new(None));
    let worker_capture = Arc::clone(&runtime_role);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingWorkerCognition {
                runtime_role: Arc::clone(&worker_capture),
            }))
        }),
    ));

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-context".to_string()),
            input: "你能解释一下这个 agent 是什么吗".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert_eq!(
        runtime_role.lock().expect("lock role capture").as_deref(),
        Some("chat")
    );
    assert_eq!(manager.active_session_count(), 0);
}

#[tokio::test]
async fn role_aware_worker_factory_receives_selected_roles() {
    let available_workers = Arc::new(Mutex::new(None));
    let constructed_roles = Arc::new(Mutex::new(HashSet::new()));
    let commander_capture = Arc::clone(&available_workers);
    let role_capture = Arc::clone(&constructed_roles);
    let manager = AgentManager::new(AgentManagerConfig::new_role_aware(
        Arc::new(move || {
            Ok(Box::new(CapturingCommanderCognition {
                available_workers: Arc::clone(&commander_capture),
            }))
        }),
        Arc::new(move |role: &RoleProfile| {
            role_capture
                .lock()
                .expect("lock constructed roles")
                .insert(role.runtime_role.clone());
            Ok(Box::new(NoopWorkerCognition))
        }),
    ));

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-factory".to_string()),
            input: CONFIGURED_DOUYIN_CONVERSION_INPUT.to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    let constructed_roles = constructed_roles.lock().expect("lock constructed roles");
    assert!(constructed_roles.contains("chat"));
    assert!(constructed_roles.contains("ops"));
    assert!(constructed_roles.contains("data"));
    assert!(constructed_roles.contains("accounting"));
    assert!(constructed_roles.contains("creative"));
    assert!(!constructed_roles.contains("engineering"));
    assert!(!constructed_roles.contains("web"));
}

#[tokio::test]
async fn commander_routes_plain_question_to_chat_role() {
    let runtime_role = Arc::new(Mutex::new(None));
    let worker_capture = Arc::clone(&runtime_role);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingWorkerCognition {
                runtime_role: Arc::clone(&worker_capture),
            }))
        }),
    ));

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-chat".to_string()),
            input: "你能解释一下这个 agent 是什么吗".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert_eq!(
        runtime_role.lock().expect("lock role capture").as_deref(),
        Some("chat")
    );
}

#[tokio::test]
async fn quick_chat_output_is_direct_reply() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-hi".to_string()),
            input: "hi".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert!(result.output.contains("测试 worker 已完成当前角色节点"));
    assert!(!result.output.contains("多智能体任务图已完成"));
    assert!(!result.output.contains("通过的角色产物"));
}

#[tokio::test]
async fn capability_question_returns_complete_chat_summary_without_internal_sections() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(CapabilityIntroWorkerCognition))),
    ));

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-capability".to_string()),
            input: "你能帮我完成什么任务呢".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert!(result.output.contains("普通问答"));
    assert!(result.output.contains("需求澄清"));
    assert!(result.output.contains("专业角色协作处理"));
    assert!(!result.output.contains("发现："));
    assert!(!result.output.contains("建议："));
    assert!(!result.output.contains("多智能体任务图已完成"));
}

#[tokio::test]
async fn task_graph_chat_short_answer_returns_direct_output() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(ShortAnswerWorkerCognition))),
    ));

    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "execution".to_string());
    metadata.insert(
        "strategy.response_mode".to_string(),
        "execution".to_string(),
    );
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "auto".to_string(),
    );

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-short-answer".to_string()),
            input: "只输出 17+25 的计算结果，不要解释。".to_string(),
            metadata,
        })
        .await
        .expect("request should complete");

    assert_eq!(result.output, "42");
    assert!(
        !result
            .output
            .contains("quality completeness: summary is too thin")
    );
    assert!(!result.output.contains("多智能体任务图已完成"));
    assert!(result.evaluation_summary.is_none());
    assert!(result.task_graphs.is_empty());
}

#[tokio::test]
async fn collab_strategy_direct_answer_bypasses_task_graph() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(ShortAnswerWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "collab".to_string());
    metadata.insert(
        "strategy.response_mode".to_string(),
        "execution".to_string(),
    );
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "manual".to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-collab-short-answer".to_string()),
            input: "只输出 17+25 的计算结果，不要解释。".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                strategy_id,
                collaboration_mode,
                route_mode,
                ..
            } if strategy_id.as_deref() == Some("collab")
                && collaboration_mode.as_deref() == Some("manual")
                && route_mode == "chat_bypass"
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::Output { content, .. } if content == "42"
        )
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::CommanderThinking { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskGraphPlanned { .. }))
    );
}

#[tokio::test]
async fn explicit_chat_role_selection_bypasses_task_graph_for_execution_strategy() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(StructuredSummaryWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "execution".to_string());
    metadata.insert(
        "strategy.response_mode".to_string(),
        "execution".to_string(),
    );
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "single".to_string(),
    );
    metadata.insert("strategy.primary_role".to_string(), "chat".to_string());
    metadata.insert("strategy.selected_roles".to_string(), "chat".to_string());

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-execution-chat".to_string()),
            input: "璇疯瘖鏂竴涓挅鍟＄洿鎾棿婕忔枟".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                strategy_id,
                collaboration_mode,
                route_mode,
                primary_runtime_role,
                support_runtime_roles,
                ..
            } if strategy_id.as_deref() == Some("execution")
                && collaboration_mode.as_deref() == Some("single")
                && route_mode == "chat_bypass"
                && primary_runtime_role.as_deref() == Some("chat")
                && support_runtime_roles.is_empty()
        )
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::CommanderThinking { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskGraphPlanned { .. }))
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::Output { content, .. } if !content.trim().is_empty()
        )
    }));
}

#[tokio::test]
async fn explicit_manual_roles_do_not_use_direct_answer_chat_bypass() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "collab".to_string());
    metadata.insert(
        "strategy.response_mode".to_string(),
        "execution".to_string(),
    );
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "manual".to_string(),
    );
    metadata.insert("strategy.primary_role".to_string(), "data".to_string());
    metadata.insert(
        "strategy.selected_roles".to_string(),
        "data,ops".to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-manual-direct-answer".to_string()),
            input: "answer only 17+25, no explanation.".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                strategy_id,
                collaboration_mode,
                route_mode,
                primary_runtime_role,
                support_runtime_roles,
                ..
            } if strategy_id.as_deref() == Some("collab")
                && collaboration_mode.as_deref() == Some("manual")
                && route_mode == "task_graph"
                && primary_runtime_role.as_deref() == Some("data")
                && support_runtime_roles == &vec!["ops".to_string()]
        )
    }));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::CommanderThinking { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskGraphPlanned { .. }))
    );
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided { route_mode, .. } if route_mode == "chat_bypass"
        )
    }));
}

#[tokio::test]
async fn fast_worker_structured_output_returns_summary_not_json() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(StructuredSummaryWorkerCognition))),
    ));

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-fast-summary".to_string()),
            input: "你能解释一下这个 agent 是什么吗".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert_eq!(result.output, "已经接入问答模型。");
    assert!(!result.output.contains("\"role_output\""));
    assert!(!result.output.contains("\"decision\""));
}

#[tokio::test]
async fn commander_uses_role_router_when_target_worker_is_missing() {
    let runtime_roles = Arc::new(Mutex::new(HashSet::new()));
    let worker_capture = Arc::clone(&runtime_roles);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingWorkerSetCognition {
                runtime_roles: Arc::clone(&worker_capture),
            }))
        }),
    ));

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-router".to_string()),
            input: "帮我分析漏斗指标和归因口径".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    let runtime_roles = runtime_roles.lock().expect("lock role set capture");
    assert!(runtime_roles.contains("data"));
    assert!(!runtime_roles.contains("chat"));
}

#[tokio::test]
async fn commander_dispatches_task_graph_roles_after_upstream_role() {
    let runtime_roles = Arc::new(Mutex::new(HashSet::new()));
    let worker_capture = Arc::clone(&runtime_roles);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingWorkerSetCognition {
                runtime_roles: Arc::clone(&worker_capture),
            }))
        }),
    ));

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-support".to_string()),
            input: CONFIGURED_DOUYIN_CONVERSION_INPUT.to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    let runtime_roles = runtime_roles.lock().expect("lock role set capture");
    assert!(runtime_roles.contains("data"));
    assert!(runtime_roles.contains("accounting"));
    assert!(runtime_roles.contains("creative"));
    assert!(result.output.contains("多智能体任务图已完成"));
    assert!(result.output.contains("数据诊断"));
    assert!(result.output.contains("数据分析师"));
    assert!(result.output.contains("预算评估"));
    assert!(result.output.contains("财务分析师"));
    assert!(result.output.contains("创意产出"));
    assert!(result.output.contains("内容创意师"));
    assert!(result.output.contains("运营方案"));
    assert!(result.output.contains("运营策略师"));
}

#[tokio::test]
async fn task_graph_result_carries_memory_snapshot_fields() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-memory-snapshot".to_string()),
            input: CONFIGURED_DOUYIN_CONVERSION_INPUT.to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert_eq!(result.task_graphs.len(), 1);
    assert!(result.worker_details.iter().any(|detail| {
        detail.role == "data"
            && detail.title == "数据诊断"
            && detail.objective.contains("分析")
            && detail
                .role_output
                .as_ref()
                .is_some_and(|output| output.summary.contains("数据角色已完成"))
            && detail
                .evaluation
                .as_ref()
                .is_some_and(|evaluation| evaluation.passed)
    }));
    assert!(result.worker_details.iter().any(|detail| {
        detail.role == "creative"
            && detail.role_output.as_ref().is_some_and(|output| {
                output
                    .recommendations
                    .iter()
                    .any(|item| item.contains("直播"))
            })
    }));
    assert!(
        result
            .role_summaries
            .iter()
            .any(|summary| summary.role == "data" && summary.summary.contains("数据角色已完成"))
    );
    assert!(
        result
            .role_summaries
            .iter()
            .any(|summary| summary.role == "creative"
                && summary.summary.contains("创意角色已形成"))
    );
    assert!(result.risks.iter().any(|risk| risk.contains("口径不一致")));
    assert!(
        result
            .evaluation_summary
            .as_deref()
            .is_some_and(|summary| summary.contains("passed=true"))
    );
}

#[tokio::test]
async fn task_graph_memory_persists_structured_output_without_prompt_leaks() {
    let memory_store = Arc::new(InMemoryMemoryStore::new());
    let mut config = AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    );
    config.memory_store = memory_store.clone();
    let manager = AgentManager::new(config);

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-memory-persist".to_string()),
            input: CONFIGURED_DOUYIN_CONVERSION_INPUT.to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    let records = memory_store.records();
    assert!(
        records
            .iter()
            .any(|record| record.kind == MemoryKind::CrossRoleContext
                && record.content.contains("数据角色已完成"))
    );
    assert!(
        records
            .iter()
            .any(|record| record.tags.contains(&"risk".to_string())
                && record.content.contains("口径不一致"))
    );

    let joined_memory = records
        .iter()
        .map(|record| record.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "system prompt",
        "developer instructions",
        "role prompt",
        "sandbox policy",
        "runtime config",
    ] {
        assert!(
            !joined_memory.to_lowercase().contains(forbidden),
            "memory should not contain {forbidden}"
        );
    }
}

#[tokio::test]
async fn stream_emits_task_graph_events() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-collab-stream".to_string()),
            input: CONFIGURED_DOUYIN_CONVERSION_INPUT.to_string(),
            metadata: Default::default(),
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

    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::WorkerStarted { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::WorkerFinished { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskGraphPlanned { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskNodeStarted { .. }))
    );
    assert!(events.iter().any(|event| matches!(
        event,
        SessionEvent::TaskGraphFinished { process, .. }
            if process.graph_id.0.starts_with("graph-")
                && !process.roles.is_empty()
                && process.panel.default_collapsed
                && process.panel.anchor == "answer_scoped_process"
    )));
}

#[tokio::test]
async fn collab_strategy_metadata_emits_route_decision_and_task_graph() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "collab".to_string());
    metadata.insert(
        "strategy.response_mode".to_string(),
        "execution".to_string(),
    );
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "manual".to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-collab-metadata".to_string()),
            input: "帮我分析漏斗指标、预算风险和创意方向".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                strategy_id,
                collaboration_mode,
                route_mode,
                ..
            } if strategy_id.as_deref() == Some("collab")
                && collaboration_mode.as_deref() == Some("manual")
                && route_mode == "task_graph"
        )
    }));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskGraphPlanned { .. }))
    );
}

#[tokio::test]
async fn manual_strategy_metadata_routes_requested_roles() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "collab".to_string());
    metadata.insert("strategy.response_mode".to_string(), "learning".to_string());
    metadata.insert(
        "strategy.learning_level".to_string(),
        "professional".to_string(),
    );
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "manual".to_string(),
    );
    metadata.insert("strategy.primary_role".to_string(), "creative".to_string());
    metadata.insert(
        "strategy.selected_roles".to_string(),
        "data,creative,ops".to_string(),
    );
    metadata.insert("model.name".to_string(), "ollama-gpt-oss-20b".to_string());

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-manual-roles".to_string()),
            input: "请让创意、数据和运营协同优化直播素材".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                response_mode,
                collaboration_mode,
                learning_level,
                model_name,
                route_mode,
                primary_runtime_role,
                support_runtime_roles,
                ..
            } if response_mode.as_deref() == Some("learning")
                && collaboration_mode.as_deref() == Some("manual")
                && learning_level.as_deref() == Some("professional")
                && model_name.as_deref() == Some("ollama-gpt-oss-20b")
                && route_mode == "task_graph"
                && primary_runtime_role.as_deref() == Some("creative")
                && support_runtime_roles == &vec!["data".to_string(), "ops".to_string()]
        )
    }));
}

#[tokio::test]
async fn single_strategy_metadata_routes_only_requested_primary_role() {
    let runtime_roles = Arc::new(Mutex::new(HashSet::new()));
    let worker_capture = Arc::clone(&runtime_roles);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingWorkerSetCognition {
                runtime_roles: Arc::clone(&worker_capture),
            }))
        }),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "single".to_string(),
    );
    metadata.insert("strategy.primary_role".to_string(), "data".to_string());
    metadata.insert(
        "strategy.selected_roles".to_string(),
        "data,creative,ops".to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-single-role".to_string()),
            input: "请只让数据角色分析漏斗".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                collaboration_mode,
                primary_runtime_role,
                support_runtime_roles,
                ..
            } if collaboration_mode.as_deref() == Some("single")
                && primary_runtime_role.as_deref() == Some("data")
                && support_runtime_roles.is_empty()
        )
    }));
    let runtime_roles = runtime_roles.lock().expect("lock role set capture");
    assert!(runtime_roles.contains("data"));
    assert!(!runtime_roles.contains("creative"));
    assert!(!runtime_roles.contains("ops"));
}

#[tokio::test]
async fn stream_bypasses_commander_for_quick_chat() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-hi-stream".to_string()),
            input: "hi".to_string(),
            metadata: Default::default(),
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

    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::WorkerStarted { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::WorkerFinished { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::Output { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::CommanderThinking { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::TaskGraphPlanned { .. }))
    );
}

#[tokio::test]
async fn stream_emits_skill_execution_events() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(SkillRequestWorkerCognition))),
    ));

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-skill-stream".to_string()),
            input: "hi".to_string(),
            metadata: Default::default(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::SkillRequested {
                skill_id,
                role,
                reason,
                ..
            } if skill_id == "coordination_agent_handoff"
                && role == "chat"
                && reason.as_deref() == Some("需要把普通问答交接给更合适的角色")
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::SkillResult {
                skill_id,
                role,
                status,
                summary,
                execution_mode,
                risk_level,
                approval_required,
                ..
            } if skill_id == "coordination_agent_handoff"
                && role == "chat"
                && status == "success"
                && summary.as_deref() == Some("Agent交接建议已生成")
                && execution_mode.as_deref() == Some("native")
                && risk_level.as_deref() == Some("low")
                && approval_required == &Some(false)
        )
    }));
}

#[tokio::test]
async fn stream_emits_autosuggested_skill_events_when_worker_omits_tool_json() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(AutosuggestSkillWorkerCognition))),
    ));

    let mut metadata = BTreeMap::new();
    metadata.insert("strategy.id".to_string(), "collab".to_string());
    metadata.insert(
        "strategy.collaboration_mode".to_string(),
        "manual".to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-autosuggest-skill".to_string()),
            input: "请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::SkillRequested {
                skill_id,
                role,
                reason,
                ..
            } if skill_id == "data_funnel_analysis"
                && role == "data"
                && reason
                    .as_deref()
                    .is_some_and(|value| value.contains("自动补偿漏斗分析"))
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::SkillResult {
                skill_id,
                role,
                status,
                execution_mode,
                risk_level,
                ..
            } if skill_id == "data_funnel_analysis"
                && role == "data"
                && status == "success"
                && execution_mode.as_deref() == Some("native")
                && risk_level.as_deref() == Some("low")
        )
    }));
}

#[tokio::test]
async fn stream_routes_default_funnel_question_to_data_skill_without_manual_collab() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(AutosuggestSkillWorkerCognition))),
    ));

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-default-funnel-skill".to_string()),
            input: "请分析转化漏斗：曝光10000 点击500 加购120 下单60 付款54".to_string(),
            metadata: Default::default(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::RouteDecided {
                route_mode,
                primary_runtime_role,
                ..
            } if route_mode == "task_graph"
                && primary_runtime_role.as_deref() == Some("data")
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::SkillRequested {
                skill_id,
                role,
                ..
            } if skill_id == "data_funnel_analysis" && role == "data"
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::SkillResult {
                skill_id,
                role,
                status,
                ..
            } if skill_id == "data_funnel_analysis"
                && role == "data"
                && status == "success"
        )
    }));
}

#[tokio::test]
async fn stream_emits_context_loaded_event_from_request_metadata() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "context.product_ids".to_string(),
        "11111111-1111-1111-1111-111111111111".to_string(),
    );
    metadata.insert("context.product_count".to_string(), "1".to_string());
    metadata.insert("context.knowledge_count".to_string(), "2".to_string());
    metadata.insert("context.source_count".to_string(), "3".to_string());
    metadata.insert(
        "context.selected_summary".to_string(),
        "Product source product:111 | name=五一咖啡组合".to_string(),
    );
    metadata.insert(
        "context.sources_json".to_string(),
        r#"[{"source_type":"product","id":"111","title":"五一咖啡组合"}]"#.to_string(),
    );

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-context-loaded".to_string()),
            input: "hi".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::ContextLoaded {
                product_ids,
                product_count,
                knowledge_count,
                source_count,
                summary,
                ..
            } if product_ids == &vec!["11111111-1111-1111-1111-111111111111".to_string()]
                && product_count == &1
                && knowledge_count == &2
                && source_count == &3
                && summary.as_deref().is_some_and(|value| value.contains("五一咖啡组合"))
        )
    }));
}

#[tokio::test]
async fn stream_does_not_emit_product_context_from_generic_context_summary() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(|| Ok(Box::new(NoopWorkerCognition))),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "context.summary".to_string(),
        "Workspace context only".to_string(),
    );
    metadata.insert("context.knowledge_count".to_string(), "2".to_string());

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-context-summary-only".to_string()),
            input: "hi".to_string(),
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

    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionEvent::ContextLoaded {
                product_ids,
                product_count,
                summary,
                ..
            } if product_ids.is_empty() && product_count == &0 && summary.is_none()
        )
    }));
}

#[tokio::test]
async fn worker_cognition_receives_product_context_metadata_and_fact() {
    let metadata_value = Arc::new(Mutex::new(None));
    let fact_value = Arc::new(Mutex::new(None));
    let metadata_capture = Arc::clone(&metadata_value);
    let fact_capture = Arc::clone(&fact_value);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingContextWorkerCognition {
                metadata_value: Arc::clone(&metadata_capture),
                fact_value: Arc::clone(&fact_capture),
            }))
        }),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "context.selected_summary".to_string(),
        "Product source product:111 | name=五一咖啡组合".to_string(),
    );

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-worker-context".to_string()),
            input: "hi".to_string(),
            metadata,
        })
        .await
        .expect("request should complete");

    assert!(
        metadata_value
            .lock()
            .expect("lock metadata result")
            .as_deref()
            .is_some_and(|value| value.contains("五一咖啡组合"))
    );
    assert!(
        fact_value
            .lock()
            .expect("lock fact result")
            .as_deref()
            .is_some_and(|value| value.contains("五一咖啡组合"))
    );
}

#[tokio::test]
async fn worker_cognition_receives_conversation_history_as_dedicated_fact() {
    let intent_value = Arc::new(Mutex::new(None));
    let metadata_value = Arc::new(Mutex::new(None));
    let fact_value = Arc::new(Mutex::new(None));
    let intent_capture = Arc::clone(&intent_value);
    let metadata_capture = Arc::clone(&metadata_value);
    let fact_capture = Arc::clone(&fact_value);
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(RouteWithoutTargetCommanderCognition))),
        Arc::new(move || {
            Ok(Box::new(CapturingConversationHistoryWorkerCognition {
                intent_value: Arc::clone(&intent_capture),
                metadata_value: Arc::clone(&metadata_capture),
                fact_value: Arc::clone(&fact_capture),
            }))
        }),
    ));
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "conversation.history_summary".to_string(),
        "User: 直播间曝光8000、点击640、加购160、下单80、付款72。\nAssistant: 已计算各段转化率并给出三个优化动作。"
            .to_string(),
    );

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("role-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("role-agent".to_string()),
            session_id: SessionId("role-session-conversation-history".to_string()),
            input: "继续沿用本会话刚才的漏斗数据，把三个动作排成今天可执行清单。".to_string(),
            metadata,
        })
        .await
        .expect("request should complete");

    assert!(
        intent_value
            .lock()
            .expect("lock intent result")
            .as_deref()
            .is_some_and(|value| {
                value.contains("同一会话历史上下文")
                    && value.contains("曝光8000")
                    && value.contains("当前用户请求")
            })
    );
    assert!(
        metadata_value
            .lock()
            .expect("lock metadata result")
            .as_deref()
            .is_some_and(|value| value.contains("曝光8000"))
    );
    assert!(
        fact_value
            .lock()
            .expect("lock fact result")
            .as_deref()
            .is_some_and(|value| {
                value.contains("conversation")
                    || (value.contains("同一会话历史上下文") && value.contains("付款72"))
            })
    );
    assert!(
        fact_value
            .lock()
            .expect("lock fact result")
            .as_deref()
            .is_some_and(|value| value.contains("不要要求用户重复"))
    );
}
