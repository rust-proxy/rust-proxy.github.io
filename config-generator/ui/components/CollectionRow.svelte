<script lang="ts">
  import FieldControl from './FieldControl.svelte';
  import { useSession } from '../state/context';
  import type { CollectionView, RowView } from '../types';

  let { collection, row, onremove }: { collection: CollectionView; row: RowView; onremove: (id: string) => void } = $props();
  const session = useSession();
</script>

<div class="cg-user" hidden={!row.visible} data-row={row.id}>
  <div class="cg-row-title">
    <strong>{collection.label} {row.number}</strong>
    <div class="cg-row-actions">
      {#if collection.generated}<button type="button" onclick={() => session.dispatch({ type: 'generate-row', collection: collection.name, id: row.id })}>{collection.generate_label}</button>{/if}
      {#if collection.removable}<button class="cg-remove" type="button" aria-label={`移除${collection.label} ${row.number}`} onclick={() => onremove(row.id)}>移除</button>{/if}
    </div>
  </div>
  <div class="cg-fields">
    {#each row.fields as field (field.key)}<FieldControl {field} row={{ collection: collection.name, id: row.id }} />{/each}
  </div>
</div>
