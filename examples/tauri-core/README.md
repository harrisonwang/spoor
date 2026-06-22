# Tauri command 最小集成形态

`parse_for_desktop` 的参数和结果形状与 Tauri command 一致，但这个最小示例
不引入 Tauri 依赖，用于直接展示 `spoor-core` 的嵌入方式。

```bash
cargo run -p spoor-tauri-core-example
```

在实际应用中为函数加上 `#[tauri::command]`，并从前端传入文件 bytes。
默认解析内存上限为 64 MiB；若前端把 `Uint8Array` 转成普通数组再传递，会产生额外
内存副本，大文件应用应设计流式读取或原生侧文件读取命令。
