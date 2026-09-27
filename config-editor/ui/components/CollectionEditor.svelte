<script lang="ts">
  import { tick } from 'svelte';
  import { m } from '../paraglide/messages';
  import CollectionRow from './CollectionRow.svelte';
  import FieldControl from './FieldControl.svelte';
  import { useSession } from '../state/context';
  import type { CollectionView } from '../types';

  let { collection }: { collection: CollectionView } = $props();
  const session = useSession();
  let root: HTMLDivElement;

  function focusRow(id: string) {
    const row = Array.from(root.querySelectorAll<HTMLElement>('[data-row]')).find(element => element.dataset.row === id);
    row?.querySelector<HTMLElement>('.field:not([hidden]) input, .field:not([hidden]) select')?.focus();
  }

  async function add() {
    const previous = new Set(collection.rows.map(row => row.id));
    session.dispatch({ type: 'add', collection: collection.name });
    await tick();
    const added = collection.rows.find(row => !previous.has(row.id));
    if (added) focusRow(added.id);
  }

  async function remove(id: string) {
    const index = collection.rows.findIndex(row => row.id === id);
    session.dispatch({ type: 'remove', collection: collection.name, id });
    await tick();
    const remaining = collection.rows.filter(row => row.visible);
    const target = remaining[Math.min(index, remaining.length - 1)];
    if (target) focusRow(target.id);
    else root.querySelector<HTMLButtonElement>('.collection__add')?.focus();
  }
</script>

<div class="collection" hidden={!collection.visible} bind:this={root}>
  <div class="collection__heading"><h3>{collection.label}</h3><span>{m.collection_item_count({ count: collection.rows.filter(row => row.visible).length })}</span></div>
  {#each collection.rows as row (row.id)}
    <CollectionRow {collection} {row} onremove={remove} />
  {/each}
  {#if collection.editable}<button class="collection__add" type="button" onclick={add}>{collection.add_label}</button>{/if}
  {#if collection.selector}<FieldControl field={collection.selector} />{/if}
  {#if collection.hint}<p class="field-hint">{collection.hint}</p>{/if}
</div>

<style>
  .collection { margin-top: 22px; }
  .collection__heading { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .collection__heading h3 { margin: 0; font-size: 13px; }
  .collection__heading > span { font-size: 11px; color: var(--color-text-muted); }
  .collection__add {
    width: 100%;
    border-style: dashed;
    font-size: 12px;
    margin-bottom: 16px;
    color: var(--color-accent);
  }
</style>
