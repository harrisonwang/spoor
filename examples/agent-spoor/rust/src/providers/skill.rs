//! 模式③Skill：不写类型化工具，丢一份 SKILL.md，用受限 run_shell 驱动 spoor CLI。
//! 最松耦合、零改 agent 逻辑；渐进式披露（系统提示只给技能目录，用到才 read_skill）。

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::provider::ToolProvider;
use crate::shell::run_shell;
use crate::tools_base::{read_file, read_file_spec};
use crate::validate::require_str;

const SKILL_SPOOR: &str = include_str!("../skills/spoor/SKILL.md");

struct SkillCard {
    name: String,
    description: String,
    body: String,
}

fn parse_skill(dir_name: &str, raw: &str) -> SkillCard {
    let mut name = dir_name.to_string();
    let mut description = String::new();
    let mut body = raw.to_string();
    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        let front = &rest[..end];
        body = rest[end + 4..].trim_start_matches('\n').to_string();
        for line in front.lines() {
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("description:") {
                description = v.trim().to_string();
            }
        }
    }
    SkillCard {
        name,
        description,
        body,
    }
}

pub struct SkillProvider {
    skills: Vec<SkillCard>,
}

impl SkillProvider {
    pub fn new() -> Self {
        Self {
            skills: vec![parse_skill("spoor", SKILL_SPOOR)],
        }
    }

    fn catalog(&self) -> String {
        if self.skills.is_empty() {
            "（无可用技能）".to_string()
        } else {
            self.skills
                .iter()
                .map(|s| format!("- {}: {}", s.name, s.description))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

impl Default for SkillProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolProvider for SkillProvider {
    async fn list_tools(&self) -> Result<Vec<Value>> {
        Ok(vec![
            read_file_spec(),
            json!({"type":"function","function":{"name":"list_skills","description":"列出可用技能（名字 + 简介）。","parameters":{"type":"object","properties":{}}}}),
            json!({"type":"function","function":{"name":"read_skill","description":"读取某个技能的完整说明（SKILL.md 正文），据此决定怎么用 run_shell。","parameters":{"type":"object","properties":{"name":{"type":"string","description":"技能名，如 spoor"}},"required":["name"]}}}),
            json!({"type":"function","function":{"name":"run_shell","description":"执行一条命令（本 demo 只放行 `spoor …`，无管道/重定向）。按技能说明调用 spoor CLI。","parameters":{"type":"object","properties":{"command":{"type":"string","description":"如 spoor data/byd.pdf --pages 1:1"}},"required":["command"]}}}),
        ])
    }

    async fn execute(&self, name: &str, args: Value) -> String {
        match name {
            "read_file" => read_file(&args),
            "list_skills" => self.catalog(),
            "read_skill" => {
                let n = match require_str(&args, "name") {
                    Ok(n) => n,
                    Err(e) => return e.to_string(),
                };
                self.skills
                    .iter()
                    .find(|s| s.name == n)
                    .map(|s| s.body.trim().to_string())
                    .unwrap_or_else(|| "没有该名字的技能".to_string())
            }
            "run_shell" => {
                let cmd = match require_str(&args, "command") {
                    Ok(c) => c,
                    Err(e) => return e.to_string(),
                };
                run_shell(cmd).await
            }
            other => format!("不支持的工具: {other}"),
        }
    }

    fn transport(&self) -> Option<String> {
        Some("Skill·spoor CLI 子进程".to_string())
    }

    fn system_addendum(&self) -> Option<String> {
        Some(format!(
            "你有以下**技能**可用（渐进式披露：先用 read_skill 读全文，再按其说明用 run_shell 执行）：\n{}\n处理非纯文本文档时，优先查看 spoor 技能。",
            self.catalog()
        ))
    }
}
