<script lang="ts">
  import { useSession } from '../state/context';
  import type { ThemeStore } from '../state/theme.svelte';
  import { writeSchemaParam } from '../state/url';

  let { theme }: { theme: ThemeStore } = $props();
  const session = useSession();
  const ui = $derived(session.ui);

  function selectSchema(name: string) {
    session.selectSchema(name);
    writeSchemaParam(session.schema);
  }
</script>

<header class="ce-header">
  <a class="ce-brand" href="./" aria-label={`${ui.title}首页`}>
    <span class="ce-mark" aria-hidden="true">{ui.mark}</span><span>{ui.brand}<small>配置工作台</small></span>
  </a>
  <div class="ce-header-controls">
    {#if session.schemas.length}
      <label class="ce-schema" for="ce-schema"><span>配置方案</span>
        <select id="ce-schema" value={session.schema} onchange={(event) => selectSchema(event.currentTarget.value)}>
          {#each session.schemas as item (item.name)}<option value={item.name}>{item.label}</option>{/each}
        </select>
      </label>
    {/if}
    <button type="button" class="ce-theme" aria-label="切换主题" aria-pressed={theme.dark} onclick={() => theme.toggle()}>{theme.dark ? '浅色' : '深色'}</button>
  </div>
</header>
