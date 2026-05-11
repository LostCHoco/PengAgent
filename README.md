# PengAgent

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

**Docker + git-managed self-host 환경을 위한 generic admin agent**. 호스트 안에서 도는 작은 daemon. VPN 안에서 client(reference: [pengport-admin](https://github.com/LostCHoco/pengport-admin), 다른 client 구현 자유)와 [PengAgent Protocol v1](docs/spec/peng_agent_protocol_v1.md)로 통신.

PengAgent는 [PengPort](https://github.com/LostCHoco/PengPort) family에서 출발했지만 **standalone** — Jellyfin·Nextcloud·게임 서버·일반 homelab 등 docker-compose + git을 쓰는 어떤 self-host 환경에도 적용 가능 (Bevy 패턴).

## 책임

- 호스트 OS 메트릭 (node_exporter proxy)
- docker 컨테이너 상태·로그·재시작 (docker-socket-proxy 통해 화이트리스트만)
- `git pull` + `docker compose up -d` (apply:compose)
- 자체 audit log

**agent가 하지 않는 것**: 임의 셸 명령, 시크릿 매니저 통합, git push, 컨테이너 생성·삭제·이미지 build, 호스트 파일 일반 접근.

## 환경변수

| 변수 | 의미 | 기본값 | 필수? |
|---|---|---|---|
| `PENGAGENT_TOKEN` | Bearer 인증 토큰 (운영자가 시크릿 매니저로 주입) | — | ✓ |
| `PENGAGENT_BIND` | HTTP 리슨 주소 (VPN IP만!) | `0.0.0.0:9000` | |
| `PENGAGENT_OPS_REPO_PATH` | git pull 대상 디렉토리 (host 마운트) | — | ✓ (compose endpoint 쓸 시) |
| `PENGAGENT_COMPOSE_FILE` | compose 파일 이름 | `docker-compose.yml` | |
| `PENGAGENT_RESTART_WHITELIST` | 재시작 허용 컨테이너 (glob, 콤마 구분) | (빈 — 아무것도 restart 불가) | |
| `PENGAGENT_METRICS_URL` | node_exporter URL | `http://localhost:9100/metrics` | |
| `PENGAGENT_DOCKER_PROXY_URL` | docker-socket-proxy URL | `http://localhost:2375` | |
| `PENGAGENT_AUDIT_DB` | audit log SQLite 경로 | `/var/lib/pengagent/audit.db` | |
| `PENGAGENT_LOG_LEVEL` | tracing 레벨 (`trace`/`debug`/`info`/`warn`/`error`) | `info` | |
| `PENGAGENT_TLS_CERT` | TLS 인증서 path (없으면 평문 HTTP) | — | |
| `PENGAGENT_TLS_KEY` | TLS 키 path | — | |

## 빠른 시작

[`docker-compose.example.yml`](docker-compose.example.yml)를 자기 환경에 맞춰 조정 후:

```bash
PENGAGENT_TOKEN="$(openssl rand -hex 32)" \
PENGAGENT_RESTART_WHITELIST="gateway,adapter-*,caddy" \
docker compose -f docker-compose.example.yml up -d
```

client에서:
```bash
curl -H "Authorization: Bearer $PENGAGENT_TOKEN" http://localhost:9000/v1/health
# "ok"
```

## 보안 권장

- **인터넷 노출 금지**: PengAgent endpoint는 VPN(Tailscale/WireGuard/Cloudflare Tunnel) 안에만. `PENGAGENT_BIND`를 VPN IP에만 바인드.
- **PENGAGENT_TOKEN은 P0 시크릿**: RCE 권한과 동등 (docker socket + compose 실행). 외부 노출 시 즉시 회전. 평문 disk 저장 금지 — 시크릿 매니저(BWS·Vault·1Password·Doppler 등)에서 환경변수로 주입.
- **docker-socket-proxy 필수**: agent에 `/var/run/docker.sock` 직접 마운트 ❌. proxy 통해 화이트리스트 (containers/* read + restart만).
- **git pull credential은 read-only**: PR-like 흐름 아니지만 push 권한 박탈로 1차 침해 시 propagation 어려움.

## 사용 시나리오

PengAgent는 docker-compose + git을 쓰는 누구든 사용 가능:

| 시나리오 | 운영자가 추가 셋업 |
|---|---|
| 자기 homelab (Jellyfin·Nextcloud 등) | 자체 client (CLI·curl·웹) 또는 fork |
| 게임 서버 운영 | 자체 GUI |
| dev VPS 자동 배포 | CLI 통합 |
| PengPort 인스턴스 운영 | [pengport-admin](https://github.com/LostCHoco/pengport-admin) Tauri 앱 |

## 빌드

```bash
cargo build --release          # binary
docker build -t peng-agent .   # multi-stage image
```

GHCR multi-arch:
```bash
docker pull ghcr.io/lostchoco/peng-agent:latest
```

## 명세

- [PengAgent Protocol v1](docs/spec/peng_agent_protocol_v1.md) — endpoint·인증·schema·capability·에러
- [본질](docs/spec/00-essence.md) — 이 프로젝트가 무엇이고 무엇이 아닌가

## 라이선스

[AGPL-3.0-only](LICENSE). PengPort family와 동일. network clause 적용 (서비스로 제공 시 소스 공개).

비영리·후원. 자기 환경에서 자유 사용·fork·확장.

## 다른 client / agent 구현

PengAgent Protocol v1만 따르면:
- **다른 client 구현 자유** — CLI·웹·다른 GUI 등 (PengAgent Protocol consumer)
- **다른 agent 구현 자유** — Go·Node·Python 등 (이 reference 구현 대체)

PSP 동형 정신.
