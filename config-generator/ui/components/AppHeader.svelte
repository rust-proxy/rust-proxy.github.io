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

<header class="cg-header">
  <a class="cg-brand" href="./" aria-label={`${ui.title}首页`}>
    <span class="cg-mark" aria-hidden="true">{ui.mark}</span><span>{ui.brand}<small>配置工作台</small></span>
  </a>
  <div class="cg-header-controls">
    {#if session.schemas.length}
      <label class="cg-schema" for="cg-schema"><span>配置方案</span>
        <select id="cg-schema" value={session.schema} onchange={(event) => selectSchema(event.currentTarget.value)}>
          {#each session.schemas as item (item.name)}<option value={item.name}>{item.label}</option>{/each}
        </select>
      </label>
    {/if}
    <button type="button" class="cg-theme" aria-label="切换主题" aria-pressed={theme.dark} onclick={() => theme.toggle()}>{theme.dark ? '浅色' : '深色'}</button>
  </div>
</header>
