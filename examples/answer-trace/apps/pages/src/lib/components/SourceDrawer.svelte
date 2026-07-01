<script lang="ts">
  import { marked } from 'marked'
  import type { Verdict } from '@answer-trace/protocol'

  let {
    open,
    markdown,
    locate,
    status,
    title,
    page,
    onClose,
  }: {
    open: boolean
    markdown: string
    locate: string | null
    status: Verdict
    title: string
    page: number | null
    onClose: () => void
  } = $props()

  marked.setOptions({ gfm: true, breaks: false })

  // 只渲染命中所在那一页(大文档下钻才不卡);无页码则整篇。
  function slicePage(md: string, p: number): string {
    const m = new RegExp(`(?:^|\\n)##[ \\t]*Page[ \\t]+${p}\\b`).exec(md)
    if (!m) return md
    const headStart = m.index + (md[m.index] === '\n' ? 1 : 0)
    const next = md.indexOf('\n## Page ', headStart + 1)
    return md.slice(headStart, next === -1 ? md.length : next)
  }

  const rendered = $derived(page != null && markdown ? slicePage(markdown, page) : markdown)
  const html = $derived(rendered ? (marked.parse(rendered) as string) : '')

  let contentEl = $state<HTMLElement | null>(null)

  // 打开 + 有目标时:等抽屉滑入后,在渲染好的原文里定位命中、高亮、滚动、闪烁。
  $effect(() => {
    if (!open || !locate || !contentEl) return
    const root = contentEl
    const needle = locate
    const cls = status
    const t = setTimeout(() => highlight(root, needle, cls), 300)
    return () => clearTimeout(t)
  })

  function highlight(root: HTMLElement, needle: string, cls: string) {
    // 清掉上次高亮
    root.querySelectorAll('mark.drawer-hl').forEach((m) => {
      m.replaceWith(document.createTextNode(m.textContent ?? ''))
    })
    root.normalize()

    const target = needle.replace(/\s+/g, '')
    if (!target) return

    // 把所有文本节点拼成"无空白串",记录每个字符回到 (node, offset) 的映射。
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT)
    const map: { node: Text; offset: number }[] = []
    let combined = ''
    let node: Node | null
    while ((node = walker.nextNode())) {
      const data = (node as Text).data
      for (let i = 0; i < data.length; i++) {
        if (/\s/.test(data[i])) continue
        map.push({ node: node as Text, offset: i })
        combined += data[i]
      }
    }

    const idx = combined.indexOf(target)
    if (idx === -1) return
    const start = map[idx]
    const end = map[idx + target.length - 1]

    const range = document.createRange()
    range.setStart(start.node, start.offset)
    range.setEnd(end.node, end.offset + 1)

    const mark = document.createElement('mark')
    mark.className = `drawer-hl ${cls}`
    try {
      range.surroundContents(mark)
      mark.scrollIntoView({ behavior: 'smooth', block: 'center' })
      mark.classList.add('flash')
      setTimeout(() => mark.classList.remove('flash'), 1200)
    } catch {
      // 命中跨越元素边界,无法整体包裹 → 退而求其次,滚到起点
      ;(start.node.parentElement ?? root).scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  }
</script>

<div
  class="fixed inset-0 z-50 {open ? 'pointer-events-auto' : 'pointer-events-none'}"
  aria-hidden={!open}
>
  <button
    type="button"
    aria-label="关闭"
    class="absolute inset-0 cursor-default bg-[#1A1D21]/20 transition-opacity duration-200 {open
      ? 'opacity-100'
      : 'opacity-0'}"
    onclick={onClose}
  ></button>

  <aside
    class="absolute top-0 right-0 flex h-full w-full max-w-[680px] flex-col border-l border-[#E7E9EC] bg-white shadow-[0_24px_80px_rgba(16,24,40,0.18)] transition-transform duration-200 ease-out {open
      ? 'translate-x-0'
      : 'translate-x-full'}"
  >
    <div class="flex h-16 shrink-0 items-center justify-between border-b border-[#E7E9EC] px-5">
      <div class="min-w-0">
        <div class="text-[13px] font-semibold text-[#1A1D21]">原文下钻</div>
        <div class="mt-0.5 truncate text-[12px] text-[#5B6370]">
          {title}{page != null ? ` · 第 ${page} 页` : ''} · 命中位置已自动定位
        </div>
      </div>
      <button
        type="button"
        onclick={onClose}
        class="grid h-9 w-9 place-items-center rounded-lg border border-[#E7E9EC] text-[#5B6370] transition hover:bg-[#F7F8FA] hover:text-[#1A1D21]"
        aria-label="关闭">✕</button
      >
    </div>

    <div bind:this={contentEl} class="drawer-md flex-1 overflow-y-auto px-6 py-6">
      <!-- 渲染的是真实 spoor 产物(byd.md 或上传文件经 pyspoor 解析的 markdown) -->
      {@html html}
    </div>
  </aside>
</div>
