import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, {
  detect_format,
  extract_media,
  parse_bytes,
} from '../../crates/spoor-wasm/pkg-web/spoor_wasm.js';

const wasm = await readFile(
  new URL('../../crates/spoor-wasm/pkg-web/spoor_wasm_bg.wasm', import.meta.url),
);
await init({ module_or_path: wasm });

const text = new TextEncoder().encode('来自 WASM 的中文文档\n');
assert.equal(detect_format(text, '说明.txt'), 'text');
assert.equal(parse_bytes(text, '说明.txt').content.value.markdown, '来自 WASM 的中文文档\n');

for (const [format, fixture] of [
  ['docx', 'docx/01_basic.docx'],
  ['xlsx', 'xlsx/01_basic.xlsx'],
  ['pdf', 'pdf/01_basic.pdf'],
  ['pptx', 'pptx/01_basic.pptx'],
  ['html', 'html/01_article.html'],
  ['epub', 'epub/01_basic.epub'],
  ['ipynb', 'ipynb/01_basic.ipynb'],
]) {
  const bytes = await readFile(new URL(
    `../../crates/spoor-cli/tests/fixtures/${fixture}`,
    import.meta.url,
  ));
  const result = parse_bytes(bytes, fixture);
  assert.equal(result.stats.format, format);
  assert.ok(result.stats.output_bytes > 0, `${format} should produce output`);
}

const mixedPdf = await readFile(new URL(
  '../../crates/spoor-cli/tests/fixtures/pdf/05_mixed_text_and_image.pdf',
  import.meta.url,
));
const mixedResult = parse_bytes(mixedPdf, 'mixed.pdf');
assert.equal(mixedResult.warnings[0].code, 'pdf_page_no_text_layer');
assert.deepEqual(mixedResult.warnings[0].location, { kind: 'page', number: 2 });

assert.throws(
  () => parse_bytes(new Uint8Array(2048), 'large.bin', undefined, 'text', 1024),
  (error) => error.code === 'parse_budget_exceeded' && error.stage === 'limits',
);

assert.throws(
  () => parse_bytes(new TextEncoder().encode('not a zip'), 'bad.docx', undefined, 'docx'),
  (error) => error.code === 'invalid_container' && error.stage === 'parse',
);

const bomb = await readFile(new URL(
  '../../crates/spoor-cli/tests/fixtures/adversarial/05_compression_bomb.docx',
  import.meta.url,
));
assert.throws(
  () => parse_bytes(bomb, 'bomb.docx', undefined, 'docx', 1024 * 1024),
  (error) => error.code === 'parse_budget_exceeded' && error.stage === 'limits',
);

assert.throws(
  () => parse_bytes(
    Uint8Array.from([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]),
    'encrypted.docx',
  ),
  (error) => error.code === 'legacy_or_encrypted_office'
    && error.stage === 'detect'
    && error.recoverable === false,
);

const imageDocx = await readFile(new URL(
  '../../crates/spoor-cli/tests/fixtures/docx/16_image_placeholders.docx',
  import.meta.url,
));
const media = extract_media(imageDocx, 'spoor-docx://word/media/image1.png', 'images.docx');
assert.ok(media instanceof Uint8Array, 'extract_media returns raw bytes');
assert.equal(new TextDecoder().decode(media), 'first-image');

assert.throws(
  () => extract_media(imageDocx, 'spoor-docx://word/media/../evil.png', 'images.docx'),
  (error) => error.code === 'parse_failed' && error.stage === 'parse',
);

// Table narrowing reaches the WASM host too (csv/01_basic.csv: Alice/Bob/Carol).
// serde_wasm_bindgen serializes row maps as JS Map, so normalize before compare.
const csv = await readFile(new URL(
  '../../crates/spoor-cli/tests/fixtures/csv/01_basic.csv',
  import.meta.url,
));
const asObject = (row) => (row instanceof Map ? Object.fromEntries(row) : row);

// columns=['Name'], limit=1, offset=1 -> just Bob, Name-only.
const filtered = parse_bytes(csv, 'data.csv', undefined, undefined, undefined, undefined, undefined, ['Name'], 1, 1);
assert.equal(filtered.content.value.tables[0].rows.length, 1);
assert.deepEqual(asObject(filtered.content.value.tables[0].rows[0]), { Name: 'Bob' });

// Inclusive 1-based row range selects the same row by its number.
const ranged = parse_bytes(csv, 'data.csv', undefined, undefined, undefined, undefined, [3, 3]);
assert.deepEqual(ranged.content.value.tables[0].rows.map((r) => asObject(r).Name), ['Bob']);

// rows is mutually exclusive with limit/offset (shared TableFilter::build contract).
assert.throws(
  () => parse_bytes(csv, 'data.csv', undefined, undefined, undefined, undefined, [2, 4], undefined, 1),
  (error) => error.code === 'parse_failed',
);

// Page filter reaches the WASM host too (02_multipage.pdf has 3 pages). The
// 11th positional arg is `pages`.
const multipagePdf = await readFile(new URL(
  '../../crates/spoor-cli/tests/fixtures/pdf/02_multipage.pdf',
  import.meta.url,
));
const pageFiltered = parse_bytes(
  multipagePdf, 'doc.pdf', undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined, [2, 2],
);
const pageMd = pageFiltered.content.value.markdown;
assert.ok(
  pageMd.includes('## Page 2') && !pageMd.includes('## Page 1') && !pageMd.includes('## Page 3'),
  pageMd,
);
