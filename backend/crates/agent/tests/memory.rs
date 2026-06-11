use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use agent::api::{
    AgentId, AgentManager, AgentManagerConfig, AgentRequest, Cognition, CognitionInput,
    CognitionResult, FileMemoryStore, FileMemoryStoreError, InMemoryMemoryStore, MemoryBudget,
    MemoryCompactionPolicy, MemoryId, MemoryKind, MemoryQuery, MemoryRecord, MemoryScope,
    MemorySource, MemoryStore, RoleMemorySummary, SessionEvent, SessionId, SessionSnapshot,
    TaskGraphHandoff, TaskGraphId, TaskGraphMemorySummary, TaskGraphProcessPanel,
    TaskGraphProcessRole, TaskGraphProcessSummary, TaskNodeId, UserId, WorkerDetail, WorkerId,
    WorkerReportStatus, WorkspaceId,
};
use async_trait::async_trait;
use serde_json::json;

struct MemoryCapturingCommander {
    captured_memory_fact: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl Cognition for MemoryCapturingCommander {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        let memory_fact = input
            .context
            .facts
            .iter()
            .find(|fact| fact.source == "memory.user_preference")
            .map(|fact| fact.content.clone());
        *self
            .captured_memory_fact
            .lock()
            .expect("lock memory fact capture") = memory_fact;

        CognitionResult::Success(json!({
            "decision": {
                "kind": "RouteTask",
                "route": {
                    "target_agent_id": "worker.ops",
                    "task_summary": "verify memory",
                    "goal": "verify memory injection",
                    "constraints": []
                },
                "handoff": {
                    "why_this_agent": "ops can verify memory",
                    "expected_output": "done"
                },
                "clarification": {
                    "question": ""
                }
            },
            "confidence": 1.0,
            "rationale": {
                "primary": "memory injection test",
                "evidence": [],
                "alternatives_considered": []
            }
        }))
    }
}

struct NoopWorker;

#[async_trait]
impl Cognition for NoopWorker {
    async fn evaluate(&self, _input: CognitionInput) -> CognitionResult {
        CognitionResult::Success(json!({
            "decision": {
                "kind": "NoAction"
            },
            "role_output": {
                "summary": "memory worker done",
                "findings": ["memory path completed"],
                "recommendations": ["keep typed memory contract"],
                "evidence": ["memory.user_preference"],
                "risks": [],
                "open_questions": []
            }
        }))
    }
}

fn test_memory_record(
    id: impl Into<String>,
    scope: MemoryScope,
    kind: MemoryKind,
    key: Option<&str>,
    content: impl Into<String>,
    confidence: f32,
) -> MemoryRecord {
    MemoryRecord {
        id: MemoryId(id.into()),
        scope,
        kind,
        key: key.map(str::to_string),
        content: content.into(),
        confidence,
        tags: vec!["test".to_string()],
        source: MemorySource {
            kind: "test".to_string(),
            description: "seed memory".to_string(),
        },
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    }
}

fn temp_memory_path(name: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_PATH: AtomicU64 = AtomicU64::new(1);
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis();
    let sequence = NEXT_TEMP_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .join("cm-agent-memory-tests")
        .join(format!("{name}-{millis}-{sequence}.json"))
}

#[test]
fn in_memory_store_filters_by_scope_kind_and_limit() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("user-1".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("agent-1".to_string())),
        session_id: None,
        task_id: None,
    };

    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            id: MemoryId("mem-1".to_string()),
            scope: MemoryScope {
                agent_id: None,
                ..scope.clone()
            },
            kind: MemoryKind::UserPreference,
            key: Some("platform".to_string()),
            content: "用户常看抖音和小红书".to_string(),
            confidence: 0.9,
            tags: vec!["profile".to_string()],
            source: MemorySource {
                kind: "test".to_string(),
                description: "seed memory".to_string(),
            },
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        },
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            id: MemoryId("mem-2".to_string()),
            scope,
            kind: MemoryKind::RoutingHint,
            key: Some("route".to_string()),
            content: "直播脚本应该路由到 creative".to_string(),
            confidence: 0.8,
            tags: vec!["routing".to_string()],
            source: MemorySource {
                kind: "test".to_string(),
                description: "seed memory".to_string(),
            },
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        },
    );

    let mut query = MemoryQuery::scoped(MemoryScope {
        user_id: Some(UserId("user-1".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("agent-1".to_string())),
        session_id: None,
        task_id: None,
    });
    query.kinds = vec![MemoryKind::UserPreference];
    query.limit = 1;

    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].kind, MemoryKind::UserPreference);
    assert!(bundle.records[0].content.contains("抖音"));
}

#[test]
fn record_is_truncated_before_write() {
    let store = InMemoryMemoryStore::with_policy(MemoryCompactionPolicy {
        max_record_chars: 40,
        ..MemoryCompactionPolicy::default()
    });
    let scope = MemoryScope {
        user_id: Some(UserId("truncate-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };
    let long_content = "alpha ".repeat(40);

    let outcome = MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "truncate",
            scope,
            MemoryKind::UserPreference,
            Some("long"),
            long_content,
            1.2,
        ),
    );

    let records = store.records();
    assert_eq!(outcome.written, 1);
    assert_eq!(outcome.compaction.truncated_records, 1);
    assert_eq!(records.len(), 1);
    assert!(records[0].content.chars().count() <= 40);
    assert!(records[0].content.contains("memory compacted"));
    assert_eq!(records[0].confidence, 1.0);
}

#[test]
fn empty_record_is_not_written() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("empty-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };

    let outcome = MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "empty",
            scope,
            MemoryKind::ConversationFact,
            Some("blank"),
            "   \n\t   ",
            0.5,
        ),
    );

    assert_eq!(outcome.written, 0);
    assert_eq!(outcome.compaction.dropped_records, 1);
    assert!(store.records().is_empty());
}

#[test]
fn same_key_upsert_does_not_grow_store() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("upsert-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };

    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "first",
            scope.clone(),
            MemoryKind::UserPreference,
            Some("platform"),
            "用户偏好抖音",
            0.5,
        ),
    );
    let outcome = MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "second",
            scope,
            MemoryKind::UserPreference,
            Some("platform"),
            "用户偏好小红书",
            0.8,
        ),
    );

    let records = store.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id.0, "first");
    assert_eq!(records[0].content, "用户偏好小红书");
    assert_eq!(outcome.compaction.merged_records, 1);
}

#[test]
fn scope_kind_compaction_prunes_unkeyed_records() {
    let store = InMemoryMemoryStore::with_policy(MemoryCompactionPolicy {
        max_records_per_scope_kind: 4,
        max_unkeyed_records_per_scope_kind: 2,
        ..MemoryCompactionPolicy::default()
    });
    let scope = MemoryScope {
        user_id: Some(UserId("prune-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };

    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "keyed-a",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("a"),
            "关键事实 A",
            0.9,
        ),
    );
    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "keyed-b",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("b"),
            "关键事实 B",
            0.8,
        ),
    );
    for index in 0..10 {
        MemoryStore::upsert_record(
            &store,
            test_memory_record(
                format!("unkeyed-{index}"),
                scope.clone(),
                MemoryKind::ConversationFact,
                None,
                format!("临时事实 {index}"),
                0.3 + (index as f32 * 0.01),
            ),
        );
    }

    let records = store.records();
    let scoped_facts = records
        .iter()
        .filter(|record| record.kind == MemoryKind::ConversationFact)
        .collect::<Vec<_>>();
    assert_eq!(scoped_facts.len(), 4);
    assert_eq!(
        scoped_facts
            .iter()
            .filter(|record| record.key.is_none())
            .count(),
        2
    );
    assert!(
        scoped_facts
            .iter()
            .any(|record| record.key.as_deref() == Some("a"))
    );
    assert!(
        scoped_facts
            .iter()
            .any(|record| record.key.as_deref() == Some("b"))
    );
}

