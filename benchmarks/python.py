#!/usr/bin/env python3
"""Compare warm PyO3, one-process-per-file CLI, and long-lived worker IPC."""

from __future__ import annotations

import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from spoor import parse_bytes

ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "crates/spoor-cli/tests/fixtures/plain/01_ascii.txt"
CLI = Path(os.environ.get("SPOOR_BIN", ROOT / "target/release/spoor"))
WARM_ITERATIONS = 10_000
PROCESS_ITERATIONS = 100
WORKER_ITERATIONS = 1_000


def ns_per_call(elapsed: float, iterations: int) -> int:
    return round(elapsed * 1_000_000_000 / iterations)


data = FIXTURE.read_bytes()

start = time.perf_counter()
for _ in range(WARM_ITERATIONS):
    parse_bytes(data, source_name=FIXTURE.name)
warm = time.perf_counter() - start

start = time.perf_counter()
for _ in range(PROCESS_ITERATIONS):
    subprocess.run([CLI, FIXTURE], stdout=subprocess.DEVNULL, check=True)
process = time.perf_counter() - start

worker = subprocess.Popen(
    [sys.executable, ROOT / "benchmarks/python_worker.py"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
)
assert worker.stdin is not None
assert worker.stdout is not None
request = json.dumps(
    {
        "data": base64.b64encode(data).decode(),
        "source_name": FIXTURE.name,
    }
)
start = time.perf_counter()
for _ in range(WORKER_ITERATIONS):
    worker.stdin.write(request + "\n")
    worker.stdin.flush()
    response = json.loads(worker.stdout.readline())
    assert response == {"ok": True, "format": "text"}
ipc = time.perf_counter() - start
worker.stdin.close()
worker.wait(timeout=5)

print(f"PyO3 warm: {ns_per_call(warm, WARM_ITERATIONS):,} ns/call")
print(
    "CLI per-process: "
    f"{ns_per_call(process, PROCESS_ITERATIONS) / 1_000_000:,.3f} ms/call"
)
print(
    "Long-lived worker IPC: "
    f"{ns_per_call(ipc, WORKER_ITERATIONS) / 1_000:,.3f} us/call"
)
