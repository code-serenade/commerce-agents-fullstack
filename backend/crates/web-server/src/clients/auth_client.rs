use axum::http::StatusCode;
use serde::Deserialize;
use toolcraft_jwt::{VerifyJwt, VerifyJwtCfg};

use crate::{
    error::{Error, Result},
    statics::request_client::get_request_client,
};

#[derive(Debug, Deserialize)]
struct JwtVerifyConfigData {
    public_key_pem: String,
    issuer: String,
    audience: String,
}

#[derive(Debug, Deserialize)]
struct CommonResponse<T> {
    code: i32,
    data: T,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    code: i32,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthUserResponse {
    pub id: String,
    pub display_user_id: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
}

pub async fn fetch_verify_jwt() -> Result<VerifyJwt> {
    let payload: CommonResponse<JwtVerifyConfigData> =
        get_internal("jwt_verify_config", None, "fetch jwt verify config").await?;

    VerifyJwt::new(VerifyJwtCfg {
        public_key_pem: payload.data.public_key_pem,
        issuer: payload.data.issuer,
        audience: payload.data.audience,
    })
    .map_err(Into::into)
}

pub async fn get_user_by_identifier(identifier: &str) -> Result<Option<AuthUserResponse>> {
    let response = get_request_client()
        .get(
            "users/by_identifier",
            Some(vec![("identifier".to_owned(), identifier.to_owned())]),
            None,
        )
        .await
        .map_err(|e| Error::Custom(format!("fetch auth user by identifier failed: {e}")))?;

    let status = response.status();
    if status == StatusCode::NOT_FOUND {
        let body = response
            .text()
            .await
            .map_err(|e| Error::Custom(format!("fetch auth user by identifier failed: {e}")))?;
        let payload: ErrorResponse = serde_json::from_str(&body).map_err(|e| {
            Error::Custom(format!(
                "decode fetch auth user by identifier error response failed: {e}"
            ))
        })?;
        let _ = payload;
        return Ok(None);
    }

    if !status.is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| Error::Custom(format!("fetch auth user by identifier failed: {e}")))?;
        let payload: ErrorResponse = serde_json::from_str(&body).map_err(|e| {
            Error::Custom(format!(
                "decode fetch auth user by identifier error response failed: {e}"
            ))
        })?;
        return Err(Error::Custom(format!(
            "fetch auth user by identifier failed: status={}, code={}, message={}",
            status, payload.code, payload.message
        )));
    }

    let payload: CommonResponse<AuthUserResponse> = response.json().await.map_err(|e| {
        Error::Custom(format!(
            "decode fetch auth user by identifier response failed: {e}"
        ))
    })?;

    if payload.code != 0 {
        return Err(Error::Custom(format!(
            "fetch auth user by identifier failed: code={}, message={}",
            payload.code, payload.message
        )));
    }

    Ok(Some(payload.data))
}

async fn get_internal<T>(
    endpoint: &str,
    query: Option<Vec<(String, String)>>,
    action: &str,
) -> Result<CommonResponse<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let response = get_request_client()
        .get(endpoint, query, None)
        .await
        .map_err(|e| Error::Custom(format!("{action} failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| Error::Custom(format!("{action} failed: {e}")))?;
        let payload: ErrorResponse = serde_json::from_str(&body)
            .map_err(|e| Error::Custom(format!("decode {action} error response failed: {e}")))?;

        return Err(Error::Custom(format!(
            "{}: code={}, message={}",
            status.as_str(),
            payload.code,
            payload.message
        )));
    }

    let payload: CommonResponse<T> = response
        .json()
        .await
        .map_err(|e| Error::Custom(format!("decode {action} response failed: {e}")))?;

    if payload.code != 0 {
        return Err(Error::Custom(format!(
            "{action} failed: code={}, message={}",
            payload.code, payload.message
        )));
    }

    Ok(payload)
}
