'use strict';

const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const test = require('node:test');
const { detectFormat, parseBytes } = require('..');

test('detects and parses text bytes', () => {
  const input = Buffer.from('hello spoor\n');
  assert.equal(detectFormat(input, 'note.txt'), 'text');
  const result = parseBytes(input, { sourceName: 'note.txt' });
  assert.equal(result.content.kind, 'document');
  assert.equal(result.content.value.markdown, 'hello spoor\n');
  assert.equal(result.stats.format, 'text');
});

test('exposes stable structured error fields', () => {
  assert.throws(
    () => parseBytes(Buffer.alloc(2048), { format: 'text', maxParseBytes: 1024 }),
    (error) => error.code === 'parse_budget_exceeded'
      && error.stage === 'limits'
      && error.recoverable === true,
  );

  assert.throws(
    () => parseBytes(Buffer.from('not a zip'), { sourceName: 'bad.docx', format: 'docx' }),
    (error) => error.code === 'invalid_container' && error.stage === 'parse',
  );

  const bomb = readFileSync(join(
    __dirname,
    '../../../crates/spoor-cli/tests/fixtures/adversarial/05_compression_bomb.docx',
  ));
  assert.throws(
    () => parseBytes(bomb, { sourceName: 'bomb.docx', format: 'docx', maxParseBytes: 1024 * 1024 }),
    (error) => error.code === 'parse_budget_exceeded' && error.stage === 'limits',
  );

  assert.throws(
    () => parseBytes(Buffer.from('d0cf11e0a1b11ae1', 'hex'), { sourceName: 'encrypted.docx' }),
    (error) => error.code === 'legacy_or_encrypted_office'
      && error.stage === 'detect'
      && error.recoverable === false,
  );
});
