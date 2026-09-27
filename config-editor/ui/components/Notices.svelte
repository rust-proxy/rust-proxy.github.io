<script lang="ts">
  import type { NoticeView } from '../types';
  let { notices }: { notices: NoticeView[] } = $props();
</script>

{#each notices as notice}
  <p class="notice" hidden={!notice.visible}>{notice.text}</p>
{/each}

<style>
  .notice {
    padding: 10px 12px;
    border-left: 3px solid var(--color-accent);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    background: var(--color-surface-raised);
    font-size: 11px;
    overflow-wrap: anywhere;
  }
</style>
