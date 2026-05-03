use std::sync::OnceLock;

use toolcraft_request::{HeaderMap, Request};

use crate::error::{Error, Result};

static REQUEST_CLIENT: OnceLock<Request> = OnceLock::new();

pub fn init_request_client(base_url: String, header: String, token: String) -> Result<()> {
    let mut client =
        Request::new().map_err(|e| Error::Custom(format!("request client init failed: {e}")))?;

    client
        .set_base_url(&base_url)
        .map_err(|e| Error::Custom(format!("invalid request client base url: {e}")))?;

    let mut default_headers = HeaderMap::new();
    default_headers
        .insert(header, token)
        .map_err(|e| Error::Custom(format!("invalid request client header: {e}")))?;
    client.set_default_headers(default_headers);

    REQUEST_CLIENT
        .set(client)
        .map_err(|_| Error::Custom("Request client already initialized".to_owned()))?;
    Ok(())
}

pub fn get_request_client() -> &'static Request {
    REQUEST_CLIENT
        .get()
        .expect("Request client not initialized")
}
