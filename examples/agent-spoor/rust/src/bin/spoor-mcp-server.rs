//! 独立的 spoor MCP server 二进制（stdio）。也可直接配进 Claude Desktop / Cursor。

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    agent_spoor::mcp_server::run().await
}
