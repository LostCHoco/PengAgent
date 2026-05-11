# PengAgent — multi-stage build (rust:alpine → alpine runtime)
# AGPL-3.0-only — source available, LICENSE included in image.

# ─── Stage 1: build ────────────────────────────────────────────────
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig

WORKDIR /build

# 의존성 빌드 캐시 (dummy src로 deps만 컴파일)
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release --bin peng-agent && \
    rm -rf src target/release/peng-agent target/release/peng-agent.d \
           target/release/deps/peng_agent* target/release/deps/libpeng_agent*

# 실제 소스 빌드
COPY src ./src
RUN touch src/main.rs && cargo build --release --bin peng-agent && \
    strip target/release/peng-agent || true

# ─── Stage 2: runtime ──────────────────────────────────────────────
FROM alpine:3.20

# docker-cli + compose plugin: agent가 `docker compose -f <file> up -d` 호출.
# git: ops repo pull.
# ca-certificates: HTTPS (node_exporter·docker proxy가 TLS인 경우).
# wget: HEALTHCHECK용.
RUN apk add --no-cache \
        docker-cli \
        docker-cli-compose \
        git \
        ca-certificates \
        wget \
    && addgroup -S pengagent \
    && adduser -S -G pengagent -u 1001 pengagent \
    && mkdir -p /var/lib/pengagent \
    && chown -R pengagent:pengagent /var/lib/pengagent

COPY --from=builder /build/target/release/peng-agent /usr/local/bin/peng-agent
COPY LICENSE /LICENSE
COPY README.md /README.md

# 운영자가 docker.sock 마운트하는 경우 docker group GID 조정 필요할 수 있음.
# 보안 권장: docker-socket-proxy 통해서만 접근 (peng-agent는 unprivileged).
USER pengagent

EXPOSE 9000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q -O- http://localhost:9000/v1/health || exit 1

ENTRYPOINT ["/usr/local/bin/peng-agent"]
