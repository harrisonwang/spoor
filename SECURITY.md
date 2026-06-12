# Security model

`spoor` parses untrusted documents using a small, deterministic Rust core. The
core is designed to fail with structured errors, but it is not a substitute
for an operating-system or WASM-runtime memory limit.

## Trust boundaries

- `spoor-core` accepts bytes plus explicit metadata. It performs no file,
  network, stdin/stdout, environment, or process I/O.
- CLI, Python, Node, and WASM adapters are responsible for acquiring bytes.
- Native parsers execute in the caller's process. Use the CLI in a constrained
  worker/container when crash or RSS isolation is required.
- No parser executes document macros, notebook code, scripts, formulas, or
  embedded binaries.

## Threats and defenses

| Threat | Defense | Default | Configurable |
| --- | --- | --- | --- |
| Oversized input | Core checks input bytes before detection/parsing | 64 MiB shared parse budget | `ParseLimits.max_parse_bytes`; CLI `--max-parse-bytes` |
| ZIP bomb: too many entries | Reject archive during central-directory inspection | 10,000 entries | No public override |
| ZIP bomb: huge entry | Reject declared or observed oversized entry | 50 MiB per entry | No public override |
| ZIP bomb: extreme ratio | Reject suspicious declared compression ratio | 200× | No public override |
| ZIP bomb: aggregate expansion | Sum declared uncompressed sizes against parse budget | Shared parse budget | `max_parse_bytes` |
| Output/token exhaustion | CLI truncates total stdout with an in-band marker or JSON warning | 256 KiB | CLI `--max-output-bytes` |
| Encrypted/legacy Office ambiguity | Intercept OLE/CFB before extension fallback | Stable `legacy_or_encrypted_office` error | No |
| Encrypted PDF | Map decryption failure to stable error | Stable `encrypted_pdf` error | No |
| Image-only PDF hallucination risk | Reject empty text layer instead of returning silent success | Stable `image_only_pdf` error | No |
| Corrupt container | Reject unreadable ZIP-based formats | Stable `invalid_container` error | No |
| Unknown parser failure or Rust panic | Catch unwind at every public core boundary and normalize to `parse_failed` with stage | Structured `SpoorError` | No |
| Infinite/very slow parse | No in-process timeout; caller must enforce deadline | Not provided | Worker/container/WASM host |
| Native dependency abort/segfault | Not recoverable in-process | Not provided | Prefer CLI worker isolation for hostile tenants |

## Stable failure contract

Every public adapter exposes `code`, `reason`, `hint`, `recoverable`, and
`stage`. Consumers must branch on `code`, never localized prose. Current codes:

- `image_only_pdf`
- `parse_budget_exceeded`
- `unsupported_format`
- `encrypted_pdf`
- `legacy_or_encrypted_office`
- `invalid_container`
- `parse_failed`

The core, CLI, Python, Node, and WASM test paths exercise shared budget,
invalid-container, compression-bomb, and CFB/OLE interception behavior.

## Reporting

Do not open a public issue for a suspected vulnerability. Send a private
security advisory through the repository's Security tab with a minimal
reproducer, affected version, and observed impact.
