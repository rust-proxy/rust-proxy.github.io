<script lang="ts">
  import type { Controller } from './controller.svelte';
  import { download } from './browser';
  let { controller }: { controller: Controller } = $props();
  let copying = $state(false);

  async function copy() {
    if (copying) return;
    copying = true;
    try {
      await navigator.clipboard.writeText(controller.export().text);
      controller.status = '已复制完整配置（含明文凭据）。';
    } catch { controller.status = '浏览器未允许复制，请使用下载配置。'; }
    finally { copying = false; }
  }

  function save() {
    try {
      const file = controller.export();
      download(file.text, file.filename);
      controller.status = `已下载 ${file.filename}（含明文凭据）。`;
    } catch { controller.status = '下载失败，请尝试复制配置。'; }
  }
</script>

<div class="cg-actions">
  <button type="button" disabled={!controller.view.valid || copying} aria-busy={copying} onclick={copy}>复制配置</button>
  <button type="button" class="cg-primary" disabled={!controller.view.valid} onclick={save}>下载配置</button>
</div>
