import { parse_bytes } from '@harrisonwang/spoor-wasm';
import './styles.css';

const $ = (selector) => document.querySelector(selector);
const state = {
  records: [],
  selectedId: null,
  filter: 'all',
  nextId: 1,
};

const elements = {
  filePicker: $('#file-picker'),
  folderPicker: $('#folder-picker'),
  loadSample: $('#load-sample'),
  dropzone: $('#dropzone'),
  intakeCount: $('#intake-count'),
  metricReady: $('#metric-ready'),
  metricErrors: $('#metric-errors'),
  metricFormats: $('#metric-formats'),
  formatList: $('#format-list'),
  metricChunks: $('#metric-chunks'),
  metricOutput: $('#metric-output'),
  exportJsonl: $('#export-jsonl'),
  exportManifest: $('#export-manifest'),
  clear: $('#clear'),
  filters: [...document.querySelectorAll('.filter')],
  fileList: $('#file-list'),
  engineState: $('#engine-state'),
  searchForm: $('#search-form'),
  query: $('#query'),
  searchButton: $('#search-form button'),
  searchSummary: $('#search-summary'),
  searchResults: $('#search-results'),
  viewerTitle: $('#viewer-title'),
  viewerMeta: $('#viewer-meta'),
  viewerOutput: $('#viewer-output'),
  copy: $('#copy'),
};

elements.filePicker.addEventListener('change', () => addFiles(elements.filePicker.files));
elements.folderPicker.addEventListener('change', () => addFiles(elements.folderPicker.files));
elements.loadSample.addEventListener('click', loadSample);
elements.clear.addEventListener('click', clearCorpus);
elements.exportJsonl.addEventListener('click', exportJsonl);
elements.exportManifest.addEventListener('click', exportManifest);
elements.searchForm.addEventListener('submit', runSearch);
elements.copy.addEventListener('click', copySelected);

for (const eventName of ['dragenter', 'dragover']) {
  elements.dropzone.addEventListener(eventName, (event) => {
    event.preventDefault();
    elements.dropzone.classList.add('active');
  });
}

for (const eventName of ['dragleave', 'drop']) {
  elements.dropzone.addEventListener(eventName, () => elements.dropzone.classList.remove('active'));
}

elements.dropzone.addEventListener('drop', (event) => {
  event.preventDefault();
  addFiles(event.dataTransfer.files);
});

for (const button of elements.filters) {
  button.addEventListener('click', () => {
    state.filter = button.dataset.filter;
    for (const candidate of elements.filters) {
      candidate.classList.toggle('active', candidate === button);
    }
    renderRegistry();
  });
}

async function addFiles(fileList) {
  const files = [...fileList];
  if (!files.length) return;

  const records = files.map((file) => ({
    id: state.nextId++,
    file,
    name: file.webkitRelativePath || file.name,
    status: 'queued',
    format: null,
    result: null,
    text: '',
    chunks: [],
    error: null,
  }));
  state.records.push(...records);
  render();

  for (const record of records) {
    record.status = 'parsing';
    elements.engineState.textContent = `正在解析 ${record.name}`;
    renderRegistry();
    await nextFrame();
    try {
      const result = parse_bytes(
        new Uint8Array(await record.file.arrayBuffer()),
        record.name,
        record.file.type || undefined,
        undefined,
        16 * 1024 * 1024,
      );
      record.result = result;
      record.format = result.stats.format;
      record.text = extractText(result);
      record.chunks = buildChunks(record);
      record.status = 'ready';
      if (state.selectedId === null) state.selectedId = record.id;
    } catch (error) {
      record.error = normalizeError(error);
      record.status = 'error';
    }
    render();
  }

  elements.engineState.textContent = 'WASM 已就绪';
  render();
}

function extractText(result) {
  if (result.content.kind === 'document') return result.content.value.markdown;
  return result.content.value.tables
    .flatMap((table) => [
      `# ${table.sheet || table.source || 'Table'}`,
      ...table.rows.map((row) => JSON.stringify(row)),
    ])
    .join('\n\n');
}

function buildChunks(record) {
  if (record.result.content.kind === 'tables') {
    return record.result.content.value.tables.flatMap((table, tableIndex) => {
      const tableMetadata = {
        table_index: tableIndex,
        sheet: table.sheet || null,
        range: table.range || null,
        headers: table.headers,
        truncated: table.truncated,
        warnings: table.warnings,
      };
      return [{
        source: record.name,
        format: record.format,
        kind: 'table_schema',
        index: tableIndex,
        ...tableMetadata,
        text: JSON.stringify(tableMetadata),
      }, ...table.rows.map((row, rowIndex) => ({
        source: record.name,
        format: record.format,
        kind: 'table_row',
        index: rowIndex,
        table_index: tableIndex,
        sheet: table.sheet || null,
        row: table.row_range?.first ? table.row_range.first + rowIndex : null,
        text: JSON.stringify(row),
      }))];
    });
  }

  return record.text
    .split(/\n{2,}/)
    .map((text) => text.trim())
    .filter(Boolean)
    .map((text, index) => ({
      source: record.name,
      format: record.format,
      kind: 'paragraph',
      index,
      text,
    }));
}

