import { readFile } from "node:fs/promises";
import type { Tool } from "../provider.js";
import { assertObject, assertString, safeResolve } from "../util/validate.js";

/** agent 自带的基础工具：读纯文本/代码文件（与 spoor 的 read_document 形成对照）。 */
export const readFileTool: Tool = {
  name: "read_file",
  description: "从当前项目读取一个文本文件，返回带行号的内容。二进制文档（PDF/Word/Excel…）请用 read_document。",
  input_schema: {
    type: "object",
    properties: {
      file_path: { type: "string", description: "相对项目根目录的文件路径，如 src/index.ts" },
    },
    required: ["file_path"],
  },
  async execute(input) {
    assertObject(input);
    const filePath = safeResolve(assertString(input.file_path, "file_path"));
    try {
      const content = await readFile(filePath, "utf-8");
      return content
        .split("\n")
        .map((line, i) => `${String(i + 1).padStart(4)}: ${line}`)
        .join("\n");
    } catch (e) {
      return `读取文件失败: ${e instanceof Error ? e.message : String(e)}`;
    }
  },
};
