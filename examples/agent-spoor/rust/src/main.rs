//! 入口：agent-spoor [--mode native|mcp|skill] [一次性问题…]

use std::io::Write;

use agent_spoor::agent::Agent;
use agent_spoor::model::Model;
use agent_spoor::provider::ToolProvider;
use agent_spoor::providers::{mcp::McpProvider, native::NativeProvider, skill::SkillProvider};
use anyhow::Result;

fn mode_note(mode: &str) -> &'static str {
    match mode {
        "mcp" => "MCP Server（独立进程，标准协议）",
        "skill" => "Skill（SKILL.md + 受限 run_shell 调 spoor CLI）",
        _ => "原生工具（spoor-core，同进程）",
    }
}

fn parse_args() -> (String, String) {
    let mut mode = "native".to_string();
    let mut rest: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--mode" {
            if let Some(v) = args.next() {
                mode = v;
            }
        } else {
            rest.push(arg);
        }
    }
    (mode, rest.join(" ").trim().to_string())
}

async fn build_provider(mode: &str) -> Result<Box<dyn ToolProvider>> {
    Ok(match mode {
        "mcp" => Box::new(McpProvider::start().await?),
        "skill" => Box::new(SkillProvider::new()),
        _ => Box::new(NativeProvider::new()),
    })
}

/// 阻塞读一行（EOF → None）。
fn read_line() -> Option<String> {
    let mut s = String::new();
    match std::io::stdin().read_line(&mut s) {
        Ok(0) => None,
        Ok(_) => Some(s),
        Err(_) => None,
    }
}

async fn run_repl(agent: &mut Agent, mode: &str) -> Result<()> {
    println!("mini-agent × spoor 已启动 —— 接入模式：{}", mode_note(mode));
    println!(
        "试试：'用 data/byd.pdf 第 1 页总结比亚迪 2024 关键财务' 或 'data/sales.csv 金额最高的三个分类'"
    );
    println!("输入 exit / quit / 退出 结束。\n");

    loop {
        print!("你: ");
        std::io::stdout().flush().ok();
        let Some(line) = tokio::task::spawn_blocking(read_line).await? else {
            break;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(
            line.to_lowercase().as_str(),
            "exit" | "quit" | "q" | "退出" | ":q"
        ) {
            break;
        }
        match agent.chat(line).await {
            Ok(answer) => println!("\nAgent: {answer}\n"),
            Err(e) => eprintln!("\n错误: {e}\n"),
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let (mode, one_shot) = parse_args();
    let model = Model::from_env()?;
    let provider = build_provider(&mode).await?;
    let mut agent = Agent::new(model, provider);

    if one_shot.is_empty() {
        run_repl(&mut agent, &mode).await?;
    } else {
        println!("[mode] {}\n", mode_note(&mode));
        println!("\nAgent: {}\n", agent.chat(&one_shot).await?);
    }

    agent.close().await;
    Ok(())
}
