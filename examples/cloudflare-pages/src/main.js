import { parse_bytes, extract_media } from '@harrisonwang/spoor-wasm';
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
const loadSample = $('#load-sample');
const mediaPanel = $('#media-panel');
const mediaGrid = $('#media-grid');
const mediaHint = $('#media-hint');
const modeButtons = [...document.querySelectorAll('.mode')];

// 占位符正则：![DOCX image N](spoor://docx/part/word/media/imageN.png)
const PLACEHOLDER = /spoor:\/\/docx\/part\/[^\s)"']+/g;

let mode = 'local';
let parsedText = '';
let paragraphs = [];
// 留存原始字节：图片提取始终在浏览器本地用 WASM 完成，与解析走本地还是边缘无关
let lastBytes = null;
let lastSource = '';

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
loadSample.addEventListener('click', loadSampleDoc);
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
  resetMedia();
  setTrace([
    `读取 ${file.name}`,
    mode === 'local' ? '浏览器内调用 WASM' : 'POST /api/parse',
    '解析与规范化',
  ], 1);

  try {
    // 字节留在浏览器，解析与图片提取都基于这份内存数据
    lastBytes = new Uint8Array(await file.arrayBuffer());
    lastSource = file.name;
    const result = mode === 'local'
      ? parse_bytes(lastBytes, file.name, file.type || undefined, undefined, 16 * 1024 * 1024)
      : await parseAtEdge(file);

    parsedText = extractText(result);
    paragraphs = parsedText.split(/\n{2,}/).map((value) => value.trim()).filter(Boolean);
    output.textContent = JSON.stringify(result, null, 2);
    renderMedia(parsedText);
    const warningCount = result.warnings?.length ?? 0;
    status.textContent = warningCount
      ? `完成但有 ${warningCount} 条完整性警告 / ${file.name}`
      : `完成 / ${file.name}`;
    stats.textContent = `${file.size.toLocaleString()} 字节 / ${result.stats.format.toUpperCase()}`;
    query.disabled = false;
    search.disabled = false;
    copy.disabled = false;
    setTrace([
      `读取 ${file.name}`,
      mode === 'local' ? '浏览器内调用 WASM' : 'Pages Function 执行 WASM',
      `识别格式 ${result.stats.format}`,
      `生成 ${paragraphs.length} 个可检索块`,
      ...(warningCount ? [`保留 ${warningCount} 条 Agent 完整性警告`] : []),
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

function resetMedia() {
  mediaPanel.hidden = true;
  mediaGrid.replaceChildren();
  mediaHint.textContent = '';
}

// 扫出 markdown 里的 spoor://docx/part/ 占位符，去重后渲染为缩略图按钮，
// 点击时才调用 extract_media 取该图字节——懒取、单资源，与 Agent 用法一致。
function renderMedia(markdown) {
  const uris = [...new Set(markdown.match(PLACEHOLDER) ?? [])];
  if (!uris.length) {
    resetMedia();
    return;
  }
  mediaPanel.hidden = false;
  mediaHint.textContent = `${uris.length} 张内嵌图片 / 点击提取（本地模式走浏览器 WASM，边缘模式走 /api/extract）`;
  mediaGrid.replaceChildren(...uris.map(createMediaCard));
}

function createMediaCard(uri) {
  const card = document.createElement('button');
  card.type = 'button';
  card.className = 'media-card';
  const label = document.createElement('span');
  label.className = 'media-uri';
  label.textContent = uri.replace('spoor://docx/part/word/media/', '');
  const slot = document.createElement('span');
  slot.className = 'media-slot';
  slot.textContent = '点击提取';
  card.append(slot, label);
  card.addEventListener('click', () => extractAndShow(uri, card, slot), { once: false });
  return card;
}

async function extractAndShow(uri, card, slot) {
  if (card.dataset.done === '1') return;
  if (!lastBytes) {
    slot.textContent = '无文档';
    return;
  }
  try {
    // 本地模式：浏览器内 WASM；边缘模式：POST 到 Pages Function 的 /api/extract
    const bytes = mode === 'local'
      ? extract_media(lastBytes, uri, lastSource, undefined, undefined, 16 * 1024 * 1024)
      : await extractAtEdge(uri);
    const url = URL.createObjectURL(new Blob([bytes]));
    const img = document.createElement('img');
    img.src = url;
    img.alt = uri;
    img.addEventListener('load', () => URL.revokeObjectURL(url), { once: true });
    slot.replaceWith(img);
    card.dataset.done = '1';
  } catch (error) {
    slot.textContent = normalizeError(error).code ?? '提取失败';
    card.classList.add('failed');
  }
}

async function extractAtEdge(uri) {
  const response = await fetch(`/api/extract?uri=${encodeURIComponent(uri)}`, {
    method: 'POST',
    headers: { 'content-type': 'application/octet-stream', 'x-filename': lastSource },
    body: lastBytes,
  });
  if (!response.ok) throw await response.json();
  return new Uint8Array(await response.arrayBuffer());
}

async function loadSampleDoc() {
  try {
    const response = await fetch('/sample-image-doc.docx');
    const blob = await response.blob();
    await handle(new File([blob], 'sample-image-doc.docx', {
      type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    }));
  } catch (error) {
    status.textContent = `示例加载失败 / ${normalizeError(error).code ?? '未知错误'}`;
  }
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
