#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release -p spoor-cli
cargo bench -p spoor-core --bench parse

fixture="crates/spoor-cli/tests/fixtures/plain/01_ascii.txt"

echo
echo "CLI cold/warm process loop (100 calls):"
/usr/bin/time -p sh -c "for _ in \$(seq 1 100); do target/release/spoor '$fixture' >/dev/null; done"

echo
echo "CLI concurrent throughput (100 calls, concurrency=8):"
/usr/bin/time -p sh -c "seq 1 100 | xargs -P 8 -I{} target/release/spoor '$fixture' >/dev/null"

echo
echo "Single CLI peak RSS:"
if /usr/bin/time -l true >/dev/null 2>&1; then
  /usr/bin/time -l target/release/spoor "$fixture" >/dev/null
else
  /usr/bin/time -v target/release/spoor "$fixture" >/dev/null
fi
