import init, { parse_bytes } from '../../crates/spoor-wasm/pkg-web/spoor_wasm.js';

await init();
const picker = document.querySelector('#picker');
const status = document.querySelector('#status');
const output = document.querySelector('#output');
const form = document.querySelector('#query');
const question = document.querySelector('#question');
const button = form.querySelector('button');
let paragraphs = [];

picker.addEventListener('change', async () => {
  const file = picker.files[0];
  if (!file) return;
  const result = parse_bytes(new Uint8Array(await file.arrayBuffer()), file.name);
  const text = result.content.kind === 'document'
    ? result.content.value.markdown
    : JSON.stringify(result.content.value.tables);
  paragraphs = text.split(/\n{2,}/).filter(Boolean);
  status.textContent = `${file.name.toUpperCase()} / ${paragraphs.length} BLOCKS`;
  question.disabled = false;
  button.disabled = false;
  output.textContent = text.slice(0, 8000);
});

form.addEventListener('submit', (event) => {
  event.preventDefault();
  const terms = question.value.toLowerCase().split(/\s+/).filter(Boolean);
  const ranked = paragraphs
    .map((text) => ({ text, score: terms.reduce((n, term) => n + text.toLowerCase().split(term).length - 1, 0) }))
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 8);
  output.textContent = ranked.length
    ? ranked.map(({ text, score }) => `[score ${score}]\n${text}`).join('\n\n---\n\n')
    : '没有找到相关段落。';
});
