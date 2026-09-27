<script lang="ts">
  import ExportActions from './ExportActions.svelte';
  import Notices from './Notices.svelte';
  import PreviewDocument from './PreviewDocument.svelte';
  import { useSession, useWorkbench } from '../state/context';

  const session = useSession();
  const workbench = useWorkbench();
  const view = $derived(session.snapshot);
  const errors = $derived(session.errors);
  const format = $derived(view.format?.value ?? 'json');
</script>

<aside id="ce-preview" class="preview" aria-label="配置预览区" tabindex="-1">
  <div class="preview__top">
    <h2>配置预览</h2>
    <span class="preview__validity" role="status" data-valid={String(view.valid)}>
      {view.valid ? '可导出' : `${errors.length} 项待填写或修正`}
    </span>
  </div>
  <div class="preview__toolbar">
    <div class="preview__tabs" role="group" aria-label="配置预览类型">
      {#each view.outputs as output (output.name)}
        <button type="button" hidden={!output.visible} aria-pressed={view.selected === output.name}
          onclick={() => session.selectOutput(output.name)}>{output.label}</button>
      {/each}
    </div>
    {#if view.format}
      {@const field = view.format}
      <select id={`ce-${field.key}`} aria-label={field.label} value={field.value}
        onchange={(event) => session.dispatch({ type: 'set', field: field.key, value: event.currentTarget.value })}>
        {#each field.options as [value, label] (value)}
          <option {value}>{label}</option>
        {/each}
      </select>
    {/if}
  </div>
  <div class="preview__filebar">
    <strong>{view.filename}</strong>
    <label for="ce-reveal">
      <input id="ce-reveal" type="checkbox" checked={session.reveal} onchange={(event) => session.setReveal(event.currentTarget.checked)} />
      显示密码
    </label>
  </div>
  <details class="preview__errors" hidden={!errors.length} open>
    <summary>查看 {errors.length} 项待修正字段</summary>
    <p>以下字段需要修正，预览中已用 &lt;placeholder&gt; 替代：</p>
    <ul>{#each errors as [key, message] (key)}
      <li><button type="button" onclick={() => workbench.navigateToField(key)}>{message}</button></li>
    {/each}</ul>
  </details>
  {#if view.preview_lines.length}
    <PreviewDocument lines={view.preview_lines} {format} />
  {:else}
    <div class="preview__code"><code>无法生成预览，请检查配置。</code></div>
  {/if}
  <ExportActions />
  <div class="preview__notes"><p class="field-hint">{view.ui.export_hint}</p>{#if view.command}<code class="preview__command">{view.command}</code>{/if}<Notices notices={view.notices} /></div>
</aside>

<style>
  .preview {
    min-width: 0;
    position: sticky;
    top: 24px;
    display: flex;
    flex-direction: column;
    max-height: calc(100dvh - 48px);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
    overflow: clip;
    scroll-margin-top: 24px;
  }
  .preview__top, .preview__toolbar, .preview__filebar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    flex-wrap: wrap;
  }
  .preview__top { border-bottom: 1px solid var(--color-border); }
  .preview__top h2 { font-size: 14px; margin: 0; }
  .preview__validity { font-size: 11px; color: var(--color-danger); }
  .preview__validity[data-valid="true"] { color: var(--color-success); }
  .preview__tabs { display: flex; flex-wrap: wrap; gap: 6px; }
  .preview__tabs button, .preview__toolbar select { font-size: 11px; min-height: 34px; padding: 6px 10px; }

  .preview__filebar {
    background: var(--color-surface-raised);
    border-block: 1px solid var(--color-border);
    font-size: 11px;
    padding-block: 10px;
    color: var(--color-text-muted);
  }
  .preview__filebar strong { overflow-wrap: anywhere; min-width: 0; color: var(--color-text); }
  .preview__filebar label { display: flex; align-items: center; gap: 7px; white-space: nowrap; }

  .preview__errors {
    padding: 12px 20px;
    font-size: 11px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-danger-soft);
    max-height: 120px;
    overflow: auto;
    flex-shrink: 0;
  }
  .preview__errors p { margin: 0 0 6px; color: var(--color-danger); }
  .preview__errors ul { padding-left: 16px; margin: 0; }
  .preview__errors button {
    background: transparent;
    padding: 4px 0;
    border: 0;
    min-height: 0;
    text-align: left;
    font-size: 11px;
    text-decoration: underline;
    color: var(--color-danger);
  }
  .preview__errors summary { font-weight: 600; color: var(--color-danger); }

  .preview__code {
    background: var(--color-surface-raised);
    max-height: 48vh;
    min-height: 100px;
    flex: 1 1 auto;
    overflow: auto;
    padding: 20px;
    scrollbar-width: thin;
    color: var(--color-text-muted);
  }
  .preview__notes { padding: 0 20px 20px; max-height: 120px; overflow: auto; flex-shrink: 0; }
  .preview__command {
    display: block;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    background: var(--color-surface-raised);
    font-size: 11px;
    overflow-wrap: anywhere;
  }
  @media (max-width: 959px) {
    .preview { position: static; max-height: none; }
    .preview__notes { max-height: none; }
  }
  @media (max-height: 740px) {
    .preview { position: static; max-height: none; }
    .preview__notes { max-height: none; }
  }
</style>
