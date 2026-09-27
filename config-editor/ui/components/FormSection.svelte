<script lang="ts">
  import CollectionEditor from './CollectionEditor.svelte';
  import FieldControl from './FieldControl.svelte';
  import Notices from './Notices.svelte';
  import type { SectionView } from '../types';
  let { section, number }: { section: SectionView; number: number } = $props();
</script>

{#snippet heading()}
  <span class="form-section__step">{String(number).padStart(2, '0')}</span>
  <span class="form-section__title">{section.label}{#if section.detail}<small>{section.detail}</small>{/if}</span>
{/snippet}
{#snippet contents()}
  <div class="form-section__body">
    <div class="field-list">
      {#each section.fields as field (field.key)}<FieldControl {field} />{/each}
    </div>
    {#each section.collections as collection (collection.name)}<CollectionEditor {collection} />{/each}
    <Notices notices={section.notices} />
  </div>
{/snippet}

{#if section.collapsed}
  <details id={`section-${section.name}`} class="form-section form-section--advanced" hidden={!section.visible}>
    <summary>{@render heading()}<span class="form-section__chevron" aria-hidden="true">⌄</span></summary>
    {@render contents()}
  </details>
{:else}
  <section id={`section-${section.name}`} class="form-section" hidden={!section.visible} aria-labelledby={`section-${section.name}-heading`}>
    <h2 id={`section-${section.name}-heading`} tabindex="-1">{@render heading()}</h2>
    {@render contents()}
  </section>
{/if}

<style>
  .form-section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    box-shadow: var(--shadow-card);
    margin: 0 0 16px;
    scroll-margin-top: 24px;
    overflow: clip;
  }
  .form-section > h2, .form-section > summary {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 16px 20px;
    font-size: 15px;
    font-weight: 650;
    list-style: none;
  }
  .form-section > summary::-webkit-details-marker { display: none; }
  .form-section > summary:hover { background: var(--color-surface-raised); }
  .form-section__step {
    display: grid;
    place-items: center;
    flex: none;
    width: 28px;
    height: 28px;
    border-radius: 9px;
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font: 11px var(--font-mono);
  }
  .form-section__title { flex: 1; min-width: 0; }
  .form-section__title small { display: block; font-size: 11px; color: var(--color-text-muted); font-weight: 400; margin-top: 3px; }
  .form-section__chevron { color: var(--color-text-muted); transition: transform .15s ease; }
  .form-section--advanced[open] .form-section__chevron { transform: rotate(180deg); }
  .form-section__body { padding: 0 20px 20px; }
  @media (max-width: 959px) {
    .form-section { scroll-margin-top: 76px; }
  }
  @media (max-width: 480px) {
    .form-section > h2, .form-section > summary { padding: 16px; }
    .form-section__body { padding: 0 16px 16px; }
  }
</style>
