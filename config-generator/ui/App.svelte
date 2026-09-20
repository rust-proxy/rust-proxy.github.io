<script lang="ts">
  import type { Controller } from './controller.svelte';
  import FormSection from './FormSection.svelte';
  import OutputPanel from './OutputPanel.svelte';
  import ConfigDescription from './ConfigDescription.svelte';

  let { controller }: { controller: Controller } = $props();
  const view = $derived(controller.view);
  let dark = $state(window.matchMedia('(prefers-color-scheme: dark)').matches);
  type Page = 'generator' | 'description';

  const query = new URLSearchParams(window.location.search);
  const requestedMode = query.get('mode');
  let page = $state<Page>(requestedMode === 'detail' || requestedMode === 'description' ? 'description' : 'generator');

  function updateUrl() {
    const url = new URL(window.location.href);
    url.searchParams.set('mode', page === 'description' ? 'detail' : 'generate');
    url.searchParams.set('schema', controller.schema);
    window.history.replaceState(window.history.state, '', url);
  }

  function selectSchema(name: string) {
    controller.selectSchema(name);
    if (page === 'description' && !controller.view.config_description) page = 'generator';
    updateUrl();
  }

  function selectPage(next: Page) {
    page = next;
    updateUrl();
  }

  queueMicrotask(() => {
    if (query.has('schema') && query.get('schema') !== controller.schema) updateUrl();
  });

  $effect(() => {
    if (page === 'description' && !view.config_description) {
      page = 'generator';
      updateUrl();
    }
  });
</script>

<svelte:head><title>{view.ui.brand} {view.ui.title}</title></svelte:head>

<div class="cg-shell" data-theme={dark ? 'dark' : 'light'}>
  <header class="cg-header">
    <a class="cg-brand" href="./" aria-label={`${view.ui.title}首页`}>
      <span class="cg-mark">{view.ui.mark}</span>{view.ui.brand}<span class="cg-brand-sub">配置工具</span>
    </a>
    <nav>
      {#if controller.schemas.length}
        <label class="cg-schema" for="cg-schema"><span>配置方案</span>
          <select id="cg-schema" value={controller.schema} onchange={(event) => selectSchema(event.currentTarget.value)}>
            {#each controller.schemas as item (item.name)}<option value={item.name}>{item.label}</option>{/each}
          </select>
        </label>
      {/if}
      <button type="button" class="cg-nav" aria-pressed={page === 'generator'} onclick={() => selectPage('generator')}>配置生成</button>
      {#if view.config_description}<button type="button" class="cg-nav" aria-pressed={page === 'description'} onclick={() => selectPage('description')}>配置详解</button>{/if}
      <a href={view.ui.reference} hidden={!view.ui.reference} target="_blank" rel="noopener noreferrer">配置说明 ↗</a>
      <button type="button" class="cg-theme" aria-label="切换主题" aria-pressed={dark} onclick={() => dark = !dark}>
        {dark ? '浅色' : '深色'}
      </button>
    </nav>
  </header>
  <main id="config-generator" data-ready="true">
    {#if page === 'description' && view.config_description}
      <ConfigDescription description={view.config_description} />
    {:else}
    <div class="cg-heading">
      <p class="cg-eyebrow">{view.ui.eyebrow}</p><h1>{view.ui.title}</h1><p>{view.ui.description}</p>
      <p class="cg-privacy"><span class="cg-dot"></span>在浏览器本地处理 · 不保存输入 · 不上传凭据</p>
    </div>
    {#if view.mode}
      {@const mode = view.mode}
      <div class="cg-modes" role="group" aria-label={mode.label}>
        {#each mode.options as [value, label] (value)}
          <button type="button" data-mode={value} aria-pressed={mode.value === value}
            onclick={() => controller.dispatch({ type: 'set', field: mode.key, value })}>{label}</button>
        {/each}
      </div>
    {/if}
    <div class="cg-workspace">
      <form class="cg-form" autocomplete="off" onsubmit={(event) => event.preventDefault()}>
        {#each view.sections as section, index (section.name)}
          <FormSection {section} number={index + 1} dispatch={controller.dispatch} />
        {/each}
      </form>
      <OutputPanel {controller} />
    </div>
    <p class="cg-status" role="status" aria-live="polite">{controller.status}</p>
    {/if}
    <footer>{view.ui.brand} 配置工具<span>本地生成，按需导出。</span></footer>
  </main>
</div>
