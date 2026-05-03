use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationMessage {
    pub title: String,
    pub content: String,
    pub level: NotificationLevel,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationMessage {
    pub fn test() -> Self {
        Self {
            title: "测试通知".to_string(),
            content: "WebSocket 小铃铛通知连接正常".to_string(),
            level: NotificationLevel::Info,
            data: None,
        }
    }
}
