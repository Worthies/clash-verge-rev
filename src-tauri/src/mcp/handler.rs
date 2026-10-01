//! Wires the per-category tool routers into a single [`rmcp::ServerHandler`].

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerConfig},
};

use super::tools::Tools;

impl Tools {
    fn combined_router() -> ToolRouter<Self> {
        Self::status_tool_router()
            + Self::proxy_tool_router()
            + Self::system_tool_router()
            + Self::profile_tool_router()
    }
}

#[rmcp::tool_handler(router = Self::combined_router())]
impl ServerHandler for Tools {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_protocol_version(ProtocolVersion::V_2025_03_26)
            .with_instructions(
                "Clash Verge Rev control surface. Read-only tools are always available. \
                 Write tools (select_proxy_node, change_proxy_mode, restart_core, \
                 toggle_system_proxy, toggle_tun_mode, import_profile, update_profile, \
                 switch_profile, reapply_profile) only succeed when the user has enabled \
                 \"Allow MCP mutations\" in Clash Verge settings."
                    .to_owned(),
            )
    }
}
