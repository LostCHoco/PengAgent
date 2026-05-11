//! Bearer 토큰 인증 middleware. constant-time 비교 (timing attack 방어).

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use subtle::ConstantTimeEq;

use crate::config::Config;
use crate::error::error_response;

pub async fn bearer_auth(
    State(cfg): State<Arc<Config>>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    let header_value = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    match header_value {
        Some(provided) => {
            // constant-time 비교. length mismatch도 일정 시간 (subtle::ConstantTimeEq).
            let expected = cfg.token.as_bytes();
            let provided_bytes = provided.as_bytes();
            if expected.ct_eq(provided_bytes).into() {
                Ok(next.run(req).await)
            } else {
                tracing::warn!("auth: invalid token");
                Err(error_response(StatusCode::UNAUTHORIZED, "unauthorized", "missing or invalid token").into_response())
            }
        }
        None => {
            tracing::debug!("auth: no bearer header");
            Err(error_response(StatusCode::UNAUTHORIZED, "unauthorized", "missing or invalid token").into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::{middleware, routing::get, Router};
    use axum_test::TestServer;

    fn cfg() -> Arc<Config> {
        Arc::new(Config {
            bind: "127.0.0.1:0".into(),
            token: "secret-test-token".into(),
            ops_repo_path: None,
            compose_file: "docker-compose.yml".into(),
            restart_whitelist: vec![],
            metrics_url: "".into(),
            docker_proxy_url: "".into(),
            audit_db: ":memory:".into(),
        })
    }

    async fn protected_handler() -> &'static str {
        "protected"
    }

    fn protected_router(cfg: Arc<Config>) -> Router {
        Router::new()
            .route("/protected", get(protected_handler))
            .route_layer(middleware::from_fn_with_state(cfg.clone(), bearer_auth))
            .with_state(cfg)
    }

    #[tokio::test]
    async fn no_header_returns_401() {
        let server = TestServer::new(protected_router(cfg())).unwrap();
        let resp = server.get("/protected").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn wrong_token_returns_401() {
        let server = TestServer::new(protected_router(cfg())).unwrap();
        let resp = server
            .get("/protected")
            .add_header("authorization", "Bearer wrong-token")
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn correct_token_passes() {
        let server = TestServer::new(protected_router(cfg())).unwrap();
        let resp = server
            .get("/protected")
            .add_header("authorization", "Bearer secret-test-token")
            .await;
        resp.assert_status_ok();
        resp.assert_text("protected");
    }

    #[tokio::test]
    async fn missing_bearer_prefix_returns_401() {
        let server = TestServer::new(protected_router(cfg())).unwrap();
        let resp = server
            .get("/protected")
            .add_header("authorization", "secret-test-token")
            .await;
        resp.assert_status_unauthorized();
    }
}
