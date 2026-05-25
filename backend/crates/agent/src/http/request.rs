use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock, RwLock},
};

use serde_json::Value;
use toolcraft_request::{HeaderMap, Request};

static HTTP_CLIENTS: OnceLock<RwLock<BTreeMap<String, Arc<Request>>>> = OnceLock::new();

pub(crate) async fn get_json(
    base_url: &str,
    endpoint: &str,
    query: Option<Vec<(String, String)>>,
    headers: Option<HeaderMap>,
    timeout_sec: u64,
) -> Result<Value, String> {
    let client = client_for_base_url(base_url, timeout_sec)?;
    let response = client
        .get(endpoint, query, headers)
        .await
        .map_err(|err| format!("external http request failed: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        return Err(format!(
            "external http request returned status {}: {}",
            status.as_u16(),
            compact_text(&body, 240)
        ));
    }
    response
        .json::<Value>()
        .await
        .map_err(|err| format!("decode external http response failed: {err}"))
}

fn client_for_base_url(base_url: &str, timeout_sec: u64) -> Result<Arc<Request>, String> {
    let timeout_sec = timeout_sec.max(1);
    let key = format!("{}|{}", base_url.trim(), timeout_sec);
    let clients = HTTP_CLIENTS.get_or_init(|| RwLock::new(BTreeMap::new()));

    if let Some(client) = clients
        .read()
        .map_err(|_| "external http client cache poisoned".to_string())?
        .get(&key)
        .cloned()
    {
        return Ok(client);
    }

    let mut request = Request::with_timeout(timeout_sec)
        .map_err(|err| format!("init external http client failed: {err}"))?;
    request
        .set_base_url(base_url)
        .map_err(|err| format!("invalid external http base_url: {err}"))?;
    let request = Arc::new(request);

    let mut write_guard = clients
        .write()
        .map_err(|_| "external http client cache poisoned".to_string())?;
    Ok(write_guard
        .entry(key)
        .or_insert_with(|| Arc::clone(&request))
        .clone())
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect()
}
