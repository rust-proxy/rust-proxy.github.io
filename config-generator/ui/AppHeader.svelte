<script lang="ts">
  import { onMount } from 'svelte';
  import type { Controller } from './controller.svelte';
  let { controller, dark = $bindable(false) }: { controller: Controller; dark: boolean } = $props();
  const ui = $derived(controller.view.ui);

  function updateUrl() {
    const url = new URL(window.location.href);
    url.searchParams.set('schema', controller.schema);
    window.history.replaceState(window.history.state, '', url);
  }
  function selectSchema(name: string) {
    controller.selectSchema(name);
    updateUrl();
  }
  onMount(() => {
    const requested = new URLSearchParams(window.location.search).get('schema');
    if (requested !== null && requested !== controller.schema) updateUrl();
  });
</script>

<header class="cg-header">
  <a class="cg-brand" href="./" aria-label={`${ui.title}首页`}>
    <span class="cg-mark" aria-hidden="true">{ui.mark}</span><span>{ui.brand}<small>配置工作台</small></span>
  </a>
  <div class="cg-header-controls">
    {#if controller.schemas.length}
      <label class="cg-schema" for="cg-schema"><span>配置方案</span>
        <select id="cg-schema" value={controller.schema} onchange={(event) => selectSchema(event.currentTarget.value)}>
          {#each controller.schemas as item (item.name)}<option value={item.name}>{item.label}</option>{/each}
        </select>
      </label>
    {/if}
    <button type="button" class="cg-theme" aria-label="切换主题" aria-pressed={dark} onclick={() => dark = !dark}>{dark ? '浅色' : '深色'}</button>
  </div>
</header>
