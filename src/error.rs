//! 에러 응답 schema (PengAgent Protocol v1).
//! 민감 정보 노출 금지 — stack trace·hostname·내부 경로 응답 본문에 절대 포함 ❌.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub schema_version: u32,
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Protocol v1 표준 에러 응답 생성.
/// 사용 예: `return error_response(StatusCode::NOT_FOUND, "container_not_found", "Container 'x' not found").into_response();`
pub fn error_response(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            schema_version: 1,
            error: ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            },
        }),
    )
}

/// PengAgent Protocol v1 에러 코드 (`docs/spec/peng_agent_protocol_v1.md`).
pub mod codes {
    pub const UNAUTHORIZED: &str = "unauthorized";
    pub const FORBIDDEN: &str = "forbidden";
    pub const NOT_FOUND: &str = "not_found";
    pub const VALIDATION_FAILED: &str = "validation_failed";
    pub const RESTART_NOT_WHITELISTED: &str = "restart_not_whitelisted";
    pub const AGENT_INTERNAL_ERROR: &str = "agent_internal_error";
    pub const AGENT_BUSY: &str = "agent_busy";
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
