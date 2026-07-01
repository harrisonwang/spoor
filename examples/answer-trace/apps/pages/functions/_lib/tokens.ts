// token 估算（启发式）：CJK(汉字)≈1 token，其余按 ~4 字符/token。偏保守，只作"是否超上下文"提示。

const CJK = /\p{Script=Han}/gu;

export function count(text: string): number {
  if (!text) return 0;
  const cjk = (text.match(CJK) ?? []).length;
  const rest = text.length - cjk;
  return Math.round(cjk + rest / 4);
}
