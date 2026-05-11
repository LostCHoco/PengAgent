//! PengAgent Protocol v1 endpoints.
//! 명세: `docs/spec/peng_agent_protocol_v1.md`

pub mod health;

// 미래 endpoint (v1 spec 정의됨):
// pub mod metrics;     // GET /v1/metrics/host[/json]
// pub mod containers;  // GET /v1/containers, POST .../restart
// pub mod logs;        // GET /v1/logs/<c>, /stream
// pub mod compose;     // POST /v1/compose/reload
// pub mod audit;       // GET /v1/audit