function render() {
  renderMetrics();
  renderRegistry();
  renderViewer();
  updateControls();
  if (elements.query.value.trim()) renderSearch(elements.query.value);
}

function renderMetrics() {
  const ready = state.records.filter((record) => record.status === 'ready');
  const errors = state.records.filter((record) => record.status === 'error');
  const formats = [...new Set(ready.map((record) => record.format))].sort();
  const chunkCount = ready.reduce((total, record) => total + record.chunks.length, 0);
  const outputBytes = ready.reduce((total, record) => total + record.result.stats.output_bytes, 0);

  elements.intakeCount.textContent = String(state.records.length).padStart(2, '0');
  elements.metricReady.textContent = ready.length;
  elements.metricErrors.textContent = `${errors.length} 个解析错误`;
  elements.metricFormats.textContent = formats.length;
  elements.formatList.textContent = formats.length ? formats.join(' / ') : '等待添加文档';
  elements.metricChunks.textContent = chunkCount.toLocaleString();
  elements.metricOutput.textContent = `已抽取 ${formatBytes(outputBytes)}`;
}

function renderRegistry() {
  const visible = state.records.filter((record) =>
    state.filter === 'all' ? true : record.status === state.filter,
  );
  if (!visible.length) {
    elements.fileList.replaceChildren(emptyItem(
      state.records.length ? '当前筛选条件下没有文件。' : '尚未添加文档。',
    ));
    return;
  }

  elements.fileList.replaceChildren(...visible.map((record) => {
    const item = document.createElement('li');
    const button = document.createElement('button');
    button.type = 'button';
    button.className = `file-card ${record.status}`;
    button.classList.toggle('selected', record.id === state.selectedId);
    button.addEventListener('click', () => {
      state.selectedId = record.id;
      renderRegistry();
      renderViewer();
    });

    const heading = document.createElement('span');
    heading.className = 'file-name';
    heading.textContent = record.name;
    const meta = document.createElement('span');
    meta.className = 'file-meta';
    meta.textContent = recordMeta(record);
    const status = document.createElement('span');
    status.className = 'file-status';
    status.textContent = statusLabel(record.status);
    button.append(heading, meta, status);
    item.append(button);
    return item;
  }));
}

function renderViewer() {
  const record = state.records.find((candidate) => candidate.id === state.selectedId);
  if (!record) {
    elements.viewerTitle.textContent = '尚未选择';
    elements.viewerMeta.replaceChildren();
    elements.viewerOutput.textContent = '选择已解析文件后查看规范化输出。';
    return;
  }

  elements.viewerTitle.textContent = record.name;
  elements.viewerMeta.replaceChildren(
    badge(statusLabel(record.status)),
    badge(record.format || '未知格式'),
    badge(formatBytes(record.file.size)),
    badge(`${record.chunks.length} 个记录`),
  );
  elements.viewerOutput.textContent = record.status === 'ready'
    ? record.text.slice(0, 30000)
    : JSON.stringify(record.error, null, 2);
}

function runSearch(event) {
  event.preventDefault();
  renderSearch(elements.query.value);
}

function renderSearch(query) {
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (!terms.length) {
    elements.searchSummary.textContent = '输入一个或多个关键词，检索完整本地语料库。';
    elements.searchResults.replaceChildren();
    return;
  }

  const matches = state.records
    .filter((record) => record.status === 'ready')
    .flatMap((record) => record.chunks)
    .map((chunk) => ({
      chunk,
      score: terms.reduce((total, term) => total + occurrences(chunk.text.toLowerCase(), term), 0),
    }))
    .filter(({ score }) => score > 0)
    .sort((left, right) => right.score - left.score || left.chunk.source.localeCompare(right.chunk.source))
    .slice(0, 24);

  elements.searchSummary.textContent = matches.length
    ? `在 ${new Set(matches.map(({ chunk }) => chunk.source)).size} 个文件中找到 ${matches.length} 条最相关结果`
    : '在已解析语料库中没有找到匹配内容。';
  elements.searchResults.replaceChildren(...matches.map(({ chunk, score }) => searchResult(chunk, score)));
}

