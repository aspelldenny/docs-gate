use std::path::Path;

use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Implementation, ListToolsResult, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::service::RoleServer;
use rmcp::{tool, tool_router};

use crate::checks;
use crate::config::Config;

use super::tools::{self, DocsDirParam, FilePathParam};

#[allow(dead_code)]
pub struct DocsGateServer {
    config: Config,
    tool_router: ToolRouter<Self>,
}

impl DocsGateServer {
    pub fn new(config: Config) -> Self {
        let tool_router = Self::tool_router();
        Self {
            config,
            tool_router,
        }
    }
}

#[tool_router]
impl DocsGateServer {
    /// Check CHANGELOG.md has a recent entry
    #[tool(name = "check_changelog")]
    fn check_changelog(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(&self.config, params.docs_dir);
        let results = vec![checks::changelog::check_changelog(&config)];
        serde_json::to_string_pretty(&results).unwrap()
    }

    /// Check ARCHITECTURE.md sections and required non-empty content
    #[tool(name = "check_architecture")]
    fn check_architecture(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(&self.config, params.docs_dir);
        let results = checks::architecture::check_architecture(&config);
        serde_json::to_string_pretty(&results).unwrap()
    }

    /// Check Discovery Report format in a specific file
    #[tool(name = "check_discovery")]
    fn check_discovery(&self, Parameters(params): Parameters<FilePathParam>) -> String {
        let path = Path::new(&params.file_path);
        let results = checks::discovery::check_discovery(path);
        serde_json::to_string_pretty(&results).unwrap()
    }

    /// Run all checks: changelog + architecture + tickets. Use this before every commit.
    #[tool(name = "check_all")]
    fn check_all(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(&self.config, params.docs_dir);
        let results = checks::run_all_checks_extended(&config);
        serde_json::to_string_pretty(&results).unwrap()
    }
}

impl rmcp::ServerHandler for DocsGateServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(String::from(
                "Use docs-gate to check documentation compliance before committing code. \
                 Run `check_all` after updating docs and BEFORE every git commit. \
                 If any check fails, fix the docs first, re-run until all checks pass, \
                 then proceed with the commit. Available checks: changelog (recent entry), \
                 architecture (required sections), tickets (type declarations), \
                 discovery (report format).",
            )),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        let tools = self.tool_router.list_all();
        std::future::ready(Ok(ListToolsResult {
            tools,
            next_cursor: None,
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        let tool_context = ToolCallContext::new(self, request, context);
        async move {
            self.tool_router
                .call(tool_context)
                .await
                .map_err(|e| rmcp::ErrorData::new(e.code, e.message, e.data))
        }
    }
}

use std::future::Future;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let server = DocsGateServer::new(Config::default());
        let info = rmcp::ServerHandler::get_info(&server);
        assert_eq!(info.server_info.name, "docs-gate");
        assert!(!info.server_info.version.is_empty());
    }

    #[test]
    fn test_tool_router_has_4_tools() {
        let server = DocsGateServer::new(Config::default());
        let tools = server.tool_router.list_all();
        let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
        assert!(names.contains(&"check_changelog".to_string()));
        assert!(names.contains(&"check_architecture".to_string()));
        assert!(names.contains(&"check_discovery".to_string()));
        assert!(names.contains(&"check_all".to_string()));
        assert_eq!(tools.len(), 4);
    }

    #[test]
    fn test_resolve_config_override() {
        let config = Config::default();
        let resolved = tools::resolve_config(&config, Some("/custom/docs".to_string()));
        assert_eq!(resolved.docs_dir, std::path::PathBuf::from("/custom/docs"));
    }

    #[test]
    fn test_resolve_config_no_override() {
        let config = Config::default();
        let resolved = tools::resolve_config(&config, None);
        assert_eq!(resolved.docs_dir, config.docs_dir);
    }
}