#[test]
fn load_bundle_respects_char_budget_even_after_store_compaction() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("budget-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };
    for index in 0..4 {
        MemoryStore::upsert_record(
            &store,
            test_memory_record(
                format!("budget-{index}"),
                scope.clone(),
                MemoryKind::UserPreference,
                Some(&format!("pref-{index}")),
                format!("偏好 {index} {}", "内容".repeat(20)),
                0.7,
            ),
        );
    }

    let mut query = MemoryQuery::scoped(scope);
    query.budget = MemoryBudget {
        max_records: 4,
        max_chars: 30,
    };

    let bundle = MemoryStore::load_bundle(&store, query);
    let total_chars = bundle
        .records
        .iter()
        .map(|record| record.content.chars().count())
        .sum::<usize>();

    assert!(total_chars <= 30);
    assert!(!bundle.records.is_empty());
}

#[test]
fn file_memory_store_writes_and_reloads_records() {
    let path = temp_memory_path("reload");
    let scope = MemoryScope {
        user_id: Some(UserId("file-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };
    {
        let store = FileMemoryStore::open(&path).expect("open file memory store");
        MemoryStore::upsert_record(
            &store,
            test_memory_record(
                "file-record",
                scope.clone(),
                MemoryKind::UserPreference,
                Some("platform"),
                "用户偏好文件持久化 memory",
                0.9,
            ),
        );
    }

    let store = FileMemoryStore::open(&path).expect("reload file memory store");
    let mut query = MemoryQuery::scoped(scope);
    query.kinds = vec![MemoryKind::UserPreference];
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].content, "用户偏好文件持久化 memory");
    assert!(
        fs::read_to_string(&path)
            .expect("read memory file")
            .contains("\"version\"")
    );
}

#[test]
fn file_memory_store_compacts_before_flushing() {
    let path = temp_memory_path("compact");
    let store = FileMemoryStore::open_with_policy(
        &path,
        MemoryCompactionPolicy {
            max_record_chars: 48,
            ..MemoryCompactionPolicy::default()
        },
    )
    .expect("open file memory store");
    let scope = MemoryScope {
        user_id: Some(UserId("file-compact-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: None,
        task_id: None,
    };

    let outcome = MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "file-long-record",
            scope,
            MemoryKind::ConversationFact,
            Some("long"),
            "durable memory ".repeat(20),
            0.9,
        ),
    );

    let stored = store.records();
    let persisted = fs::read_to_string(&path).expect("read memory file");
    assert_eq!(outcome.written, 1);
    assert_eq!(outcome.compaction.truncated_records, 1);
    assert_eq!(stored.len(), 1);
    assert!(stored[0].content.chars().count() <= 48);
    assert!(persisted.contains("memory compacted"));
}

#[test]
fn file_memory_store_persists_session_snapshot() {
    let path = temp_memory_path("session");
    let scope = MemoryScope {
        user_id: Some(UserId("file-session-user".to_string())),
        workspace_id: None,
        agent_id: None,
        session_id: Some(SessionId("file-session".to_string())),
        task_id: None,
    };
    {
        let store = FileMemoryStore::open(&path).expect("open file memory store");
        let outcome = MemoryStore::persist_session(
            &store,
            &scope,
            SessionSnapshot {
                summary: "本次 session 形成了文件持久化 memory".to_string(),
                final_output: "final output".to_string(),
                blackboard: [("decision".to_string(), "使用文件 store".to_string())].into(),
                ..SessionSnapshot::default()
            },
        );
        assert!(outcome.written >= 2);
    }

    let store = FileMemoryStore::open(&path).expect("reload file memory store");
    let bundle = MemoryStore::load_bundle(&store, MemoryQuery::scoped(scope));

    assert!(
        bundle
            .records
            .iter()
            .any(|record| record.kind == MemoryKind::SessionSummary)
    );
    assert!(
        bundle
            .records
            .iter()
            .any(|record| record.kind == MemoryKind::ConversationFact
                && record.key.as_deref() == Some("decision"))
    );
    assert!(
        bundle
            .records
            .iter()
            .any(|record| record.key.as_deref() == Some("memory_card:facts")
                && record.content.contains("文件持久化 memory"))
    );
}

#[test]
fn file_memory_store_reports_corrupt_file_on_open() {
    let path = temp_memory_path("corrupt");
    fs::create_dir_all(path.parent().expect("temp path has parent")).expect("create temp dir");
    fs::write(&path, "{not valid json").expect("write corrupt memory file");

    let error = FileMemoryStore::open(&path).expect_err("corrupt file should fail");

    assert!(matches!(error, FileMemoryStoreError::Json { .. }));
}

#[test]
fn session_snapshot_persists_role_risks_questions_and_evaluation() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("snapshot-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("snapshot-agent".to_string())),
        session_id: Some(SessionId("snapshot-session".to_string())),
        task_id: None,
    };
    let role_summary = RoleMemorySummary {
        graph_id: TaskGraphId("graph-snapshot".to_string()),
        node_id: TaskNodeId("node-data".to_string()),
        worker_id: WorkerId("worker.data".to_string()),
        role: "data".to_string(),
        summary: "数据角色完成归因口径诊断".to_string(),
        findings: vec!["漏斗指标需要拆解".to_string()],
        recommendations: vec!["统一渠道口径".to_string()],
        evidence: vec!["原始任务".to_string()],
        risks: vec!["口径不一致会导致误判".to_string()],
        open_questions: vec!["需要确认 GMV 统计口径".to_string()],
        status: WorkerReportStatus::Completed,
    };
    let worker_detail = WorkerDetail {
        graph_id: TaskGraphId("graph-snapshot".to_string()),
        node_id: TaskNodeId("node-data".to_string()),
        task_id: agent::api::TaskId("graph-snapshot:node-data".to_string()),
        worker_id: WorkerId("worker.data".to_string()),
        role: "data".to_string(),
        title: "数据诊断".to_string(),
        objective: "分析数据指标归因".to_string(),
        attempt: 1,
        status: WorkerReportStatus::Completed,
        content: "数据角色完成归因口径诊断".to_string(),
        role_output: None,
        evidence: vec!["原始任务".to_string()],
        risks: vec!["口径不一致会导致误判".to_string()],
        open_questions: vec!["需要确认 GMV 统计口径".to_string()],
        diagnostics: Vec::new(),
        evaluation: None,
    };
    let process_summary = TaskGraphProcessSummary {
        graph_id: TaskGraphId("graph-snapshot".to_string()),
        root_task: "分析数据指标归因".to_string(),
        primary: Some(TaskGraphProcessRole {
            node_id: TaskNodeId("node-data".to_string()),
            task_id: Some(agent::api::TaskId("graph-snapshot:node-data".to_string())),
            worker_id: WorkerId("worker.data".to_string()),
            role: "data".to_string(),
            role_label: "数据分析师".to_string(),
            title: "数据诊断".to_string(),
            objective: "分析数据指标归因".to_string(),
            status: "passed".to_string(),
            attempt: 1,
            is_primary: true,
            summary: Some("数据角色完成归因口径诊断".to_string()),
            finding: Some("漏斗指标需要拆解".to_string()),
            recommendation: Some("统一渠道口径".to_string()),
            evidence: vec!["原始任务".to_string()],
            risks: vec!["口径不一致会导致误判".to_string()],
            open_questions: vec!["需要确认 GMV 统计口径".to_string()],
        }),
        roles: vec![TaskGraphProcessRole {
            node_id: TaskNodeId("node-data".to_string()),
            task_id: Some(agent::api::TaskId("graph-snapshot:node-data".to_string())),
            worker_id: WorkerId("worker.data".to_string()),
            role: "data".to_string(),
            role_label: "数据分析师".to_string(),
            title: "数据诊断".to_string(),
            objective: "分析数据指标归因".to_string(),
            status: "passed".to_string(),
            attempt: 1,
            is_primary: true,
            summary: Some("数据角色完成归因口径诊断".to_string()),
            finding: Some("漏斗指标需要拆解".to_string()),
            recommendation: Some("统一渠道口径".to_string()),
            evidence: vec!["原始任务".to_string()],
            risks: vec!["口径不一致会导致误判".to_string()],
            open_questions: vec!["需要确认 GMV 统计口径".to_string()],
        }],
        handoffs: vec![TaskGraphHandoff {
            from_node_id: TaskNodeId("node-data".to_string()),
            to_node_id: TaskNodeId("node-ops".to_string()),
            from_role: "data".to_string(),
            to_role: "ops".to_string(),
            from_role_label: "数据分析师".to_string(),
            to_role_label: "运营负责人".to_string(),
            summary: "数据口径交给运营拆执行节奏".to_string(),
        }],
        next_actions: vec!["统一渠道口径".to_string()],
        check_items: vec!["确认 GMV 统计口径".to_string()],
        deliverables: vec!["final_report：归因诊断报告".to_string()],
        risks: vec!["口径不一致会导致误判".to_string()],
        open_questions: vec!["需要确认 GMV 统计口径".to_string()],
        incomplete: Vec::new(),
        panel: TaskGraphProcessPanel {
            default_collapsed: true,
            anchor: "answer_scoped_process".to_string(),
            label: "本次回答过程".to_string(),
            reason: "过程面板绑定本次回答，历史回放时默认折叠。".to_string(),
        },
    };
    let outcome = MemoryStore::persist_session(
        &store,
        &scope,
        SessionSnapshot {
            summary: "结构化 snapshot 已完成".to_string(),
            final_output: "final".to_string(),
            task_graphs: vec![TaskGraphMemorySummary {
                graph_id: TaskGraphId("graph-snapshot".to_string()),
                root_task: "分析数据指标归因".to_string(),
                roles: vec!["data".to_string()],
                process: Some(process_summary),
                role_summaries: vec![role_summary.clone()],
                worker_details: vec![worker_detail.clone()],
                risks: role_summary.risks.clone(),
                open_questions: role_summary.open_questions.clone(),
                evaluation_summary: Some("node-data passed=true score=1.00".to_string()),
            }],
            role_summaries: vec![role_summary],
            worker_details: vec![worker_detail],
            risks: vec!["口径不一致会导致误判".to_string()],
            open_questions: vec!["需要确认 GMV 统计口径".to_string()],
            evaluation_summary: Some("node-data passed=true score=1.00".to_string()),
            ..SessionSnapshot::default()
        },
    );

    let records = store.records();
    assert!(outcome.written >= 5);
    assert!(
        records
            .iter()
            .any(|record| record.kind == MemoryKind::CrossRoleContext
                && record.content.contains("数据角色完成归因口径诊断"))
    );
    assert!(
        records
            .iter()
            .any(|record| record.tags.contains(&"risk".to_string())
                && record.content.contains("口径不一致"))
    );
    assert!(
        records
            .iter()
            .any(|record| record.tags.contains(&"open_question".to_string())
                && record.content.contains("GMV"))
    );
    assert!(
        records
            .iter()
            .any(|record| record.tags.contains(&"evaluation".to_string())
                && record.content.contains("passed=true"))
    );
    assert!(records.iter().any(|record| {
        record.key.as_deref() == Some("memory_card:facts")
            && record.tags.contains(&"memory_card".to_string())
            && record.tags.iter().any(|tag| tag.starts_with("entity:"))
            && record.content.contains("漏斗指标需要拆解")
    }));
    assert!(records.iter().any(|record| {
        record.key.as_deref() == Some("memory_card:decisions")
            && record.content.contains("统一渠道口径")
    }));
    assert!(records.iter().any(|record| {
        record.key.as_deref() == Some("memory_card:constraints")
            && record.content.contains("口径不一致")
    }));
    assert!(records.iter().any(|record| {
        record.key.as_deref() == Some("memory_card:open_questions")
            && record.content.contains("GMV")
    }));
    let task_graph_memory = records
        .iter()
        .find(|record| {
            record.kind == MemoryKind::CrossRoleContext
                && record.key.as_deref() == Some("task_graph:graph-snapshot")
        })
        .expect("task graph memory should be persisted");
    assert!(task_graph_memory.content.contains("Process panel"));
    assert!(task_graph_memory.content.contains("default_collapsed=true"));
    assert!(task_graph_memory.content.contains("answer_scoped_process"));
    assert!(
        task_graph_memory
            .content
            .contains("Primary role: 数据分析师")
    );
    assert!(
        task_graph_memory
            .content
            .contains("数据口径交给运营拆执行节奏")
    );
    assert!(
        task_graph_memory
            .content
            .contains("final_report：归因诊断报告")
    );
}

