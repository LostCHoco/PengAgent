//! PengAgent — generic admin agent for docker-compose + git-managed self-host.
//! PengAgent Protocol v1 reference implementation.
//!
//! See `docs/spec/peng_agent_protocol_v1.md` for the protocol.

use anyhow::Result;
use axum::{middleware, routing::get, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod auth;
mod config;
mod endpoints;
mod error;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cfg = Arc::new(Config::from_env()?);
    tracing::info!(
        bind = %cfg.bind,
        whitelist_count = cfg.restart_whitelist.len(),
        ops_repo = ?cfg.ops_repo_path,
        "PengAgent starting"
    );

    let app = build_router(cfg.clone());

    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!(bind = %cfg.bind, "listening");
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    let level = std::env::var("PENGAGENT_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let filter = EnvFilter::try_new(&level).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn build_router(cfg: Arc<Config>) -> Router {
    // Public (no auth) routes
    let public = Router::new().route("/v1/health", get(endpoints::health::health));

    // Authenticated routes는 endpoint 추가될 때 함께 정의 (빈 Router에 route_layer는 no-op + panic).
    // 미래 추가 예: with_auth(cfg, |r| r.route("/v1/metrics/host", get(endpoints::metrics::host)))

    Router::new()
        .merge(public)
        .layer(TraceLayer::new_for_http())
        .with_state(cfg)
}

/// Helper to build an authenticated sub-router. Use when adding the first protected endpoint.
#[allow(dead_code)]
fn with_auth<F>(cfg: Arc<Config>, build: F) -> Router<Arc<Config>>
where
    F: FnOnce(Router<Arc<Config>>) -> Router<Arc<Config>>,
{
    build(Router::new()).route_layer(middleware::from_fn_with_state(cfg, auth::bearer_auth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    fn test_config() -> Arc<Config> {
        Arc::new(Config {
            bind: "127.0.0.1:0".to_string(),
            token: "test-token".to_string(),
            ops_repo_path: None,
            compose_file: "docker-compose.yml".to_string(),
            restart_whitelist: vec![],
            metrics_url: "http://localhost:9100/metrics".to_string(),
            docker_proxy_url: "http://localhost:2375".to_string(),
            audit_db: ":memory:".to_string(),
        })
    }

    #[tokio::test]
    async fn health_returns_ok_without_auth() {
        let app = build_router(test_config());
        let server = TestServer::new(app).unwrap();
        let resp = server.get("/v1/health").await;
        resp.assert_status_ok();
        resp.assert_text("ok");
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = build_router(test_config());
        let server = TestServer::new(app).unwrap();
        let resp = server.get("/v1/nonexistent").await;
        resp.assert_status_not_found();
    }
}
