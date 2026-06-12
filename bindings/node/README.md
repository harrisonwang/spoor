# @harrisonwang/spoor

Native Node.js bindings for the spoor document engine.

```js
const { parseBytes } = require('@harrisonwang/spoor');

const result = parseBytes(Buffer.from('hello\n'), { sourceName: 'note.txt' });
```

Native addons are distributed as optional platform packages. Thrown spoor
errors expose `code`, `reason`, `hint`, `recoverable`, and `stage`.
