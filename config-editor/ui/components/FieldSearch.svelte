<script lang="ts">
  import { useSession, useWorkbench } from '../state/context';

  let query = $state('');
  const session = useSession();
  const workbench = useWorkbench();

  function escape(event: KeyboardEvent) {
    if (event.key === 'Escape') { query = ''; document.getElementById('ce-search')?.focus(); }
  }

  const matches = $derived.by(() => {
    const term = query.trim().toLocaleLowerCase();
    if (!term) return [];
    return session.searchable.filter(entry =>
      [entry.label, entry.hint, entry.path].some(text => text.toLocaleLowerCase().includes(term)));
  });
</script>

<div class="ce-search" role="search" aria-label="查找配置项">
  <label for="ce-search">查找配置项</label>
  <div class="ce-search-input"><input id="ce-search" type="search" onkeydown={escape} bind:value={query} placeholder="搜索名称、说明或配置路径" autocomplete="off" />
    {#if query}<button type="button" onclick={() => { query = ''; document.getElementById('ce-search')?.focus(); }}>清除</button>{/if}
  </div>
  {#if query.trim()}
    <div class="ce-search-results">
      <p role="status">{matches.length ? `找到 ${matches.length} 个配置项` : '没有匹配的可见配置项'}</p>
      <ul>{#each matches as match (match.path)}
        <li><button type="button" onkeydown={escape} onclick={() => { query = ''; workbench.navigateToField(match.path); }}><strong>{match.label}</strong><span>{match.section} · {match.path}</span></button></li>
      {/each}</ul>
    </div>
  {/if}
</div>
