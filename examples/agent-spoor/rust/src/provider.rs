//! 能力供给层。三种接入模式各实现一份，agent 主循环只依赖它 ——
//! "内核不变，只换能力从哪来"。

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ToolProvider: Send {
    /// 给模型的工具 schema（OpenAI function 格式）。
    async fn list_tools(&self) -> Result<Vec<Value>>;
    /// 按名字执行一次工具调用，返回给模型看的文本。
    async fn execute(&self, name: &str, args: Value) -> String;
    /// 工具"跑在哪"的短标签，仅用于日志，让 native/mcp/skill 的差别可见。
    fn transport(&self) -> Option<String> {
        None
    }
    /// 往系统提示追加内容（skill 模式注入技能目录）。
    fn system_addendum(&self) -> Option<String> {
        None
    }
    /// 清理（mcp 模式断开子进程）。
    async fn close(&mut self) {}
}
