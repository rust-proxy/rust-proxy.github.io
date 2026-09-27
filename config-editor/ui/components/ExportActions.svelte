<script lang="ts">
  import { m } from '../paraglide/messages';
  import { download } from '../lib/dom';
  import { useSession } from '../state/context';

  const session = useSession();
  let copying = $state(false);

  async function copy() {
    if (copying) return;
    copying = true;
    try {
      await navigator.clipboard.writeText(session.export().text);
      session.status = m.export_copied();
    } catch { session.status = m.export_copy_denied(); }
    finally { copying = false; }
  }

  function save() {
    try {
      const file = session.export();
      download(file.text, file.filename);
      session.status = m.export_downloaded({ filename: file.filename });
    } catch { session.status = m.export_download_failed(); }
  }
</script>

<div class="export-actions">
  <button type="button" disabled={!session.valid || copying} aria-busy={copying} onclick={copy}>{m.export_copy()}</button>
  <button type="button" class="button--primary" disabled={!session.valid} onclick={save}>{m.export_download()}</button>
</div>

<style>
  .export-actions { display: flex; gap: 10px; padding: 12px 16px; border-top: 1px solid var(--color-border); }
  .export-actions button { flex: 1; font-size: 12px; }
</style>