#[test]
fn load_bundle_prioritizes_records_related_to_current_question() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("relevance-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("relevance-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "mem-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 校园冷萃咖啡的试卖结论是先测价格带。",
            0.95,
        ),
    );
    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "mem-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:constraints"),
            "Constraints:\n- 直播复盘里退款率偏高，今天先处理成交承接和客服解释。",
            0.7,
        ),
    );

    let mut query =
        MemoryQuery::scoped(scope).for_text("继续 直播 复盘，今天先处理 退款率 和 成交承接");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:constraints")
    );
    assert!(bundle.records[0].content.contains("退款率偏高"));
}

#[test]
fn load_bundle_prioritizes_matching_entity_when_query_has_no_spaces() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("entity-relevance-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("entity-relevance-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "entity-high-confidence-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 校园冷萃咖啡转化稳定，下一轮先测试优惠券。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:便携保温杯".to_string(),
            ],
            ..test_memory_record(
                "entity-lower-confidence-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- 便携保温杯复盘先看短视频种草与直播间承接。",
                0.62,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text("继续便携保温杯复盘，重点看承接");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("便携保温杯"));
}

#[test]
fn load_bundle_uses_explicit_context_terms_for_entity_relevance() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("explicit-context-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("explicit-context-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "explicit-context-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 校园冷萃咖啡上一轮已经确认价格带。",
            0.98,
        ),
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-001".to_string(),
                "entity:便携保温杯".to_string(),
            ],
            ..test_memory_record(
                "explicit-context-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-001 便携保温杯本轮先优化直播间成交承接。",
                0.61,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope)
        .for_text("继续刚才\nproduct:thermos-001\n便携保温杯正在做直播复盘");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-001"));
}

