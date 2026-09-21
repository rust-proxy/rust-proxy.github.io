<script lang="ts">
  import type { Controller } from './controller.svelte';
  import { download, focusError } from './browser';
  import Notices from './Notices.svelte';
  import PreviewDocument from './PreviewDocument.svelte';

  let { controller }: { controller: Controller } = $props();
  const view = $derived(controller.view);
  const errors = $derived(Object.entries(view.errors));
  let reveal = $state(false);
  const format = $derived(view.format?.value ?? 'json');

  async function copy() {
    try {
      await navigator.clipboard.writeText(controller.export().text);
      controller.status = '已复制完整配置（含明文凭据）。';
    } catch { controller.status = '浏览器未允许复制，请使用下载配置。'; }
  }

  function save() {
    try {
      const file = controller.export();
      download(file.text, file.filename);
      controller.status = `已下载 ${file.filename}（含明文凭据）。`;
    } catch { controller.status = '下载失败，请尝试复制配置。'; }
  }
</script>

<aside class="cg-output" aria-label="配置预览区">
  <div class="cg-output-top">
    <strong>配置预览</strong>
    <span class="cg-validity" role="status" data-valid={String(view.valid)}>
      {view.valid ? '可导出' : `${errors.length} 项待填写或修正`}
    </span>
  </div>
  <div class="cg-toolbar">
    <div class="cg-sides" role="group" aria-label="配置预览类型">
      {#each view.outputs as output (output.name)}
        <button type="button" hidden={!output.visible} aria-pressed={view.selected === output.name}
          onclick={() => controller.select(output.name)}>{output.label}</button>
      {/each}
    </div>
    {#if view.format}
      {@const field = view.format}
      <select id={`cg-${field.key}`} aria-label={field.label} value={field.value}
        onchange={(event) => controller.dispatch({ type: 'set', field: field.key, value: event.currentTarget.value })}>
        {#each field.options as [value, label] (value)}
          <option {value}>{label}</option>
        {/each}
      </select>
    {/if}
  </div>
  <div class="cg-filebar">
    <strong>{view.filename}</strong>
    <label for="cg-reveal">
      <input id="cg-reveal" type="checkbox" bind:checked={reveal} onchange={(event) => controller.showSecrets(event.currentTarget.checked)} />
      显示密码
    </label>
  </div>
  <div class="cg-errors" hidden={!errors.length}>
    <p>完成以下字段后即可生成：</p>
    <ul>{#each errors as [key, message] (key)}
      <li><button type="button" onclick={() => focusError(key)}>{message}</button></li>
    {/each}</ul>
  </div>
  {#if view.preview_lines.length}
    <PreviewDocument lines={view.preview_lines} {format} />
  {:else}
    <div class="cg-code"><code>填写左侧配置，预览将在校验通过后显示。</code></div>
  {/if}
  <div class="cg-actions">
    <button type="button" disabled={!view.valid} onclick={copy}>复制配置</button>
    <button type="button" class="cg-primary" disabled={!view.valid} onclick={save}>下载配置</button>
  </div>
  <p class="cg-hint">{view.ui.export_hint}</p><code class="cg-command">{view.command}</code>
  <Notices notices={view.notices} />
</aside>
