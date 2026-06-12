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

const text = new TextEncoder().encode('hello wasm\n');
assert.equal(detect_format(text, 'note.txt'), 'text');
assert.equal(parse_bytes(text, 'note.txt').content.value.markdown, 'hello wasm\n');

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
