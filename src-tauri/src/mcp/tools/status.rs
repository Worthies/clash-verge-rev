//! Read-only status/inspection tools. Always available, never gated by the mutation flag.

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool,
};
use serde::Deserialize;
use serde_json::json;

use super::{Tools, anyhow_to_mcp};
use crate::{config::Config, core::CoreManager};

fn json_result(value: &serde_json::Value) -> CallToolResult {
    let text = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    CallToolResult::success(vec![ContentBlock::text(text)])
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct DelayTestRequest {
    /// URL to measure round-trip latency against (default: http://cp.cloudflare.com/generate_204)
    #[serde(default)]
    pub url: Option<String>,
}

#[rmcp::tool_router(router = status_tool_router, vis = "pub(crate)")]
impl Tools {
    #[tool(
        description = "Get Clash Verge Rev application status: current profile, system proxy, TUN, external controller and ports"
    )]
    pub(crate) async fn get_app_status(&self) -> Result<CallToolResult, McpError> {
        let verge = Config::verge().await.latest_arc();
        let clash_info = Config::clash().await.data_arc().get_client_info();
        let profiles = Config::profiles().await.latest_arc();

        Ok(json_result(&json!({
            "currentProfile": profiles.current,
            "systemProxyEnabled": verge.enable_system_proxy.unwrap_or(false),
            "tunModeEnabled": verge.enable_tun_mode.unwrap_or(false),
            "externalControllerEnabled": verge.enable_external_controller.unwrap_or(false),
            "allowMutations": verge.mcp_server_allow_mutations.unwrap_or(false),
            "mcpServerRunning": crate::mcp::mcp_server_running(),
            "controllerServer": clash_info.server,
            "mixedPort": clash_info.mixed_port,
            "socksPort": clash_info.socks_port,
            "httpPort": clash_info.port,
        })))
    }

    #[tool(description = "Get the current proxy mode (rule, global, direct or script)")]
    pub(crate) async fn get_proxy_mode(&self) -> Result<CallToolResult, McpError> {
        let mode = Config::clash().await.data_arc().get_mode();
        Ok(json_result(&json!({ "mode": mode })))
    }

    #[tool(description = "List all proxy groups and nodes, including which node is currently selected in each group")]
    pub(crate) async fn list_proxies(&self) -> Result<CallToolResult, McpError> {
        let view = crate::cmd::proxy_view(None)
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        let value = serde_json::to_value(view).map_err(super::serde_to_mcp)?;
        Ok(json_result(&value))
    }

    #[tool(description = "List all configured profiles/subscriptions and which one is active")]
    pub(crate) async fn list_profiles(&self) -> Result<CallToolResult, McpError> {
        let profiles = Config::profiles().await.data_arc();
        let value = serde_json::to_value(&*profiles).map_err(super::serde_to_mcp)?;
        Ok(json_result(&value))
    }

    #[tool(description = "Get recent core (mihomo) log lines")]
    pub(crate) async fn get_core_logs(&self) -> Result<CallToolResult, McpError> {
        let logs = CoreManager::global().get_clash_logs().await.map_err(anyhow_to_mcp)?;
        Ok(json_result(&json!({ "logs": logs })))
    }

    #[tool(
        description = "Test round-trip delay to a URL through the current proxy (or directly, if no proxy is enabled)"
    )]
    pub(crate) async fn test_delay(
        &self,
        Parameters(DelayTestRequest { url }): Parameters<DelayTestRequest>,
    ) -> Result<CallToolResult, McpError> {
        let url = url.unwrap_or_else(|| "http://cp.cloudflare.com/generate_204".to_owned());
        let millis = crate::feat::test_delay(url.clone().into())
            .await
            .map_err(anyhow_to_mcp)?;
        Ok(json_result(&json!({ "url": url, "delayMs": millis })))
    }
}
