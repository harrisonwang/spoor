//! 模式②的服务端：一个独立的 spoor MCP Server（stdio）。
//! 把它配进 Claude Desktop / Cursor，那些 agent 也立刻能读本地文档。
//! 引擎用 spoor-core（同进程），但对 MCP 客户端透明。

use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParam, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ErrorData as McpError, ServiceExt};
use serde_json::Value;

use crate::spoor_tools::{run_spoor_tool, spoor_tool_specs};

#[derive(Clone)]
pub struct SpoorServer;

impl ServerHandler for SpoorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "spoor".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            },
            instructions: Some(
                "spoor 文档解析：read_document / extract_document_image".to_string(),
            ),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _req: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = spoor_tool_specs()
            .into_iter()
            .filter_map(|spec| {
                let f = spec.get("function")?;
                let name = f.get("name")?.as_str()?.to_string();
                let desc = f.get("description")?.as_str()?.to_string();
                let params = f.get("parameters")?.as_object()?.clone();
                Some(Tool::new(name, desc, Arc::new(params)))
            })
            .collect();
        Ok(ListToolsResult {
            tools,
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        req: CallToolRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = req.name.to_string();
        let args = Value::Object(req.arguments.unwrap_or_default());
        // 日志走 stderr（stdout 归协议）：在 MCP 模式下亲眼看到"独立 server 进程"收到调用。
        eprintln!(
            "[spoor-mcp pid={}] ← 调用 {name} {args}",
            std::process::id()
        );
        let text = tokio::task::spawn_blocking(move || run_spoor_tool(&name, &args))
            .await
            .unwrap_or_else(|e| format!("panic: {e}"));
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

pub async fn run() -> anyhow::Result<()> {
    eprintln!(
        "[spoor-mcp pid={}] server ready on stdio",
        std::process::id()
    );
    let service = SpoorServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
