# pyspoor

Typed Python adapter for `spoor-core`.

```python
from spoor import parse_path

result = parse_path("report.pdf")
print(result.content.value.markdown)
for warning in result.warnings:
    print(warning["code"], warning.get("location"))
```

Agents must inspect `result.warnings`; a successful parse can still report a
missing PDF text layer, suspicious text, merged-table degradation, or omitted
visuals.

Build locally with `maturin develop`.
