import { extractMedia } from '../../src/edge-spoor';

const MAX_REQUEST_BYTES = 16 * 1024 * 1024;

// POST /api/extract?uri=spoor-docx://word/media/*
// body 为原始 DOCX 字节，按占位符返回单张内嵌图片的原始字节（懒取、单资源）。
export const onRequestPost = async (context: { request: Request }): Promise<Response> => {
  const { request } = context;
  const uri = new URL(request.url).searchParams.get('uri');
  if (!uri) {
    return errorResponse('missing_uri', '缺少 uri 查询参数（spoor-docx://word/media/*）。', 400);
  }

  const declaredLength = Number(request.headers.get('content-length') ?? 0);
  if (declaredLength > MAX_REQUEST_BYTES) {
    return errorResponse('request_too_large', '请求超过此演示的 16 MiB 上限。', 413);
  }

  const bytes = new Uint8Array(await request.arrayBuffer());
  if (bytes.byteLength > MAX_REQUEST_BYTES) {
    return errorResponse('request_too_large', '请求超过此演示的 16 MiB 上限。', 413);
  }

  try {
    const media = extractMedia(
      bytes,
      uri,
      request.headers.get('x-filename') ?? undefined,
      request.headers.get('content-type') ?? undefined,
      undefined,
      MAX_REQUEST_BYTES,
    );
    return new Response(media, {
      headers: { ...responseHeaders(), 'content-type': mediaContentType(uri) },
    });
  } catch (error) {
    return Response.json(normalizeError(error), { status: 422, headers: responseHeaders() });
  }
};

function mediaContentType(uri: string): string {
  const ext = uri.split('.').pop()?.toLowerCase();
  if (ext === 'png') return 'image/png';
  if (ext === 'jpg' || ext === 'jpeg') return 'image/jpeg';
  if (ext === 'gif') return 'image/gif';
  if (ext === 'webp') return 'image/webp';
  return 'application/octet-stream';
}

function errorResponse(code: string, message: string, status: number): Response {
  return Response.json({ code, message }, { status, headers: responseHeaders() });
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
    'cache-control': 'no-store',
    'x-content-type-options': 'nosniff',
  };
}
