<script lang="ts">
  import { useSession, useWorkbench } from '../state/context';
  import { focusSection } from '../lib/dom';

  const session = useSession();
  const workbench = useWorkbench();
  let active = $state('');

  $effect(() => {
    const names = session.sections.filter(section => section.visible).map(section => section.name);
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
      {#each session.visibleSections as section (section.name)}<option value={section.name}>{section.label}{session.errorCount(section) ? ` · ${session.errorCount(section)} 项待修正` : ''}</option>{/each}
    </select>
  </label>
  {#each session.sections as section, index (section.name)}
    {#if section.visible}
      <button type="button" aria-current={active === section.name ? 'location' : undefined} onclick={() => focusSection(section.name)}>
        <span class="cg-nav-number">{String(index + 1).padStart(2, '0')}</span><span>{section.label}</span>
        {#if session.errorCount(section)}<span class="cg-error-count" aria-label={`${session.errorCount(section)} 项待修正`}>{session.errorCount(section)}</span>{/if}
      </button>
    {/if}
  {/each}
  <a href="#cg-preview" onclick={(event) => { event.preventDefault(); workbench.showPreview(); }}>查看配置预览 →</a>
</nav>
