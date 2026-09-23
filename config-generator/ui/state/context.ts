import { getContext, setContext } from 'svelte';
import type { SessionStore } from './session.svelte';
import type { WorkbenchStore } from './workbench.svelte';

const SESSION = Symbol('cg-session');
const WORKBENCH = Symbol('cg-workbench');

export function setSessions(session: SessionStore, workbench: WorkbenchStore): void {
  setContext(SESSION, session);
  setContext(WORKBENCH, workbench);
}

export function useSession(): SessionStore {
  return getContext<SessionStore>(SESSION);
}

export function useWorkbench(): WorkbenchStore {
  return getContext<WorkbenchStore>(WORKBENCH);
}
