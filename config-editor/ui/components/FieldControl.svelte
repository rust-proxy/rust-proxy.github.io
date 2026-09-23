<script lang="ts">
  import { useSession } from '../state/context';
  import type { FieldView } from '../types';

  let { field, row }: { field: FieldView; row?: { collection: string; id: string } } = $props();
  const session = useSession();
  const id = $derived(`ce-${field.path}`);
  const described = $derived([field.hint && `${id}-hint`, field.error && `${id}-error`].filter(Boolean).join(' ') || undefined);

  function change(value: string) {
    session.dispatch(row ? { type: 'set-row', ...row, field: field.key, value } : { type: 'set', field: field.key, value });
  }
</script>

<div class:ce-toggle={field.kind === 'toggle'} class="ce-field" hidden={!field.visible}>
  <label for={id}>{field.label}</label>
  {#if field.kind === 'toggle'}
    <input {id} type="checkbox" checked={field.value === 'true'}
      aria-describedby={described} aria-invalid={!!field.error}
      onchange={(event) => change(String(event.currentTarget.checked))} />
  {:else if field.kind === 'select'}
    <select {id} value={field.value} aria-describedby={described} aria-invalid={!!field.error}
      onchange={(event) => change(event.currentTarget.value)}>
      {#each field.options as [value, label] (value)}
        <option {value}>{label}</option>
      {/each}
    </select>
  {:else}
    <input {id} type={field.kind === 'number' ? 'text' : field.kind}
      value={field.value} placeholder={field.placeholder} autocomplete="off" spellcheck="false"
      inputmode={field.kind === 'number' ? 'numeric' : undefined}
      aria-describedby={described} aria-invalid={!!field.error}
      oninput={(event) => change(event.currentTarget.value)} />
  {/if}
  {#if field.hint}<span id={`${id}-hint`} class="ce-hint">{field.hint}</span>{/if}
  {#if field.error}<span id={`${id}-error`} class="ce-error">{field.error}</span>{/if}
  {#if !row && field.generated}
    <button type="button" onclick={() => session.dispatch({ type: 'generate', field: field.key })}>生成随机值</button>
  {/if}
</div>
