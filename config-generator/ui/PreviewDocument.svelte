<script lang="ts">
  import type { PreviewLine } from './types';
  import { highlightLines } from './prism';

  let { lines, format }: { lines: PreviewLine[]; format: string } = $props();
  const rendered = $derived(highlightLines(lines.map(line => line.text), format));
  let tip = $state<{ text: string; top: number; left: number; above: boolean } | null>(null);

  function place(event: Event, text: string) {
    if (!text) { tip = null; return; }
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const width = Math.min(460, window.innerWidth - 120);
    const left = Math.max(12, Math.min(rect.left + 42, window.innerWidth - width - 16));
    const above = rect.top > 120;
    tip = { text, top: above ? rect.top - 8 : rect.bottom + 8, left, above };
  }
  function clear() { tip = null; }
</script>

<div class="cg-desc-code" aria-label="生成配置预览">
  {#each lines as line, index (`${index}-${line.text}`)}
    <button type="button" class="cg-desc-line" aria-label={line.description ? `${line.text}。${line.description}` : line.text}
      onmouseenter={(event) => place(event, line.description)}
      onmouseleave={clear}
      onfocus={(event) => place(event, line.description)}
      onblur={clear}>
      <span class="cg-line-number" aria-hidden="true">{index + 1}</span>
      <code class="cg-hl">{@html rendered[index] ?? ''}</code>
    </button>
  {/each}
</div>
{#if tip}
  <span class="cg-tooltip" role="tooltip" style={`top: ${tip.top}px; left: ${tip.left}px; transform: translateY(${tip.above ? '-100%' : '0'})`}>{tip.text}</span>
{/if}
