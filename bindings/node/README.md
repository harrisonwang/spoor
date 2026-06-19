# @harrisonwang/spoor

Native Node.js bindings for the spoor document engine.

```js
const { parseBytes } = require('@harrisonwang/spoor');

const result = parseBytes(Buffer.from('hello\n'), { sourceName: 'note.txt' });
for (const warning of result.warnings) {
  console.warn(warning.code, warning.location);
}
```

Native addons are distributed as optional platform packages. Thrown spoor
errors expose `code`, `reason`, `hint`, `recoverable`, and `stage`.
Successful parses can still contain typed integrity warnings; agents must
inspect `result.warnings`.

## Tables: narrowing and pagination

For CSV/XLSX, `parseBytes` accepts the same narrowing options as the CLI, so
pipelines can page through full tables instead of the default 100-row preview:

```js
// A slice by inclusive 1-based [first, last] row range (mutually exclusive with limit/offset)
parseBytes(data, { sourceName: 'data.xlsx', sheet: 'Sheet1', rows: [5, 104] });

// Or paginate by limit/offset and keep only some columns
parseBytes(data, { sourceName: 'data.xlsx', columns: ['分类', '金额'], limit: 100, offset: 200 });
```

## PDFs: page ranges

For page-oriented PDFs, pass `pages` (inclusive 1-based) to parse only a slice
and avoid reading a large document end to end:

```js
parseBytes(data, { sourceName: 'report.pdf', pages: [1, 3] }); // only pages 1–3
```

## Extracting embedded media

Resolve a safe media URI emitted in the output (DOCX images, extractable PDF
images) to a `Buffer` for handing to an external vision model:

```js
const { extractMedia } = require('@harrisonwang/spoor');

const image = extractMedia(data, 'spoor-docx://word/media/image1.png', { sourceName: 'report.docx' });
```
