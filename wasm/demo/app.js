import init, { parse_bytes } from '../../crates/spoor-wasm/pkg-web/spoor_wasm.js';

const dropzone = document.querySelector('#dropzone');
const picker = document.querySelector('#picker');
const output = document.querySelector('#output');
const status = document.querySelector('#status');
const stats = document.querySelector('#stats');

await init();
status.textContent = 'WASM ENGINE / READY';

for (const eventName of ['dragenter', 'dragover']) {
  dropzone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropzone.classList.add('active');
  });
}

for (const eventName of ['dragleave', 'drop']) {
  dropzone.addEventListener(eventName, () => dropzone.classList.remove('active'));
}

dropzone.addEventListener('drop', (event) => {
  event.preventDefault();
  handle(event.dataTransfer.files[0]);
});
picker.addEventListener('change', () => handle(picker.files[0]));

async function handle(file) {
  if (!file) return;
  status.textContent = `PARSING / ${file.name.toUpperCase()}`;
  stats.textContent = `${file.size.toLocaleString()} bytes`;
  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const result = parse_bytes(bytes, file.name, file.type || undefined, undefined, undefined);
    output.textContent = JSON.stringify(result, null, 2);
    status.textContent = `DONE / ${result.stats.format.toUpperCase()}`;
  } catch (error) {
    output.textContent = JSON.stringify(error, null, 2) || String(error);
    status.textContent = 'FAILED / SEE ERROR';
  }
}
