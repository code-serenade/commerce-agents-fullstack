use model_gateway_rs::{
    llm::Llm,
    model::llm::{ChatContentPart, ChatMessage, LlmInput},
};
use serde_json::Value;

use super::{VisionPreprocessAgentConfig, VisionPreprocessOutput};
use crate::{
    agent_error::{CognitionError, LlmError, Result},
    agent_utils::prompt::Prompt,
    llm_routing::{AgentLlmCfg, AgentLlmRegistry, AgentLlmUsageCfg},
};

#[derive(Debug, Clone)]
pub struct VisionPreprocessInput {
    pub data_url: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub object_key: String,
    pub bucket: String,
    pub file_size: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct VisionPreprocessResult {
    pub analysis_kind: String,
    pub prompt_key: String,
    pub prompt_version: String,
    pub llm_name: String,
    pub llm_model: Option<String>,
    pub summary: String,
    pub result_json: Value,
}

pub struct VisionPreprocessAgent {
    config: VisionPreprocessAgentConfig,
    llms: AgentLlmRegistry,
}

impl VisionPreprocessAgent {
    pub fn new(
        config: VisionPreprocessAgentConfig,
        llm_configs: &[AgentLlmCfg],
        default_llm: Option<&str>,
        usage: &AgentLlmUsageCfg,
    ) -> Result<Self> {
        Ok(Self {
            config,
            llms: AgentLlmRegistry::new(llm_configs, default_llm, usage)?,
        })
    }

    pub async fn run(&self, input: VisionPreprocessInput) -> Result<VisionPreprocessResult> {
        let usage = self.llms.usage();
        let llm_name = self.config.usage_name(&usage)?;
        let llm_model = self.llms.model_by_name(&llm_name).map(str::to_string);
        let llm = self.llms.client_by_name(&llm_name)?;
        let prompt = Prompt::load_from_repo(format!("config/prompts/{}", self.config.prompt))?;
        tracing::debug!(
            system_agent_id = %self.config.id,
            object_key = %input.object_key,
            llm_name = %llm_name,
            llm_model = ?llm_model,
            data_url_bytes = input.data_url.len(),
            "vision preprocess llm request starting"
        );
        let messages = vec![
            prompt.to_message(model_gateway_rs::model::role::Role::System),
            ChatMessage::user_with_parts(vec![
                ChatContentPart::text(render_user_prompt(&input)),
                ChatContentPart::image_url_detail(input.data_url, "high"),
            ]),
        ];

        let raw = llm
            .chat_once(LlmInput { messages })
            .await
            .map_err(|err| {
                LlmError::message(format!("vision preprocess llm request failed: {err}"))
            })?
            .get_content()
            .to_string();
        tracing::debug!(
            system_agent_id = %self.config.id,
            object_key = %input.object_key,
            llm_name = %llm_name,
            output_chars = raw.chars().count(),
            "vision preprocess llm response received"
        );
        let output = decode_output(&raw)?;
        let (summary, result_json) = output.into_result_parts()?;
        tracing::debug!(
            system_agent_id = %self.config.id,
            object_key = %input.object_key,
            summary_chars = summary.chars().count(),
            "vision preprocess output decoded"
        );

        Ok(VisionPreprocessResult {
            analysis_kind: self.config.analysis_kind.clone(),
            prompt_key: self.config.prompt.clone(),
            prompt_version: self.config.prompt_version.clone(),
            llm_name,
            llm_model,
            summary,
            result_json,
        })
    }

    pub fn config(&self) -> &VisionPreprocessAgentConfig {
        &self.config
    }
}

fn render_user_prompt(input: &VisionPreprocessInput) -> String {
    format!(
        "请对这张会话上传图片做一次预识别，并只返回符合 system prompt 要求的 \
         JSON。\n\n文件信息：\n- bucket: {}\n- object_key: {}\n- file_name: {}\n- mime_type: \
         {}\n- file_size: {}\n\n注意：不要输出 Markdown，不要输出 JSON 之外的解释。",
        input.bucket,
        input.object_key,
        input.file_name.as_deref().unwrap_or("unknown"),
        input.mime_type.as_deref().unwrap_or("unknown"),
        input
            .file_size
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    )
}

fn decode_output(raw: &str) -> Result<VisionPreprocessOutput> {
    let Some(json_text) = extract_json_value(raw.trim()) else {
        return Err(CognitionError::decode("vision preprocess output is not JSON").into());
    };
    match serde_json::from_str::<VisionPreprocessOutput>(json_text) {
        Ok(output) => Ok(output),
        Err(first_err) => {
            let repaired = repair_invalid_json_string_escapes(json_text);
            serde_json::from_str::<VisionPreprocessOutput>(&repaired).map_err(|second_err| {
                CognitionError::decode(format!(
                    "invalid vision preprocess JSON: {first_err}; repair failed: {second_err}"
                ))
                .into()
            })
        }
    }
}

fn extract_json_value(input: &str) -> Option<&str> {
    for start_char in ['{', '['] {
        if let Some(candidate) = extract_balanced_json(input, start_char) {
            return Some(candidate);
        }
    }
    None
}

fn extract_balanced_json(input: &str, start_char: char) -> Option<&str> {
    let end_char = match start_char {
        '{' => '}',
        '[' => ']',
        _ => return None,
    };
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
            ch if ch == start_char => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            ch if ch == end_char && depth > 0 => {
                depth -= 1;
                if depth == 0
                    && let Some(start_index) = start
                {
                    return Some(&input[start_index ..= index]);
                }
            }
            _ => {}
        }
    }

    None
}

fn repair_invalid_json_string_escapes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut unicode_escape_remaining = 0usize;

    while let Some(ch) = chars.next() {
        if unicode_escape_remaining > 0 {
            if ch.is_ascii_hexdigit() {
                unicode_escape_remaining -= 1;
                output.push(ch);
            } else {
                unicode_escape_remaining = 0;
                output.push(ch);
            }
            continue;
        }

        if !in_string {
            if ch == '"' {
                in_string = true;
            }
            output.push(ch);
            continue;
        }

        match ch {
            '"' => {
                in_string = false;
                output.push(ch);
            }
            '\\' => match chars.peek().copied() {
                Some('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') => {
                    output.push(ch);
                    output.push(chars.next().expect("peeked escape char exists"));
                }
                Some('u') => {
                    output.push(ch);
                    output.push(chars.next().expect("peeked unicode escape exists"));
                    unicode_escape_remaining = 4;
                }
                Some(_) => {
                    output.push_str("\\\\");
                }
                None => output.push_str("\\\\"),
            },
            _ => output.push(ch),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::repair_invalid_json_string_escapes;

    #[test]
    fn repairs_invalid_json_string_escapes() {
        let input = r#"{"summary":"a \* b","scene":"c \ d","ok":"line\nnext"}"#;
        let repaired = repair_invalid_json_string_escapes(input);
        let value: serde_json::Value = serde_json::from_str(&repaired).expect("repaired json");

        assert_eq!(value["summary"], "a \\* b");
        assert_eq!(value["scene"], "c \\ d");
        assert_eq!(value["ok"], "line\nnext");
    }
}
