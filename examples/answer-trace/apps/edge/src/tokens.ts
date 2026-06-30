// token 估算。原 apps/api 用 tiktoken(o200k_base)；边缘为避免再背一份 ~2MB 编码表
// WASM，这里用启发式近似：CJK(汉字)≈1 token，其余按 ~4 字符/token。
// 用途只是前端"是否超上下文"的提示，偏保守（说塞得下基本塞得下），不需精确。

const CJK = /\p{Script=Han}/gu;

export function count(text: string): number {
  if (!text) return 0;
  const cjk = (text.match(CJK) ?? []).length;
  const rest = text.length - cjk;
  return Math.round(cjk + rest / 4);
}
