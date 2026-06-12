import assert from 'node:assert/strict';
import { handler } from './index.mjs';

const response = await handler({
  filename: 'note.txt',
  body: 'hello lambda\n',
  isBase64Encoded: false,
});

assert.equal(response.statusCode, 200);
assert.equal(response.body, 'hello lambda\n');
