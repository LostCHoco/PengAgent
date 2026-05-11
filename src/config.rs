//! 환경변수 기반 설정. PengAgent는 시크릿 매니저 통합 코드 없음 —
//! 운영자가 외부에서 환경변수로 주입 (BWS·Vault·1Password·docker secrets 등 자유).

use anyhow::{anyhow, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    /// HTTP 리슨 주소. VPN IP에만 바인드 권장 (예: `100.x.x.x:9000`).
    pub bind: String,

    /// Bearer 인증 토큰. P0 시크릿 — 평문 disk 저장 금지, 시크릿 매니저에서 주입.
    pub token: String,

    /// `apply:compose` endpoint가 git pull/compose up할 ops repo 경로 (host 마운트).
    /// None이면 compose endpoint 비활성.
    pub ops_repo_path: Option<String>,

    /// `docker compose -f <FILE>` 파일 이름 (ops_repo_path 기준).
    pub compose_file: String,

    /// 재시작 허용 컨테이너 (glob 지원). 비어 있으면 어떤 컨테이너도 restart 불가.
    pub restart_whitelist: Vec<String>,

    /// node_exporter URL (`/v1/metrics/host` 응답 source).
    pub metrics_url: String,

    /// docker-socket-proxy URL. agent는 docker socket 직접 마운트 ❌.
    pub docker_proxy_url: String,

    /// audit log SQLite 경로 (write action 영구 기록).
    pub audit_db: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let token = env::var("PENGAGENT_TOKEN")
            .map_err(|_| anyhow!("PENGAGENT_TOKEN env required (운영자가 시크릿 매니저로 주입)"))?;
        if token.is_empty() {
            return Err(anyhow!("PENGAGENT_TOKEN must not be empty"));
        }
        if token.len() < 16 {
            return Err(anyhow!(
                "PENGAGENT_TOKEN too short ({}<16). 보안 권장: 32+ chars (openssl rand -hex 32)",
                token.len()
            ));
        }

        let bind = env::var("PENGAGENT_BIND").unwrap_or_else(|_| "0.0.0.0:9000".to_string());

        let ops_repo_path = env::var("PENGAGENT_OPS_REPO_PATH")
            .ok()
            .filter(|s| !s.is_empty());

        let compose_file =
            env::var("PENGAGENT_COMPOSE_FILE").unwrap_or_else(|_| "docker-compose.yml".to_string());

        let restart_whitelist = env::var("PENGAGENT_RESTART_WHITELIST")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        let metrics_url = env::var("PENGAGENT_METRICS_URL")
            .unwrap_or_else(|_| "http://localhost:9100/metrics".to_string());

        let docker_proxy_url = env::var("PENGAGENT_DOCKER_PROXY_URL")
            .unwrap_or_else(|_| "http://localhost:2375".to_string());

        let audit_db = env::var("PENGAGENT_AUDIT_DB")
            .unwrap_or_else(|_| "/var/lib/pengagent/audit.db".to_string());

        Ok(Self {
            bind,
            token,
            ops_repo_path,
            compose_file,
            restart_whitelist,
            metrics_url,
            docker_proxy_url,
            audit_db,
        })
    }

    /// 컨테이너 이름이 restart 화이트리스트에 매칭되는지 (glob).
    /// `*`만 지원 (suffix/prefix/contains). 정규식 안 씀 (단순성 + 안전성).
    pub fn is_restart_allowed(&self, container_name: &str) -> bool {
        self.restart_whitelist
            .iter()
            .any(|pattern| glob_match(pattern, container_name))
    }
}

/// 매우 단순한 glob: `*` 만 지원. `gateway` / `adapter-*` / `*-mc` / `*` 정도 케이스.
fn glob_match(pattern: &str, s: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    match pattern.split_once('*') {
        None => pattern == s,
        Some((prefix, suffix)) => s.starts_with(prefix) && s.ends_with(suffix) && s.len() >= prefix.len() + suffix.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_exact() {
        assert!(glob_match("gateway", "gateway"));
        assert!(!glob_match("gateway", "gateway2"));
    }

    #[test]
    fn glob_prefix_star() {
        assert!(glob_match("adapter-*", "adapter-mc"));
        assert!(glob_match("adapter-*", "adapter-modded"));
        assert!(!glob_match("adapter-*", "modded"));
    }

    #[test]
    fn glob_suffix_star() {
        assert!(glob_match("*-mc", "adapter-mc"));
        assert!(!glob_match("*-mc", "mc-adapter"));
    }

    #[test]
    fn glob_all() {
        assert!(glob_match("*", "anything"));
    }
}
