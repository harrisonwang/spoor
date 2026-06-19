'use strict';

const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const test = require('node:test');
const { detectFormat, parseBytes, extractMedia } = require('..');

test('detects and parses text bytes', () => {
  const input = Buffer.from('hello spoor\n');
  assert.equal(detectFormat(input, 'note.txt'), 'text');
  const result = parseBytes(input, { sourceName: 'note.txt' });
  assert.equal(result.content.kind, 'document');
  assert.equal(result.content.value.markdown, 'hello spoor\n');
  assert.equal(result.stats.format, 'text');
});

test('exposes stable warning code and location', () => {
  const mixedPdf = readFileSync(join(
    __dirname,
    '../../../crates/spoor-cli/tests/fixtures/pdf/05_mixed_text_and_image.pdf',
  ));
  const result = parseBytes(mixedPdf, { sourceName: 'mixed.pdf' });

  assert.equal(result.warnings[0].code, 'pdf_page_no_text_layer');
  assert.deepEqual(result.warnings[0].location, { kind: 'page', number: 2 });
});

test('table filter paginates and selects columns', () => {
  const csv = readFileSync(join(
    __dirname,
    '../../../crates/spoor-cli/tests/fixtures/csv/01_basic.csv',
  ));
  // Alice(row 2), Bob(row 3), Carol(row 4); columns Name/Score/Note.
  const result = parseBytes(csv, { sourceName: 'data.csv', columns: ['Name'], limit: 1, offset: 1 });
  assert.equal(result.content.kind, 'tables');
  assert.deepEqual(result.content.value.tables[0].rows, [{ Name: 'Bob' }]);

  const ranged = parseBytes(csv, { sourceName: 'data.csv', rows: [3, 3] });
  assert.deepEqual(ranged.content.value.tables[0].rows.map((r) => r.Name), ['Bob']);
});

test('table filter rejects rows combined with limit', () => {
  const csv = readFileSync(join(
    __dirname,
    '../../../crates/spoor-cli/tests/fixtures/csv/01_basic.csv',
  ));
  assert.throws(
    () => parseBytes(csv, { sourceName: 'data.csv', rows: [2, 4], limit: 1 }),
    (error) => error.code === 'parse_failed',
  );
});

test('pages filter limits PDF to requested pages', () => {
  const pdf = readFileSync(join(
    __dirname,
    '../../../crates/spoor-cli/tests/fixtures/pdf/02_multipage.pdf',
  ));
  const md = parseBytes(pdf, { sourceName: 'doc.pdf', pages: [2, 2] }).content.value.markdown;
  assert.ok(md.includes('## Page 2'), md);
  assert.ok(!md.includes('## Page 1'), md);
  assert.ok(!md.includes('## Page 3'), md);

  assert.throws(
    () => parseBytes(pdf, { sourceName: 'doc.pdf', pages: [3, 1] }),
    (error) => error.code === 'parse_failed',
  );
});

test('extract_media returns safe docx resource bytes', () => {
  const docx = readFileSync(join(
    __dirname,
    '../../../crates/spoor-cli/tests/fixtures/docx/16_image_placeholders.docx',
  ));
  const image = extractMedia(docx, 'spoor-docx://word/media/image1.png', { sourceName: 'images.docx' });
  assert.equal(Buffer.from(image).toString(), 'first-image');

  assert.throws(
    () => extractMedia(docx, 'word/media/image1.png', { sourceName: 'images.docx' }),
    (error) => error.code === 'parse_failed',
  );
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
