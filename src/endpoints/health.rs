//! `GET /v1/health` — ping endpoint.
//!
//! 명세 (peng_agent_protocol_v1.md):
//! - 인증 불필요 (capability: none)
//! - 응답: `"ok"` (text/plain)
//!
//! 다른 endpoint 추가 후에도 health는 unauthenticated 유지 — 단순 liveness probe용.
//! agent 자체가 살아 있는지만 알려주고, 호스트·컨테이너 정보는 인증된 endpoint에서.

use axum::response::IntoResponse;

pub async fn health() -> impl IntoResponse {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use axum_test::TestServer;

    #[tokio::test]
    async fn health_returns_ok_text() {
        let app = Router::new().route("/v1/health", get(health));
        let server = TestServer::new(app).unwrap();
        let resp = server.get("/v1/health").await;
        resp.assert_status_ok();
        resp.assert_text("ok");
    }
}
