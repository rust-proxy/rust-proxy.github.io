<script lang="ts">
  import Field from './Field.svelte';
  import type { CollectionView, Dispatch } from './types';
  let { collection, dispatch }: { collection: CollectionView; dispatch: Dispatch } = $props();
</script>

<div class="cg-collection" hidden={!collection.visible}>
  <div class="cg-collection-heading"><h3>{collection.label}</h3><span>{collection.rows.filter(row => row.visible).length} 项</span></div>
  {#each collection.rows as row (row.id)}
    <div class="cg-user" hidden={!row.visible}>
      <div class="cg-row-title">
        <strong>{collection.label} {row.number}</strong>
        <div class="cg-row-actions">
          {#if collection.generated}<button type="button" onclick={() => dispatch({ type: 'generate-row', collection: collection.name, id: row.id })}>{collection.generate_label}</button>{/if}
          {#if collection.removable}<button class="cg-remove" type="button" aria-label={`移除${collection.label} ${row.number}`} onclick={() => dispatch({ type: 'remove', collection: collection.name, id: row.id })}>移除</button>{/if}
        </div>
      </div>
      <div class="cg-fields">
        {#each row.fields as field (field.key)}<Field {field} {dispatch} row={{ collection: collection.name, id: row.id }} />{/each}
      </div>
    </div>
  {/each}
  {#if collection.editable}<button class="cg-add" type="button" onclick={() => dispatch({ type: 'add', collection: collection.name })}>{collection.add_label}</button>{/if}
  {#if collection.selector}<Field field={collection.selector} {dispatch} />{/if}
  {#if collection.hint}<p class="cg-hint">{collection.hint}</p>{/if}
</div>
