---
name: PengAgent Protocol v1 명세 핵심
description: PengAgent의 표준 인터페이스. client(PengPort-admin 등) ↔ PengAgent 통신. PSP v1 동형 구조 — endpoint / 인증 / schema / capability scoping / 에러. PengAgent는 generic standalone이며 이 protocol은 PengPort 종속 없음 (어떤 docker-compose + git-managed self-host에도 적용 가능)
type: project
---

# PengAgent Protocol v1

**PengAgent의 표준 인터페이스**. client(reference impl: `pengport-admin` Tauri 앱, 다른 구현 CLI·웹 등 자유) ↔ PengAgent 컨테이너 통신을 정의. **PSP가 player ↔ instance 통신 표준이면, PengAgent Protocol은 operator ↔ self-hosted Docker host 통신 표준** — PengPort 외 환경(Jellyfin·Nextcloud·게임서버·일반 homelab)에도 적용 가능.

## 설계 원칙

- **PSP 정신 동형**: protocol 정의자 + reference 구현. 시스템 종속·운영자 환경 종속 코드는 agent 본체에 박지 말고 환경변수로 분리
- **PengPort 종속성 0**: agent 본체에 "PengPort" 단어 등장 0회 목표. docker-compose + git을 쓰는 어떤 self-host도 PengAgent 쓸 수 있어야 함
- **least privilege**: 각 endpoint capability 명시. agent는 그 이상 권한 가지지 않음
- **read 위주**: write 작업은 명시적 화이트리스트만 (`/catalog/reload`, `/container/restart`). 임의 명령 실행 endpoint 없음 (= PSP v1이 `native_command` 정의 안 한 것과 동일 정신)
- **외부 입력 방어**: 로그·메트릭 응답은 client 측에서 untrusted로 spotlight 적용 (글로벌 hook + 룰)

## 엔드포인트 (v1)

agent는 다음 endpoint들을 노출. 모두 **VPN 안만** (인터넷 노출 금지).

### 메트릭

| 경로 | 메서드 | 응답 | capability |
|---|---|---|---|
| `GET /v1/health` | — | `"ok"` | (없음, ping) |
| `GET /v1/metrics/host` | — | OpenMetrics text (node_exporter passthrough) | `read:metrics` |
| `GET /v1/metrics/host/json` | — | JSON 요약 (CPU·메모리·디스크·네트워크) | `read:metrics` |

### 컨테이너

| 경로 | 메서드 | 응답 | capability |
|---|---|---|---|
| `GET /v1/containers` | — | `[{id, name, image, state, status, ports, created_at}, ...]` | `read:containers` |
| `GET /v1/containers/<name>` | — | 단일 container 상세 | `read:containers` |
| `POST /v1/containers/<name>/restart` | body 없음 | `{success: bool, message}` | `restart:container` (화이트리스트만) |

### 로그

| 경로 | 메서드 | 응답 | capability |
|---|---|---|---|
| `GET /v1/logs/<container>?tail=N&since=...` | — | text/plain (tail N 줄) | `read:logs` |
| `GET /v1/logs/<container>/stream?since=...` | — | SSE stream (event: `log`, `error`) | `read:logs` |

### Compose reload (git-managed 운영)

| 경로 | 메서드 | 응답 | capability |
|---|---|---|---|
| `POST /v1/compose/reload` | body 없음 | `{git_pulled, compose_up, services_changed}` | `apply:compose` |

→ **편집은 client 측에서**: 운영자 laptop이 ops repo(예: PengPort의 경우 `pengdoll-ops`, 다른 환경은 자기 ops repo)를 직접 편집·검증·commit·push. agent는 `git pull` + `docker compose up -d` 만. 권한 분리(git write는 laptop, agent는 git read만).

PengPort 특수 catalog(`services.d/*.toml` PSP service entry)는 PengPort-admin client가 다루는 PengPort 영역. PengAgent는 그게 무엇인지 모름 — generic git pull + compose up만.

### Audit

| 경로 | 메서드 | 응답 | capability |
|---|---|---|---|
| `GET /v1/audit?since=...` | — | `[{ts, who, action, target, result}, ...]` | `read:audit` |

agent는 모든 write 작업(`restart`, `catalog/reload`)을 자체 audit log에 기록. client가 정기적으로 fetch.

