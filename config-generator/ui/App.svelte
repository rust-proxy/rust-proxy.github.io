<script lang="ts">
  import type { Controller } from './controller.svelte';
  import AppHeader from './AppHeader.svelte';
  import SectionNavigation from './SectionNavigation.svelte';
  import FormSection from './FormSection.svelte';
  import OutputPanel from './OutputPanel.svelte';

  let { controller }: { controller: Controller } = $props();
  const view = $derived(controller.view);
  let dark = $state(window.matchMedia('(prefers-color-scheme: dark)').matches);
</script>

<svelte:head><title>{view.ui.brand} {view.ui.title}</title></svelte:head>

<div class="cg-shell" data-theme={dark ? 'dark' : 'light'}>
  <a class="cg-skip" href="#cg-editor">跳转到配置表单</a>
  <AppHeader {controller} bind:dark />
  <main id="config-generator" data-ready="true">
    <div class="cg-heading">
      <div><p class="cg-eyebrow">{view.ui.eyebrow}</p><h1>{view.ui.title}</h1><p class="cg-intro">{view.ui.description}</p></div>
      <p class="cg-privacy"><span class="cg-dot"></span>本地处理<span>不保存输入 · 不上传凭据</span></p>
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
      <SectionNavigation sections={view.sections} />
      <form id="cg-editor" class="cg-form" autocomplete="off" onsubmit={(event) => event.preventDefault()}>
        <div class="cg-region-heading"><h2>选择配置</h2><span>修改后实时更新预览</span></div>
        {#key controller.schema}
          {#each view.sections as section, index (section.name)}
            <FormSection {section} number={index + 1} dispatch={controller.dispatch} />
          {/each}
        {/key}
      </form>
      <OutputPanel {controller} />
    </div>
    <div class="cg-status" class:cg-status-visible={!!controller.status} role="status" aria-live="polite">{controller.status}</div>
    <footer><span>{view.ui.brand} 配置工具</span><span>本地生成，按需导出。</span></footer>
  </main>
</div>
