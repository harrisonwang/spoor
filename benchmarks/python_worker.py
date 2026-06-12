#!/usr/bin/env python3
"""Long-lived pyspoor worker used only by the benchmark harness."""

from __future__ import annotations

import base64
import json
import sys

from spoor import SpoorError, parse_bytes


for line in sys.stdin:
    request = json.loads(line)
    try:
        result = parse_bytes(
            base64.b64decode(request["data"]),
            source_name=request.get("source_name"),
        )
        response = {"ok": True, "format": result.stats.format}
    except SpoorError as error:
        response = {"ok": False, "code": error.code}
    print(json.dumps(response), flush=True)
