# pyspoor

Typed Python adapter for `spoor-core`.

```python
from spoor import parse_path

result = parse_path("report.pdf")
print(result.content.value.markdown)
```

Build locally with `maturin develop`.
