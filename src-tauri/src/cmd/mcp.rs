/// Whether the MCP server is currently listening, for the settings UI to show live status.
#[tauri::command]
pub fn get_mcp_server_running() -> bool {
    crate::mcp::mcp_server_running()
}
