//! 模式①原生工具：把 spoor 直接写成 Tool，spoor-core 同进程调用（源头，最原生）。

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::provider::ToolProvider;
use crate::spoor_tools::{run_spoor_tool, spoor_tool_specs};
use crate::tools_base::{read_file, read_file_spec};

pub struct NativeProvider {
    transport: String,
}

impl NativeProvider {
    pub fn new() -> Self {
        Self {
            transport: format!("原生·同进程 spoor-core (pid={})", std::process::id()),
        }
    }
}

impl Default for NativeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolProvider for NativeProvider {
    async fn list_tools(&self) -> Result<Vec<Value>> {
        let mut tools = vec![read_file_spec()];
        tools.extend(spoor_tool_specs());
        Ok(tools)
    }

    async fn execute(&self, name: &str, args: Value) -> String {
        if name == "read_file" {
            return read_file(&args);
        }
        // spoor-core 是同步 CPU 工作，丢到阻塞线程池，避免卡住事件循环。
        let name = name.to_string();
        tokio::task::spawn_blocking(move || run_spoor_tool(&name, &args))
            .await
            .unwrap_or_else(|e| format!("工具执行 panic: {e}"))
    }

    fn transport(&self) -> Option<String> {
        Some(self.transport.clone())
    }
}
