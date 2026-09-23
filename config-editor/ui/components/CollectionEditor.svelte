<script lang="ts">
  import { tick } from 'svelte';
  import CollectionRow from './CollectionRow.svelte';
  import FieldControl from './FieldControl.svelte';
  import { useSession } from '../state/context';
  import type { CollectionView } from '../types';

  let { collection }: { collection: CollectionView } = $props();
  const session = useSession();
  let root: HTMLDivElement;

  function focusRow(id: string) {
    const row = Array.from(root.querySelectorAll<HTMLElement>('[data-row]')).find(element => element.dataset.row === id);
    row?.querySelector<HTMLElement>('.ce-field:not([hidden]) input, .ce-field:not([hidden]) select')?.focus();
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
    else root.querySelector<HTMLButtonElement>('.ce-add')?.focus();
  }
</script>

<div class="ce-collection" hidden={!collection.visible} bind:this={root}>
  <div class="ce-collection-heading"><h3>{collection.label}</h3><span>{collection.rows.filter(row => row.visible).length} 项</span></div>
  {#each collection.rows as row (row.id)}
    <CollectionRow {collection} {row} onremove={remove} />
  {/each}
  {#if collection.editable}<button class="ce-add" type="button" onclick={add}>{collection.add_label}</button>{/if}
  {#if collection.selector}<FieldControl field={collection.selector} />{/if}
  {#if collection.hint}<p class="ce-hint">{collection.hint}</p>{/if}
</div>
