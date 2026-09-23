<script lang="ts">
  import CollectionEditor from './CollectionEditor.svelte';
  import FieldControl from './FieldControl.svelte';
  import Notices from './Notices.svelte';
  import type { SectionView } from '../types';
  let { section, number }: { section: SectionView; number: number } = $props();
</script>

{#snippet heading()}
  <span class="ce-step">{String(number).padStart(2, '0')}</span>
  <span class="ce-section-title">{section.label}{#if section.detail}<small>{section.detail}</small>{/if}</span>
{/snippet}
{#snippet contents()}
  <div class="ce-section-body">
    <div class="ce-fields">
      {#each section.fields as field (field.key)}<FieldControl {field} />{/each}
    </div>
    {#each section.collections as collection (collection.name)}<CollectionEditor {collection} />{/each}
    <Notices notices={section.notices} />
  </div>
{/snippet}

{#if section.collapsed}
  <details id={`ce-section-${section.name}`} class="ce-section ce-advanced" hidden={!section.visible}>
    <summary>{@render heading()}<span class="ce-chevron" aria-hidden="true">⌄</span></summary>
    {@render contents()}
  </details>
{:else}
  <section id={`ce-section-${section.name}`} class="ce-section" hidden={!section.visible} aria-labelledby={`ce-heading-${section.name}`}>
    <h2 id={`ce-heading-${section.name}`} tabindex="-1">{@render heading()}</h2>
    {@render contents()}
  </section>
{/if}
