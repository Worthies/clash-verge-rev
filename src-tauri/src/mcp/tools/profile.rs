//! Write tools for managing subscription profiles. Delegates to the same `cmd::profile` entry
//! points the frontend uses, so validation, locking, and notifications stay identical.

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool,
};
use serde::Deserialize;

use super::Tools;
use crate::cmd::CommandFailure;

fn text_result(message: impl Into<std::string::String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(message.into())])
}

fn command_failure_to_mcp(failure: CommandFailure) -> McpError {
    McpError::internal_error(failure.to_string(), None)
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct ImportProfileRequest {
    /// Subscription URL to import as a new profile
    pub url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub(super) struct ProfileUidRequest {
    /// UID of the profile, as returned by `list_profiles`
    pub uid: String,
}

#[rmcp::tool_router(router = profile_tool_router, vis = "pub(crate)")]
impl Tools {
    #[tool(description = "Import a new profile/subscription from a URL. Requires MCP mutations to be enabled")]
    pub(crate) async fn import_profile(
        &self,
        Parameters(ImportProfileRequest { url }): Parameters<ImportProfileRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        crate::cmd::import_profile(url.clone(), None)
            .await
            .map_err(command_failure_to_mcp)?;
        Ok(text_result(format!("Profile imported from '{url}'")))
    }

    #[tool(description = "Refresh/update a profile from its subscription source. Requires MCP mutations to be enabled")]
    pub(crate) async fn update_profile(
        &self,
        Parameters(ProfileUidRequest { uid }): Parameters<ProfileUidRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        crate::cmd::update_profile(uid.clone().into(), None)
            .await
            .map_err(command_failure_to_mcp)?;
        Ok(text_result(format!("Profile '{uid}' updated")))
    }

    #[tool(description = "Switch the active profile by UID. Requires MCP mutations to be enabled")]
    pub(crate) async fn switch_profile(
        &self,
        Parameters(ProfileUidRequest { uid }): Parameters<ProfileUidRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        let outcome = crate::cmd::patch_profiles_config_by_profile_index(uid.clone().into())
            .await
            .map_err(command_failure_to_mcp)?;
        if outcome.is_valid() {
            Ok(text_result(format!("Switched to profile '{uid}'")))
        } else {
            Err(McpError::internal_error(
                format!("profile switch validation failed: {outcome}"),
                None,
            ))
        }
    }

    #[tool(description = "Re-apply (enhance) the current profile configuration. Requires MCP mutations to be enabled")]
    pub(crate) async fn reapply_profile(&self) -> Result<CallToolResult, McpError> {
        self.require_mutations_allowed().await?;
        let outcome = crate::cmd::enhance_profiles().await.map_err(command_failure_to_mcp)?;
        if outcome.is_valid() {
            Ok(text_result("Current profile re-applied"))
        } else {
            Err(McpError::internal_error(
                format!("profile re-apply validation failed: {outcome}"),
                None,
            ))
        }
    }
}
