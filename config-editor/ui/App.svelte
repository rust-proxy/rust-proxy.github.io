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

<div class="shell" data-theme={theme.current}>
  <a class="skip-link" href="#ce-editor" onclick={(event) => { event.preventDefault(); workbench.skipToEditor(); }}>跳转到配置表单</a>
  <AppHeader {theme} />
  <main id="config-editor" data-ready="true">
    <div class="page-heading">
      <div><p class="page-heading__eyebrow">{view.ui.eyebrow}</p><h1>{view.ui.title}</h1><p class="page-heading__intro">{view.ui.description}</p></div>
      <p class="privacy-note"><span class="privacy-note__dot"></span>本地处理<span>不保存输入 · 不上传凭据</span></p>
    </div>
    {#if view.mode}
      {@const mode = view.mode}
      <div class="schema-modes" role="group" aria-label={mode.label}>
        {#each mode.options as [value, label] (value)}
          <button type="button" data-mode={value} aria-pressed={mode.value === value}
            onclick={() => session.dispatch({ type: 'set', field: mode.key, value })}>{label}</button>
        {/each}
      </div>
    {/if}
    <div class="pane-switch" role="group" aria-label="工作区视图">
      <button type="button" aria-pressed={workbench.pane === 'editor'} onclick={() => workbench.switchPane('editor')}>编辑配置</button>
      <button type="button" aria-pressed={workbench.pane === 'preview'} onclick={() => workbench.switchPane('preview')}>预览与导出{view.valid ? '' : ` · ${session.errors.length}`}</button>
    </div>
    <div class="workspace" data-pane={workbench.pane}>
      <SectionNav />
      <form id="ce-editor" class="editor-form" autocomplete="off" onsubmit={(event) => event.preventDefault()}>
        <div class="region-heading"><h2>选择配置</h2><span>修改后实时更新预览</span></div>
        {#key session.schema}
          <FieldSearch />
          {#each view.sections as section, index (section.name)}
            <FormSection {section} number={index + 1} />
          {/each}
        {/key}
      </form>
      <PreviewPanel />
    </div>
    <div class={`status-bar${session.status ? ' status-bar--visible' : ''}`} role="status" aria-live="polite">{session.status}</div>
    <footer><span>{view.ui.brand} 配置工具</span><span>本地生成，按需导出。</span></footer>
  </main>
</div>

<style>
  .shell {
    min-height: 100vh;
    color: var(--color-text);
    background:
      radial-gradient(1100px 520px at 82% -12%, color-mix(in srgb, var(--color-accent) 12%, transparent), transparent 70%),
      var(--color-bg);
    font-size: 14px;
    line-height: 1.6;
  }
  .skip-link {
    position: fixed;
    top: -100px;
    left: 16px;
    z-index: 50;
    padding: 12px 16px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: var(--shadow-pop);
  }
  .skip-link:focus { top: 10px; }
  #config-editor {
    max-width: 1616px;
    padding: 26px 28px 22px;
    margin: auto;
  }
  .page-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    margin-bottom: 22px;
  }
  .page-heading > div { max-width: 800px; }
  .page-heading__eyebrow {
    color: var(--color-accent);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: .12em;
    text-transform: uppercase;
    margin: 0 0 8px;
  }
  .page-heading h1 {
    font-size: clamp(22px, 2.2vw, 30px);
    letter-spacing: -.03em;
    line-height: 1.25;
    margin: 0 0 12px;
  }
  .page-heading__intro { color: var(--color-text-muted); margin: 0; font-size: 13px; }
  .privacy-note {
    flex: none;
    display: grid;
    grid-template-columns: 9px auto;
    align-items: center;
    gap: 4px 8px;
    padding: 10px 14px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    background: var(--color-surface);
    font-size: 12px;
    color: var(--color-success);
  }
  .privacy-note > span:last-child { grid-column: 2; font-size: 11px; color: var(--color-text-muted); }
  .privacy-note__dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-success);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--color-success) 18%, transparent);
  }
  .schema-modes { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 24px; }
  .schema-modes button { font-size: 12px; padding-inline: 16px; }
  .pane-switch { display: none; }
  .workspace {
    display: grid;
    grid-template-columns: 188px minmax(0, 1fr) minmax(0, .78fr);
    gap: 20px;
    align-items: start;
  }
  .editor-form { min-width: 0; scroll-margin-top: 24px; }
  .region-heading {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin: 0 0 16px;
    gap: 12px;
  }
  .region-heading h2 { margin: 0; font-size: 14px; }
  .region-heading > span { color: var(--color-text-muted); font-size: 11px; }
  .status-bar {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 30;
    max-width: calc(100vw - 32px);
    font-size: 12px;
  }
  .status-bar--visible {
    padding: 12px 18px;
    border: 1px solid var(--color-border-strong);
    border-radius: 999px;
    background: var(--color-text);
    color: var(--color-surface);
    box-shadow: var(--shadow-float);
  }
  footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-top: 32px;
    padding-top: 18px;
    border-top: 1px solid var(--color-border);
    color: var(--color-text-muted);
    font-size: 11px;
  }
  @media (max-width: 1279px) {
    .workspace { grid-template-columns: minmax(0, 1fr) minmax(0, .78fr); }
    .page-heading { align-items: flex-start; }
  }
  @media (max-width: 959px) {
    .workspace { grid-template-columns: minmax(0, 1fr); gap: 16px; }
    .page-heading { display: block; }
    .privacy-note { display: flex; flex-wrap: wrap; margin: 12px 0 0; width: fit-content; }
    .pane-switch {
      position: sticky;
      top: 0;
      z-index: 15;
      display: flex;
      gap: 8px;
      padding: 10px 0;
      margin-bottom: 16px;
      background: var(--color-bg);
    }
    .pane-switch button { flex: 1; }
    .pane-switch button[aria-pressed="true"] { background: var(--color-accent); color: var(--color-on-accent); border-color: var(--color-accent); }
    [data-pane="editor"] > :global(.preview),
    [data-pane="preview"] > .editor-form,
    [data-pane="preview"] > :global(.section-nav) { display: none; }
  }
  @media (max-width: 480px) {
    #config-editor { padding: 18px 16px; }
    .page-heading { margin-bottom: 12px; }
    .region-heading { flex-wrap: wrap; }
  }
  .pane-switch button { transition: none; }
</style>
