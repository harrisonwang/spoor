// 受限 run_shell：只放行单条 `spoor …`，无 shell 元字符，cwd 锁项目根。
// 既真实演示"技能驱动 CLI"，又不暴露任意命令执行。

import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { safeResolve } from "./validate.js";

const FORBIDDEN = /[;&|`$><\n\r()]/; // 拒绝管道/重定向/子命令/换行

function tokenize(cmd: string): string[] {
  const out: string[] = [];
  const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(cmd)) !== null) out.push(m[1] ?? m[2] ?? m[3] ?? "");
  return out;
}

function run(
  cmd: string,
  args: string[],
  cwd: string,
): Promise<{ code: number; stdout: Buffer; stderr: string }> {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { cwd });
    const out: Buffer[] = [];
    let err = "";
    child.stdout.on("data", (d: Buffer) => out.push(Buffer.from(d)));
    child.stderr.on("data", (d: Buffer) => (err += d.toString()));
    child.on("error", reject);
    child.on("close", (code) => resolve({ code: code ?? 0, stdout: Buffer.concat(out), stderr: err }));
  });
}

export async function runShell(command: string): Promise<string> {
  const cmd = command.trim();
  if (FORBIDDEN.test(cmd)) {
    throw new Error("命令含不允许的 shell 元字符（此工具只放行单条 spoor 命令，无管道/重定向）");
  }
  const argv = tokenize(cmd);
  if (argv[0] !== "spoor") throw new Error("run_shell 只放行 `spoor …` 命令");
  const args = argv.slice(1);

  // 用本地已装的 spoor CLI（@harrisonwang/spoor-cli 提供 bin）。
  const { code, stdout, stderr } = await run("npx", ["--no-install", "spoor", ...args], process.cwd());

  // --extract：stdout 是二进制媒体；skill 模式没有 > 重定向，这里替它存文件。
  const extractIdx = args.indexOf("--extract");
  if (extractIdx !== -1) {
    if (code !== 0) return `spoor 提取失败（exit ${code}）:\n${stderr.trim()}`;
    const uri = args[extractIdx + 1] ?? "media";
    const outDir = safeResolve(".spoor-media");
    await mkdir(outDir, { recursive: true });
    const name = uri.replace(/[^a-zA-Z0-9._-]/g, "_").replace(/^_+/, "").slice(-48) || "media";
    await writeFile(path.join(outDir, name), stdout);
    return `已提取内嵌资源 → .spoor-media/${name}（${stdout.length} bytes）。可交给 VLM。`;
  }

  const text = stdout.toString("utf8").trimEnd();
  const warn = stderr.trim() ? `\n\n〔stderr / warnings〕\n${stderr.trim()}` : "";
  if (code !== 0 && !text) return `spoor 执行失败（exit ${code}）:\n${stderr.trim()}`;
  return text + warn;
}
