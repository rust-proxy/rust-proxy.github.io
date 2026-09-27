<script lang="ts">
  import { m } from '../paraglide/messages';
  import { useSession, useWorkbench } from '../state/context';

  let query = $state('');
  const session = useSession();
  const workbench = useWorkbench();

  function escape(event: KeyboardEvent) {
    if (event.key === 'Escape') { query = ''; document.getElementById('editor-search')?.focus(); }
  }

  const matches = $derived.by(() => {
    const term = query.trim().toLocaleLowerCase();
    if (!term) return [];
    return session.searchable.filter(entry =>
      [entry.label, entry.hint, entry.path].some(text => text.toLocaleLowerCase().includes(term)));
  });
</script>

<div class="field-search" role="search" aria-label={m.search_label()}>
  <label for="editor-search">{m.search_label()}</label>
  <div class="field-search__input"><input id="editor-search" type="search" onkeydown={escape} bind:value={query} placeholder={m.search_placeholder()} autocomplete="off" />
    {#if query}<button type="button" onclick={() => { query = ''; document.getElementById('editor-search')?.focus(); }}>{m.search_clear()}</button>{/if}
  </div>
  {#if query.trim()}
    <div class="field-search__results" data-testid="search-results">
      <p role="status">{matches.length ? m.search_found({ count: matches.length }) : m.search_empty()}</p>
      <ul>{#each matches as match (match.path)}
        <li><button type="button" data-testid="search-result" onkeydown={escape} onclick={() => { query = ''; workbench.navigateToField(match.path); }}><strong>{match.label}</strong><span>{match.section} · {match.path}</span></button></li>
      {/each}</ul>
    </div>
  {/if}
</div>

<style>
  .field-search { position: relative; margin-bottom: 20px; }
  .field-search > label { display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; }
  .field-search__input { display: flex; gap: 8px; }
  .field-search__input input { min-width: 0; width: 100%; }
  .field-search__results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 10;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-pop);
    max-height: 360px;
    overflow: auto;
  }
  .field-search__results p { padding: 10px 14px; margin: 0; color: var(--color-text-muted); font-size: 12px; }
  .field-search__results ul { list-style: none; padding: 0; margin: 0; }
  .field-search__results button { display: grid; width: 100%; text-align: left; border: 0; border-radius: 0; gap: 2px; }
  .field-search__results span { font-size: 11px; color: var(--color-text-muted); overflow-wrap: anywhere; font-weight: 400; }
</style>
