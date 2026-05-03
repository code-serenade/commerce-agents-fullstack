use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use uuid::Uuid;

use crate::dto::ws::WsServerMessage;

type ConnectionMap = HashMap<Uuid, UnboundedSender<WsServerMessage>>;

#[derive(Default)]
pub struct WsHub {
    clients: Mutex<HashMap<String, ConnectionMap>>,
}

impl WsHub {
    pub fn register(&self, user_id: String) -> (Uuid, UnboundedReceiver<WsServerMessage>) {
        let (tx, rx) = unbounded_channel();
        let connection_id = Uuid::new_v4();
        let mut clients = self.clients.lock().expect("websocket hub poisoned");

        clients
            .entry(user_id)
            .or_default()
            .insert(connection_id, tx);

        (connection_id, rx)
    }

    pub fn unregister(&self, user_id: &str, connection_id: Uuid) {
        let mut clients = self.clients.lock().expect("websocket hub poisoned");

        if let Some(connections) = clients.get_mut(user_id) {
            connections.remove(&connection_id);

            if connections.is_empty() {
                clients.remove(user_id);
            }
        }
    }

    #[allow(dead_code)]
    pub fn push_to_user(&self, user_id: &str, message: WsServerMessage) -> usize {
        let mut clients = self.clients.lock().expect("websocket hub poisoned");
        let Some(connections) = clients.get_mut(user_id) else {
            return 0;
        };

        let mut sent = 0;
        connections.retain(|_, tx| {
            let ok = tx.send(message.clone()).is_ok();
            if ok {
                sent += 1;
            }
            ok
        });
        sent
    }

    #[allow(dead_code)]
    pub fn broadcast(&self, message: WsServerMessage) -> usize {
        let mut clients = self.clients.lock().expect("websocket hub poisoned");
        let mut sent = 0;

        clients.retain(|_, connections| {
            connections.retain(|_, tx| {
                let ok = tx.send(message.clone()).is_ok();
                if ok {
                    sent += 1;
                }
                ok
            });

            !connections.is_empty()
        });

        sent
    }
}

static WS_HUB: OnceLock<WsHub> = OnceLock::new();

pub fn get_ws_hub() -> &'static WsHub {
    WS_HUB.get_or_init(WsHub::default)
}
