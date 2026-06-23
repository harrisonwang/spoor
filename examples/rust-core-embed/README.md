# spoor-core 最小嵌入（无框架）

**本示例唯一证明：把 `spoor-core`（纯 Rust 库）嵌入自己 Rust 程序的最小形态——不引入任何框架（连 Tauri 都不依赖），只演示 `ParseRequest` → `parse` 的调用形状与结果。** 这是「Rust 直接嵌入」交付形态的最小起点；要看它在完整 GUI 桌面里的样子，见 [`../tauri-desktop`](../tauri-desktop/)。

`parse_for_desktop` 的参数和结果形状与 Tauri command 一致，但这个最小示例
不引入 Tauri 依赖，用于直接展示 `spoor-core` 的嵌入方式。

```bash
cargo run -p spoor-rust-core-embed-example
```

在实际应用中为函数加上 `#[tauri::command]`，并从前端传入文件 bytes。
默认解析内存上限为 64 MiB；若前端把 `Uint8Array` 转成普通数组再传递，会产生额外
内存副本，大文件应用应设计流式读取或原生侧文件读取命令。
