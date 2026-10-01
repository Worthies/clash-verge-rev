//! Write tools that change the active proxy selection, mode, or core state.
//!
//! All gated by [`Tools::require_mutations_allowed`].

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool,
};
use serde::Deserialize;

use super::{Tools, anyhow_to_mcp};
use crate::core::{CoreManager, handle::Handle};

fn text_result(message: impl Into<std::string::String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(message.into())])
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct SelectProxyRequest {
    /// Name of the proxy group (e.g. "Proxy", "GLOBAL")
    pub group: String,
    /// Name of the proxy node to select inside that group
    pub node: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct ChangeModeRequest {
    /// Target proxy mode
    #[schemars(regex(pattern = r"^(rule|global|direct|script)$"))]
    pub mode: String,
}

#[rmcp::tool_router(router = proxy_tool_router, vis = "pub(crate)")]
impl Tools {
    #[tool(description = "Select a proxy node inside a proxy group. Requires MCP mutations to be enabled")]
    pub(crate) async fn select_proxy_node(
        &self,
        Parameters(SelectProxyRequest { group, node }): Parameters<SelectProxyRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        crate::feat::switch_proxy_node(&group, &node).await;
        Ok(text_result(format!("Selected '{node}' for group '{group}'")))
    }

    #[tool(description = "Switch the proxy mode: rule, global, direct or script. Requires MCP mutations to be enabled")]
    pub(crate) async fn change_proxy_mode(
        &self,
        Parameters(ChangeModeRequest { mode }): Parameters<ChangeModeRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        crate::feat::change_clash_mode(mode.clone().into())
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        Ok(text_result(format!("Proxy mode changed to '{mode}'")))
    }

    #[tool(description = "Restart the clash/mihomo core process. Requires MCP mutations to be enabled")]
    pub(crate) async fn restart_core(&self) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        CoreManager::global().restart_core().await.map_err(anyhow_to_mcp)?;
        Handle::refresh_clash();
        Ok(text_result("Core restarted"))
    }
}
