<script lang="ts">
  import { tick } from 'svelte';
  import Field from './Field.svelte';
  import type { CollectionView, Dispatch } from './types';
  let { collection, dispatch }: { collection: CollectionView; dispatch: Dispatch } = $props();
  let root: HTMLDivElement;
  async function add() {
    const previous = new Set(collection.rows.map(row => row.id));
    dispatch({ type: 'add', collection: collection.name });
    await tick();
    const added = collection.rows.find(row => !previous.has(row.id));
    if (added) focusRow(added.id);
  }
  function focusRow(id: string) {
    const row = Array.from(root.querySelectorAll<HTMLElement>('[data-row]')).find(element => element.dataset.row === id);
    row?.querySelector<HTMLElement>('.cg-field:not([hidden]) input, .cg-field:not([hidden]) select')?.focus();
  }
  async function remove(id: string) {
    const index = collection.rows.findIndex(row => row.id === id);
    dispatch({ type: 'remove', collection: collection.name, id });
    await tick();
    const remaining = collection.rows.filter(row => row.visible);
    const target = remaining[Math.min(index, remaining.length - 1)];
    if (target) focusRow(target.id);
    else root.querySelector<HTMLButtonElement>('.cg-add')?.focus();
  }
</script>

<div class="cg-collection" hidden={!collection.visible} bind:this={root}>
  <div class="cg-collection-heading"><h3>{collection.label}</h3><span>{collection.rows.filter(row => row.visible).length} 项</span></div>
  {#each collection.rows as row (row.id)}
    <div class="cg-user" hidden={!row.visible} data-row={row.id}>
      <div class="cg-row-title">
        <strong>{collection.label} {row.number}</strong>
        <div class="cg-row-actions">
          {#if collection.generated}<button type="button" onclick={() => dispatch({ type: 'generate-row', collection: collection.name, id: row.id })}>{collection.generate_label}</button>{/if}
          {#if collection.removable}<button class="cg-remove" type="button" aria-label={`移除${collection.label} ${row.number}`} onclick={() => remove(row.id)}>移除</button>{/if}
        </div>
      </div>
      <div class="cg-fields">
        {#each row.fields as field (field.key)}<Field {field} {dispatch} row={{ collection: collection.name, id: row.id }} />{/each}
      </div>
    </div>
  {/each}
  {#if collection.editable}<button class="cg-add" type="button" onclick={add}>{collection.add_label}</button>{/if}
  {#if collection.selector}<Field field={collection.selector} {dispatch} />{/if}
  {#if collection.hint}<p class="cg-hint">{collection.hint}</p>{/if}
</div>
