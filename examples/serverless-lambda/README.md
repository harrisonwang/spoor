# AWS Lambda example

This handler expects the `spoor` Linux binary in a Lambda Layer at
`/opt/bin/spoor`. Invoke with `{ "filename": "report.pdf", "body": "...",
"isBase64Encoded": true }`. The parsing process remains isolated from the
Node.js runtime and inherits the CLI parse/output limits.

Run the integration smoke test against a locally built binary:

```bash
cargo build -p spoor-cli
SPOOR_BIN="$PWD/target/debug/spoor" npm --prefix examples/serverless-lambda test
```
