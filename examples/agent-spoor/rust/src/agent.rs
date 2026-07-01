//! Agent 主循环。只依赖 ToolProvider —— 三种模式共用。

use anyhow::Result;
use serde_json::{Value, json};

use crate::model::Model;
use crate::provider::ToolProvider;

const SYSTEM_PROMPT: &str = "你是一个能读本地文档的智能体，运行在当前项目目录中。\n\
- 查看纯文本/代码文件用 read_file；读取 PDF/Word/Excel/PPT/EPUB/网页等文档时，用文档工具或技能，不要臆测内容。\n\
- 文档解析可能带完整性 warnings（如某页是扫描件、无文本层），要如实转达用户。\n\
- 回答简洁、准确；涉及具体数字时尽量给出处（页码）。";

pub struct Agent {
    model: Model,
    provider: Box<dyn ToolProvider>,
    messages: Vec<Value>,
    tools: Option<Vec<Value>>,
}

impl Agent {
    pub fn new(model: Model, provider: Box<dyn ToolProvider>) -> Self {
        let content = match provider.system_addendum() {
            Some(add) => format!("{SYSTEM_PROMPT}\n\n{add}"),
            None => SYSTEM_PROMPT.to_string(),
        };
        Self {
            model,
            provider,
            messages: vec![json!({"role": "system", "content": content})],
            tools: None,
        }
    }

    pub async fn chat(&mut self, user: &str) -> Result<String> {
        if self.tools.is_none() {
            self.tools = Some(self.provider.list_tools().await?);
        }
        self.messages.push(json!({"role": "user", "content": user}));

        loop {
            let tools = self.tools.clone().unwrap_or_default();
            let (message, usage) = self.model.call(&self.messages, &tools).await?;
            self.messages.push(message.clone());

            let tool_calls = message
                .get("tool_calls")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();

            if tool_calls.is_empty() {
                if let Some((input, output)) = usage {
                    println!("[tokens] 输入: {input}, 输出: {output}");
                }
                return Ok(message
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string());
            }

            for call in tool_calls {
                if call.get("type").and_then(Value::as_str) != Some("function") {
                    continue;
                }
                let name = call["function"]["name"].as_str().unwrap_or("").to_string();
                let args_str = call["function"]["arguments"].as_str().unwrap_or("{}");
                let args: Value = serde_json::from_str(args_str).unwrap_or_else(|_| json!({}));

                let via = self
                    .provider
                    .transport()
                    .map(|t| format!("  ⟨跑在: {t}⟩"))
                    .unwrap_or_default();
                println!("\n🔧 调用工具: {name}{via}");
                println!("  参数: {args}");

                let result = self.provider.execute(&name, args).await;
                let preview = if result.chars().count() > 300 {
                    format!("{}\n...", result.chars().take(300).collect::<String>())
                } else {
                    result.clone()
                };
                println!("  结果: {preview}");

                let id = call.get("id").and_then(Value::as_str).unwrap_or("");
                self.messages
                    .push(json!({"role": "tool", "tool_call_id": id, "content": result}));
            }
        }
    }

    pub async fn close(&mut self) {
        self.provider.close().await;
    }
}
