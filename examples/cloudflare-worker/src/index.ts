import { extractMedia, parseBytes } from './spoor';

const MAX_REQUEST_BYTES = 16 * 1024 * 1024;

export default {
  async fetch(request: Request): Promise<Response> {
    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: responseHeaders() });
    }

    const url = new URL(request.url);

    if (request.method === 'GET') {
      return Response.json({
        name: 'spoor-document-cleaner',
        runtime: 'cloudflare-workers',
        max_request_bytes: MAX_REQUEST_BYTES,
        formats: ['docx', 'xlsx', 'pdf', 'pptx', 'html', 'epub', 'ipynb', 'markdown', 'text', 'csv'],
        endpoints: {
          'POST /': 'POST 原始文档字节解析为结构化结果',
          'POST /extract?uri=<spoor:// 占位符>': 'POST 原始字节，按解析结果里的 spoor:// 占位符返回单个内嵌资源：DOCX/PPTX 图片（spoor://{docx,pptx}/part/.../media/*）、PDF 内嵌图（spoor://pdf/obj/{id}/{gen}）、PDF 整页矢量图（spoor://pdf/page/{n}，返回 SVG）',
        },
      }, { headers: responseHeaders() });
    }

    if (request.method !== 'POST') {
      return Response.json(
        { usage: 'POST 原始文档字节，并设置 x-filename；content-type 可选。' },
        { status: 405, headers: responseHeaders() },
      );
    }

    const declaredLength = Number(request.headers.get('content-length') ?? 0);
    if (declaredLength > MAX_REQUEST_BYTES) {
      return Response.json(
        { code: 'request_too_large', message: '请求超过此演示的 16 MiB 上限。' },
        { status: 413, headers: responseHeaders() },
      );
    }

    const bytes = new Uint8Array(await request.arrayBuffer());
    if (bytes.byteLength > MAX_REQUEST_BYTES) {
      return Response.json(
        { code: 'request_too_large', message: '请求超过此演示的 16 MiB 上限。' },
        { status: 413, headers: responseHeaders() },
      );
    }

    const filename = request.headers.get('x-filename') ?? undefined;
    const contentType = request.headers.get('content-type') ?? undefined;

    // /extract：按占位符返回单张内嵌图片的原始字节（懒取、单资源）
    if (url.pathname === '/extract') {
      const uri = url.searchParams.get('uri');
      if (!uri) {
        return Response.json(
          { code: 'missing_uri', message: '缺少 uri 查询参数（spoor://{docx,pptx}/part/{root}/media/*）。' },
          { status: 400, headers: responseHeaders() },
        );
      }
      try {
        const media = extractMedia(bytes, uri, filename, contentType, undefined, MAX_REQUEST_BYTES);
        return new Response(media, {
          headers: { ...responseHeaders(), 'content-type': mediaContentType(media) },
        });
      } catch (error) {
        return Response.json(normalizeError(error), { status: 422, headers: responseHeaders() });
      }
    }

    try {
      const result = parseBytes(bytes, filename, contentType, undefined, MAX_REQUEST_BYTES);
      return Response.json(result, { headers: responseHeaders() });
    } catch (error) {
      return Response.json(normalizeError(error), {
        status: 422,
        headers: responseHeaders(),
      });
    }
  },
};

function mediaContentType(bytes: Uint8Array): string {
  // 按字节魔数判定，而非 URI 后缀：PDF 整页图是 SVG、PDF 内嵌图可能是 PNG/JPEG，
  // 它们的 spoor://pdf/... 占位符都没有扩展名可循。
  if (bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4e && bytes[3] === 0x47) return 'image/png';
  if (bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff) return 'image/jpeg';
  if (bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46) return 'image/gif';
  if (
    bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 &&
    bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50
  ) return 'image/webp';
  const head = new TextDecoder().decode(bytes.subarray(0, 64)).trimStart().toLowerCase();
  if (head.startsWith('<?xml') || head.startsWith('<svg')) return 'image/svg+xml';
  return 'application/octet-stream';
}

function normalizeError(error: unknown): unknown {
  if (error && typeof error === 'object') {
    const entries = Object.entries(error);
    if (entries.length > 0) return Object.fromEntries(entries);
    if (error instanceof Error) {
      return { code: 'parse_failed', message: error.message };
    }
  }
  return { code: 'parse_failed', message: String(error) };
}

function responseHeaders(): HeadersInit {
  return {
    'access-control-allow-origin': '*',
    'access-control-allow-methods': 'GET, POST, OPTIONS',
    'access-control-allow-headers': 'content-type, x-filename',
    'cache-control': 'no-store',
    'x-content-type-options': 'nosniff',
  };
}
