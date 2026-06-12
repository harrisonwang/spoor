'use strict';

const assert = require('node:assert/strict');
const { parseBytes } = require('@harrisonwang/spoor');

const result = parseBytes(Buffer.from('来自 Electron 的中文文档\n'), {
  sourceName: '说明.txt',
});

assert.equal(result.stats.format, 'text');
assert.equal(result.content.value.markdown, '来自 Electron 的中文文档\n');
console.log('Electron 原生 binding smoke test passed.');
