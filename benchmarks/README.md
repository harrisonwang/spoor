# Benchmarks and size budgets

Run:

```bash
./benchmarks/run.sh
./benchmarks/wasm-size.sh
SPOOR_BIN="$PWD/target/release/spoor" .venv/bin/python benchmarks/python.py
```

Baseline measured on 2026-06-11 on Apple Silicon:

| Measurement | Result |
| --- | ---: |
| Warm `spoor-core` plain-text parse | 245 ns/call |
| Warm PyO3 plain-text parse | 2,941 ns/call |
| Long-lived Python worker IPC | 36.564 µs/call |
| Python-driven CLI process per file | 13.006 ms/call |
| CLI process loop, 100 calls | 0.20 s total / 2.0 ms per call |
| Concurrent CLI, 100 calls at concurrency 8 | 0.03 s total / ~3,333 calls/s |
| Single CLI maximum resident set size | 2,686,976 bytes |
| `spoor-core` crate archive | < 140 KiB |
| macOS arm64 CLI binary | 4,905,744 bytes |
| Published core-format WASM raw / gzip | 1,426,062 / 588,506 bytes |
| Published full-format WASM raw / gzip | 2,232,597 / 858,149 bytes |
| Cloudflare Worker bundle raw / gzip | ~1.40 MiB / 578.85 KiB |
| macOS arm64 `pyspoor` abi3 wheel | 1,379,367 bytes |
| macOS arm64 Node native addon | 2,917,408 bytes |

`benchmarks/python.py` measures the architecture comparison explicitly:
in-process warm PyO3, one CLI process per file, and a long-lived Python worker
using newline-delimited JSON IPC.

WASM package sizes are measured from `wasm-pack` output with the intentionally
disabled, currently incompatible `wasm-opt` step. Core-format gzip remains
below the 3 MiB P0 budget and full-format gzip remains below the 10 MiB budget.