## 인증

### Bearer 토큰 (v1)

모든 요청 헤더: `Authorization: Bearer <PENGAGENT_TOKEN>`

- `PENGAGENT_TOKEN`: P0 시크릿. 운영자가 자기 시크릿 매니저(BWS·Vault·1Password·Doppler 등 어떤 것이든)로 관리. agent 자체엔 시크릿 매니저 코드 없음 — 외부에서 환경변수로 주입.
- agent 측 비교: constant-time (`subtle::ConstantTimeEq`) — timing attack 방어
- 토큰 누락·잘못된 토큰 → `401 Unauthorized`, response body 비어 있음 (정보 누출 방지)

### Capability 분리 (v1 → 미래)

v1은 단일 토큰으로 모든 capability. v2 에서 capability-scoped 토큰 도입 가능 (예: `read-only` 토큰을 모바일/타 운영자에게 발급). v1 spec은 capability 명시로 v2 호환 준비.

### Transport

- HTTPS 권장 (self-signed OK, Tailscale 안에서는 평문도 무방하지만 일관성 위해 TLS)
- VPN 안 (Tailscale 100.x.x.x / WireGuard 10.x.x.x / Cloudflare Tunnel): agent endpoint를 VPN IP에만 바인드
- 인터넷 노출 절대 금지

## Schema 예시

### `GET /v1/metrics/host/json`
```json
{
  "schema_version": 1,
  "ts": "2026-05-11T10:30:00Z",
  "cpu": { "load_1m": 0.42, "load_5m": 0.38, "cores": 4 },
  "memory": { "total_mb": 24576, "used_mb": 3210, "available_mb": 21366 },
  "disk": [
    { "mount": "/", "total_gb": 49, "used_gb": 12, "available_gb": 37 }
  ],
  "network": { "rx_bytes": 102938472, "tx_bytes": 50293847 },
  "uptime_seconds": 1234567
}
```

### `GET /v1/containers`
```json
{
  "schema_version": 1,
  "containers": [
    {
      "id": "abc123def456",
      "name": "gateway",
      "image": "ghcr.io/lostchoco/pengport-gateway:latest",
      "state": "running",
      "status": "Up 3 days",
      "ports": ["8080/tcp"],
      "created_at": "2026-05-08T14:22:00Z",
      "restart_count": 0,
      "health": "healthy"
    }
  ]
}
```

### `POST /v1/compose/reload`
```json
{
  "schema_version": 1,
  "git_pulled": true,
  "git_head_before": "fd8b45c8...",
  "git_head_after": "7b4bea1...",
  "compose_up": true,
  "services_changed": ["service-a", "service-b"],
  "elapsed_ms": 4280
}
```

### `GET /v1/audit`
```json
{
  "schema_version": 1,
  "events": [
    {
      "ts": "2026-05-11T10:32:15Z",
      "actor": "operator",
      "action": "compose_reload",
      "target": "git",
      "result": "success",
      "metadata": { "from": "fd8b45c8", "to": "7b4bea1" }
    }
  ]
}
```

## 에러 응답

```json
{
  "schema_version": 1,
  "error": {
    "code": "container_not_found",
    "message": "Container 'foo' not found",
    "details": null
  }
}
```

에러 코드 명세 (v1):
- `unauthorized` — 401, 토큰 누락/잘못됨
- `forbidden` — 403, capability 없음 (현재는 401과 동일하지만 v2에서 분리)
- `not_found` — 404
- `validation_failed` — 400
- `restart_not_whitelisted` — 403, 컨테이너가 restart 허용 목록에 없음
- `agent_internal_error` — 500
- `agent_busy` — 503 (회전·재시작 중)

**민감 정보 노출 금지**: 에러 메시지에 stack trace·hostname·내부 경로 노출 안 함. verbose mode는 dev 빌드 only.

## Capability 명세 (v1)

agent 컨테이너의 권한:

