import { parse_bytes } from '@harrisonwang/spoor-wasm';

export default {
  async fetch(request: Request): Promise<Response> {
    if (request.method !== 'POST') {
      return Response.json(
        { usage: 'POST raw document bytes; set x-filename and content-type headers' },
        { status: 405 },
      );
    }

    const bytes = new Uint8Array(await request.arrayBuffer());
    try {
      const result = parse_bytes(
        bytes,
        request.headers.get('x-filename') ?? undefined,
        request.headers.get('content-type') ?? undefined,
        undefined,
        16 * 1024 * 1024,
      );
      return Response.json(result);
    } catch (error) {
      return Response.json(error, { status: 422 });
    }
  },
};
