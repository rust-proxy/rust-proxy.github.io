<script lang="ts">
  import FieldControl from '../components/FieldControl.svelte';
  import { setSessions } from '../state/context';
  import { SessionStore } from '../state/session.svelte';
  import { WorkbenchStore } from '../state/workbench.svelte';
  import type { FieldView } from '../types';

  let { schema = 'tuic-server', path }: { schema?: string; path: string } = $props();
  // svelte-ignore state_referenced_locally
  const session = new SessionStore(schema);
  const workbench = new WorkbenchStore(() => false);
  setSessions(session, workbench);

  const field = $derived.by<FieldView | undefined>(() => {
    const match = (fields: FieldView[]) => fields.find(item => item.path === path);
    for (const section of session.snapshot.sections) {
      const direct = match(section.fields);
      if (direct) return direct;
      for (const collection of section.collections) {
        if (collection.selector?.path === path) return collection.selector;
        for (const row of collection.rows) {
          const nested = match(row.fields);
          if (nested) return nested;
        }
      }
    }
    return undefined;
  });
</script>

{#if field}<FieldControl {field} />{/if}
