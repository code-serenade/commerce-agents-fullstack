use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsClientMessage {
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WsServerMessage {
    #[serde(rename = "type")]
    pub kind: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
