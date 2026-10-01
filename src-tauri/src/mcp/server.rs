//! Lifecycle (start/stop) and axum wiring for the MCP Streamable HTTP endpoint.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::{Context as _, Result};
use axum::{Router, middleware};
use clash_verge_logging::{Type, logging};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

use super::{auth::AuthState, tools::Tools};
use crate::config::{Config, IVerge};

static MCP_RUNNING: AtomicBool = AtomicBool::new(false);
static MCP_SHUTDOWN: OnceCell<Mutex<Option<CancellationToken>>> = OnceCell::new();

pub fn mcp_server_running() -> bool {
    MCP_RUNNING.load(Ordering::Acquire)
}

fn random_token() -> Result<std::string::String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).context("failed to generate MCP server token")?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Returns the configured MCP token, generating and persisting one on first use.
async fn resolve_token(verge: &IVerge) -> Result<std::string::String> {
    if let Some(token) = verge.mcp_server_token.clone() {
        return Ok(token.into());
    }

    let token = random_token()?;
    crate::feat::patch_verge(
        &IVerge {
            mcp_server_token: Some(token.clone().into()),
            ..IVerge::default()
        },
        false,
    )
    .await
    .context("failed to persist generated MCP server token")?;
    Ok(token)
}

/// Starts the MCP server if `enable_mcp_server` is set. No-op if already running or disabled.
pub fn start_mcp_server() {
    crate::process::AsyncHandler::spawn(|| async {
        if let Err(error) = try_start_mcp_server().await {
            logging!(error, Type::Mcp, "failed to start MCP server: {error:#}");
        }
    });
}

async fn try_start_mcp_server() -> Result<()> {
    if MCP_RUNNING.load(Ordering::Acquire) {
        return Ok(());
    }

    let verge = Config::verge().await.latest_arc();
    if !verge.enable_mcp_server.unwrap_or(false) {
        logging!(debug, Type::Mcp, "MCP server disabled, not starting");
        return Ok(());
    }

    let port = verge
        .mcp_server_port
        .unwrap_or(crate::constants::network::ports::DEFAULT_MCP_SERVER);
    let token = resolve_token(&verge).await?;
    drop(verge);

    let cancellation_token = CancellationToken::new();
    MCP_SHUTDOWN.get_or_init(|| Mutex::new(None));
    if let Some(cell) = MCP_SHUTDOWN.get() {
        *cell.lock() = Some(cancellation_token.clone());
    }

    let service = StreamableHttpService::new(
        || Ok(Tools),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().with_cancellation_token(cancellation_token.child_token()),
    );

    let auth_state = AuthState { token };
    let router = Router::new()
        .nest_service("/mcp", service)
        .route_layer(middleware::from_fn_with_state(
            auth_state,
            super::auth::require_bearer_token,
        ));

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .with_context(|| format!("failed to bind MCP server to 127.0.0.1:{port}"))?;
    let bound_port = listener
        .local_addr()
        .context("failed to read MCP server address")?
        .port();

    logging!(info, Type::Mcp, "MCP server listening on 127.0.0.1:{bound_port}");
    MCP_RUNNING.store(true, Ordering::Release);

    crate::process::AsyncHandler::spawn(move || async move {
        let shutdown = cancellation_token.clone();
        let serve_result = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
            })
            .await;
        MCP_RUNNING.store(false, Ordering::Release);
        if let Err(error) = serve_result {
            logging!(error, Type::Mcp, "MCP server stopped unexpectedly: {error}");
        } else {
            logging!(info, Type::Mcp, "MCP server stopped");
        }
    });

    Ok(())
}

/// Stops the MCP server if running. Safe to call even if it was never started.
pub fn stop_mcp_server() {
    let Some(cell) = MCP_SHUTDOWN.get() else {
        return;
    };
    let token = cell.lock().take();
    if let Some(token) = token {
        token.cancel();
    }
}

/// Restarts the server to pick up a config change (enable flag, port, token, …).
pub async fn restart_mcp_server_for_config_change() {
    stop_mcp_server();
    // Give the previous listener a moment to release the port before rebinding.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    start_mcp_server();
}
