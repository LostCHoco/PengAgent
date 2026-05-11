---
name: PengAgent 본질
description: docker-compose + git-managed self-host를 위한 generic admin agent. PengPort family 출신이지만 standalone (Bevy 패턴). PengAgent Protocol v1 reference impl. AGPL-3.0
type: project
---

# PengAgent

## 본질 한 줄

**Docker + git-managed self-host 환경의 운영을 위한 generic admin agent**. host machine 안에서 도는 작은 daemon. VPN 안에서 client(reference: PengPort-admin Tauri 앱, 다른 client 구현 자유)와 PengAgent Protocol v1로 통신.

## 무엇이고 무엇이 아닌가

**이것**:
- docker-compose + git으로 운영하는 self-host 서비스 환경의 일상 운영(상태 확인·로그·재시작·git pull + compose up)을 SSH 없이 가능하게 하는 daemon
- PengAgent Protocol v1 reference 구현 (Rust, GHCR multi-arch image)
- PengPort family에서 출발했지만 standalone — Jellyfin·Nextcloud·게임 서버·일반 homelab 등 docker-compose 쓰는 누구든 사용 가능
- AGPL-3.0, 비영리

**아닌 것**:
- PengPort 의존 (의존성·코드·문서에 "PengPort" 단어 등장 0회 목표)
- 컨테이너 오케스트레이터 (Kubernetes·Portainer 대체 아님)
- 모니터링 SaaS / 메트릭 시계열 DB (Prometheus·Grafana 대체 아님 — 호스트 메트릭은 node_exporter proxy로 위임)
- 임의 명령 실행 도구 (capability 화이트리스트만)
- 시크릿 매니저 (외부 시크릿 매니저가 환경변수로 주입)
- 운영자별 SaaS / 호스팅

## Bevy 패턴 (왜 family 출신인데 standalone인가)

