# Electron 桌面应用：原生 Node.js binding

**本示例唯一证明：在 Electron 桌面应用里用 spoor 的原生 Node.js addon `@harrisonwang/spoor` 解析文档——本机原生执行、文件不出设备，binding 是 Node addon（对照 [`../tauri-desktop`](../tauri-desktop/) 的 Rust core 直链）。** 适合 JS/Node 技术栈的桌面应用。

该示例在 Electron 主进程中使用原生 Node.js binding
`@harrisonwang/spoor`。Renderer 开启 `contextIsolation`、禁用 Node
integration，并只通过窄 preload API 传递本地文件 bytes。

原生包包含 DOCX、XLSX、PDF、PPTX、HTML、EPUB、IPYNB 及基础格式。
示例为每个文件设置 64 MiB 解析内存上限；解析仍发生在 Electron 主进程内，
生产应用如需崩溃隔离或严格超时，应放入 Utility Process 或独立 worker。

```bash
cd examples/electron-desktop
npm install
npm run check
npm start
```

构建未签名的本地应用：

```bash
npm run package
```

产物写入 `dist/`。当前原生 binding 发布目标为 macOS arm64/x64、
Linux x64 GNU 和 Windows x64 MSVC。
