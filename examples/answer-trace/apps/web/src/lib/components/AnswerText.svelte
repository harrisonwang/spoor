<script lang="ts">
  import { mount, unmount } from 'svelte'
  import { marked } from 'marked'
  import { autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom'
  import type { AnswerPart, Evidence, Verdict } from '@answer-trace/protocol'
  import { STATUS } from '$lib/status'
  import ClaimPopover from './ClaimPopover.svelte'

  let {
    parts,
    evidence,
    onOpenSource,
  }: {
    parts: AnswerPart[]
    evidence: Evidence[]
    onOpenSource?: (evidence: Evidence) => void
  } = $props()

  const evMap = $derived(new Map(evidence.map((e) => [e.id, e])))

  // 用控制字符 U+0001..0003 作 sentinel 包住每个 claim：marked 当普通文本、不转义，
  // 整篇渲染后再用正则把 sentinel 换成可交互 span。控制字符在真实文本里几乎不会出现。
  const OPEN = String.fromCharCode(1)
  const SEP = String.fromCharCode(2)
  const CLOSE = String.fromCharCode(3)

  // 源码里转义 <>& 挡住 LLM 可能的原始 HTML（marked 的 escape 识别已有实体，不会二次转义）。
  const esc = (t: string) => t.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')

  // 把 claim 前导的块级标记（列表符/标题号/引用/换行）留在 sentinel 外，避免破坏 marked 的块解析。
  function claimToken(eid: string, text: string): string {
    const lead = text.match(/^\s+/)?.[0] ?? ''
    const marker = text.slice(lead.length).match(/^((?:[-*+]|\d+\.)\s+|#{1,6}\s+|>\s+)/)?.[0] ?? ''
    const prefix = lead + marker
    const body = text.slice(prefix.length)
    if (!body) return esc(text)
    return esc(prefix) + OPEN + eid + SEP + esc(body) + CLOSE
  }

  function buildHtml(parts: AnswerPart[], evMap: Map<string, Evidence>): string {
    const src = parts
      .map((p) => (p.type === 'claim' && p.text ? claimToken(p.evidenceIds[0], p.text) : esc(p.text)))
      .join('')
    const out = marked.parse(src, { async: false, breaks: true, gfm: true }) as string
    const re = new RegExp(`${OPEN}(.*?)${SEP}([\\s\\S]*?)${CLOSE}`, 'g')
    // 常驻观感全交给 CSS 按 verdict 分级：已核验静默、需复核琥珀虚线、无法核验红色波浪线。
    return out.replace(re, (_m, eid: string, inner: string) => {
      const verdict = (evMap.get(eid)?.verdict ?? 'unsupported') as Verdict
      return `<span class="at-claim" data-eid="${eid}" data-verdict="${verdict}" role="button" tabindex="0">${inner}</span>`
    })
  }

  const html = $derived(buildHtml(parts, evMap))

  let container = $state<HTMLElement | null>(null)

  // 水合：给渲染出的 .at-claim span 挂 hover/click 弹层（floating-ui + 动态 mount ClaimPopover）。
  $effect(() => {
    html // 依赖：内容变了重新水合
    const root = container
    if (!root) return

    let popEl: HTMLElement | null = null
    let popInstance: ReturnType<typeof mount> | null = null
    let currentRef: HTMLElement | null = null
    let cleanupAuto: (() => void) | null = null
    let showTimer: ReturnType<typeof setTimeout> | undefined
    let hideTimer: ReturnType<typeof setTimeout> | undefined

    // 只在弹层打开时给一层浅底，示意「当前聚焦项」；常态背景交还给 CSS（即无背景）。
    function paint(span: HTMLElement, active: boolean) {
      const v = (span.dataset.verdict ?? 'unsupported') as Verdict
      span.style.background = active ? STATUS[v].soft : ''
    }

    function close() {
      clearTimeout(showTimer)
      clearTimeout(hideTimer)
      cleanupAuto?.()
      cleanupAuto = null
      if (popInstance) {
        unmount(popInstance)
        popInstance = null
      }
      if (popEl) {
        popEl.remove()
        popEl = null
      }
      if (currentRef) paint(currentRef, false)
      currentRef = null
    }

    function open(ref: HTMLElement, ev: Evidence) {
      if (currentRef === ref && popEl) return
      close()
      currentRef = ref
      paint(ref, true)
      popEl = document.createElement('div')
      popEl.style.cssText = 'position:fixed;left:0;top:0;z-index:100;'
      popEl.addEventListener('mouseenter', () => clearTimeout(hideTimer))
      popEl.addEventListener('mouseleave', scheduleClose)
      document.body.appendChild(popEl)
      popInstance = mount(ClaimPopover, { target: popEl, props: { evidence: ev, onOpenSource } })
      cleanupAuto = autoUpdate(ref, popEl, () => {
        computePosition(ref, popEl!, {
          strategy: 'fixed',
          placement: 'bottom-start',
          middleware: [offset(8), flip(), shift({ padding: 8 })],
        }).then(({ x, y }) => {
          if (popEl) Object.assign(popEl.style, { left: `${x}px`, top: `${y}px` })
        })
      })
    }

    function scheduleOpen(ref: HTMLElement, ev: Evidence) {
      clearTimeout(hideTimer)
      clearTimeout(showTimer)
      showTimer = setTimeout(() => open(ref, ev), 120)
    }
    function scheduleClose() {
      clearTimeout(showTimer)
      clearTimeout(hideTimer)
      hideTimer = setTimeout(close, 200)
    }

    const cleanups: Array<() => void> = []
    for (const span of Array.from(root.querySelectorAll<HTMLElement>('.at-claim'))) {
      const ev = evMap.get(span.dataset.eid ?? '')
      if (!ev) continue
      const enter = () => scheduleOpen(span, ev)
      const leave = () => scheduleClose()
      const toggle = () => (currentRef === span && popEl ? close() : open(span, ev))
      const key = (e: KeyboardEvent) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          toggle()
        } else if (e.key === 'Escape') {
          close()
        }
      }
      span.addEventListener('mouseenter', enter)
      span.addEventListener('mouseleave', leave)
      span.addEventListener('focus', enter)
      span.addEventListener('blur', leave)
      span.addEventListener('click', toggle)
      span.addEventListener('keydown', key)
      cleanups.push(() => {
        span.removeEventListener('mouseenter', enter)
        span.removeEventListener('mouseleave', leave)
        span.removeEventListener('focus', enter)
        span.removeEventListener('blur', leave)
        span.removeEventListener('click', toggle)
        span.removeEventListener('keydown', key)
      })
    }

    return () => {
      for (const c of cleanups) c()
      close()
    }
  })
</script>

<!-- eslint-disable svelte/no-at-html-tags -->
<div bind:this={container} class="at-answer text-[15px] leading-7 text-[#1A1D21]">{@html html}</div>

<style>
  .at-answer :global(p) {
    margin: 0 0 0.5rem;
  }
  .at-answer :global(p:last-child) {
    margin-bottom: 0;
  }
  .at-answer :global(ul),
  .at-answer :global(ol) {
    margin: 0.25rem 0 0.5rem;
    padding-left: 1.35rem;
  }
  .at-answer :global(ul) {
    list-style: disc;
  }
  .at-answer :global(ol) {
    list-style: decimal;
  }
  .at-answer :global(li) {
    margin: 0.15rem 0;
  }
  .at-answer :global(li::marker) {
    color: #9aa2ad;
  }
  .at-answer :global(strong) {
    font-weight: 600;
  }
  .at-answer :global(em) {
    font-style: italic;
  }
  .at-answer :global(code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.88em;
    background: #f3f4f6;
    padding: 0.05em 0.3em;
    border-radius: 4px;
  }
  .at-answer :global(h1),
  .at-answer :global(h2),
  .at-answer :global(h3) {
    font-weight: 600;
    margin: 0.6rem 0 0.35rem;
    line-height: 1.35;
  }
  .at-answer :global(h1) {
    font-size: 1.15em;
  }
  .at-answer :global(h2) {
    font-size: 1.08em;
  }
  .at-answer :global(h3) {
    font-size: 1em;
  }
  .at-answer :global(table) {
    border-collapse: collapse;
    margin: 0.4rem 0;
    font-size: 0.94em;
  }
  .at-answer :global(th),
  .at-answer :global(td) {
    border: 1px solid #e7e9ec;
    padding: 0.28rem 0.55rem;
    text-align: left;
  }
  .at-answer :global(th) {
    background: #f7f8fa;
    font-weight: 600;
  }
  .at-answer :global(a) {
    color: #2563eb;
    text-decoration: underline;
  }
  .at-answer :global(blockquote) {
    margin: 0.4rem 0;
    padding-left: 0.75rem;
    border-left: 3px solid #e7e9ec;
    color: #4b5563;
  }
  /* 高亮强度跟随「不确定程度」，而非「是不是 claim」。 */
  .at-answer :global(.at-claim) {
    cursor: help;
    border-radius: 2px;
    text-underline-offset: 3px;
    -webkit-box-decoration-break: clone;
    box-decoration-break: clone;
    transition:
      background 0.15s,
      text-decoration-color 0.15s;
  }
  /* 已核验：事实信息静默呈现，仅 hover/聚焦时浮出一条细虚线示意可溯源。 */
  .at-answer :global(.at-claim[data-verdict='supported']) {
    text-decoration: underline dotted transparent;
    text-decoration-thickness: 1px;
  }
  .at-answer :global(.at-claim[data-verdict='supported']:hover),
  .at-answer :global(.at-claim[data-verdict='supported']:focus-visible) {
    text-decoration-color: rgba(21, 163, 74, 0.5);
  }
  /* 需复核：常驻琥珀色虚线，提示「数字接近但要核对」。 */
  .at-answer :global(.at-claim[data-verdict='partial']) {
    text-decoration: underline dashed rgba(217, 119, 6, 0.8);
    text-decoration-thickness: 1.5px;
  }
  /* 无法核验/疑似有误：红色波浪线，最醒目——把注意力集中到这里。 */
  .at-answer :global(.at-claim[data-verdict='unsupported']) {
    text-decoration: underline wavy rgba(220, 38, 38, 0.85);
    text-decoration-thickness: 1px;
  }
</style>
