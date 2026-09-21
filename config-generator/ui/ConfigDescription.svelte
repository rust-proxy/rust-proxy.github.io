<script lang="ts">
  import type { ConfigDescription } from './types';
  import { highlightLines } from './prism';

  let { description }: { description: ConfigDescription } = $props();
  let selected = $state('');
  let selectedFormat = $state('yaml');
  let selections = $state<Record<string, string>>({});
  let status = $state('');
  const config = $derived(description.configs.find(item => item.name === selected) ?? description.configs[0]);
  const format = $derived(config?.formats.find(item => item.name === selectedFormat) ?? config?.formats[0]);
  const lines = $derived(format?.lines.filter(line => line.conditions.every(([name, value]) => selections[name] === value)) ?? []);
  const rendered = $derived(highlightLines(lines.map(line => line.text), format?.name ?? 'yaml'));

  function selectConfig(name: string) {
    selected = name;
    status = '';
    const next = description.configs.find(item => item.name === name);
    selections = Object.fromEntries(next?.selectors.map(selector => [selector.name, selector.default]) ?? []);
    if (next && !next.formats.some(item => item.name === selectedFormat)) selectedFormat = next.formats[0]?.name ?? '';
  }

  function setSelection(name: string, value: string) {
    selections = { ...selections, [name]: value };
    status = '';
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(`${lines.map(line => line.text).join('\n')}\n`);
      status = '已复制当前显示的配置。';
    } catch {
      status = '浏览器未允许复制，请手动选择文本。';
    }
  }

  $effect(() => {
    if (!description.configs.some(item => item.name === selected)) selectConfig(description.configs[0]?.name ?? '');
  });
</script>

<section class="cg-desc" aria-labelledby="cg-desc-title">
  <div class="cg-desc-heading">
    <p class="cg-eyebrow">CONFIG DESC · {format?.label ?? ''}</p>
    <h1 id="cg-desc-title">{description.title}</h1>
    <p>{description.description}</p>
  </div>
  <div class="cg-desc-workspace">
    <aside class="cg-desc-controls" aria-label="配置类型、格式与变体">
      <fieldset>
        <legend>配置类型</legend>
        <div class="cg-desc-kinds">
          {#each description.configs as item (item.name)}
            <button type="button" aria-pressed={item.name === selected} onclick={() => selectConfig(item.name)}>{item.label}</button>
          {/each}
        </div>
      </fieldset>
      {#if config}
        {#if config.formats.length > 1}
          <label class="cg-desc-selector" for="cg-desc-format">
            <span>配置格式</span>
            <select id="cg-desc-format" value={selectedFormat} onchange={(event) => { selectedFormat = event.currentTarget.value; status = ''; }}>
              {#each config.formats as item (item.name)}<option value={item.name}>{item.label}</option>{/each}
            </select>
          </label>
        {/if}
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
      <p class="cg-hint">选择格式与枚举分支，右侧示例会同步展示对应的配置形态。</p>
    </aside>
    {#if config && format}
      <article class="cg-desc-document" aria-label={`${config.label} ${format.label} 配置详解`}>
        <header>
          <strong>{config.filename}.{format.extension}</strong>
          <div class="cg-desc-meta">
            <span>悬浮或聚焦任意配置项查看说明</span>
            <button type="button" onclick={copy}>复制配置</button>
          </div>
        </header>
        <p class="cg-desc-status" role="status" aria-live="polite">{status}</p>
        <div class="cg-desc-code">
          {#each lines as line, index (`${line.text}-${index}`)}
            <button type="button" class="cg-desc-line" aria-label={`${line.text}。${line.description}`}>
              <span class="cg-line-number" aria-hidden="true">{index + 1}</span>
              <code class="cg-hl">{@html rendered[index] ?? ''}</code>
              <span class="cg-tooltip" role="tooltip">{line.description}</span>
            </button>
          {/each}
        </div>
      </article>
    {/if}
  </div>
</section>
