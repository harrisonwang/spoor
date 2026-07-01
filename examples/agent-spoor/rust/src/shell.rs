//! 受限 run_shell：只放行单条 `spoor …`，无 shell 元字符，cwd 锁项目根。

use anyhow::{Result, bail};
use tokio::process::Command;

use crate::validate::safe_resolve;

pub async fn run_shell(command: &str) -> String {
    run_shell_inner(command)
        .await
        .unwrap_or_else(|e| format!("错误: {e}"))
}

async fn run_shell_inner(command: &str) -> Result<String> {
    let cmd = command.trim();
    if cmd.chars().any(|c| {
        matches!(
            c,
            ';' | '&' | '|' | '`' | '$' | '>' | '<' | '\n' | '\r' | '(' | ')'
        )
    }) {
        bail!("命令含不允许的 shell 元字符（此工具只放行单条 spoor 命令，无管道/重定向）");
    }
    let argv = tokenize(cmd);
    if argv.first().map(String::as_str) != Some("spoor") {
        bail!("run_shell 只放行 `spoor …` 命令");
    }
    let args = &argv[1..];
    let bin = std::env::var("SPOOR_BIN").unwrap_or_else(|_| "spoor".to_string());
    let output = Command::new(&bin).args(args).output().await?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // --extract：stdout 是二进制媒体；skill 模式没有 > 重定向，这里替它存文件。
    if let Some(i) = args.iter().position(|a| a == "--extract") {
        if !output.status.success() {
            return Ok(format!(
                "spoor 提取失败（{}）:\n{}",
                output.status,
                stderr.trim()
            ));
        }
        let uri = args.get(i + 1).map(String::as_str).unwrap_or("media");
        let out_dir = safe_resolve(".spoor-media")?;
        std::fs::create_dir_all(&out_dir)?;
        let mut name: String = uri
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        name = name.trim_start_matches('_').to_string();
        if name.len() > 48 {
            name = name[name.len() - 48..].to_string();
        }
        if name.is_empty() {
            name = "media".to_string();
        }
        std::fs::write(out_dir.join(&name), &output.stdout)?;
        return Ok(format!(
            "已提取内嵌资源 → .spoor-media/{name}（{} bytes）。可交给 VLM。",
            output.stdout.len()
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let warn = if stderr.trim().is_empty() {
        String::new()
    } else {
        format!("\n\n〔stderr / warnings〕\n{}", stderr.trim())
    };
    if !output.status.success() && text.is_empty() {
        return Ok(format!(
            "spoor 执行失败（{}）:\n{}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(format!("{text}{warn}"))
}

fn tokenize(cmd: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for c in cmd.chars() {
        match c {
            '"' => in_quote = !in_quote,
            c if c.is_whitespace() && !in_quote => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
