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
          'POST /extract?uri=spoor://{docx,pptx}/part/{root}/media/*': 'POST 原始 DOCX/PPTX 字节，按占位符返回单张内嵌图片字节',
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
          headers: { ...responseHeaders(), 'content-type': mediaContentType(uri) },
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

function mediaContentType(uri: string): string {
  const ext = uri.split('.').pop()?.toLowerCase();
  if (ext === 'png') return 'image/png';
  if (ext === 'jpg' || ext === 'jpeg') return 'image/jpeg';
  if (ext === 'gif') return 'image/gif';
  if (ext === 'webp') return 'image/webp';
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
