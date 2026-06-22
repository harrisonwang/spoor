# Electron 本地文档桌面示例

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
