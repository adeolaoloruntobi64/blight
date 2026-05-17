use std::{error::Error, fmt::Display};

use axum::{http::{Response, StatusCode}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BareErrorCode {
    UNKNOWN,
    HOST_NOT_FOUND,
    CONNECTION_RESET,
    CONNECTION_REFUSED,
    CONNECTION_TIMEOUT,
    MISSING_BARE_HEADER,
    INVALID_BARE_HEADER,
    #[allow(unused)]
    UNKNOWN_BARE_HEADER,
    INVALID_HEADER,
    FORBIDDEN_BARE_HEADER,
}

impl BareErrorCode {
    pub fn to_str(&self) -> &str {
        use BareErrorCode::*;
        match self {
            UNKNOWN => "UNKNOWN",
            HOST_NOT_FOUND => "HOST_NOT_FOUND",
            CONNECTION_RESET => "CONNECTION_RESET",
            CONNECTION_REFUSED => "CONNECTION_REFUSED",
            CONNECTION_TIMEOUT => "CONNECTION_TIMEOUT",
            MISSING_BARE_HEADER => "MISSING_BARE_HEADER",
            INVALID_BARE_HEADER => "INVALID_BARE_HEADER",
            UNKNOWN_BARE_HEADER => "UNKNOWN_BARE_HEADER",
            INVALID_HEADER => "INVALID_HEADER",
            FORBIDDEN_BARE_HEADER => "FORBIDDEN_BARE_HEADER",
        }
    }
    pub fn to_status(&self) -> StatusCode {
        use BareErrorCode::*;
        match self {
            UNKNOWN | HOST_NOT_FOUND | CONNECTION_RESET | CONNECTION_REFUSED
            | CONNECTION_TIMEOUT => StatusCode::INTERNAL_SERVER_ERROR,
            MISSING_BARE_HEADER | INVALID_BARE_HEADER | UNKNOWN_BARE_HEADER
            | INVALID_HEADER => StatusCode::BAD_REQUEST,
            FORBIDDEN_BARE_HEADER => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BareError {
    pub code: BareErrorCode,
    pub id: String,
    pub message: String
}

impl BareError {
    pub fn into_json_str(&self) -> String {
        json!({
            "code": self.code.to_str(),
            "id": self.id,
            "message": self.message.as_str(),
        }).to_string()
    }
}

impl IntoResponse for BareError {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header("Content-Type", "application/json")
            .header("X-Bare-Error-Code", self.code.to_str())
            .header("X-Bare-Error-ID", self.id.as_str())
            .header("X-Bare-Error-Message", self.message.as_str())
            .status(self.code.to_status())
            .body(serde_json::to_value(&self).unwrap().to_string().into())
            .unwrap()
    }
}

impl Display for BareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_value(&self).unwrap().to_string())
    }
}

impl Error for BareError {}