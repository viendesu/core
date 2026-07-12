//! MCP (Model Context Protocol) server for viendesu.
//!
//! Exposes service domains as MCP tools over the streamable HTTP
//! transport, mountable into any axum router:
//!
//! ```ignore
//! let mcp = viendesu_mcp::router(service.clone(), viendesu_mcp::catalog::read_only());
//! let router = viendesu_http::server::make_router::<T>(service).nest("/mcp", mcp);
//! ```
//!
//! Tool input/output schemas are derived from the protocol types via
//! `schemars`, so the tool surface always matches the HTTP API. Callers
//! authenticate the same way: `Authorization: Bearer <session token>`.

pub mod catalog;
pub mod registry;
pub mod rpc;

pub use registry::Tools;
pub use rpc::router;
