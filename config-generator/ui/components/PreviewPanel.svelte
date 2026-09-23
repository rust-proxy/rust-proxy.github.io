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

<aside id="cg-preview" class="cg-output" aria-label="配置预览区" tabindex="-1">
  <div class="cg-output-top">
    <h2>配置预览</h2>
    <span class="cg-validity" role="status" data-valid={String(view.valid)}>
      {view.valid ? '可导出' : `${errors.length} 项待填写或修正`}
    </span>
  </div>
  <div class="cg-toolbar">
    <div class="cg-sides" role="group" aria-label="配置预览类型">
      {#each view.outputs as output (output.name)}
        <button type="button" hidden={!output.visible} aria-pressed={view.selected === output.name}
          onclick={() => session.selectOutput(output.name)}>{output.label}</button>
      {/each}
    </div>
    {#if view.format}
      {@const field = view.format}
      <select id={`cg-${field.key}`} aria-label={field.label} value={field.value}
        onchange={(event) => session.dispatch({ type: 'set', field: field.key, value: event.currentTarget.value })}>
        {#each field.options as [value, label] (value)}
          <option {value}>{label}</option>
        {/each}
      </select>
    {/if}
  </div>
  <div class="cg-filebar">
    <strong>{view.filename}</strong>
    <label for="cg-reveal">
      <input id="cg-reveal" type="checkbox" checked={session.reveal} onchange={(event) => session.setReveal(event.currentTarget.checked)} />
      显示密码
    </label>
  </div>
  <details class="cg-errors" hidden={!errors.length} open>
    <summary>查看 {errors.length} 项待修正字段</summary>
    <p>以下字段需要修正，预览中已用 &lt;placeholder&gt; 替代：</p>
    <ul>{#each errors as [key, message] (key)}
      <li><button type="button" onclick={() => workbench.navigateToField(key)}>{message}</button></li>
    {/each}</ul>
  </details>
  {#if view.preview_lines.length}
    <PreviewDocument lines={view.preview_lines} {format} />
  {:else}
    <div class="cg-code"><code>无法生成预览，请检查配置。</code></div>
  {/if}
  <ExportActions />
  <div class="cg-export-notes"><p class="cg-hint">{view.ui.export_hint}</p>{#if view.command}<code class="cg-command">{view.command}</code>{/if}<Notices notices={view.notices} /></div>
</aside>
