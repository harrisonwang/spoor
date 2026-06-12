from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any


@dataclass(slots=True)
class SpoorError(Exception):
    code: str
    reason: str
    hint: str
    recoverable: bool
    stage: str | None = None

    def __str__(self) -> str:
        return f"{self.code}: {self.reason}"

    @classmethod
    def from_native(cls, error: Exception) -> "SpoorError":
        payload: dict[str, Any] = json.loads(str(error))
        return cls(
            code=payload["code"],
            reason=payload["reason"],
            hint=payload["hint"],
            recoverable=payload["recoverable"],
            stage=payload.get("stage"),
        )
