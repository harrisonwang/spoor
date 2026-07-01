//! 模式②MCP：agent 当 MCP client，把 spoor MCP server 的工具桥接进主循环。
//! 松耦合、标准协议：同一个 server 也能插进 Claude Desktop / Cursor。

use anyhow::{Context, Result};
use async_trait::async_trait;
use rmcp::model::CallToolRequestParam;
use rmcp::service::{RoleClient, RunningService, ServiceExt};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use serde_json::{Value, json};
use tokio::process::Command;

use crate::provider::ToolProvider;
use crate::tools_base::{read_file, read_file_spec};

pub struct McpProvider {
    service: Option<RunningService<RoleClient, ()>>,
    tools: Vec<Value>, // 桥接后的 OpenAI 工具 schema
}

impl McpProvider {
    pub async fn start() -> Result<Self> {
        // 以子进程拉起同目录的 spoor-mcp-server 二进制。
        let server_bin = std::env::current_exe()?
            .parent()
            .context("无法定位可执行文件目录")?
            .join("spoor-mcp-server");
        let cwd = std::env::current_dir()?;
        let service = ()
            .serve(TokioChildProcess::new(Command::new(server_bin).configure(
                |cmd| {
                    cmd.current_dir(cwd);
                },
            ))?)
            .await?;

        let list = service.list_tools(Default::default()).await?;
        let mut tools = vec![read_file_spec()]; // agent 自带的 read_file 仍在本地
        for t in list.tools {
            tools.push(json!({
                "type": "function",
                "function": {
                    "name": t.name.to_string(),
                    "description": t.description.map(|d| d.to_string()).unwrap_or_default(),
                    "parameters": Value::Object((*t.input_schema).clone()),
                }
            }));
        }
        Ok(Self {
            service: Some(service),
            tools,
        })
    }
}

#[async_trait]
impl ToolProvider for McpProvider {
    async fn list_tools(&self) -> Result<Vec<Value>> {
        Ok(self.tools.clone())
    }

    async fn execute(&self, name: &str, args: Value) -> String {
        if name == "read_file" {
            return read_file(&args);
        }
        let Some(service) = self.service.as_ref() else {
            return "MCP 会话已关闭".to_string();
        };
        let arguments = args.as_object().cloned();
        match service
            .call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments,
            })
            .await
        {
            Ok(result) => result
                .content
                .into_iter()
                .filter_map(|c| c.as_text().map(|t| t.text.clone()))
                .collect::<Vec<_>>()
                .join("\n"),
            Err(e) => format!("MCP 工具 {name} 调用失败: {e}"),
        }
    }

    fn transport(&self) -> Option<String> {
        Some("MCP·独立 server 子进程（stdio 往返；真实 pid 见 [spoor-mcp] 日志）".to_string())
    }

    async fn close(&mut self) {
        if let Some(service) = self.service.take() {
            let _ = service.cancel().await;
        }
    }
}
