<script lang="ts">
  import { highlightLines } from '../lib/highlight';
  import type { PreviewLine } from '../types';

  let { lines, format }: { lines: PreviewLine[]; format: string } = $props();
  const rendered = $derived(highlightLines(lines.map(line => line.text), format));
  let tip = $state<{ text: string; top: number; left: number; above: boolean } | null>(null);

  function place(event: Event, text: string) {
    if (!text) { tip = null; return; }
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const width = Math.min(420, window.innerWidth - 32);
    const left = Math.max(16, Math.min(rect.left + 42, window.innerWidth - width - 16));
    const above = rect.top > 120;
    tip = { text, top: above ? rect.top - 8 : rect.bottom + 8, left, above };
  }
  function clear() { tip = null; }
  $effect(() => { lines; format; clear(); });
</script>

<svelte:window onresize={clear} onkeydown={(event) => { if (event.key === 'Escape') clear(); }} />

<div class="preview-doc" aria-label="生成配置预览">
  {#each lines as line, index (index)}
    <button type="button" class="preview-doc__line" aria-label={line.description ? `${line.text}。${line.description}` : line.text}
      onmouseenter={(event) => place(event, line.description)}
      onmouseleave={clear}
      onfocus={(event) => place(event, line.description)}
      onblur={clear}>
      <span class="preview-doc__number" aria-hidden="true">{index + 1}</span>
      <code class="preview-doc__code">{@html rendered[index] ?? ''}</code>
    </button>
  {/each}
</div>
{#if tip}
  <span class="preview-tooltip" role="tooltip" style={`top: ${tip.top}px; left: ${tip.left}px; transform: translateY(${tip.above ? '-100%' : '0'})`}>{tip.text}</span>
{/if}

<style>
  .preview-doc {
    background: var(--color-surface-raised);
    max-height: 48vh;
    min-height: 100px;
    flex: 1 1 auto;
    overflow: auto;
    padding: 14px 0;
    scrollbar-width: thin;
  }
  .preview-doc__line {
    display: grid;
    grid-template-columns: 48px 1fr;
    width: max-content;
    min-width: 100%;
    min-height: 26px;
    padding: 0 20px 0 0;
    border: 0;
    border-radius: 0;
    background: transparent;
    text-align: left;
    font-weight: 400;
  }
  .preview-doc__line:hover, .preview-doc__line:focus-visible { background: var(--color-accent-soft); outline-offset: -2px; }
  .preview-doc__line code { white-space: pre; font-size: 12px; line-height: 26px; }
  .preview-doc__number {
    position: sticky;
    left: 0;
    background: var(--color-surface-raised);
    color: var(--color-text-muted);
    text-align: right;
    padding-right: 14px;
    font: 10px/26px var(--font-mono);
    user-select: none;
  }
  .preview-doc__line:hover .preview-doc__number, .preview-doc__line:focus-visible .preview-doc__number { background: var(--color-accent-soft); }
  .preview-tooltip {
    position: fixed;
    z-index: 20;
    width: min(420px, calc(100vw - 32px));
    max-height: 100px;
    overflow: auto;
    padding: 10px 12px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: var(--color-text);
    color: var(--color-surface);
    box-shadow: var(--shadow-pop);
    font-size: 12px;
    pointer-events: none;
  }
  @media (max-width: 959px) {
    .preview-doc { max-height: 60dvh; }
  }

  .preview-doc :global(.token.property), .preview-doc :global(.token.atrule) { color: var(--syntax-key); }
  .preview-doc :global(.token.string) { color: var(--syntax-string); }
  .preview-doc :global(.token.number) { color: var(--syntax-number); }
  .preview-doc :global(.token.boolean), .preview-doc :global(.token.important) { color: var(--syntax-literal); }
  .preview-doc :global(.token.class-name), .preview-doc :global(.token.table), .preview-doc :global(.token.tag) { color: var(--syntax-section); }
  .preview-doc :global(.token.punctuation) { color: var(--syntax-punctuation); }
  .preview-doc :global(.token.comment) { color: var(--syntax-comment); }
</style>
