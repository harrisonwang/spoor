# Cloudflare Worker example

The Worker accepts raw document bytes in a `POST` request. Set `x-filename`
and `content-type` so spoor can detect the input reliably.

```bash
cd crates/spoor-wasm
npm run build
cd ../../wasm/cloudflare-worker
npm install
npm run dev
```