[Bevy](https://bevyengine.org/)는 ECS 게임 엔진. 자체 ecosystem(`bevy_*` plugin들)이 있지만 핵심은 standalone — Bevy를 모르는 사람도 자기 프로젝트에 갖다 쓸 수 있음. plugin들은 Bevy를 사용하는 사용자 편의일 뿐 Bevy 본체가 plugin에 의존하지 않음.

PengAgent도 동일:
- **PengAgent family에서 태어남** (PengPort 운영 필요에서 출발)
- **standalone 사용 가능** (PengPort 없는 사람도 자기 환경에 docker admin agent로 사용)
- **PengPort family 통합은 그 위의 layer** (PengPort-admin client가 PengAgent에 의존하지만 그 반대 X)

## PengPort family에서의 위치

```
LostCHoco/PengPort                    (메인 client, player용)
LostCHoco/pengport-shared             (PSP schema 라이브러리, 본가 workspace 멤버)
LostCHoco/pengport-gateway            (PSP gateway, public 서비스)
LostCHoco/pengport-adapter-minecraft  (PSP MC 어댑터, public 서비스)
LostCHoco/pengport-ops (private)      (펭돌서버 운영 환경)
LostCHoco/pengport-admin              (운영자 client, PengPort 전용 — 신설 예정)
LostCHoco/PengAgent                   (generic admin agent, standalone — 신설 예정)
```

PengAgent는 LostCHoco/PengAgent로 별도 repo. CamelCase는 standalone 강조 (PengPort family의 standalone member들 — PengPort, PengAgent).

## 4대 원칙 (PengAgent 맥락)

본가 4대 원칙(`core_principles.md`)을 PengAgent에도 적용. PengPort 종속 표현 제거 버전:

1. **항구성 가정 금지** — 운영 호스트(Oracle·AWS·자체 서버 등), VPN(Tailscale·WireGuard·Cloudflare Tunnel), 시크릿 매니저(BWS·Vault·1Password 등), ops repo 위치 모두 가변. 환경변수로 분리.
2. **Software vs Instance 분리** — "PengAgent"=소프트웨어, 운영자별 배포=인스턴스. 코드에 특정 운영자(LostCHoco)·특정 ops repo 이름(pengdoll-ops) 박지 말 것.
3. **Protocol-First Extension (PengAgent Protocol)** — `peng_agent_protocol_v1.md` 표준. 다른 client 구현 자유.
4. **Generic + Kiosk 양립** — 흐름은 본가와 동일하지만 PengAgent에서 "kiosk"는 적게 의미. agent 자체는 항상 generic. 운영자별 환경은 환경변수로 주입.

## 책임 범위

`peng_agent_protocol_v1.md`의 endpoint 그대로:
- 호스트 OS 메트릭 (node_exporter proxy)
- docker 컨테이너 상태 (docker-socket-proxy)
- 컨테이너 로그 tail/stream
- 컨테이너 재시작 (화이트리스트만)
- git pull + `docker compose up -d` (ops repo)
- 자체 audit log

agent가 **하지 않는** 일:
- 시크릿 매니저 직접 접근 (외부에서 환경변수로 주입)
- git push
- 임의 셸 명령
- 컨테이너 생성·삭제·이미지 빌드
- docker network·volume 조작
- 호스트 파일시스템 일반 접근

## 의존성 추상화 포인트

agent를 generic하게 만드는 핵심: 환경 종속 부분을 환경변수로:

| 종속 부분 | 환경변수 | default | 예 |
|---|---|---|---|
| 인증 토큰 | `PENGAGENT_TOKEN` | 필수 | (시크릿 매니저에서 주입) |
| bind 주소 | `PENGAGENT_BIND` | `0.0.0.0:9000` | `100.x.x.x:9000` (Tailscale IP만) |
| ops repo path | `PENGAGENT_OPS_REPO_PATH` | 필수 | `/srv/ops` (컨테이너 안 마운트) |
| compose file | `PENGAGENT_COMPOSE_FILE` | `docker-compose.yml` | (ops repo 상대 경로) |
| 재시작 화이트리스트 | `PENGAGENT_RESTART_WHITELIST` | (없음) | `gateway,adapter-*,caddy` (glob) |
| 메트릭 source | `PENGAGENT_METRICS_URL` | `http://localhost:9100/metrics` | (node_exporter 위치) |
| docker socket proxy | `PENGAGENT_DOCKER_PROXY_URL` | `http://localhost:2375` | (docker-socket-proxy 위치) |
| audit log 위치 | `PENGAGENT_AUDIT_DB` | `/var/lib/pengagent/audit.db` | (SQLite) |

agent에 "PengPort"·"pengdoll" 단어 등장 0회. 운영자별 환경은 docker-compose나 시크릿 매니저로 주입.

## 사용 시나리오 (PengPort 외)

| 시나리오 | client | PengAgent로 무엇? |
|---|---|---|
| Jellyfin homelab 운영 | CLI 또는 자체 웹 UI | `jellyfin` 컨테이너 상태·로그·재시작, compose 업데이트 |
| Nextcloud + Collabora | CLI · `pengport-admin` fork | 둘 다 상태 모니터링, nightly cron 후 재시작 트리거 |
| 게임 서버 (MC 외) | 자체 GUI | 서버 컨테이너 health·로그·재시작 |
| 일반 dev VPS | CLI | 자기 프로젝트 컨테이너·git 배포 자동화 |
| 펭돌서버 (PengPort 운영) | `pengport-admin` Tauri | PSP gateway·adapter들 + caddy + 자기 mc 서버 모니터링 |

각 시나리오에서 PengAgent 본체는 동일 binary. 환경변수만 다름.

## 시크릿 처리 (어디까지 agent 책임?)

**agent 책임 0** — 시크릿 매니저 통합은 agent에 포함 안 시킴. 이유:
- BWS·Vault·1Password·Doppler·CyberArk 등 매니저별 SDK 통합 = agent 비대화
- 시크릿 매니저는 보통 운영자 환경 specific (사용자가 이미 쓰는 것 있음)
- 시크릿은 docker-compose의 env_file 또는 docker secrets, 또는 `bws run --` 같은 wrapper로 주입하는 게 표준

→ agent는 환경변수만 읽음. 시크릿 매니저는 운영자가 외부에서 처리. **이것이 generic이 가능한 이유** — agent가 특정 매니저에 종속되지 않음.

PengPort 운영자는 BWS 사용 (`start.sh`가 BWS run wrap하여 환경변수 주입). 다른 운영자는 다른 매니저 자유 사용.

## 라이선스 및 비즈니스

- **AGPL-3.0-only** (PengPort family와 동일)
- 비영리·후원 모델
- 다른 운영자가 자기 환경에서 사용 자유. AGPL network clause 적용 (서비스로 제공 시 소스 공개 의무)
- 다른 운영자가 fork·확장 자유

## repo 구조 (예정)

```
LostCHoco/PengAgent/
├── Cargo.toml                # workspace? or 단일 crate?
├── src/
│   ├── main.rs               # HTTP 서버 entry
│   ├── auth.rs               # PENGAGENT_TOKEN 검증
│   ├── endpoints/
│   │   ├── health.rs
│   │   ├── metrics.rs        # node_exporter proxy
│   │   ├── containers.rs     # docker-socket-proxy proxy
│   │   ├── logs.rs           # SSE stream
│   │   ├── compose.rs        # git pull + compose up
│   │   └── audit.rs          # SQLite read
│   ├── audit.rs              # write to SQLite
│   └── config.rs             # 환경변수 파싱
├── Dockerfile
├── docker-compose.example.yml  # 운영자 통합 가이드 예
├── .github/workflows/
│   └── release.yml           # GHCR multi-arch build
├── README.md                 # 어떤 시나리오에 쓸 수 있는지, 환경변수 카탈로그
├── docs/spec/
│   └── peng_agent_protocol_v1.md   # protocol 명세 본체
├── LICENSE                   # AGPL-3.0-only
└── CLAUDE.md (gitignored)    # 작업 가이드
```

명세 본체(`peng_agent_protocol_v1.md`)는 repo의 `docs/spec/`에 git tracked (PengPort 본가의 글로벌 정책 "docs/ gitignored"와 달리, PengAgent는 protocol 공개가 본질이라 spec은 commit. 분실 위험 방지).

## 본질 표현 일관성

본질 한 줄("docker-compose + git-managed self-host 환경의 운영을 위한 generic admin agent")이 4곳 일치해야:

1. `PengAgent/docs/spec/00-essence.md` § 1
2. `PengAgent/CLAUDE.md` 첫 줄
3. 이 memory 파일 description + 첫 줄
4. PengAgent 자체 MEMORY.md (repo 신설 시) 인덱스 한 줄 요약

본가 PengPort의 `MEMORY.md`(이 파일이 있는 곳)에서도 PengAgent 본질 한 줄 짧게 언급 (cross-reference).

## 참고

- 명세: `peng_agent_protocol_v1.md`
- PengPort-admin 본질 (client 측): `admin_essence.md`
- PengPort family 본가: `project_overview.md`, `core_principles.md`
