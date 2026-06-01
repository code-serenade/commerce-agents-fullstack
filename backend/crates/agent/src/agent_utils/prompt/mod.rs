use std::path::{Path, PathBuf};

use model_gateway_rs::model::{llm::ChatMessage, role::Role};
use serde_json::Value;
use utils::llm_json;

use crate::{
    agent_error::{PromptError, Result},
    log_error,
};

#[derive(Debug, Clone)]
pub struct Prompt {
    pub path: PathBuf,
    pub content: String,
}

impl Prompt {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = std::fs::read_to_string(&path).map_err(|err| {
            let prompt_err = if err.kind() == std::io::ErrorKind::NotFound {
                PromptError::not_found(path.display().to_string())
            } else {
                PromptError::io(err.to_string())
            };
            log_error!(prompt_err);
            prompt_err
        })?;
        Ok(Self { path, content })
    }

    pub fn load_from_repo(relative: impl AsRef<Path>) -> Result<Self> {
        let relative = relative.as_ref();
        if relative.exists() {
            return Self::load(relative);
        }
        let manifest_relative = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative);
        Self::load(manifest_relative)
    }

    pub fn to_message(&self, role: Role) -> ChatMessage {
        ChatMessage {
            role,
            content: self.content.clone().into(),
        }
    }

    pub fn extract_json_schema(&self) -> Result<Value> {
        let marker = "JSON schema:";
        let start = self.content.find(marker).ok_or_else(|| {
            let err = PromptError::internal("missing 'JSON schema:' section in prompt");
            log_error!(err);
            err
        })?;

        let schema_region = &self.content[start + marker.len()..];
        let schema_text = llm_json::extract_json_value(schema_region).ok_or_else(|| {
            let err = PromptError::internal("failed to extract JSON schema from prompt");
            log_error!(err);
            err
        })?;

        let schema = serde_json::from_str(schema_text).map_err(|err| {
            let prompt_err = PromptError::internal(format!("invalid JSON schema in prompt: {err}"));
            log_error!(prompt_err);
            prompt_err
        })?;
        Ok(schema)
    }

    /// Render prompt content by replacing {{key}} with provided values.
    pub fn render(&self, entries: &[(&str, String)]) -> String {
        let mut rendered = self.content.clone();
        for (key, value) in entries {
            let token = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&token, value);
        }
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::Prompt;

    #[test]
    fn extracts_json_schema_from_prompt_content() {
        let prompt = Prompt {
            path: "inline".into(),
            content: r#"
Header

JSON schema:
{
  "route": {
    "agent": "",
    "message": ""
  }
}

Footer
"#
            .to_string(),
        };

        let schema = prompt.extract_json_schema().expect("schema should parse");
        assert_eq!(schema["route"]["agent"], "");
        assert_eq!(schema["route"]["message"], "");
    }
}
