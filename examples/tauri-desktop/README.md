# Tauri 2 桌面应用：把 spoor-core 直接链接进二进制

**本示例唯一证明：把 `spoor-core`（纯 Rust 库）直接静态链接进一个完整的 Tauri 2 桌面二进制——解析在本机原生执行，零外部进程、零网络、零运行时依赖，文件不出设备。** 与 [`../electron-desktop`](../electron-desktop/) 的区别就在 binding：这里是 **Rust core 直链**，那边是 **Node 原生 addon**；同一种本地桌面解析，两种技术栈各一条。

这是一个完整 Tauri 2 应用，直接把 `spoor-core` 链接进桌面二进制。前端通过
窄 `parse_document` command 传递本地 bytes，并在 renderer 中执行段落检索。
原生 core 包含 DOCX、XLSX、PDF、PPTX、HTML、EPUB、IPYNB 及基础格式。

```bash
cd examples/tauri-desktop
npm install
npm run check
npm run tauri:dev
```

构建桌面应用：

```bash
npm run tauri:build
```

示例使用 core 默认的 64 MiB 解析内存上限。当前 command 通过
`Array.from(Uint8Array)` 转换后再传给 Rust，会同时保留多个内存副本，因此它
用于说明集成结构，不适合作为超大文件传输方案。最小依赖形态见
[`../rust-core-embed`](../rust-core-embed/)。

`src-tauri/Cargo.lock` 有意把 `time` 固定在 `0.3.47`；`0.3.48` 当前会通过
Tauri 的传递依赖与 Rust 1.96 冲突。
