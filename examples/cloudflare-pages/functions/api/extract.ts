import { extractMedia } from '../../src/edge-spoor';

const MAX_REQUEST_BYTES = 16 * 1024 * 1024;

// POST /api/extract?uri=<spoor:// 占位符>
// body 为原始文档字节，按占位符返回单个内嵌资源的原始字节（懒取、单资源）：
// DOCX/PPTX 图片、PDF 内嵌图（spoor://pdf/obj/...）、PDF 整页矢量图（spoor://pdf/page/...，SVG）。
export const onRequestPost = async (context: { request: Request }): Promise<Response> => {
  const { request } = context;
  const uri = new URL(request.url).searchParams.get('uri');
  if (!uri) {
    return errorResponse('missing_uri', '缺少 uri 查询参数（spoor:// 占位符，如 spoor://pdf/page/1）。', 400);
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
      headers: { ...responseHeaders(), 'content-type': mediaContentType(media) },
    });
  } catch (error) {
    return Response.json(normalizeError(error), { status: 422, headers: responseHeaders() });
  }
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
