import { stdin, stdout } from "node:process";
import * as readline from "node:readline/promises";
import { Agent } from "./agent.js";
import type { ToolProvider } from "./provider.js";
import { mcpProvider } from "./providers/mcp.js";
import { nativeProvider } from "./providers/native.js";
import { skillProvider } from "./providers/skill.js";

type Mode = "native" | "mcp" | "skill";
const EXIT_COMMANDS = new Set(["exit", "quit", "q", "退出", ":q"]);

function parseArgs(argv: string[]): { mode: Mode; oneShot: string } {
  let mode: Mode = "native";
  const rest: string[] = [];
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--mode") {
      const v = argv[++i];
      if (v === "native" || v === "mcp" || v === "skill") mode = v;
      else throw new Error(`未知 --mode: ${v}（可选 native|mcp|skill）`);
    } else {
      rest.push(argv[i]);
    }
  }
  return { mode, oneShot: rest.join(" ").trim() };
}

async function buildProvider(mode: Mode): Promise<ToolProvider> {
  if (mode === "mcp") return mcpProvider();
  if (mode === "skill") return skillProvider();
  return nativeProvider();
}

const MODE_NOTE: Record<Mode, string> = {
  native: "原生工具（@harrisonwang/spoor，同进程）",
  mcp: "MCP Server（独立进程，标准协议）",
  skill: "Skill（SKILL.md + 受限 run_shell 调 spoor CLI）",
};

async function runOnce(mode: Mode, message: string): Promise<void> {
  const agent = new Agent(await buildProvider(mode));
  try {
    console.log(`[mode] ${MODE_NOTE[mode]}\n`);
    console.log(`\nAgent: ${await agent.chat(message)}\n`);
  } finally {
    await agent.close();
  }
}

async function runRepl(mode: Mode): Promise<void> {
  stdin.setEncoding("utf8");
  if (stdin.isTTY) stdin.setRawMode(false);
  const rl = readline.createInterface({ input: stdin, output: stdout, terminal: false });
  const agent = new Agent(await buildProvider(mode));

  process.on("SIGINT", () => {
    if (agent.abort()) {
      stdout.write("\n");
      return;
    }
    void agent.close().finally(() => {
      console.log("\n再见！");
      process.exit(130);
    });
  });

  console.log(`mini-agent × spoor 已启动 —— 接入模式：${MODE_NOTE[mode]}`);
  console.log(`试试：'用 data/byd.pdf 第 1 页总结比亚迪 2024 关键财务' 或 'data/sales.csv 金额最高的三个分类'`);
  console.log("输入 exit / quit / 退出 结束。\n");

  while (true) {
    const userInput = await rl.question("你: ");
    if (!userInput.trim()) continue;
    if (EXIT_COMMANDS.has(userInput.trim().toLowerCase())) break;
    try {
      console.log(`\nAgent: ${await agent.chat(userInput)}\n`);
    } catch (e) {
      if (e instanceof Error && e.name === "AbortError") {
        console.log("\n[已中断]\n");
        continue;
      }
      console.error(`\n错误: ${e instanceof Error ? e.message : String(e)}\n`);
    }
  }

  rl.close();
  await agent.close();
  console.log("\n再见！");
  process.exit(0);
}

async function main(): Promise<void> {
  const { mode, oneShot } = parseArgs(process.argv.slice(2));
  if (oneShot) await runOnce(mode, oneShot);
  else await runRepl(mode);
}

main().catch((e) => {
  console.error(e instanceof Error ? e.message : String(e));
  process.exit(1);
});
