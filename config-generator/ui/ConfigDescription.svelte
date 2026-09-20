<script lang="ts">
  import type { ConfigDescription } from './types';

  let { description, selected, onselect }: {
    description: ConfigDescription;
    selected: string;
    onselect: (name: string) => void;
  } = $props();
  let selections = $state<Record<string, string>>({});
  let initialized = $state('');
  const config = $derived(description.configs.find(item => item.name === selected) ?? description.configs[0]);
  const lines = $derived(config?.lines.filter(line => line.conditions.every(([name, value]) => selections[name] === value)) ?? []);

  function selectConfig(name: string) {
    onselect(name);
  }

  function setSelection(name: string, value: string) {
    selections = { ...selections, [name]: value };
  }

  $effect(() => {
    if (!description.configs.some(item => item.name === selected)) onselect(description.configs[0]?.name ?? '');
    else if (config && initialized !== config.name) {
      initialized = config.name;
      selections = Object.fromEntries(config.selectors.map(selector => [selector.name, selector.default]));
    }
  });
</script>

<section class="cg-desc" aria-labelledby="cg-desc-title">
  <div class="cg-desc-heading">
    <p class="cg-eyebrow">CONFIG DESC · YAML</p>
    <h1 id="cg-desc-title">{description.title}</h1>
    <p>{description.description}</p>
  </div>
  <div class="cg-desc-workspace">
    <aside class="cg-desc-controls" aria-label="配置类型与变体">
      <fieldset>
        <legend>配置类型</legend>
        <div class="cg-desc-kinds">
          {#each description.configs as item (item.name)}
            <button type="button" aria-pressed={item.name === selected} onclick={() => selectConfig(item.name)}>{item.label}</button>
          {/each}
        </div>
      </fieldset>
      {#if config}
        {#each config.selectors as selector (selector.name)}
          <label class="cg-desc-selector" for={`cg-desc-${selector.name}`}>
            <span>{selector.label}</span>
            <select id={`cg-desc-${selector.name}`} value={selections[selector.name] ?? selector.default}
              onchange={(event) => setSelection(selector.name, event.currentTarget.value)}>
              {#each selector.choices as [value, label] (value)}<option {value}>{label}</option>{/each}
            </select>
          </label>
        {/each}
      {/if}
      <p class="cg-hint">选择不同枚举分支，右侧示例会同步展示该配置形态。</p>
    </aside>
    {#if config}
      <article class="cg-desc-yaml" aria-label={`${config.label} YAML 配置详解`}>
        <header><strong>{config.filename}</strong><span>悬浮或聚焦任意配置项查看说明</span></header>
        <div class="cg-desc-code">
          {#each lines as line, index (`${line.yaml}-${index}`)}
            {@const yaml = `${'  '.repeat(line.indent)}${line.yaml}`}
            <button type="button" class="cg-desc-line" aria-label={`${yaml}。${line.description}`}>
              <span class="cg-line-number" aria-hidden="true">{index + 1}</span>
              <code>{yaml}</code>
              <span class="cg-tooltip" role="tooltip">{line.description}</span>
            </button>
          {/each}
        </div>
      </article>
    {/if}
  </div>
</section>
