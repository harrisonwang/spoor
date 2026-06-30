<script lang="ts">
  import { autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom'
  import type { Snippet } from 'svelte'
  import type { Evidence, Verdict } from '@answer-trace/protocol'
  import { STATUS } from '$lib/status'
  import ClaimPopover from './ClaimPopover.svelte'

  let {
    status,
    evidence,
    onOpenSource,
    children,
  }: {
    status: Verdict
    evidence: Evidence | null
    onOpenSource?: (evidence: Evidence) => void
    children: Snippet
  } = $props()

  const s = $derived(STATUS[status])

  let open = $state(false)
  let refEl = $state<HTMLElement | null>(null)
  let popEl = $state<HTMLElement | null>(null)
  let showTimer: ReturnType<typeof setTimeout> | undefined
  let hideTimer: ReturnType<typeof setTimeout> | undefined

  function show() {
    clearTimeout(hideTimer)
    showTimer = setTimeout(() => (open = true), 120)
  }
  function hide() {
    clearTimeout(showTimer)
    hideTimer = setTimeout(() => (open = false), 200)
  }

  $effect(() => {
    if (!open || !refEl || !popEl) return
    return autoUpdate(refEl, popEl, () => {
      computePosition(refEl!, popEl!, {
        strategy: 'fixed',
        placement: 'bottom-start',
        middleware: [offset(8), flip(), shift({ padding: 8 })],
      }).then(({ x, y }) => {
        if (popEl) Object.assign(popEl.style, { left: `${x}px`, top: `${y}px` })
      })
    })
  })

  // 命中高亮:行内 background 渐变 + box-decoration-break:clone —— 文本正常跨行,
  // 不再是 inline-block 那种整块原子换行(会在行尾留空、硬挤到下一行)。
  const hl = $derived(
    `background: linear-gradient(transparent 60%, ${open ? s.color + '33' : s.wash} 60%);` +
      '-webkit-box-decoration-break: clone; box-decoration-break: clone;',
  )
</script>

<span
  bind:this={refEl}
  class="cursor-default rounded-[2px] font-medium text-[#1A1D21] outline-none transition-[background]"
  style={hl}
  role="button"
  tabindex="0"
  aria-haspopup="true"
  onmouseenter={show}
  onmouseleave={hide}
  onfocus={show}
  onblur={hide}
  onclick={() => (open = !open)}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      open = !open
    } else if (e.key === 'Escape') {
      open = false
    }
  }}>{@render children()}</span
>{#if open && evidence}<div
    bind:this={popEl}
    class="z-[100]"
    style="position: fixed; left: 0; top: 0;"
    role="group"
    onmouseenter={show}
    onmouseleave={hide}
  >
    <ClaimPopover {evidence} {onOpenSource} />
  </div>{/if}