#[test]
fn load_bundle_prefers_recent_memory_when_query_has_no_specific_match() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("recent-context-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("recent-context-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    let now = SystemTime::now();
    let mut older_high_confidence = test_memory_record(
        "recent-old-high-confidence",
        scope.clone(),
        MemoryKind::ConversationFact,
        Some("memory_card:facts"),
        "Facts:\n- 校园冷萃咖啡上一轮只确认了试饮反馈。",
        0.99,
    );
    older_high_confidence.updated_at = now - Duration::from_secs(60);
    let mut newer_low_confidence = test_memory_record(
        "recent-new-low-confidence",
        scope.clone(),
        MemoryKind::ConversationFact,
        Some("memory_card:decisions"),
        "Decisions:\n- 便携保温杯上一轮决定先改直播间成交承接。",
        0.3,
    );
    newer_low_confidence.updated_at = now;

    MemoryStore::upsert_record(&store, older_high_confidence);
    MemoryStore::upsert_record(&store, newer_low_confidence);

    let mut query = MemoryQuery::scoped(scope).for_text("继续刚才");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("便携保温杯"));
}

#[test]
fn load_bundle_ignores_continuation_only_terms_for_recent_memory_fallback() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("continuation-context-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("continuation-context-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    let now = SystemTime::now();
    let mut older_generic_match = test_memory_record(
        "continuation-old-generic-match",
        scope.clone(),
        MemoryKind::ConversationFact,
        Some("memory_card:facts"),
        "Facts:\n- 校园冷萃咖啡上一轮写过继续跟进和上面修改事项。",
        0.99,
    );
    older_generic_match.updated_at = now - Duration::from_secs(60);
    let mut newer_recent_context = test_memory_record(
        "continuation-new-recent-context",
        scope.clone(),
        MemoryKind::ConversationFact,
        Some("memory_card:decisions"),
        "Decisions:\n- 便携保温杯最近一轮决定先检查直播间成交承接。",
        0.3,
    );
    newer_recent_context.updated_at = now;

    MemoryStore::upsert_record(&store, older_generic_match);
    MemoryStore::upsert_record(&store, newer_recent_context);

    let mut query = MemoryQuery::scoped(scope).for_text("继续刚才上面修改");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("便携保温杯"));
}

#[test]
fn load_bundle_strips_continuation_prefix_before_cjk_ngram_matching() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("continuation-prefix-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("continuation-prefix-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        test_memory_record(
            "continuation-prefix-generic",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 校园冷萃咖啡记录包含继续修改上面这些泛化动作。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:便携保温杯".to_string(),
            ],
            ..test_memory_record(
                "continuation-prefix-entity",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- 便携保温杯本轮先修正直播间成交承接。",
                0.45,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text("继续刚才上面修改便携保温杯");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("便携保温杯"));
}

#[test]
fn load_bundle_prefers_exact_entity_over_partial_ngram_overlap() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("exact-entity-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("exact-entity-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:便携水杯".to_string(),
            ],
            ..test_memory_record(
                "partial-ngram-overlap",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- 便携水杯上一轮主要看外观卖点和价格带。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:便携保温杯".to_string(),
            ],
            ..test_memory_record(
                "exact-entity-match",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- 便携保温杯本轮先优化直播间成交承接。",
                0.35,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text("继续便携保温杯");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("便携保温杯"));
}

#[test]
fn load_bundle_prefers_exact_context_ref_over_shared_ref_fragments() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("exact-context-ref-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("exact-context-ref-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-archive".to_string(),
            ],
            ..test_memory_record(
                "shared-ref-fragment-old",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-archive 旧款保温杯上一轮只看价格带。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-001".to_string(),
            ],
            ..test_memory_record(
                "exact-context-ref-new",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-001 本轮先优化直播间成交承接。",
                0.35,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text("继续刚才\nproduct:thermos-001");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 240,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-001"));
}

#[test]
fn load_bundle_uses_request_prefixed_context_ref_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("request-prefixed-ref-user".to_string())),
        workspace_id: Some(WorkspaceId("request-prefixed-ref-workspace".to_string())),
        agent_id: Some(AgentId("request-prefixed-ref-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-archive".to_string(),
            ],
            ..test_memory_record(
                "request-prefix-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-archive 旧款保温杯上一轮只看价格带。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-002".to_string(),
            ],
            ..test_memory_record(
                "request-prefix-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-002 本轮先核对资料包里的成交承接反馈。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.refs".to_string(),
        "product:thermos-002".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("request-prefixed-ref-user".to_string())),
        workspace_id: Some(WorkspaceId("request-prefixed-ref-workspace".to_string())),
        agent_id: AgentId("request-prefixed-ref-agent".to_string()),
        session_id: SessionId("request-prefixed-ref-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-002"));
}

#[test]
fn load_bundle_normalizes_context_ref_case_from_metadata() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("metadata-ref-case-user".to_string())),
        workspace_id: Some(WorkspaceId("metadata-ref-case-workspace".to_string())),
        agent_id: Some(AgentId("metadata-ref-case-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-archive".to_string(),
            ],
            ..test_memory_record(
                "metadata-ref-case-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-archive 旧款保温杯上一轮只看价格带。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-003".to_string(),
            ],
            ..test_memory_record(
                "metadata-ref-case-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-003 本轮先核对资料包里的成交承接反馈。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.refs".to_string(),
        "Product:Thermos-003".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("metadata-ref-case-user".to_string())),
        workspace_id: Some(WorkspaceId("metadata-ref-case-workspace".to_string())),
        agent_id: AgentId("metadata-ref-case-agent".to_string()),
        session_id: SessionId("metadata-ref-case-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-003"));
}

