<script lang="ts">
  import { focusSection } from './browser';
  import type { SectionView } from './types';
  let { sections, preview }: { sections: SectionView[]; preview: () => void } = $props();
  let active = $state('');
  function countErrors(section: SectionView) {
    return section.fields.filter(field => field.visible && field.error).length + section.collections.filter(collection => collection.visible).reduce((count, collection) => count + collection.rows.filter(row => row.visible).flatMap(row => row.fields).filter(field => field.visible && field.error).length + Number(!!collection.selector?.visible && !!collection.selector.error), 0);
  }
  $effect(() => {
    const names = sections.filter(section => section.visible).map(section => section.name);
    let frame = 0;
    const update = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const elements = names.map(name => document.getElementById(`cg-section-${name}`)).filter((element): element is HTMLElement => !!element && element.getClientRects().length > 0);
        if (!elements.length) return;
        const atBottom = window.scrollY > 0 && window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2;
        const current = atBottom ? elements.at(-1) : elements.filter(element => element.getBoundingClientRect().top <= 140).at(-1) ?? elements[0];
        active = current?.id.replace('cg-section-', '') ?? '';
      });
    };
    update();
    const observer = new ResizeObserver(update);
    const form = document.getElementById('cg-editor');
    if (form) observer.observe(form);
    window.addEventListener('scroll', update, { passive: true });
    window.addEventListener('resize', update);
    return () => { observer.disconnect(); cancelAnimationFrame(frame); window.removeEventListener('scroll', update); window.removeEventListener('resize', update); };
  });
</script>

<nav class="cg-section-nav" aria-label="配置分区">
  <p>配置目录</p>
  <label class="cg-section-picker" for="cg-section-picker">配置分区
    <select id="cg-section-picker" value={active} onchange={(event) => focusSection(event.currentTarget.value)}>
      {#each sections.filter(section => section.visible) as section (section.name)}<option value={section.name}>{section.label}{countErrors(section) ? ` · ${countErrors(section)} 项待修正` : ''}</option>{/each}
    </select>
  </label>
  {#each sections as section, index (section.name)}
    {#if section.visible}
      <button type="button" aria-current={active === section.name ? 'location' : undefined} onclick={() => focusSection(section.name)}>
        <span class="cg-nav-number">{String(index + 1).padStart(2, '0')}</span><span>{section.label}</span>
        {#if countErrors(section)}<span class="cg-error-count" aria-label={`${countErrors(section)} 项待修正`}>{countErrors(section)}</span>{/if}
      </button>
    {/if}
  {/each}
  <a href="#cg-preview" onclick={(event) => { event.preventDefault(); preview(); }}>查看配置预览 →</a>
</nav>
