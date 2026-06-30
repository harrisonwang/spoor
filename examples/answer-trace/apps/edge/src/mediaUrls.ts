// markdown 预处理：把 spoor 产物的安全 URI 重写为前端可请求的 /api/media 链接。
// 等价于 apps/api/app/services/media_urls.py，但用 Worker 自身 origin 拼绝对地址
// （跨源 <img> 显示无需 CORS），并带上 corpus，让图片请求落到正确的 KV 语料。

const SPOOR_IMG = /!\[([^\]]*)\]\((spoor:\/\/[^)]+)\)/g;

export function rewriteSpoorImages(
  markdown: string,
  origin: string,
  docIndex = 0,
  corpusId?: string,
): string {
  return markdown.replace(SPOOR_IMG, (_match, alt: string, uri: string) => {
    const encoded = encodeURIComponent(uri);
    const corpus = corpusId ? `&corpus=${encodeURIComponent(corpusId)}` : "";
    return `![${alt}](${origin}/api/media?uri=${encoded}&doc=${docIndex}${corpus})`;
  });
}
