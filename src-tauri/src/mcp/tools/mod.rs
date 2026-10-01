mod profile;
mod proxy;
mod status;
mod system;

use rmcp::model::ErrorData as McpError;

use crate::config::Config;

/// Shared state for every MCP tool call.
#[derive(Clone, Default)]
pub(super) struct Tools;

impl Tools {
    /// Rejects a write tool unless the user opted into mutations in Settings.
    async fn require_mutations_allowed(&self) -> Result<(), McpError> {
        let allowed = Config::verge()
            .await
            .latest_arc()
            .mcp_server_allow_mutations
            .unwrap_or(false);
        if allowed {
            Ok(())
        } else {
            Err(McpError::invalid_request(
                "MCP write operations are disabled. Enable \"Allow MCP mutations\" in Clash Verge settings first.",
                None,
            ))
        }
    }
}

pub(super) fn anyhow_to_mcp(error: anyhow::Error) -> McpError {
    McpError::internal_error(format!("{error:#}"), None)
}

pub(super) fn serde_to_mcp(error: serde_json::Error) -> McpError {
    McpError::invalid_params(error.to_string(), None)
}
