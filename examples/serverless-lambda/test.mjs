import assert from 'node:assert/strict';
import { handler } from './index.mjs';

const response = await handler({
  filename: '说明.txt',
  body: '来自 Lambda 的中文文档\n',
  isBase64Encoded: false,
});

assert.equal(response.statusCode, 200);
assert.equal(response.body, '来自 Lambda 的中文文档\n');
