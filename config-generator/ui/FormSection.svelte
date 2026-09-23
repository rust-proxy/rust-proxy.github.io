<script lang="ts">
  import CollectionEditor from './CollectionEditor.svelte';
  import Field from './Field.svelte';
  import Notices from './Notices.svelte';
  import type { Dispatch, SectionView } from './types';
  let { section, number, dispatch }: { section: SectionView; number: number; dispatch: Dispatch } = $props();
</script>

{#snippet heading()}
  <span class="cg-step">{String(number).padStart(2, '0')}</span>
  <span class="cg-section-title">{section.label}{#if section.detail}<small>{section.detail}</small>{/if}</span>
{/snippet}
{#snippet contents()}
  <div class="cg-section-body">
    <div class="cg-fields">
      {#each section.fields as field (field.key)}<Field {field} {dispatch} />{/each}
    </div>
    {#each section.collections as collection (collection.name)}<CollectionEditor {collection} {dispatch} />{/each}
    <Notices notices={section.notices} />
  </div>
{/snippet}

{#if section.collapsed}
  <details id={`cg-section-${section.name}`} class="cg-section cg-advanced" hidden={!section.visible}>
    <summary>{@render heading()}<span class="cg-chevron" aria-hidden="true">⌄</span></summary>
    {@render contents()}
  </details>
{:else}
  <section id={`cg-section-${section.name}`} class="cg-section" hidden={!section.visible} aria-labelledby={`cg-heading-${section.name}`}>
    <h2 id={`cg-heading-${section.name}`} tabindex="-1">{@render heading()}</h2>
    {@render contents()}
  </section>
{/if}
