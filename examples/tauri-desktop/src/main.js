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
const mediaPanel = document.querySelector('#media-panel');
const mediaGrid = document.querySelector('#media-grid');
const mediaHint = document.querySelector('#media-hint');

// 占位符正则：![DOCX image N](spoor://docx/part/word/media/imageN.png)
const PLACEHOLDER = /spoor:\/\/docx\/part\/[^\s)"']+/g;

let parsedText = '';
let paragraphs = [];
// 留存当前文档的字节与文件名，供按需提取图片复用
let lastBytes = null;
let lastSource = '';
let lastContentType = null;

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
  resetMedia();
  setTrace([`读取 ${file.name}`, 'invoke parse_document', 'spoor-core 解析'], 1);

  try {
    lastBytes = new Uint8Array(await file.arrayBuffer());
    lastSource = file.name;
    lastContentType = file.type || null;
    const result = JSON.parse(await invoke('parse_document', {
      bytes: Array.from(lastBytes),
      sourceName: lastSource,
      contentType: lastContentType,
    }));
    parsedText = result.content.kind === 'document'
      ? result.content.value.markdown
      : JSON.stringify(result.content.value.tables, null, 2);
    paragraphs = parsedText.split(/\n{2,}/).map((text) => text.trim()).filter(Boolean);
    status.textContent = `完成 / ${file.name}`;
    stats.textContent = `${result.stats.input_bytes.toLocaleString()} 字节 / ${result.stats.format.toUpperCase()}`;
    output.textContent = JSON.stringify(result, null, 2);
    renderMedia(parsedText);
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

function resetMedia() {
  mediaPanel.hidden = true;
  mediaGrid.replaceChildren();
  mediaHint.textContent = '';
}

// 扫出 spoor://docx/part/ 占位符，去重后列成缩略图；点击才 invoke 提取该图字节——
// 懒取、单资源，与 Agent 只取相关图的用法一致。
function renderMedia(markdown) {
  const uris = [...new Set(markdown.match(PLACEHOLDER) ?? [])];
  if (!uris.length) {
    resetMedia();
    return;
  }
  mediaPanel.hidden = false;
  mediaHint.textContent = `${uris.length} 张内嵌图片 / 点击经 extract_document_media 提取`;
  mediaGrid.replaceChildren(...uris.map(createMediaCard));
}

function createMediaCard(uri) {
  const card = document.createElement('button');
  card.type = 'button';
  card.className = 'media-card';
  const slot = document.createElement('span');
  slot.className = 'media-slot';
  slot.textContent = '点击提取';
  const label = document.createElement('span');
  label.className = 'media-uri';
  label.textContent = uri.replace('spoor://docx/part/word/media/', '');
  card.append(slot, label);
  card.addEventListener('click', () => extractAndShow(uri, card, slot));
  return card;
}

async function extractAndShow(uri, card, slot) {
  if (card.dataset.done === '1' || !lastBytes) return;
  try {
    // command 返回 tauri::ipc::Response，invoke 直接拿到 ArrayBuffer（二进制 IPC）
    const buffer = await invoke('extract_document_media', {
      bytes: Array.from(lastBytes),
      sourceName: lastSource,
      resource: uri,
      contentType: lastContentType,
    });
    const url = URL.createObjectURL(new Blob([buffer]));
    const img = document.createElement('img');
    img.src = url;
    img.alt = uri;
    img.addEventListener('load', () => URL.revokeObjectURL(url), { once: true });
    slot.replaceWith(img);
    card.dataset.done = '1';
  } catch (error) {
    let code = '提取失败';
    try { code = JSON.parse(error).code ?? code; } catch { /* 非 JSON 错误 */ }
    slot.textContent = code;
    card.classList.add('failed');
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
