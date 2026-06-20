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

## Tables: narrowing and pagination

For CSV/XLSX, `parse_bytes` / `parse_path` accept the same narrowing options as
the CLI, so callers can page through full tables instead of the default
100-row preview:

```python
from spoor import parse_bytes

# A slice by inclusive 1-based row range (mutually exclusive with limit/offset)
parse_bytes(data, source_name="data.xlsx", sheet="Sheet1", rows=(5, 104))

# Or paginate by limit/offset and keep only some columns
parse_bytes(data, source_name="data.xlsx", columns=["分类", "金额"], limit=100, offset=200)
```

## PDFs: page ranges

For page-oriented PDFs, pass `pages` (inclusive 1-based) to parse only a slice
and avoid reading a large document end to end:

```python
parse_path("report.pdf", pages=(1, 3))  # only pages 1–3
```

## Extracting embedded media

Resolve a safe media URI emitted in the output (DOCX images, extractable PDF
images) to raw bytes for handing to an external vision model:

```python
from spoor import extract_media

image = extract_media(data, "spoor-docx://word/media/image1.png", source_name="report.docx")
```

Build locally with `maturin develop`.
