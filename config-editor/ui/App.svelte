<script lang="ts">
  import AppHeader from './components/AppHeader.svelte';
  import FieldSearch from './components/FieldSearch.svelte';
  import FormSection from './components/FormSection.svelte';
  import PreviewPanel from './components/PreviewPanel.svelte';
  import SectionNav from './components/SectionNav.svelte';
  import { setSessions } from './state/context';
  import type { SessionStore } from './state/session.svelte';
  import type { ThemeStore } from './state/theme.svelte';
  import type { WorkbenchStore } from './state/workbench.svelte';

  let { session, theme, workbench }: { session: SessionStore; theme: ThemeStore; workbench: WorkbenchStore } = $props();
  // svelte-ignore state_referenced_locally
  setSessions(session, workbench);
  const view = $derived(session.snapshot);

  $effect(() => {
    void session.schema;
    workbench.reset();
  });
</script>

<svelte:head><title>{view.ui.brand} {view.ui.title}</title></svelte:head>

<div class="ce-shell" data-theme={theme.dark ? 'dark' : 'light'}>
  <a class="ce-skip" href="#ce-editor" onclick={(event) => { event.preventDefault(); workbench.skipToEditor(); }}>跳转到配置表单</a>
  <AppHeader {theme} />
  <main id="config-editor" data-ready="true">
    <div class="ce-heading">
      <div><p class="ce-eyebrow">{view.ui.eyebrow}</p><h1>{view.ui.title}</h1><p class="ce-intro">{view.ui.description}</p></div>
      <p class="ce-privacy"><span class="ce-dot"></span>本地处理<span>不保存输入 · 不上传凭据</span></p>
    </div>
    {#if view.mode}
      {@const mode = view.mode}
      <div class="ce-modes" role="group" aria-label={mode.label}>
        {#each mode.options as [value, label] (value)}
          <button type="button" data-mode={value} aria-pressed={mode.value === value}
            onclick={() => session.dispatch({ type: 'set', field: mode.key, value })}>{label}</button>
        {/each}
      </div>
    {/if}
    <div class="ce-pane-switch" role="group" aria-label="工作区视图">
      <button type="button" aria-pressed={workbench.pane === 'editor'} onclick={() => workbench.switchPane('editor')}>编辑配置</button>
      <button type="button" aria-pressed={workbench.pane === 'preview'} onclick={() => workbench.switchPane('preview')}>预览与导出{view.valid ? '' : ` · ${session.errors.length}`}</button>
    </div>
    <div class="ce-workspace" data-pane={workbench.pane}>
      <SectionNav />
      <form id="ce-editor" class="ce-form" autocomplete="off" onsubmit={(event) => event.preventDefault()}>
        <div class="ce-region-heading"><h2>选择配置</h2><span>修改后实时更新预览</span></div>
        {#key session.schema}
          <FieldSearch />
          {#each view.sections as section, index (section.name)}
            <FormSection {section} number={index + 1} />
          {/each}
        {/key}
      </form>
      <PreviewPanel />
    </div>
    <div class="ce-status" class:ce-status-visible={!!session.status} role="status" aria-live="polite">{session.status}</div>
    <footer><span>{view.ui.brand} 配置工具</span><span>本地生成，按需导出。</span></footer>
  </main>
</div>
