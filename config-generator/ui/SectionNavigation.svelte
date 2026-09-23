<script lang="ts">
  import { focusSection } from './browser';
  import type { SectionView } from './types';
  let { sections }: { sections: SectionView[] } = $props();
</script>

<nav class="cg-section-nav" aria-label="配置分区">
  <p>配置目录</p>
  {#each sections as section, index (section.name)}
    {#if section.visible}
      <button type="button" onclick={() => focusSection(section.name)}>
        <span class="cg-nav-number">{String(index + 1).padStart(2, '0')}</span><span>{section.label}</span>
      </button>
    {/if}
  {/each}
  <a href="#cg-preview">查看配置预览 ↓</a>
</nav>
