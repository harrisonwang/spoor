//! 无 LLM 冒烟：验证三种 provider 的执行路径（spoor-core / CLI / MCP 往返）。

use agent_spoor::provider::ToolProvider;
use agent_spoor::providers::native::NativeProvider;
use agent_spoor::providers::skill::SkillProvider;
use rmcp::model::CallToolRequestParam;
use rmcp::service::ServiceExt;
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use serde_json::json;
use tokio::process::Command;

#[tokio::test]
async fn native_reads_table() {
    let provider = NativeProvider::new();
    let out = provider
        .execute(
            "read_document",
            json!({"path": "data/sales.csv", "limit": 3}),
        )
        .await;
    assert!(out.contains("分类"), "native read_document 输出异常: {out}");
    assert!(provider.transport().unwrap().contains("同进程"));
}

#[tokio::test]
async fn skill_runs_cli() {
    let provider = SkillProvider::new();
    let out = provider
        .execute("run_shell", json!({"command": "spoor data/sales.csv"}))
        .await;
    assert!(
        out.contains("schema_version") || out.contains("分类"),
        "skill run_shell 输出异常: {out}"
    );
    let catalog = provider.execute("list_skills", json!({})).await;
    assert!(catalog.contains("spoor"), "list_skills 输出异常: {catalog}");
}

#[tokio::test]
async fn mcp_roundtrip() {
    let server = env!("CARGO_BIN_EXE_spoor-mcp-server");
    let service = ()
        .serve(
            TokioChildProcess::new(Command::new(server).configure(|c| {
                c.current_dir(env!("CARGO_MANIFEST_DIR"));
            }))
            .unwrap(),
        )
        .await
        .unwrap();

    let tools = service.list_tools(Default::default()).await.unwrap();
    let names: Vec<String> = tools.tools.iter().map(|t| t.name.to_string()).collect();
    assert!(
        names.iter().any(|n| n == "read_document"),
        "MCP 工具缺失: {names:?}"
    );

    let result = service
        .call_tool(CallToolRequestParam {
            name: "read_document".into(),
            arguments: json!({"path": "data/sales.csv", "limit": 1})
                .as_object()
                .cloned(),
        })
        .await
        .unwrap();
    let text: String = result
        .content
        .into_iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect();
    assert!(
        text.contains("分类") || text.contains("format"),
        "MCP read_document 结果异常: {text}"
    );

    service.cancel().await.unwrap();
}
