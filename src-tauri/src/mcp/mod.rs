//! Built-in MCP (Model Context Protocol) server.
//!
//! Lets AI agents inspect and, when explicitly allowed, control Clash Verge over a local
//! Streamable HTTP endpoint. Disabled by default; bound to 127.0.0.1 only; requires a Bearer
//! token; and gates any state-changing tool behind a separate opt-in flag.

mod auth;
mod handler;
mod server;
mod tools;

pub use server::{mcp_server_running, restart_mcp_server_for_config_change, start_mcp_server, stop_mcp_server};
