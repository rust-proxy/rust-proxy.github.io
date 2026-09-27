<script lang="ts">
  import { m } from '../paraglide/messages';
  import FieldControl from './FieldControl.svelte';
  import { useSession } from '../state/context';
  import type { CollectionView, RowView } from '../types';

  let { collection, row, onremove }: { collection: CollectionView; row: RowView; onremove: (id: string) => void } = $props();
  const session = useSession();
</script>

<div class="collection-row" hidden={!row.visible} data-row={row.id}>
  <div class="collection-row__title">
    <strong>{collection.label} {row.number}</strong>
    <div class="collection-row__actions">
      {#if collection.generated}<button type="button" onclick={() => session.dispatch({ type: 'generate-row', collection: collection.name, id: row.id })}>{collection.generate_label}</button>{/if}
      {#if collection.removable}<button class="collection-row__remove" type="button" aria-label={m.row_remove_aria({ label: collection.label, number: row.number })} onclick={() => onremove(row.id)}>{m.row_remove()}</button>{/if}
    </div>
  </div>
  <div class="field-list">
    {#each row.fields as field (field.key)}<FieldControl {field} row={{ collection: collection.name, id: row.id }} />{/each}
  </div>
</div>

<style>
  .collection-row {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
    padding: 16px;
    margin-bottom: 12px;
  }
  .collection-row__title {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 12px;
    margin-bottom: 16px;
  }
  .collection-row__title strong { font-size: 12px; }
  .collection-row__actions { display: flex; flex-wrap: wrap; gap: 6px; }
  .collection-row__actions button { font-size: 11px; min-height: 32px; padding: 5px 10px; }
  .collection-row__remove { color: var(--color-danger); }
  .collection-row__remove:hover:not(:disabled) { border-color: var(--color-danger); background: var(--color-danger-soft); }
  @media (max-width: 480px) {
    .collection-row { padding: 12px; }
  }
</style>
