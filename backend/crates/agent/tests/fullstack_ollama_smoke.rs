use std::{env, sync::Arc, time::Duration};

use agent::api::{
    AgentId, AgentManager, AgentManagerConfig, AgentRequest, Cognition, CognitionEngine,
    CognitionInput, CognitionResult, RoleCognitionFactory, RoleProfile, SessionId, UserId,
};
use async_trait::async_trait;
use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::llm::{ChatMessage, LlmInput},
};
use serde_json::{Value, json};
use utils::llm_json;

const ENABLE_ENV: &str = "CM_AGENT_RUN_LOCAL_LLM_SMOKE";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_MODEL: &str = "gemma4:26b";
const DEFAULT_TASK: &str = "请用一句话解释这个 agent 框架现在是否可以运行。";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fullstack_workspace_runs_agent_with_local_ollama() {
    if env::var(ENABLE_ENV).as_deref() != Ok("1") {
        eprintln!("skipped: set {ENABLE_ENV}=1 to run local Ollama smoke test");
        return;
    }

    let base_url = env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_BASE_URL.into());
    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.into());
    let task = env::var("CM_AGENT_SMOKE_TASK").unwrap_or_else(|_| DEFAULT_TASK.to_string());
    let llm = Arc::new(
        ChatCompletionsLlm::new(&base_url, &model, None)
            .expect("Ollama client should initialize")
            .with_temperature(Some(0.2))
            .with_max_tokens(Some(512)),
    );
    let manager = build_manager(llm).expect("manager should initialize");

    let result = manager
        .run_request(AgentRequest {
            user_id: Some(UserId("fullstack-smoke-user".to_string())),
            workspace_id: None,
            agent_id: AgentId("fullstack-smoke-agent".to_string()),
            session_id: SessionId("fullstack-ollama-smoke".to_string()),
            input: task,
            metadata: Default::default(),
        })
        .await
        .expect("agent session should complete");

    assert!(
        !result.output.trim().is_empty(),
        "agent output should not be empty"
    );
    assert_eq!(result.task_graphs.len(), 1);
    assert!(!result.worker_details.is_empty());
    assert!(result.worker_details.iter().any(|detail| {
        detail
            .role_output
            .as_ref()
            .is_some_and(|output| !output.summary.trim().is_empty())
    }));
    assert_eq!(manager.active_session_count(), 0);
}

fn build_manager(llm: Arc<ChatCompletionsLlm>) -> agent::api::Result<AgentManager> {
    let commander_llm = Arc::clone(&llm);
    let commander = Arc::new(move || {
        Ok(Box::new(CognitionEngine::new(
            Arc::clone(&commander_llm),
            "config/prompts/zh/cognition/commander_routing.md",
        )?) as Box<dyn Cognition + Send>)
    });

    let worker_llm = Arc::clone(&llm);
    let worker: RoleCognitionFactory = Arc::new(move |role: &RoleProfile| {
        Ok(Box::new(LocalOllamaWorker {
            llm: Arc::clone(&worker_llm),
            role: role.clone(),
        }) as Box<dyn Cognition + Send>)
    });

    let mut config = AgentManagerConfig::new_role_aware(commander, worker);
    config.runtime.response_timeout = Duration::from_secs(180);
    config.runtime.commander_fast_route = true;
    config.runtime.commander_fast_route_min_score = 2.0;
    Ok(AgentManager::new(config))
}

struct LocalOllamaWorker {
    llm: Arc<ChatCompletionsLlm>,
    role: RoleProfile,
}

#[async_trait]
impl Cognition for LocalOllamaWorker {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        let system = format!(
            r#"你是 fullstack backend 内置 agent crate 的专用 worker。

当前角色：
- id: {}
- name: {}
- runtime_role: {}

你只完成当前 assignment 节点，不调度其他 worker。
必须只返回合法 JSON，不要 Markdown，不要代码块：
{{
  "decision": {{"kind": "NoAction"}},
  "role_output": {{
    "summary": "本角色一句话结论",
    "findings": ["关键发现"],
    "recommendations": ["建议或动作"],
    "evidence": ["使用的任务事实或上游输入"],
    "risks": ["风险或边界"],
    "open_questions": []
  }}
}}
"#,
            self.role.id.0, self.role.name, self.role.runtime_role
        );
        let user = render_worker_input(&input);
        let raw = match self
            .llm
            .chat_once(LlmInput {
                messages: vec![ChatMessage::system(system), ChatMessage::user(user)],
            })
            .await
        {
            Ok(output) => output.get_content().trim().to_string(),
            Err(err) => {
                return CognitionResult::Success(fallback_output(
                    &self.role.runtime_role,
                    &format!("LLM 调用失败：{err}"),
                ));
            }
        };

        let value = parse_json_object(&raw)
            .filter(has_role_output_summary)
            .unwrap_or_else(|| fallback_output(&self.role.runtime_role, &raw));
        CognitionResult::Success(value)
    }
}

fn render_worker_input(input: &CognitionInput) -> String {
    let facts = input
        .context
        .facts
        .iter()
        .map(|fact| format!("- {}: {}", fact.source, fact.content))
        .collect::<Vec<_>>()
        .join("\n");
    let metadata = input
        .context
        .metadata
        .iter()
        .map(|(key, value)| format!("- {key}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Intent:\n{}\n\nMetadata:\n{}\n\nFacts:\n{}",
        input.intent.description, metadata, facts
    )
}

fn fallback_output(role: &str, raw: &str) -> Value {
    json!({
        "decision": { "kind": "NoAction" },
        "role_output": {
            "summary": format!("{role} 角色已完成兜底输出：{}", truncate(raw, 240)),
            "findings": [format!("{role} 已读取当前任务和上下游上下文"), "模型输出已被测试兜底包装"],
            "recommendations": ["继续按当前任务图推进", "检查 worker_details 是否包含本节点", "前端可读取 role_output 展示详情"],
            "evidence": ["local_ollama_smoke"],
            "risks": [format!("模型未返回严格 JSON，原始输出已截断：{}", truncate(raw, 240))],
            "open_questions": []
        }
    })
}

fn parse_json_object(raw: &str) -> Option<Value> {
    llm_json::parse_llm_json_value(raw).ok()
}

fn has_role_output_summary(value: &Value) -> bool {
    value
        .pointer("/role_output/summary")
        .and_then(Value::as_str)
        .is_some_and(|summary| !summary.trim().is_empty())
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut output = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}
