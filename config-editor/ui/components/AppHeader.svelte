<script lang="ts">
  import { useSession } from '../state/context';
  import { THEMES, type ThemeStore } from '../state/theme.svelte';
  import { writeSchemaParam } from '../state/url';

  let { theme }: { theme: ThemeStore } = $props();
  const session = useSession();
  const ui = $derived(session.ui);

  function selectSchema(name: string) {
    session.selectSchema(name);
    writeSchemaParam(session.schema);
  }
</script>

<header class="app-header">
  <a class="app-header__brand" href="./" aria-label={`${ui.title}首页`}>
    <span class="app-header__mark" aria-hidden="true">{ui.mark}</span><span>{ui.brand}<small>配置工作台</small></span>
  </a>
  <div class="app-header__controls">
    {#if session.schemas.length}
      <label class="app-header__schema" for="ce-schema"><span>配置方案</span>
        <select id="ce-schema" value={session.schema} onchange={(event) => selectSchema(event.currentTarget.value)}>
          {#each session.schemas as item (item.name)}<option value={item.name}>{item.label}</option>{/each}
        </select>
      </label>
    {/if}
    <label class="app-header__theme" for="ce-theme"><span>主题</span>
      <select id="ce-theme" value={theme.current} onchange={(event) => theme.select(event.currentTarget.value)}>
        {#each THEMES as option (option.id)}<option value={option.id}>{option.label}</option>{/each}
      </select>
    </label>
  </div>
</header>

<style>
  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 16px max(28px, calc((100vw - 1560px) / 2));
    background: color-mix(in srgb, var(--color-surface) 88%, transparent);
    border-bottom: 1px solid var(--color-border);
    backdrop-filter: blur(10px);
  }
  .app-header__brand {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--color-text);
    font-size: 19px;
    font-weight: 750;
    text-decoration: none;
  }
  .app-header__brand small {
    display: block;
    color: var(--color-text-muted);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: .14em;
  }
  .app-header__mark {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: var(--radius-md);
    background: linear-gradient(145deg, var(--color-accent), var(--color-accent-strong));
    color: var(--color-on-accent);
    box-shadow: 0 8px 18px -10px color-mix(in srgb, var(--color-accent) 80%, transparent);
  }
  .app-header__controls, .app-header__schema { display: flex; align-items: center; gap: 12px; }
  .app-header__schema { font-size: 12px; color: var(--color-text-muted); }
  .app-header__theme { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--color-text-muted); }
  .app-header__theme select { max-width: 150px; }
  @media (max-width: 480px) {
    .app-header { padding: 12px 16px; gap: 12px; flex-wrap: wrap; }
    .app-header__brand { font-size: 16px; gap: 8px; }
    .app-header__mark { width: 34px; height: 34px; }
    .app-header__brand small { font-size: 9px; }
    .app-header__controls { gap: 8px; flex-wrap: wrap; }
    .app-header__schema { gap: 8px; }
    .app-header__schema select { max-width: 170px; }
  }
</style>
