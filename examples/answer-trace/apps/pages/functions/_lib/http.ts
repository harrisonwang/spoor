// 同源前后端，无需 CORS；统一 JSON 响应 + 不缓存。
export function json(data: unknown, status = 200): Response {
  return Response.json(data, {
    status,
    headers: { "cache-control": "no-store", "x-content-type-options": "nosniff" },
  });
}
