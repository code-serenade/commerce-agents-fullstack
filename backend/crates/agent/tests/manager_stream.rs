use std::{sync::Arc, time::Duration};

use agent::api::{
    AgentId, AgentManager, AgentManagerConfig, AgentRequest, Cognition, CognitionInput,
    CognitionResult, RoleCognitionFactory, SessionEvent, SessionId, UserId,
};
use async_trait::async_trait;
use serde_json::json;

struct TestCommanderCognition;

#[async_trait]
impl Cognition for TestCommanderCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "RouteTask",
                "route": {
                    "target_agent_id": "worker.ops",
                    "task_summary": "stream integration",
                    "goal": "finish stream integration",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "default worker",
                    "expected_output": "done"
                },
                "clarification": {
                    "question": ""
                }
            },
            "confidence": 1.0,
            "rationale": {
                "primary": "stream integration test",
                "evidence": [],
                "alternatives_considered": []
            }
        }))
    }
}

struct TestWorkerCognition;

#[async_trait]
impl Cognition for TestWorkerCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            }
        }))
    }
}

struct SlowConstructedWorkerCognition;

#[async_trait]
impl Cognition for SlowConstructedWorkerCognition {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            }
        }))
    }
}

#[tokio::test]
async fn manager_stream_finishes_and_cleans_active_session() {
    let manager = AgentManager::new(AgentManagerConfig::new(
        Arc::new(|| Ok(Box::new(TestCommanderCognition))),
        Arc::new(|| Ok(Box::new(TestWorkerCognition))),
    ));

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("test-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("test-agent".to_string()),
            session_id: SessionId("test-session".to_string()),
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

    assert!(matches!(events.first(), Some(SessionEvent::Started { .. })));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::Output { .. }))
    );
    let output = events
        .iter()
        .find_map(|event| match event {
            SessionEvent::Output { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .expect("stream should emit final output");
    let chunks = events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::LlmChunk { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!chunks.is_empty());
    assert_eq!(chunks.join(""), output);
    assert!(matches!(events.last(), Some(SessionEvent::Finished { .. })));
    assert_eq!(manager.active_session_count(), 0);
}

#[tokio::test]
async fn manager_stream_emits_started_before_slow_worker_construction() {
    let worker_factory: RoleCognitionFactory = Arc::new(|_| {
        std::thread::sleep(Duration::from_millis(250));
        Ok(Box::new(SlowConstructedWorkerCognition))
    });
    let manager = AgentManager::new(AgentManagerConfig::new_role_aware(
        Arc::new(|| Ok(Box::new(TestCommanderCognition))),
        worker_factory,
    ));

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("test-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("test-agent".to_string()),
            session_id: SessionId("test-slow-startup".to_string()),
            input: "hi".to_string(),
            metadata: Default::default(),
        })
        .expect("stream should start");

    let first = tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("started should be emitted before worker cognition construction completes")
        .expect("stream should stay open");

    assert!(matches!(first, SessionEvent::Started { .. }));
}
