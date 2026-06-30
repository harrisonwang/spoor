<script lang="ts">
  import type { AnswerTrace, Evidence, Verdict } from '@answer-trace/protocol'
  import AnswerText from './AnswerText.svelte'

  let {
    trace,
    onOpenSource,
  }: {
    trace: AnswerTrace
    onOpenSource?: (evidence: Evidence) => void
  } = $props()

  function count(v: Verdict): number {
    return trace.answer.filter((p) => p.type === 'claim' && p.verdict === v).length
  }
  const cited = $derived(count('supported') + count('partial'))
  const partial = $derived(count('partial'))
  const unsupported = $derived(count('unsupported'))
</script>

<section class="space-y-4">
  <div class="flex justify-end">
    <div
      class="max-w-[78%] rounded-2xl rounded-tr-md border border-[#E7E9EC] bg-white px-4 py-3 text-[14px] leading-6 text-[#1A1D21] shadow-[0_1px_2px_rgba(16,24,40,0.03)]"
    >
      {trace.question}
    </div>
  </div>

  <div class="flex items-start gap-3">
    <div class="mt-1 grid h-7 w-7 shrink-0 place-items-center rounded-full bg-[#EEF2F7] text-[#2563EB]">
      <svg viewBox="0 0 24 24" fill="currentColor" class="h-3.5 w-3.5" aria-hidden="true">
        <path d="M12 3l1.6 6.4L20 11l-6.4 1.6L12 19l-1.6-6.4L4 11l6.4-1.6z" />
      </svg>
    </div>
    <div class="min-w-0 flex-1">
      <div
        class="rounded-2xl rounded-tl-md border border-[#E7E9EC] bg-white p-4 shadow-[0_1px_2px_rgba(16,24,40,0.03)]"
      >
        <AnswerText parts={trace.answer} evidence={trace.evidence} {onOpenSource} />
      </div>
    </div>
  </div>
</section>
