// 模式③Skill：不写新的类型化工具，丢一份 SKILL.md，用受限 run_shell 驱动 spoor CLI。
// 最松耦合、零改 agent 逻辑；渐进式披露（系统提示只给技能目录，用到才 read_skill 读全文）。

import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { staticProvider, type Tool, type ToolProvider } from "../provider.js";
import { readFileTool } from "../tools/base.js";
import { runShell } from "../util/shell.js";
import { assertObject, assertString } from "../util/validate.js";

interface SkillCard {
  name: string;
  description: string;
  body: string;
}

function parseSkill(dirName: string, raw: string): SkillCard {
  let name = dirName;
  let description = "";
  let body = raw;
  const m = raw.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
  if (m) {
    body = m[2];
    for (const line of m[1].split("\n")) {
      const kv = line.match(/^(\w+):\s*(.*)$/);
      if (!kv) continue;
      if (kv[1] === "name") name = kv[2].trim();
      if (kv[1] === "description") description = kv[2].trim();
    }
  }
  return { name, description, body };
}

function loadSkills(): SkillCard[] {
  const dir = fileURLToPath(new URL("../skills", import.meta.url));
  if (!existsSync(dir)) return [];
  const cards: SkillCard[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const file = path.join(dir, entry.name, "SKILL.md");
    if (existsSync(file)) cards.push(parseSkill(entry.name, readFileSync(file, "utf8")));
  }
  return cards;
}

export function skillProvider(): ToolProvider {
  const skills = loadSkills();
  const byName = new Map(skills.map((s) => [s.name, s]));

  const catalog = () => skills.map((s) => `- ${s.name}: ${s.description}`).join("\n") || "（无可用技能）";

  const listSkills: Tool = {
    name: "list_skills",
    description: "列出可用技能（名字 + 简介）。",
    input_schema: { type: "object", properties: {} },
    async execute() {
      return catalog();
    },
  };

  const readSkill: Tool = {
    name: "read_skill",
    description: "读取某个技能的完整说明（SKILL.md 正文），据此决定怎么用 run_shell。",
    input_schema: {
      type: "object",
      properties: { name: { type: "string", description: "技能名，如 spoor" } },
      required: ["name"],
    },
    async execute(input) {
      assertObject(input);
      const s = byName.get(assertString(input.name, "name"));
      return s ? s.body.trim() : "没有该名字的技能";
    },
  };

  const runShellTool: Tool = {
    name: "run_shell",
    description: "执行一条命令（本 demo 只放行 `spoor …`，无管道/重定向）。按技能说明调用 spoor CLI。",
    input_schema: {
      type: "object",
      properties: { command: { type: "string", description: "如 spoor data/byd.pdf --pages 1:1" } },
      required: ["command"],
    },
    async execute(input) {
      assertObject(input);
      return runShell(assertString(input.command, "command"));
    },
  };

  const addendum = () =>
    `你有以下**技能**可用（渐进式披露：先用 read_skill 读全文，再按其说明用 run_shell 执行）：\n${catalog()}\n处理非纯文本文档时，优先查看 spoor 技能。`;

  return staticProvider([readFileTool, listSkills, readSkill, runShellTool], {
    transport: "Skill·spoor CLI 子进程",
    systemAddendum: addendum,
  });
}