#[test]
fn load_bundle_uses_bare_product_id_metadata_as_context_ref() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("bare-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("bare-product-id-workspace".to_string())),
        agent_id: Some(AgentId("bare-product-id-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-archive".to_string(),
            ],
            ..test_memory_record(
                "bare-product-id-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-archive 旧款保温杯上一轮只看价格带。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-004".to_string(),
            ],
            ..test_memory_record(
                "bare-product-id-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-004 本轮先核对直播成交承接反馈。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.product_ids".to_string(),
        "Thermos-004".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("bare-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("bare-product-id-workspace".to_string())),
        agent_id: AgentId("bare-product-id-agent".to_string()),
        session_id: SessionId("bare-product-id-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-004"));
}

#[test]
fn load_bundle_uses_camel_case_product_id_metadata_as_context_ref() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("camel-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("camel-product-id-workspace".to_string())),
        agent_id: Some(AgentId("camel-product-id-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-archive".to_string(),
            ],
            ..test_memory_record(
                "camel-product-id-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-archive 旧商品上一轮只看标题。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-010".to_string(),
            ],
            ..test_memory_record(
                "camel-product-id-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-010 本轮先复盘售后退款率。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("context.productIds".to_string(), "Thermos-010".to_string());
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("camel-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("camel-product-id-workspace".to_string())),
        agent_id: AgentId("camel-product-id-agent".to_string()),
        session_id: SessionId("camel-product-id-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-010"));
}

#[test]
fn load_bundle_uses_singular_product_id_metadata_as_context_ref() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("singular-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("singular-product-id-workspace".to_string())),
        agent_id: Some(AgentId("singular-product-id-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-old".to_string(),
            ],
            ..test_memory_record(
                "singular-product-id-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-old 旧商品上一轮只看包装。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-021".to_string(),
            ],
            ..test_memory_record(
                "singular-product-id-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-021 本轮先复盘客服解释。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.productId".to_string(),
        "Thermos-021".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("singular-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("singular-product-id-workspace".to_string())),
        agent_id: AgentId("singular-product-id-agent".to_string()),
        session_id: SessionId("singular-product-id-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-021"));
}

#[test]
fn load_bundle_uses_bare_campaign_id_metadata_as_context_ref() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("bare-campaign-id-user".to_string())),
        workspace_id: Some(WorkspaceId("bare-campaign-id-workspace".to_string())),
        agent_id: Some(AgentId("bare-campaign-id-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:campaign:launch-archive".to_string(),
            ],
            ..test_memory_record(
                "bare-campaign-id-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- campaign:launch-archive 旧活动上一轮只看预算消耗。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:campaign:launch-006".to_string(),
            ],
            ..test_memory_record(
                "bare-campaign-id-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- campaign:launch-006 本轮先核对活动承接和素材反馈。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.campaign_ids".to_string(),
        "Launch-006".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("bare-campaign-id-user".to_string())),
        workspace_id: Some(WorkspaceId("bare-campaign-id-workspace".to_string())),
        agent_id: AgentId("bare-campaign-id-agent".to_string()),
        session_id: SessionId("bare-campaign-id-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("campaign:launch-006"));
}

#[test]
fn load_bundle_uses_rag_product_id_metadata_as_context_ref() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("rag-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("rag-product-id-workspace".to_string())),
        agent_id: Some(AgentId("rag-product-id-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:product:thermos-archive".to_string(),
            ],
            ..test_memory_record(
                "rag-product-id-shared-fragment",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- product:thermos-archive 旧商品上一轮只看价格带。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-008".to_string(),
            ],
            ..test_memory_record(
                "rag-product-id-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-008 本轮先核对 RAG 命中的售后承接资料。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("rag.product_ids".to_string(), "Thermos-008".to_string());
    metadata.insert("rag.query_terms".to_string(), "售后承接".to_string());
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("rag-product-id-user".to_string())),
        workspace_id: Some(WorkspaceId("rag-product-id-workspace".to_string())),
        agent_id: AgentId("rag-product-id-agent".to_string()),
        session_id: SessionId("rag-product-id-session".to_string()),
        input: "继续看检索资料".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("product:thermos-008"));
}

#[test]
fn load_bundle_uses_conversation_history_summary_for_low_signal_followup() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("conversation-history-user".to_string())),
        workspace_id: Some(WorkspaceId("conversation-history-workspace".to_string())),
        agent_id: Some(AgentId("conversation-history-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:校园冷萃咖啡".to_string(),
            ],
            ..test_memory_record(
                "conversation-history-unrelated",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- 校园冷萃咖啡旧复盘先看试饮转化。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:QA矩阵智能保温杯".to_string(),
            ],
            ..test_memory_record(
                "conversation-history-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- QA矩阵智能保温杯 本轮先处理退款率偏高和直播间成交承接。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "conversation.history_summary".to_string(),
        "上一轮正在复盘 QA矩阵智能保温杯，问题集中在退款率偏高和成交承接。".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("conversation-history-user".to_string())),
        workspace_id: Some(WorkspaceId("conversation-history-workspace".to_string())),
        agent_id: AgentId("conversation-history-agent".to_string()),
        session_id: SessionId("conversation-history-session".to_string()),
        input: "继续刚才".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("QA矩阵智能保温杯"));
}

#[test]
fn load_bundle_uses_conversation_memory_summary_json_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("conversation-memory-json-user".to_string())),
        workspace_id: Some(WorkspaceId(
            "conversation-memory-json-workspace".to_string(),
        )),
        agent_id: Some(AgentId("conversation-memory-json-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "conversation-memory-json-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 校园冷萃咖啡上一轮只讨论海报色彩。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "conversation-memory-json-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:decisions"),
            "Decisions:\n- 夏季防晒衣需要先按直播间成交承接重写脚本，预算不能超过 3000 元。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.conversation.memory_summary_json".to_string(),
        json!({
            "kind": "conversation_memory_summary",
            "rollingSummary": "上一轮确认夏季防晒衣需要提升点击转化。",
            "facts": [
                {"source": "conversation_message:old", "text": "用户主推轻薄防晒衣，退款率偏高。"}
            ],
            "decisions": [
                {"text": "先按直播间成交承接重写脚本。"}
            ],
            "constraints": [
                {"text": "预算不能超过 3000 元。"}
            ],
            "openQuestions": [
                {"text": "是否需要按渠道拆分预算？"}
            ],
            "lastUserInput": "不应进入召回的最后输入",
            "lastAssistantOutput": "不应进入召回的最后输出",
            "sourceMap": {
                "lastSessionId": "secret-session",
                "memoryBudget": {"memoryTextMaxChars": 2400}
            }
        })
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("conversation-memory-json-user".to_string())),
        workspace_id: Some(WorkspaceId(
            "conversation-memory-json-workspace".to_string(),
        )),
        agent_id: AgentId("conversation-memory-json-agent".to_string()),
        session_id: SessionId("conversation-memory-json-session".to_string()),
        input: "继续完善方案".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("预算不能超过 3000"));
}

#[test]
fn load_bundle_uses_conversation_memory_summary_json_camel_alias_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("conversation-memory-camel-user".to_string())),
        workspace_id: Some(WorkspaceId(
            "conversation-memory-camel-workspace".to_string(),
        )),
        agent_id: Some(AgentId("conversation-memory-camel-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "conversation-memory-camel-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 旧活动只讨论赠品库存。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "conversation-memory-camel-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:decisions"),
            "Decisions:\n- 校园保温杯活动需要先修正客服解释口径和直播脚本。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "conversation.memorySummaryJson".to_string(),
        json!({
            "rollingSummary": "上一轮确认校园保温杯活动要先处理退款率。",
            "facts": [
                {"text": "客服解释口径需要和直播脚本一起调整。"}
            ],
            "lastUserInput": "不应进入召回的最后输入",
            "sourceMap": {"lastSessionId": "secret-session"}
        })
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("conversation-memory-camel-user".to_string())),
        workspace_id: Some(WorkspaceId(
            "conversation-memory-camel-workspace".to_string(),
        )),
        agent_id: AgentId("conversation-memory-camel-agent".to_string()),
        session_id: SessionId("conversation-memory-camel-session".to_string()),
        input: "继续".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("客服解释口径"));
}

#[test]
fn load_bundle_uses_workspace_source_titles_for_material_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("source-title-user".to_string())),
        workspace_id: Some(WorkspaceId("source-title-workspace".to_string())),
        agent_id: Some(AgentId("source-title-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-title-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 竞品价格表上一轮只核对券后价。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-title-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:decisions"),
            "Decisions:\n- 直播成交承接表 显示客服解释和退款率需要一起复盘。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "workspaceContext.source_titles".to_string(),
        "conversation_file:file-7 title=直播成交承接表".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("source-title-user".to_string())),
        workspace_id: Some(WorkspaceId("source-title-workspace".to_string())),
        agent_id: AgentId("source-title-agent".to_string()),
        session_id: SessionId("source-title-session".to_string()),
        input: "继续看资料".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("直播成交承接表"));
}

