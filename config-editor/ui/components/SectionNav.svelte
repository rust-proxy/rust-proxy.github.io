<script lang="ts">
  import { m } from '../paraglide/messages';
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
        const elements = names.map(name => document.getElementById(`section-${name}`)).filter((element): element is HTMLElement => !!element && element.getClientRects().length > 0);
        if (!elements.length) return;
        const atBottom = window.scrollY > 0 && window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2;
        const current = atBottom ? elements.at(-1) : elements.filter(element => element.getBoundingClientRect().top <= 140).at(-1) ?? elements[0];
        active = current?.id.replace('section-', '') ?? '';
      });
    };
    update();
    const observer = new ResizeObserver(update);
    const form = document.getElementById('editor-form');
    if (form) observer.observe(form);
    window.addEventListener('scroll', update, { passive: true });
    window.addEventListener('resize', update);
    return () => { observer.disconnect(); cancelAnimationFrame(frame); window.removeEventListener('scroll', update); window.removeEventListener('resize', update); };
  });
</script>

<nav class="section-nav" aria-label={m.nav_sections()}>
  <p>{m.nav_directory()}</p>
  <label class="section-nav__picker" for="editor-section-picker">{m.nav_sections()}
    <select id="editor-section-picker" value={active} onchange={(event) => focusSection(event.currentTarget.value)}>
      {#each session.visibleSections as section (section.name)}<option value={section.name}>{section.label}{session.errorCount(section) ? m.nav_option_error({ count: session.errorCount(section) }) : ''}</option>{/each}
    </select>
  </label>
  {#each session.sections as section, index (section.name)}
    {#if section.visible}
      <button type="button" data-testid="section-nav-item" aria-current={active === section.name ? 'location' : undefined} onclick={() => focusSection(section.name)}>
        <span class="section-nav__number">{String(index + 1).padStart(2, '0')}</span><span>{section.label}</span>
        {#if session.errorCount(section)}<span class="section-nav__error-count" aria-label={m.nav_error_count({ count: session.errorCount(section) })}>{session.errorCount(section)}</span>{/if}
      </button>
    {/if}
  {/each}
  <a href="#editor-preview" onclick={(event) => { event.preventDefault(); workbench.showPreview(); }}>{m.nav_view_preview()}</a>
</nav>

<style>
  .section-nav { position: sticky; top: 24px; display: grid; gap: 5px; }
  .section-nav p {
    font-size: 11px;
    letter-spacing: .14em;
    text-transform: uppercase;
    color: var(--color-text-muted);
    margin: 4px 0 14px 10px;
  }
  .section-nav button {
    display: flex;
    align-items: baseline;
    gap: 10px;
    border-color: transparent;
    background: transparent;
    text-align: left;
    font-size: 12px;
    padding: 10px;
  }
  .section-nav__number { color: var(--color-text-muted); font: 10px var(--font-mono); }
  .section-nav a { margin: 18px 10px; font-size: 12px; text-decoration: none; }
  .section-nav button[aria-current] {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 28%, transparent);
  }
  .section-nav button > span:nth-child(2) { flex: 1; }
  .section-nav__error-count {
    border-radius: 999px;
    padding: 0 7px;
    color: var(--color-danger);
    background: var(--color-danger-soft);
    font-size: 11px;
  }
  .section-nav__picker { display: none; }
  @media (max-width: 1279px) {
    .section-nav {
      grid-column: 1 / -1;
      position: static;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 10px 14px;
      border: 1px solid var(--color-border);
      border-radius: var(--radius-md);
      background: var(--color-surface);
      box-shadow: var(--shadow-card);
    }
    .section-nav > button, .section-nav > p { display: none; }
    .section-nav a { margin: 0; }
    .section-nav__picker { display: flex; align-items: center; gap: 12px; min-width: 0; font-size: 12px; }
    .section-nav__picker select { min-width: 0; max-width: 100%; }
  }
  @media (max-width: 959px) {
    .section-nav > a { display: none; }
    .section-nav__picker { width: 100%; }
    .section-nav__picker select { flex: 1; }
  }
</style>
