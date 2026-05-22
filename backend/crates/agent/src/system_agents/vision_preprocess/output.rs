use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionPreprocessTextOutput {
    #[serde(default)]
    pub ocr: Vec<String>,
    #[serde(default)]
    pub uncertain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionPreprocessStyleOutput {
    #[serde(default)]
    pub visual_style: String,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub mood: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionPreprocessCommerceHintsOutput {
    #[serde(default)]
    pub possible_use_cases: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionPreprocessOutput {
    pub summary: String,
    #[serde(default)]
    pub scene: String,
    #[serde(default)]
    pub subjects: Vec<String>,
    #[serde(default)]
    pub text: Option<VisionPreprocessTextOutput>,
    #[serde(default)]
    pub style: Option<VisionPreprocessStyleOutput>,
    #[serde(default)]
    pub commerce_hints: Option<VisionPreprocessCommerceHintsOutput>,
    #[serde(default)]
    pub agent_context: String,
    #[serde(default)]
    pub uncertainties: Vec<String>,
}

impl VisionPreprocessOutput {
    pub fn into_result_parts(self) -> crate::agent_error::Result<(String, Value)> {
        let summary = if self.agent_context.trim().is_empty() {
            self.summary.trim().to_string()
        } else {
            self.agent_context.trim().to_string()
        };
        let value = serde_json::to_value(self).map_err(|err| {
            crate::agent_error::CognitionError::decode(format!(
                "vision preprocess result serialization failed: {err}"
            ))
        })?;
        Ok((summary, value))
    }
}