#[test]
fn load_bundle_uses_quality_learning_summary_for_revision_followup() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("quality-learning-user".to_string())),
        workspace_id: Some(WorkspaceId("quality-learning-workspace".to_string())),
        agent_id: Some(AgentId("quality-learning-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "quality-learning-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 竞品价格表上一轮只核对券后价。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "quality-learning-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:constraints"),
            "Constraints:\n- 复盘回答需要优先说明退款率异常和客服解释口径。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "qualityLearning.learningSummary".to_string(),
        "[quality_issue 0.82] 复盘回答需要优先说明退款率异常和客服解释口径。".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("quality-learning-user".to_string())),
        workspace_id: Some(WorkspaceId("quality-learning-workspace".to_string())),
        agent_id: AgentId("quality-learning-agent".to_string()),
        session_id: SessionId("quality-learning-session".to_string()),
        input: "继续改回答".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:constraints")
    );
    assert!(bundle.records[0].content.contains("退款率异常"));
}

#[test]
fn load_bundle_uses_context_summary_alias_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("context-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("context-summary-workspace".to_string())),
        agent_id: Some(AgentId("context-summary-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "context-summary-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 竞品价格表上一轮只核对券后价。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "context-summary-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:constraints"),
            "Constraints:\n- 上下文资料提到直播退款率和客服解释口径需要一起核对。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.summary".to_string(),
        "Selected context notes mention 直播退款率 and 客服解释口径.".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("context-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("context-summary-workspace".to_string())),
        agent_id: AgentId("context-summary-agent".to_string()),
        session_id: SessionId("context-summary-session".to_string()),
        input: "继续看上下文".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:constraints")
    );
    assert!(bundle.records[0].content.contains("直播退款率"));
}

#[test]
fn load_bundle_uses_image_summary_metadata_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("image-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("image-summary-workspace".to_string())),
        agent_id: Some(AgentId("image-summary-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "image-summary-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 竞品价格表上一轮只核对券后价。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "image-summary-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:decisions"),
            "Decisions:\n- 低糖冷萃 海报主标题和买二送一促销点需要一起复盘。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.attachments.image_summaries_json".to_string(),
        json!([{
            "sourceRef": "conversation_file:image-9",
            "objectKey": "private/images/image-9.png",
            "summary": "图片显示冷萃咖啡促销海报。",
            "agentContext": "海报主标题是低糖冷萃，促销点是买二送一。",
            "analysisId": "analysis-secret",
            "llmName": "vision-model"
        }])
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("image-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("image-summary-workspace".to_string())),
        agent_id: AgentId("image-summary-agent".to_string()),
        session_id: SessionId("image-summary-session".to_string()),
        input: "继续看图片".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("低糖冷萃"));
}

#[test]
fn load_bundle_uses_image_text_metadata_aliases_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("image-text-user".to_string())),
        workspace_id: Some(WorkspaceId("image-text-workspace".to_string())),
        agent_id: Some(AgentId("image-text-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "image-text-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 竞品价格表上一轮只核对券后价。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "image-text-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:decisions"),
            "Decisions:\n- 冷萃咖啡海报的低糖0脂和19.9元价格需要补直播承接话术。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.attachments.image_titles".to_string(),
        "冷萃咖啡海报".to_string(),
    );
    metadata.insert(
        "request.context.vision_summary".to_string(),
        "画面中心是低糖冷萃咖啡瓶，底部突出19.9元价格。".to_string(),
    );
    metadata.insert(
        "attachments.ocr_text".to_string(),
        "低糖0脂 冷萃咖啡 19.9元".to_string(),
    );
    metadata.insert(
        "request.context.image_notes".to_string(),
        "海报卖点需要补充直播承接话术。".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("image-text-user".to_string())),
        workspace_id: Some(WorkspaceId("image-text-workspace".to_string())),
        agent_id: AgentId("image-text-agent".to_string()),
        session_id: SessionId("image-text-session".to_string()),
        input: "继续看这张图".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("低糖0脂"));
}

#[test]
fn load_bundle_uses_source_summaries_metadata_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("source-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("source-summary-workspace".to_string())),
        agent_id: Some(AgentId("source-summary-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-summary-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 竞品价格表上一轮只核对券后价。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-summary-related",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:decisions"),
            "Decisions:\n- profit.xlsx 里 GMV 12000 元、广告费 1200 元，reviews.docx 里客服响应偏长。",
            0.35,
        ),
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.source_summaries".to_string(),
        json!([
            {
                "ref": "conversation_file:profit-1",
                "title": "profit.xlsx",
                "summary": "GMV 12000 元，成本 7000 元，广告费 1200 元，订单 180。"
            },
            {
                "sourceRef": "conversation_file:reviews-1",
                "fileName": "reviews.docx",
                "ocr_text": "用户反馈发货慢，客服响应时间偏长。"
            }
        ])
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("source-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("source-summary-workspace".to_string())),
        agent_id: AgentId("source-summary-agent".to_string()),
        session_id: SessionId("source-summary-session".to_string()),
        input: "继续提取证据".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("GMV 12000"));
}

#[test]
fn load_bundle_uses_workspace_pack_source_summaries_metadata_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("pack-source-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("pack-source-summary-workspace".to_string())),
        agent_id: Some(AgentId("pack-source-summary-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "pack-source-summary-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 旧资料只讨论券后价和包装。 ",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:refund-88".to_string(),
            ],
            ..test_memory_record(
                "pack-source-summary-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:refund-88 refund.csv 需要先核对尺码偏小和发货慢。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.workspaceContextPack.source_summaries".to_string(),
        json!([{
            "ref": "conversation_file:refund-88",
            "title": "refund.csv",
            "summary": "退款原因集中在尺码偏小和发货慢。",
            "objectKey": "private/source/raw.json"
        }])
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("pack-source-summary-user".to_string())),
        workspace_id: Some(WorkspaceId("pack-source-summary-workspace".to_string())),
        agent_id: AgentId("pack-source-summary-agent".to_string()),
        session_id: SessionId("pack-source-summary-session".to_string()),
        input: "继续看资料摘要".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("refund.csv"));
}

#[test]
fn load_bundle_uses_source_summary_ocr_text_alias_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("source-ocr-alias-user".to_string())),
        workspace_id: Some(WorkspaceId("source-ocr-alias-workspace".to_string())),
        agent_id: Some(AgentId("source-ocr-alias-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-ocr-alias-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 旧图片只讨论主图背景颜色。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:ocr-88".to_string(),
            ],
            ..test_memory_record(
                "source-ocr-alias-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:ocr-88 comments.png 的评论截图先处理尺码偏小和发货慢。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "workspaceContext.sourceSummaries".to_string(),
        json!([{
            "sourceRef": "conversation_file:ocr-88",
            "fileName": "comments.png",
            "ocrText": "评论截图显示尺码偏小和发货慢。",
            "objectKey": "private/ocr/source.png"
        }])
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("source-ocr-alias-user".to_string())),
        workspace_id: Some(WorkspaceId("source-ocr-alias-workspace".to_string())),
        agent_id: AgentId("source-ocr-alias-agent".to_string()),
        session_id: SessionId("source-ocr-alias-session".to_string()),
        input: "继续看截图文字".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("comments.png"));
}

