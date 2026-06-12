import { invoke } from '@tauri-apps/api/core';
import './styles.css';

const picker = document.querySelector('#picker');
const dropzone = document.querySelector('#dropzone');
const status = document.querySelector('#status');
const stats = document.querySelector('#stats');
const output = document.querySelector('#output');
const form = document.querySelector('#query');
const question = document.querySelector('#question');
const search = form.querySelector('button');
const copy = document.querySelector('#copy');
const trace = document.querySelector('#trace');

let parsedText = '';
let paragraphs = [];

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
  parseFile(event.dataTransfer.files[0]);
});
picker.addEventListener('change', () => parseFile(picker.files[0]));

form.addEventListener('submit', (event) => {
  event.preventDefault();
  const terms = question.value.toLowerCase().split(/\s+/).filter(Boolean);
  if (!terms.length) {
    output.textContent = parsedText;
    return;
  }
  const ranked = paragraphs
    .map((text) => ({
      text,
      score: terms.reduce((sum, term) => sum + text.toLowerCase().split(term).length - 1, 0),
    }))
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 10);
  output.textContent = ranked.length
    ? ranked.map(({ score, text }) => `[相关度 ${score}]\n${text}`).join('\n\n———\n\n')
    : '没有找到相关段落。';
});

copy.addEventListener('click', async () => {
  await navigator.clipboard.writeText(output.textContent);
  copy.textContent = '已复制';
  setTimeout(() => { copy.textContent = '复制结果'; }, 1200);
});

async function parseFile(file) {
  if (!file) return;

  status.textContent = `解析中 / ${file.name}`;
  stats.textContent = `${file.size.toLocaleString()} 字节 / RUST`;
  output.textContent = '正在调用 parse_document…';
  setTrace([`读取 ${file.name}`, 'invoke parse_document', 'spoor-core 解析'], 1);

  try {
    const result = JSON.parse(await invoke('parse_document', {
      bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
      sourceName: file.name,
      contentType: file.type || null,
    }));
    parsedText = result.content.kind === 'document'
      ? result.content.value.markdown
      : JSON.stringify(result.content.value.tables, null, 2);
    paragraphs = parsedText.split(/\n{2,}/).map((text) => text.trim()).filter(Boolean);
    status.textContent = `完成 / ${file.name}`;
    stats.textContent = `${result.stats.input_bytes.toLocaleString()} 字节 / ${result.stats.format.toUpperCase()}`;
    output.textContent = JSON.stringify(result, null, 2);
    question.disabled = false;
    search.disabled = false;
    copy.disabled = false;
    setTrace([`读取 ${file.name}`, `spoor-core 解析 ${result.stats.format}`, `生成 ${paragraphs.length} 个检索块`]);
  } catch (error) {
    let normalized;
    try {
      normalized = JSON.parse(error);
    } catch {
      normalized = { code: 'parse_failed', message: String(error) };
    }
    status.textContent = `失败 / ${normalized.code}`;
    output.textContent = JSON.stringify(normalized, null, 2);
    setTrace(['读取文档', '解析失败', normalized.code], 2);
  }
}

function setTrace(items, activeIndex = items.length - 1) {
  trace.replaceChildren(...items.map((item, index) => {
    const li = document.createElement('li');
    li.textContent = item;
    li.classList.toggle('active', index === activeIndex);
    return li;
  }));
}
