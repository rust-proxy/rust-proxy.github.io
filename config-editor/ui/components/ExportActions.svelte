<script lang="ts">
  import { download } from '../lib/dom';
  import { useSession } from '../state/context';

  const session = useSession();
  let copying = $state(false);

  async function copy() {
    if (copying) return;
    copying = true;
    try {
      await navigator.clipboard.writeText(session.export().text);
      session.status = '已复制完整配置（含明文凭据）。';
    } catch { session.status = '浏览器未允许复制，请使用下载配置。'; }
    finally { copying = false; }
  }

  function save() {
    try {
      const file = session.export();
      download(file.text, file.filename);
      session.status = `已下载 ${file.filename}（含明文凭据）。`;
    } catch { session.status = '下载失败，请尝试复制配置。'; }
  }
</script>

<div class="ce-actions">
  <button type="button" disabled={!session.valid || copying} aria-busy={copying} onclick={copy}>复制配置</button>
  <button type="button" class="ce-primary" disabled={!session.valid} onclick={save}>下载配置</button>
</div>
