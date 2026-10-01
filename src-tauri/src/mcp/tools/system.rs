//! Write tools for system-level toggles (system proxy, TUN mode).

use rmcp::{
    ErrorData as McpError,
    model::{CallToolResult, ContentBlock},
    tool,
};

use super::Tools;

fn text_result(message: impl Into<std::string::String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(message.into())])
}

#[rmcp::tool_router(router = system_tool_router, vis = "pub(crate)")]
impl Tools {
    #[tool(description = "Toggle the OS-level system proxy on or off. Requires MCP mutations to be enabled")]
    pub(crate) async fn toggle_system_proxy(&self) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        match crate::feat::toggle_system_proxy().await {
            Some(enabled) => Ok(text_result(format!(
                "System proxy is now {}",
                if enabled { "on" } else { "off" }
            ))),
            None => Err(McpError::internal_error("failed to toggle system proxy", None)),
        }
    }

    #[tool(description = "Toggle TUN mode on or off. Requires MCP mutations to be enabled")]
    pub(crate) async fn toggle_tun_mode(&self) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        let enabled = crate::feat::toggle_tun_mode(None).await;
        Ok(text_result(format!(
            "TUN mode is now {}",
            if enabled { "on" } else { "off" }
        )))
    }
}
