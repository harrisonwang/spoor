import path from "node:path";

/** 确保 input 是普通对象 */
export function assertObject(input: unknown): asserts input is Record<string, unknown> {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    throw new Error("Input 必须是一个普通对象");
  }
}

/** 确保字段是 string 并返回 */
export function assertString(value: unknown, field: string): string {
  if (typeof value !== "string") throw new Error(`${field} 必须是一个字符串`);
  return value;
}

/** 把相对路径解析成项目内绝对路径；拒绝 ../ 越界（文件不出本项目，呼应 spoor 的本地处理）。 */
export function safeResolve(userPath: string): string {
  const root = path.resolve(process.cwd());
  const resolved = path.resolve(root, userPath);
  if (resolved !== root && !resolved.startsWith(root + path.sep)) {
    throw new Error(`路径在项目外: ${userPath}`);
  }
  return resolved;
}

// —— 宽松强制转换（模型给的参数可能缺省或类型不对，取不到就当没传）——
export const optStr = (v: unknown): string | undefined => (typeof v === "string" ? v : undefined);
export const optNum = (v: unknown): number | undefined => (typeof v === "number" ? v : undefined);

export function pair(v: unknown): [number, number] | undefined {
  return Array.isArray(v) && v.length === 2 && v.every((n) => typeof n === "number")
    ? [v[0] as number, v[1] as number]
    : undefined;
}

export function strArr(v: unknown): string[] | undefined {
  return Array.isArray(v) && v.every((x) => typeof x === "string") ? (v as string[]) : undefined;
}