#[test]
fn load_bundle_uses_source_summary_index_aliases_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("source-index-alias-user".to_string())),
        workspace_id: Some(WorkspaceId("source-index-alias-workspace".to_string())),
        agent_id: Some(AgentId("source-index-alias-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-index-alias-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 旧知识只讨论杯身颜色。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:product:thermos-099".to_string(),
            ],
            ..test_memory_record(
                "source-index-alias-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- product:thermos-099 knowledge-99 售后知识摘要要先核对杯盖漏水。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.rag.source_summaries".to_string(),
        json!([{
            "sourceId": "knowledge-99",
            "productId": "Thermos-099",
            "name": "售后知识摘要",
            "summary": "退款评论集中在杯盖漏水。",
            "metadata": {"raw": "hidden"}
        }])
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("source-index-alias-user".to_string())),
        workspace_id: Some(WorkspaceId("source-index-alias-workspace".to_string())),
        agent_id: AgentId("source-index-alias-agent".to_string()),
        session_id: SessionId("source-index-alias-session".to_string()),
        input: "继续看知识摘要".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("杯盖漏水"));
}

#[test]
fn load_bundle_uses_source_index_metadata_for_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("source-index-user".to_string())),
        workspace_id: Some(WorkspaceId("source-index-workspace".to_string())),
        agent_id: Some(AgentId("source-index-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "source-index-unrelated",
            scope.clone(),
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "Facts:\n- 旧资料只讨论价格表。",
            0.99,
        ),
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:file-77".to_string(),
            ],
            ..test_memory_record(
                "source-index-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:file-77 refunds.xlsx 先核对退款率和客服解释。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.sources_json".to_string(),
        json!([
            {
                "sourceType": "conversation_file",
                "id": "file-77",
                "refKey": "conversation_file:file-77",
                "fileName": "refunds.xlsx",
                "objectKey": "private/source/body.json"
            }
        ])
        .to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("source-index-user".to_string())),
        workspace_id: Some(WorkspaceId("source-index-workspace".to_string())),
        agent_id: AgentId("source-index-agent".to_string()),
        session_id: SessionId("source-index-session".to_string()),
        input: "继续看来源".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(bundle.records[0].content.contains("refunds.xlsx"));
}

#[test]
fn load_bundle_uses_workspace_pack_content_refs_for_material_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("pack-content-ref-user".to_string())),
        workspace_id: Some(WorkspaceId("pack-content-ref-workspace".to_string())),
        agent_id: Some(AgentId("pack-content-ref-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:conversation_file:old-content".to_string(),
            ],
            ..test_memory_record(
                "pack-content-ref-unrelated",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- conversation_file:old-content 只记录了旧资料价格。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:content-77".to_string(),
            ],
            ..test_memory_record(
                "pack-content-ref-related",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:content-77 资料包里先核对售后证据。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.workspaceContextPack.contentRefs".to_string(),
        "conversation_file:content-77".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("pack-content-ref-user".to_string())),
        workspace_id: Some(WorkspaceId("pack-content-ref-workspace".to_string())),
        agent_id: AgentId("pack-content-ref-agent".to_string()),
        session_id: SessionId("pack-content-ref-session".to_string()),
        input: "继续看资料包".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(
        bundle.records[0]
            .content
            .contains("conversation_file:content-77")
    );
}

#[test]
fn load_bundle_uses_combined_context_ref_for_material_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("combined-ref-user".to_string())),
        workspace_id: Some(WorkspaceId("combined-ref-workspace".to_string())),
        agent_id: Some(AgentId("combined-ref-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:conversation_file:file-old".to_string(),
            ],
            ..test_memory_record(
                "combined-ref-old-file",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- conversation_file:file-old 旧资料只记录了商品外观反馈。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:file-combined".to_string(),
            ],
            ..test_memory_record(
                "combined-ref-exact-file",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:file-combined 资料里先核对直播成交承接反馈。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.combined_refs".to_string(),
        "conversation_file:file-combined".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("combined-ref-user".to_string())),
        workspace_id: Some(WorkspaceId("combined-ref-workspace".to_string())),
        agent_id: AgentId("combined-ref-agent".to_string()),
        session_id: SessionId("combined-ref-session".to_string()),
        input: "继续分析资料".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(
        bundle.records[0]
            .content
            .contains("conversation_file:file-combined")
    );
}

#[test]
fn load_bundle_uses_context_refs_alias_for_material_relevance() {
    let store = Arc::new(InMemoryMemoryStore::new());
    let scope = MemoryScope {
        user_id: Some(UserId("context-refs-alias-user".to_string())),
        workspace_id: Some(WorkspaceId("context-refs-alias-workspace".to_string())),
        agent_id: Some(AgentId("context-refs-alias-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:conversation_file:file-old".to_string(),
            ],
            ..test_memory_record(
                "context-refs-alias-old-file",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- conversation_file:file-old 旧资料只记录了包装反馈。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:file-context-ref".to_string(),
            ],
            ..test_memory_record(
                "context-refs-alias-current-file",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:file-context-ref 资料里先核对退款说明。",
                0.35,
            )
        },
    );

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert(
        "request.context.contextRefs".to_string(),
        "conversation_file:file-context-ref".to_string(),
    );
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("context-refs-alias-user".to_string())),
        workspace_id: Some(WorkspaceId("context-refs-alias-workspace".to_string())),
        agent_id: AgentId("context-refs-alias-agent".to_string()),
        session_id: SessionId("context-refs-alias-session".to_string()),
        input: "继续分析资料".to_string(),
        metadata,
    });

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(
        bundle.records[0]
            .content
            .contains("conversation_file:file-context-ref")
    );
}

#[test]
fn load_bundle_prioritizes_exact_context_ref_over_generic_term_score() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("ref-priority-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("ref-priority-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:conversation_file:file-old".to_string(),
            ],
            ..test_memory_record(
                "ref-priority-generic-score",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- conversation_file:file-old livestream conversion retention checkout pricing material evidence review refund response customer product campaign traffic content audit worksheet operations notes 都指向旧资料。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "entity:conversation_file:file-combined".to_string(),
            ],
            ..test_memory_record(
                "ref-priority-exact-ref",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- conversation_file:file-combined 先核对最新资料。",
                0.35,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text(
        "继续分析资料 livestream conversion retention checkout pricing material evidence review refund response customer product campaign traffic content audit worksheet operations notes conversation_file:file-combined",
    );
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 260,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(
        bundle.records[0]
            .content
            .contains("conversation_file:file-combined")
    );
}

#[test]
fn load_bundle_prioritizes_multiple_exact_context_refs_case_insensitively() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("multi-ref-priority-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("multi-ref-priority-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:campaign:launch-77".to_string(),
            ],
            ..test_memory_record(
                "multi-ref-single-match-generic-score",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- campaign:launch-77 livestream conversion retention checkout pricing material evidence review refund response customer product traffic content audit worksheet operations notes 都指向旧活动。",
                0.99,
            )
        },
    );
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "decision".to_string(),
                "ENTITY:workspace_source:Source-A".to_string(),
                "entity:campaign:Launch-77".to_string(),
            ],
            ..test_memory_record(
                "multi-ref-double-match-current-context",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:decisions"),
                "Decisions:\n- workspace_source:Source-A 和 campaign:Launch-77 本轮先核对最新资料。",
                0.35,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text(
        "继续分析 workspace_source:source-a campaign:launch-77 livestream conversion retention checkout pricing material evidence review refund response customer product traffic content audit worksheet operations notes",
    );
    query.kinds = vec![MemoryKind::ConversationFact];
    query.limit = 2;
    query.budget = MemoryBudget {
        max_records: 2,
        max_chars: 320,
    };
    let bundle = MemoryStore::load_bundle(&store, query);

    assert_eq!(
        bundle
            .records
            .first()
            .and_then(|record| record.key.as_deref()),
        Some("memory_card:decisions")
    );
    assert!(
        bundle.records[0]
            .content
            .contains("workspace_source:Source-A")
    );
    assert!(bundle.records[0].content.contains("campaign:Launch-77"));
}