| capability | 의미 | 구현 |
|---|---|---|
| `read:metrics` | host OS 메트릭 읽기 | node_exporter proxy. 자체 권한 없음 (호스트 메트릭은 node_exporter가) |
| `read:containers` | docker container 정보 | docker-socket-proxy `containers/json` 화이트리스트 |
| `read:logs` | docker logs tail | docker-socket-proxy `containers/<id>/logs` 화이트리스트 |
| `restart:container` | 특정 container 재시작 | docker-socket-proxy `containers/<id>/restart`, 컨테이너 화이트리스트 (`PENGAGENT_RESTART_WHITELIST` 환경변수, glob 지원, 예: `"gateway,adapter-*,caddy"`) |
| `apply:compose` | git pull + compose up -d | agent 내부에서 ops repo checkout 보유 (`PENGAGENT_OPS_REPO_PATH`). `git pull` (read-only credential) + `docker compose up -d` 실행 |
| `read:audit` | agent 자체 audit log 읽기 | agent 내부 SQLite/파일 |

**agent가 가지지 않는 권한** (어떤 환경에서 쓰든 동일):
- git push (운영자 laptop만)
- 시크릿 매니저 write (운영자 laptop만)
- 임의 명령 실행
- 시크릿 변경
- 컨테이너 생성·삭제 (restart만 허용)
- docker network·volume 조작
- 호스트 파일시스템 일반 read/write (마운트된 ops repo 외)

## 외부 입력 방어 (이 protocol 특수)

agent → client 응답은 **untrusted external content**. client 측에서:

- 로그 텍스트(`/v1/logs/<container>`)에 instruction-like 문자열이 있을 수 있음 (악성 사용자가 채팅·이메일·웹 폼으로 박은 텍스트가 서비스 로그에 들어가는 등) → 사용자 표시 외 자동 행동 트리거 금지
- 메트릭 응답은 비교적 안전하지만 schema 검증 후 사용
- container status·name 등을 UI에 표시할 때 XSS 방어 (React 기본 escape에 의존, dangerouslySetInnerHTML 금지)

## v2 / 미래 확장 후보

- WebSocket 으로 메트릭 push (현재 SSE)
- Capability-scoped 토큰 (read-only 토큰 등)
- mTLS 인증 (Bearer 대체)
- 시크릿 회전 trigger endpoint (agent가 운영자 시크릿 매니저 트리거? 또는 laptop trigger만)
- 다중 host 관리 (한 client에서 여러 PengAgent 인스턴스)
- 컨테이너 logs filtering server-side (정규식·로그 레벨)

v1은 위 미실현 기능을 정의 안 함 — 명세 차원에서 보수적.

## reference 구현 vs 다른 구현

- **agent reference**: `LostCHoco/PengAgent` (Rust, GHCR multi-arch image). PengAgent Protocol v1 완전 구현.
- **client reference**: `LostCHoco/pengport-admin` (Tauri+React). PengPort 운영자 전용 UX.
- **다른 client 구현 가능**: CLI(`curl + jq`로 직접), 웹, 다른 GUI (`pengctl`·`agent-tui` 등 누군가 만들 수 있음). PengAgent Protocol v1만 따르면 PengAgent와 호환.
- **다른 agent 구현 가능**: Go·Node·Python 등. agent의 사양만 만족하면 됨. 이론적으로는.

## 참고

- 본가 PSP 명세: `psp_spec_core.md` (동형 구조 참고)
- PengAgent 자체 본질: `peng_agent_essence.md`
- admin client 본질: `admin_essence.md`
- 신뢰 모델: `admin_trust_model.md` (작성 예정)
- 아키텍처: `admin_architecture.md` (작성 예정)

## 명세 vs 구현

v1 spec은 protocol 정의자. 본가 PSP와 동일하게 **여러 구현이 가능**:

- reference 구현: `pengport-admin-agent` (Rust, GHCR multi-arch image)
- 다른 운영자가 자체 agent 구현 가능 (예: Go·Node 등 다른 언어). Admin Protocol v1만 따르면 PengPort-admin 클라이언트 호환.

client 구현은 단일 (reference만, fork 가능): `pengport-admin` (Tauri+React, GitHub Releases zip)

## 참고

- 본가 PSP 명세: `psp_spec_core.md` (동형 구조 참고)
- 신뢰 모델: `admin_trust_model.md` (operator boundary 정의)
- 아키텍처: `admin_architecture.md` (다이어그램·컴포넌트 매핑)
- repo 구조: `admin_repo_structure.md`
