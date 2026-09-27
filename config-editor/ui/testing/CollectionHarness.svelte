<script lang="ts">
  import CollectionEditor from '../components/CollectionEditor.svelte';
  import { setSessions } from '../state/context';
  import { SessionStore } from '../state/session.svelte';
  import { WorkbenchStore } from '../state/workbench.svelte';

  let { schema = 'tuic-server', name }: { schema?: string; name: string } = $props();
  // svelte-ignore state_referenced_locally
  const session = new SessionStore(schema);
  const workbench = new WorkbenchStore(() => false);
  setSessions(session, workbench);

  const collection = $derived(session.snapshot.sections.flatMap(section => section.collections).find(item => item.name === name));
</script>

{#if collection}<CollectionEditor {collection} />{/if}