function searchResult(chunk, score) {
  const item = document.createElement('li');
  const header = document.createElement('div');
  const source = document.createElement('strong');
  source.textContent = chunk.source;
  const count = document.createElement('span');
  count.textContent = `相关度 ${score} / ${chunk.kind.replace('_', ' ')}`;
  header.append(source, count);
  const text = document.createElement('p');
  text.textContent = chunk.text.slice(0, 700);
  item.append(header, text);
  return item;
}

function updateControls() {
  const hasRecords = state.records.length > 0;
  const hasReady = state.records.some((record) => record.status === 'ready');
  const hasSelection = state.records.some((record) => record.id === state.selectedId);
  elements.clear.disabled = !hasRecords;
  elements.exportJsonl.disabled = !hasReady;
  elements.exportManifest.disabled = !hasRecords;
  elements.query.disabled = !hasReady;
  elements.searchButton.disabled = !hasReady;
  elements.copy.disabled = !hasSelection;
  if (!hasReady) {
    elements.searchResults.replaceChildren();
    elements.searchSummary.textContent = hasRecords
      ? '等待文档解析成功。'
      : '添加文档或加载中文样例后开始检索。';
  }
}

function loadSample() {
  addFiles([
    new File([
      '# 北港改造项目现场报告\n\n港区改造计划的目标是把船舶平均等待时间降低 22%。\n\n## 主要风险\n\n审批仍在关键路径上，东侧泊位需要补充第二轮环境评估。\n\n## 行动项\n\n周五前确认审批负责人，并发布修订后的施工顺序。',
    ], '北港现场报告.md', { type: 'text/markdown' }),
    new File([
      '负责人,工作流,状态,截止日期\n米娜,审批,存在风险,2026-06-19\n西奥,施工,按计划,2026-07-03\n艾瑞丝,运营,按计划,2026-06-26\n',
    ], '交付登记表.csv', { type: 'text/csv' }),
    new File([
      '<article><h1>北港项目简报</h1><p>改造计划在提高泊位吞吐量的同时保障本地就业。</p><h2>待决策事项</h2><p>进场施工前，为东侧泊位补充环境评估预算。</p></article>',
    ], '项目简报.html', { type: 'text/html' }),
  ]);
}

function clearCorpus() {
  state.records = [];
  state.selectedId = null;
  elements.filePicker.value = '';
  elements.folderPicker.value = '';
  elements.query.value = '';
  render();
}

function exportJsonl() {
  const lines = state.records
    .filter((record) => record.status === 'ready')
    .flatMap((record) => record.chunks)
    .map((chunk) => JSON.stringify(chunk));
  download('spoor-corpus.jsonl', `${lines.join('\n')}\n`, 'application/x-ndjson');
}

function exportManifest() {
  const manifest = state.records.map((record) => ({
    source: record.name,
    status: record.status,
    format: record.format,
    input_bytes: record.file.size,
    output_bytes: record.result?.stats.output_bytes ?? null,
    chunks: record.chunks.length,
    error: record.error,
  }));
  download('spoor-corpus-manifest.json', `${JSON.stringify(manifest, null, 2)}\n`, 'application/json');
}

async function copySelected() {
  await navigator.clipboard.writeText(elements.viewerOutput.textContent);
  elements.copy.textContent = '已复制';
  setTimeout(() => { elements.copy.textContent = '复制'; }, 1200);
}

function recordMeta(record) {
  if (record.status === 'ready') {
    return `${record.format} / ${record.chunks.length} 个记录 / ${formatBytes(record.file.size)}`;
  }
  if (record.status === 'error') return record.error.code || 'unknown_error';
  return `${formatBytes(record.file.size)} / ${record.status}`;
}

function emptyItem(message) {
  const item = document.createElement('li');
  item.className = 'empty-state';
  item.textContent = message;
  return item;
}

function badge(text) {
  const element = document.createElement('span');
  element.textContent = text;
  return element;
}

function statusLabel(status) {
  return {
    queued: '等待解析',
    parsing: '解析中',
    ready: '解析成功',
    error: '解析失败',
  }[status] || status;
}

function occurrences(text, term) {
  return text.split(term).length - 1;
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MiB`;
}

function normalizeError(error) {
  if (error && typeof error === 'object') return error;
  return { code: 'unknown_error', reason: String(error) };
}

function download(filename, content, type) {
  const link = document.createElement('a');
  link.href = URL.createObjectURL(new Blob([content], { type }));
  link.download = filename;
  link.click();
  URL.revokeObjectURL(link.href);
}

function nextFrame() {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}

render();
