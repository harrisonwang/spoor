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
