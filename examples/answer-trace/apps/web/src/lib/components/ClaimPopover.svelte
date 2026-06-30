<script lang="ts">
  import type { Evidence } from '@answer-trace/protocol'
  import { STATUS } from '$lib/status'

  let {
    evidence,
    onOpenSource,
  }: {
    evidence: Evidence
    onOpenSource?: (evidence: Evidence) => void
  } = $props()

  const s = $derived(STATUS[evidence.verdict])
</script>

<div
  class="w-[380px] rounded-xl border border-[#E7E9EC] bg-white p-3.5 shadow-[0_12px_40px_rgba(16,24,40,0.14)]"
>
  {#if evidence.kind === 'quote'}
    <div class="rounded-lg bg-[#F7F8FA] px-3 py-2">
      <div class="text-[12.5px] leading-5 text-[#5B6370]">
        <span class="text-[#9AA1A9]">…</span>{evidence.before}<span
          class="rounded-[3px] font-medium text-[#1A1D21]"
          style="box-shadow: inset 0 -0.42em 0 {s.wash}">{evidence.hit}</span
        >{evidence.after}<span class="text-[#9AA1A9]">…</span>
      </div>
    </div>
  {:else if evidence.kind === 'cell'}
    <div class="rounded-lg bg-[#F7F8FA] px-3 py-2">
      <div class="flex flex-wrap items-center gap-1 text-[12px] text-[#5B6370]">
        <span>{evidence.table}</span>
        <span class="text-[#9AA1A9]">›</span>
        <span>{evidence.row}</span>
        <span class="text-[#9AA1A9]">›</span>
        <span>{evidence.column}</span>
        <span class="mx-1 text-[#9AA1A9]">=</span>
        <span
          class="rounded-[3px] px-0.5 font-mono text-[12.5px] font-semibold text-[#1A1D21] tabular-nums"
          style="box-shadow: inset 0 -0.42em 0 {s.wash}">{evidence.value}</span
        >
      </div>
    </div>
  {:else}
    <div class="rounded-lg bg-[#F7F8FA] px-3 py-2 text-[12.5px] leading-5 text-[#5B6370]">
      <div class="mb-1 flex items-center gap-1.5 text-[11px] font-medium text-[#991B1B]">
        🔍 全文检索未命中
      </div>
      {evidence.reason}
      {#if evidence.expectedTruth}
        <div class="mt-1 font-medium text-[#1A1D21]">{evidence.expectedTruth}</div>
      {/if}
    </div>
  {/if}

  {#if evidence.kind !== 'none' && evidence.note}
    <div
      class="mt-2 rounded-lg border px-2.5 py-1.5 text-[11.5px] leading-4"
      style="color:{s.text};background:{s.soft};border-color:{s.border}"
    >
      {evidence.note}
    </div>
  {/if}

  {#if evidence.kind !== 'none'}
    <div class="mt-2 flex items-center justify-between text-[11px]">
      {#if evidence.page != null}
        <span class="text-[#9AA1A9]">第 {evidence.page} 页</span>
      {:else}
        <span></span>
      {/if}
      {#if onOpenSource}
        <button
          type="button"
          class="inline-flex items-center gap-1 font-medium text-[#2563EB] hover:text-[#1D4ED8]"
          onclick={(e) => {
            e.stopPropagation()
            onOpenSource?.(evidence)
          }}
        >
          定位原文 ↗
        </button>
      {/if}
    </div>
  {/if}
</div>
