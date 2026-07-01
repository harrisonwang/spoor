//! 参数校验与相对路径解析（呼应 spoor 的本地处理：文件不出项目）。

use anyhow::{Result, anyhow, bail};
use serde_json::Value;
use std::path::PathBuf;

/// 把相对路径解析成项目内绝对路径；拒绝 ../ 越界。
pub fn safe_resolve(user_path: &str) -> Result<PathBuf> {
    let root = std::env::current_dir()?.canonicalize()?;
    let candidate = root.join(user_path);
    // 目标文件可能还不存在（.spoor-media），逐段规范化父目录即可。
    let resolved = normalize(&candidate);
    if resolved != root && !resolved.starts_with(&root) {
        bail!("路径在项目外: {user_path}");
    }
    Ok(resolved)
}

/// 纯词法规范化（去掉 `.` / `..`），不触碰文件系统。
fn normalize(path: &std::path::Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

pub fn require_str<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("缺少字符串参数 {key}"))
}

pub fn opt_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

pub fn opt_usize(args: &Value, key: &str) -> Option<usize> {
    args.get(key).and_then(Value::as_u64).map(|n| n as usize)
}

/// 从 JSON 数组取 `[a, b]`（1-based 闭区间）。
pub fn pair(args: &Value, key: &str) -> Option<(usize, usize)> {
    let arr = args.get(key)?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    Some((arr[0].as_u64()? as usize, arr[1].as_u64()? as usize))
}

pub fn str_vec(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}