#[test]
fn memory_bundle_record_summaries_do_not_expose_content() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("summary-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("summary-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec![
                "memory_card".to_string(),
                "fact".to_string(),
                "entity:便携保温杯".to_string(),
            ],
            ..test_memory_record(
                "summary-record",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- 便携保温杯上一轮确认退款率偏高，需要先处理客服解释。",
                0.82,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text("继续便携保温杯");
    query.kinds = vec![MemoryKind::ConversationFact];
    let bundle = MemoryStore::load_bundle(&store, query);
    let summaries = bundle.record_summaries();

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].kind, "conversation_fact");
    assert_eq!(summaries[0].key.as_deref(), Some("memory_card:facts"));
    assert!(summaries[0].tags.contains(&"entity:便携保温杯".to_string()));
    assert!(summaries[0].char_count > 0);
    let debug = format!("{summaries:?}");
    assert!(!debug.contains("退款率偏高"));
    assert!(!debug.contains("客服解释"));
}

#[test]
fn memory_bundle_record_summaries_mark_truncated_records() {
    let store = InMemoryMemoryStore::new();
    let scope = MemoryScope {
        user_id: Some(UserId("truncated-summary-user".to_string())),
        workspace_id: None,
        agent_id: Some(AgentId("truncated-summary-agent".to_string())),
        session_id: None,
        task_id: None,
    };
    MemoryStore::upsert_record(
        &store,
        MemoryRecord {
            tags: vec!["memory_card".to_string(), "fact".to_string()],
            ..test_memory_record(
                "truncated-summary-record",
                scope.clone(),
                MemoryKind::ConversationFact,
                Some("memory_card:facts"),
                "Facts:\n- 便携保温杯上一轮确认退款率偏高，需要先处理客服解释和直播话术承接。",
                0.82,
            )
        },
    );

    let mut query = MemoryQuery::scoped(scope).for_text("便携保温杯");
    query.kinds = vec![MemoryKind::ConversationFact];
    query.budget = MemoryBudget {
        max_records: 1,
        max_chars: 18,
    };
    let bundle = MemoryStore::load_bundle(&store, query);
    let summaries = bundle.record_summaries();

    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].truncated);
    assert!(!summaries[0].tags.contains(&"memory_truncated".to_string()));
    assert!(summaries[0].char_count <= 18);
}

#[tokio::test]
async fn manager_loads_memory_bundle_into_commander_context() {
    let store = Arc::new(InMemoryMemoryStore::new());
    MemoryStore::upsert_record(
        store.as_ref(),
        MemoryRecord {
            id: MemoryId("mem-preference".to_string()),
            scope: MemoryScope {
                user_id: Some(UserId("memory-user".to_string())),
                workspace_id: None,
                agent_id: Some(AgentId("memory-agent".to_string())),
                session_id: None,
                task_id: None,
            },
            kind: MemoryKind::UserPreference,
            key: Some("platforms".to_string()),
            content: "用户偏好先看抖音 GMV，再看小红书内容效率".to_string(),
            confidence: 0.92,
            tags: vec!["profile".to_string()],
            source: MemorySource {
                kind: "test".to_string(),
                description: "seed memory".to_string(),
            },
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        },
    );

    let captured_memory_fact = Arc::new(Mutex::new(None));
    let commander_capture = Arc::clone(&captured_memory_fact);
    let mut config = AgentManagerConfig::new(
        Arc::new(move || {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::clone(&commander_capture),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store.clone();
    config.runtime.commander_fast_route = false;
    let manager = AgentManager::new(config);

    manager
        .run_request(AgentRequest {
            user_id: Some(UserId("memory-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("memory-agent".to_string()),
            session_id: SessionId("memory-session".to_string()),
            input: "验证记忆注入".to_string(),
            metadata: Default::default(),
        })
        .await
        .expect("request should complete");

    assert_eq!(
        captured_memory_fact
            .lock()
            .expect("lock memory fact capture")
            .as_deref(),
        Some("用户偏好先看抖音 GMV，再看小红书内容效率")
    );
    assert!(
        store
            .records()
            .iter()
            .any(|record| record.kind == MemoryKind::SessionSummary)
    );
}

#[test]
fn manager_memory_preview_uses_same_scope_and_safe_summaries() {
    let store = Arc::new(InMemoryMemoryStore::new());
    MemoryStore::upsert_record(
        store.as_ref(),
        test_memory_record(
            "preview-memory",
            MemoryScope {
                user_id: Some(UserId("memory-user".to_string())),
                workspace_id: Some(WorkspaceId("workspace-1".to_string())),
                agent_id: Some(AgentId("web-agent".to_string())),
                session_id: None,
                task_id: None,
            },
            MemoryKind::ConversationFact,
            Some("memory_card:facts"),
            "便携保温杯上一轮直播复盘确认退款率偏高，需要优先检查承接话术。",
            0.9,
        ),
    );

    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = store;
    let manager = AgentManager::new(config);
    let bundle = manager.load_memory_bundle_preview(&AgentRequest {
        user_id: Some(UserId("memory-user".to_string())),
        workspace_id: Some(WorkspaceId("workspace-1".to_string())),
        agent_id: AgentId("web-agent".to_string()),
        session_id: SessionId("context-preview".to_string()),
        input: "继续看便携保温杯复盘".to_string(),
        metadata: Default::default(),
    });

    assert_eq!(bundle.records.len(), 1);
    let summaries = bundle.record_summaries();
    assert_eq!(summaries[0].key.as_deref(), Some("memory_card:facts"));
    assert_eq!(summaries[0].kind, "conversation_fact");
    let summary_text = serde_json::to_string(&summaries).expect("serialize summaries");
    assert!(!summary_text.contains("退款率偏高"));
    assert!(!summary_text.contains("承接话术"));
}

#[tokio::test]
async fn stream_reports_memory_load_and_persist_events() {
    let mut config = AgentManagerConfig::new(
        Arc::new(|| {
            Ok(Box::new(MemoryCapturingCommander {
                captured_memory_fact: Arc::new(Mutex::new(None)),
            }))
        }),
        Arc::new(|| Ok(Box::new(NoopWorker))),
    );
    config.memory_store = Arc::new(InMemoryMemoryStore::new());
    let manager = AgentManager::new(config);

    let mut rx = manager
        .run_stream(AgentRequest {
            user_id: Some(UserId("memory-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("memory-agent".to_string()),
            session_id: SessionId("memory-stream-session".to_string()),
            input: "验证记忆事件".to_string(),
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
            .any(|event| matches!(event, SessionEvent::MemoryLoaded { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::MemoryCompacted { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, SessionEvent::MemoryPersisted { .. }))
    );
}
