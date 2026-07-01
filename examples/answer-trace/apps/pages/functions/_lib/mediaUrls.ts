// 把 spoor:// 安全 URI 重写为同源 /api/media 相对链接（前后端同域，无需绝对地址/CORS）。

const SPOOR_IMG = /!\[([^\]]*)\]\((spoor:\/\/[^)]+)\)/g;

export function rewriteSpoorImages(markdown: string, docIndex = 0, corpusId?: string): string {
  return markdown.replace(SPOOR_IMG, (_match, alt: string, uri: string) => {
    const encoded = encodeURIComponent(uri);
    const corpus = corpusId ? `&corpus=${encodeURIComponent(corpusId)}` : "";
    return `![${alt}](/api/media?uri=${encoded}&doc=${docIndex}${corpus})`;
  });
}
