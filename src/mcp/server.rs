use std::path::{Path, PathBuf};

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
use crate::config;

use super::tools::{self, DocsDirParam, FilePathParam};

pub struct DocsGateServer {
    config_path: Option<PathBuf>,
    tool_router: ToolRouter<Self>,
}

impl DocsGateServer {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let tool_router = Self::tool_router();
        Self {
            config_path,
            tool_router,
        }
    }

    /// Load fresh config from disk on each tool call so MCP server reflects edits to
    /// `.docs-gate.toml` without needing a restart.
    fn load_fresh_config(&self) -> config::Config {
        config::load_config(self.config_path.as_deref())
    }
}

#[tool_router]
impl DocsGateServer {
    /// Check CHANGELOG.md has a recent entry
    #[tool(name = "check_changelog")]
    fn check_changelog(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(self.load_fresh_config(), params.docs_dir);
        let results = vec![checks::changelog::check_changelog(&config)];
        serde_json::to_string_pretty(&results).unwrap()
    }

    /// Check ARCHITECTURE.md sections and required non-empty content
    #[tool(name = "check_architecture")]
    fn check_architecture(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(self.load_fresh_config(), params.docs_dir);
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

    /// Check git staged files: CHANGELOG must be staged with code changes, and file-to-docs rules must be satisfied
    #[tool(name = "check_staged")]
    fn check_staged(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(self.load_fresh_config(), params.docs_dir);
        let mut results = vec![checks::staged::check_changelog_staged(&config)];
        results.extend(checks::staged::check_rules(&config));
        serde_json::to_string_pretty(&results).unwrap()
    }

    /// Run all checks: changelog + architecture + staged + rules + tickets. Use this before every commit.
    #[tool(name = "check_all")]
    fn check_all(&self, Parameters(params): Parameters<DocsDirParam>) -> String {
        let config = tools::resolve_config(self.load_fresh_config(), params.docs_dir);
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
    use crate::config::Config;

    #[test]
    fn test_server_info() {
        let server = DocsGateServer::new(None);
        let info = rmcp::ServerHandler::get_info(&server);
        assert_eq!(info.server_info.name, "docs-gate");
        assert!(!info.server_info.version.is_empty());
    }

    #[test]
    fn test_tool_router_has_5_tools() {
        let server = DocsGateServer::new(None);
        let tools = server.tool_router.list_all();
        let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
        assert!(names.contains(&"check_changelog".to_string()));
        assert!(names.contains(&"check_architecture".to_string()));
        assert!(names.contains(&"check_discovery".to_string()));
        assert!(names.contains(&"check_staged".to_string()));
        assert!(names.contains(&"check_all".to_string()));
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_resolve_config_override() {
        let resolved = tools::resolve_config(Config::default(), Some("/custom/docs".to_string()));
        assert_eq!(resolved.docs_dir, std::path::PathBuf::from("/custom/docs"));
    }

    #[test]
    fn test_resolve_config_no_override() {
        let default_dir = Config::default().docs_dir;
        let resolved = tools::resolve_config(Config::default(), None);
        assert_eq!(resolved.docs_dir, default_dir);
    }

    #[test]
    fn test_load_fresh_config_no_path_returns_default() {
        let server = DocsGateServer::new(None);
        let config = server.load_fresh_config();
        // Without an explicit path, load_config falls back to ".docs-gate.toml" in CWD;
        // when that file doesn't exist or fails, we get Config::default(). Either way,
        // calling load_fresh_config must not panic and must return a usable Config.
        assert!(!config.changelog.is_empty());
    }

    #[test]
    fn test_load_fresh_config_picks_up_edits() {
        // Verify the core hot-reload contract: editing the config file between calls
        // must change what the server sees on the next call.
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".docs-gate.toml");

        std::fs::write(&config_path, "docs_dir = \"docs-v1\"\n").unwrap();
        let server = DocsGateServer::new(Some(config_path.clone()));
        let first = server.load_fresh_config();
        assert_eq!(first.docs_dir, std::path::PathBuf::from("docs-v1"));

        std::fs::write(&config_path, "docs_dir = \"docs-v2\"\n").unwrap();
        let second = server.load_fresh_config();
        assert_eq!(second.docs_dir, std::path::PathBuf::from("docs-v2"));
    }
}
