use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::llm::{ChatMessage, LlmInput},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    agent_error::{Error, LlmError, Result, SettingsError},
    api::{
        AgentManager, AgentManagerConfig, Cognition, CognitionEngine, CognitionInput,
        CognitionResult, RoleCognitionFactory, RoleProfile,
    },
};

pub type AgentLlmClient = ChatCompletionsLlm;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentLlmCfg {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub reasoning_effort: Option<String>,
    pub chat_completions_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentRuntimeCfg {
    #[serde(default)]
    pub llm: Vec<AgentLlmCfg>,
    #[serde(default)]
    pub default_llm: Option<String>,
}

impl AgentRuntimeCfg {
    pub fn from_llm(llm: Vec<AgentLlmCfg>) -> Self {
        Self {
            llm,
            default_llm: None,
        }
    }
}

pub fn build_agent_manager(config: AgentRuntimeCfg) -> Result<AgentManager> {
    let llm = Arc::new(build_default_llm(config)?);

    let commander_llm = Arc::clone(&llm);
    let commander = Arc::new(move || {
        Ok(Box::new(CognitionEngine::new(
            Arc::clone(&commander_llm),
            "prompts/zh/cognition/commander_routing.md",
        )?) as Box<dyn Cognition + Send>)
    });

    let worker_llm = Arc::clone(&llm);
    let worker: RoleCognitionFactory = Arc::new(move |role: &RoleProfile| {
        Ok(Box::new(AgentRoleWorkerCognition {
            llm: Arc::clone(&worker_llm),
            role: role.clone(),
        }) as Box<dyn Cognition + Send>)
    });

    let mut manager_config = AgentManagerConfig::new_role_aware(commander, worker);
    manager_config.runtime.response_timeout = Duration::from_secs(900);
    manager_config.runtime.commander_fast_route = true;
    manager_config.runtime.commander_fast_route_min_score = 2.0;

    Ok(AgentManager::new(manager_config))
}

fn build_default_llm(config: AgentRuntimeCfg) -> Result<AgentLlmClient> {
    if config.llm.is_empty() {
        return Err(SettingsError::missing_llm_usage(
            "agent runtime requires at least one [[llm]] entry",
        )
        .into());
    }

    let selected = match config.default_llm.as_deref() {
        Some(name) => config
            .llm
            .iter()
            .find(|llm| llm.name == name)
            .ok_or_else(|| SettingsError::missing_llm_usage(format!("llm '{name}' not found")))?,
        None => config
            .llm
            .first()
            .expect("checked non-empty llm config above"),
    };

    let mut client = ChatCompletionsLlm::new(
        &selected.base_url,
        &selected.model,
        selected.api_key.as_deref(),
    )
    .map_err(|err| {
        Error::from(LlmError::message(format!(
            "init llm client '{}' failed: {err}",
            selected.name
        )))
    })?
    .with_max_tokens(selected.max_tokens)
    .with_temperature(selected.temperature)
    .with_reasoning_effort(selected.reasoning_effort.as_deref());

    if let Some(endpoint) = &selected.chat_completions_endpoint {
        client = client.with_chat_completions_endpoint(endpoint.clone());
    }

    Ok(client)
}

struct AgentRoleWorkerCognition {
    llm: Arc<AgentLlmClient>,
    role: RoleProfile,
}

#[async_trait]
impl Cognition for AgentRoleWorkerCognition {
    async fn evaluate(&self, input: CognitionInput) -> CognitionResult {
        let system = format!(
            r#"你是 commerce agent runtime 的专用角色 worker。

当前角色：
- id: {}
- name: {}
- runtime_role: {}

你只负责完成当前 assignment 的本节点产物，不调度其他 worker。
如果 Context facts 中包含 upstream report，请把它们作为输入。

你必须只返回合法 JSON，不要 Markdown，不要代码块：
{{
  "decision": {{"kind": "NoAction"}},
  "role_output": {{
    "summary": "本角色一句话结论",
    "findings": ["本角色关键发现"],
    "recommendations": ["本角色建议或动作"],
    "evidence": ["你实际使用了哪些上游事实"],
    "risks": ["风险、边界或失败条件"],
    "open_questions": []
  }}
}}

除非确实无法继续，否则 role_output.open_questions 必须为空数组。
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
                return CognitionResult::Success(json!({
                    "decision": { "kind": "NoAction" },
                    "role_output": {
                        "summary": format!("LLM 调用失败：{err}"),
                        "findings": [],
                        "recommendations": [],
                        "evidence": [],
                        "risks": [format!("LLM 调用失败：{err}")],
                        "open_questions": [format!("LLM 调用失败：{err}")]
                    }
                }));
            }
        };

        if let Some(parsed) = parse_role_json(&raw)
            && has_role_output_content(&parsed)
        {
            return CognitionResult::Success(parsed);
        }

        CognitionResult::Success(parse_or_wrap_role_output(&raw))
    }
}

fn render_worker_input(input: &CognitionInput) -> String {
    let facts = if input.context.facts.is_empty() {
        "- none".to_string()
    } else {
        input
            .context
            .facts
            .iter()
            .map(|fact| {
                format!(
                    "- source: {}\n  reliability: {}\n  content: {}",
                    fact.source, fact.reliability, fact.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let metadata = if input.context.metadata.is_empty() {
        "- none".to_string()
    } else {
        input
            .context
            .metadata
            .iter()
            .map(|(key, value)| format!("- {key}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Intent:\n- id: {}\n- kind: {:?}\n- description: {}\n\nContext metadata:\n{}\n\nContext \
         facts:\n{}",
        input.intent.id, input.intent.kind, input.intent.description, metadata, facts
    )
}

fn parse_or_wrap_role_output(raw: &str) -> Value {
    match parse_role_json(raw) {
        Some(value) if has_role_output_content(&value) => value,
        Some(value) => json!({
            "decision": value.get("decision").cloned().unwrap_or_else(|| json!({ "kind": "NoAction" })),
            "role_output": {
                "summary": format!("模型未按契约返回有效 role_output，原始输出：{}", raw),
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": ["模型输出缺少结构化 role_output"],
                "open_questions": ["需要 worker 重新提交结构化 role_output"]
            }
        }),
        None => json!({
            "decision": { "kind": "NoAction" },
            "role_output": {
                "summary": raw,
                "findings": [],
                "recommendations": [],
                "evidence": [],
                "risks": [],
                "open_questions": []
            }
        }),
    }
}

fn parse_role_json(raw: &str) -> Option<Value> {
    serde_json::from_str(raw)
        .ok()
        .or_else(|| extract_json_object(raw).and_then(|value| serde_json::from_str(value).ok()))
}

fn has_role_output_content(value: &Value) -> bool {
    let Some(role_output) = value.get("role_output") else {
        return false;
    };
    if let Some(text) = role_output.as_str() {
        return !text.trim().is_empty();
    }
    let Some(role_output) = role_output.as_object() else {
        return false;
    };
    role_output
        .get("summary")
        .and_then(|summary| summary.as_str())
        .map(str::trim)
        .is_some_and(|summary| !summary.is_empty())
}

fn extract_json_object(input: &str) -> Option<&str> {
    let mut in_string = false;
    let mut escape = false;
    let mut depth = 0usize;
    let mut start = None;

    for (index, ch) in input.char_indices() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    return start.map(|start| &input[start ..= index]);
                }
            }
            _ => {}
        }
    }

    None
}
