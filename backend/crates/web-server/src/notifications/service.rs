use serde_json::to_value;

use crate::{
    dto::ws::WsServerMessage, notifications::dto::NotificationMessage, statics::ws_hub::get_ws_hub,
};

pub struct NotificationService;

impl NotificationService {
    pub fn send_to_user(user_id: &str, notification: NotificationMessage) -> usize {
        get_ws_hub().push_to_user(user_id, Self::into_ws_message(notification))
    }

    pub fn broadcast(notification: NotificationMessage) -> usize {
        get_ws_hub().broadcast(Self::into_ws_message(notification))
    }

    pub fn test_notification() -> NotificationMessage {
        NotificationMessage::test()
    }

    fn into_ws_message(notification: NotificationMessage) -> WsServerMessage {
        WsServerMessage {
            kind: "notification",
            user_id: None,
            payload: Some(to_value(notification).expect("notification message must serialize")),
            message: None,
        }
    }
}
