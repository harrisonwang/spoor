import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, {
  detect_format,
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
