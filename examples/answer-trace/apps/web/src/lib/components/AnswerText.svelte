<script lang="ts">
  import type { AnswerPart, Evidence } from '@answer-trace/protocol'
  import InlineClaim from './InlineClaim.svelte'

  let {
    parts,
    evidence,
    onOpenSource,
  }: {
    parts: AnswerPart[]
    evidence: Evidence[]
    onOpenSource?: (evidence: Evidence) => void
  } = $props()

  const map = $derived(new Map(evidence.map((e) => [e.id, e])))
</script>

<p class="text-[15px] leading-7 text-[#1A1D21]"
  >{#each parts as part}{#if part.type === 'claim'}<InlineClaim
        status={part.verdict}
        evidence={map.get(part.evidenceIds[0]) ?? null}
        {onOpenSource}>{part.text}</InlineClaim
      >{:else}{part.text}{/if}{/each}</p
>
