import { parse_bytes } from '@harrisonwang/spoor-wasm';
import './styles.css';

const $ = (selector) => document.querySelector(selector);
const picker = $('#picker');
const dropzone = $('#dropzone');
const output = $('#output');
const status = $('#status');
const stats = $('#stats');
const query = $('#query');
const search = $('#search');
const copy = $('#copy');
const privacy = $('#privacy');
const runtimeStatus = $('#runtime-status');
const trace = $('#trace');
const modeButtons = [...document.querySelectorAll('.mode')];

let mode = 'local';
let parsedText = '';
let paragraphs = [];

runtimeStatus.textContent = 'WASM 已就绪 / 边缘函数已就绪';
setTrace(['WASM 模块已装载', '等待文档']);

for (const button of modeButtons) {
  button.addEventListener('click', () => {
    mode = button.dataset.mode;
    for (const candidate of modeButtons) {
      const active = candidate === button;
      candidate.classList.toggle('active', active);
      candidate.setAttribute('aria-checked', String(active));
    }
    privacy.textContent = mode === 'local' ? '本地 / 不上传' : '边缘 / 加密传输';
  });
}

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
search.addEventListener('click', runSearch);
query.addEventListener('keydown', (event) => {
  if (event.key === 'Enter') runSearch();
});
copy.addEventListener('click', async () => {
  await navigator.clipboard.writeText(output.textContent);
  copy.textContent = '已复制';
  setTimeout(() => { copy.textContent = '复制结果'; }, 1400);
});

async function handle(file) {
  if (!file) return;

  status.textContent = `${mode === 'local' ? '本地解析' : '边缘解析'} / ${file.name}`;
  stats.textContent = `${file.size.toLocaleString()} 字节 / 正在识别`;
  output.textContent = '解析中…';
  query.disabled = true;
  search.disabled = true;
  copy.disabled = true;
  setTrace([
    `读取 ${file.name}`,
    mode === 'local' ? '浏览器内调用 WASM' : 'POST /api/parse',
    '解析与规范化',
  ], 1);

  try {
    const result = mode === 'local'
      ? parse_bytes(
          new Uint8Array(await file.arrayBuffer()),
          file.name,
          file.type || undefined,
          undefined,
          16 * 1024 * 1024,
        )
      : await parseAtEdge(file);

    parsedText = extractText(result);
    paragraphs = parsedText.split(/\n{2,}/).map((value) => value.trim()).filter(Boolean);
    output.textContent = JSON.stringify(result, null, 2);
    status.textContent = `完成 / ${file.name}`;
    stats.textContent = `${file.size.toLocaleString()} 字节 / ${result.stats.format.toUpperCase()}`;
    query.disabled = false;
    search.disabled = false;
    copy.disabled = false;
    setTrace([
      `读取 ${file.name}`,
      mode === 'local' ? '浏览器内调用 WASM' : 'Pages Function 执行 WASM',
      `识别格式 ${result.stats.format}`,
      `生成 ${paragraphs.length} 个可检索块`,
    ]);
  } catch (error) {
    const normalized = normalizeError(error);
    output.textContent = JSON.stringify(normalized, null, 2);
    status.textContent = `失败 / ${normalized.code ?? '未知错误'}`;
    stats.textContent = `${file.size.toLocaleString()} 字节 / 失败`;
    setTrace(['读取文档', '解析失败', normalized.code ?? 'unknown_error'], 2);
  }
}

async function parseAtEdge(file) {
  const response = await fetch('/api/parse', {
    method: 'POST',
    headers: {
      'content-type': file.type || 'application/octet-stream',
      'x-filename': file.name,
    },
    body: file,
  });
  const payload = await response.json();
  if (!response.ok) throw payload;
  return payload;
}

function runSearch() {
  const terms = query.value.toLowerCase().split(/\s+/).filter(Boolean);
  if (!terms.length) {
    output.textContent = parsedText;
    return;
  }
  const ranked = paragraphs
    .map((text) => ({
      text,
      score: terms.reduce(
        (total, term) => total + text.toLowerCase().split(term).length - 1,
        0,
      ),
    }))
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 8);

  output.textContent = ranked.length
    ? ranked.map(({ text, score }) => `[相关度 ${score}]\n${text}`).join('\n\n———\n\n')
    : '没有找到相关段落。';
}

function extractText(result) {
  return result.content.kind === 'document'
    ? result.content.value.markdown
    : JSON.stringify(result.content.value.tables, null, 2);
}

function normalizeError(error) {
  if (error && typeof error === 'object') return error;
  return { code: 'unknown_error', message: String(error) };
}

function setTrace(items, activeIndex = items.length - 1) {
  trace.replaceChildren(...items.map((item, index) => {
    const li = document.createElement('li');
    li.textContent = item;
    li.classList.toggle('active', index === activeIndex);
    return li;
  }));
}
