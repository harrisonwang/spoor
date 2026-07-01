//! agent 自带的基础工具：读纯文本/代码文件（与 spoor 的 read_document 形成对照）。

use serde_json::{Value, json};

use crate::validate::{require_str, safe_resolve};

pub fn read_file_spec() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "从当前项目读取一个文本文件，返回带行号的内容。二进制文档（PDF/Word/Excel…）请用 read_document。",
            "parameters": {
                "type": "object",
                "properties": {
                    "file_path": {"type": "string", "description": "相对项目根目录的文件路径，如 src/main.rs"}
                },
                "required": ["file_path"]
            }
        }
    })
}

pub fn read_file(args: &Value) -> String {
    let path = match require_str(args, "file_path") {
        Ok(p) => p,
        Err(e) => return e.to_string(),
    };
    let abs = match safe_resolve(path) {
        Ok(p) => p,
        Err(e) => return e.to_string(),
    };
    match std::fs::read_to_string(&abs) {
        Ok(content) => content
            .split('\n')
            .enumerate()
            .map(|(i, line)| format!("{:>4}: {line}", i + 1))
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => format!("读取文件失败: {e}"),
    }
}
