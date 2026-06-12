import { parseBytes } from '../../src/edge-spoor';

const MAX_REQUEST_BYTES = 16 * 1024 * 1024;

export const onRequestGet = async (): Promise<Response> => Response.json({
  name: 'spoor-pages-demo',
  runtime: 'cloudflare-pages-functions',
  max_request_bytes: MAX_REQUEST_BYTES,
  formats: ['docx', 'xlsx', 'pdf', 'pptx', 'html', 'epub', 'ipynb', 'markdown', 'text', 'csv'],
});

export const onRequestPost = async (context: { request: Request }): Promise<Response> => {
  const { request } = context;
  const declaredLength = Number(request.headers.get('content-length') ?? 0);
  if (declaredLength > MAX_REQUEST_BYTES) {
    return errorResponse('request_too_large', '请求超过此演示的 16 MiB 上限。', 413);
  }

  const bytes = new Uint8Array(await request.arrayBuffer());
  if (bytes.byteLength > MAX_REQUEST_BYTES) {
    return errorResponse('request_too_large', '请求超过此演示的 16 MiB 上限。', 413);
  }

  try {
    const result = parseBytes(
      bytes,
      request.headers.get('x-filename') ?? undefined,
      request.headers.get('content-type') ?? undefined,
      undefined,
      MAX_REQUEST_BYTES,
    );
    return Response.json(result, { headers: responseHeaders() });
  } catch (error) {
    return Response.json(normalizeError(error), {
      status: 422,
      headers: responseHeaders(),
    });
  }
};

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
