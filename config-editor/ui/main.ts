import { mount, unmount } from 'svelte';
import init from '../pkg/engine';
import wasmUrl from '../pkg/engine_bg.wasm?url';
import { m } from './paraglide/messages';
import App from './App.svelte';
import { LocaleStore } from './state/locale.svelte';
import { SessionStore } from './state/session.svelte';
import { ThemeStore } from './state/theme.svelte';
import { ViewportStore } from './state/viewport.svelte';
import { readSchemaParam, writeSchemaParam } from './state/url';
import { WorkbenchStore } from './state/workbench.svelte';
import './styles/index.css';

async function start() {
  const loading = document.getElementById('editor-loading');
  const locale = new LocaleStore();
  try {
    await init({ module_or_path: wasmUrl });
    const target = document.getElementById('app');
    if (!target) throw new Error(m.main_mount_missing());
    const request = readSchemaParam(window.location.search);
    const session = new SessionStore(request ?? '', locale.current);
    if (request !== null && request !== session.schema) writeSchemaParam(session.schema);
    const viewport = new ViewportStore();
    const theme = new ThemeStore();
    const workbench = new WorkbenchStore(() => viewport.narrow);
    const app = mount(App, { target, props: { session, theme, workbench, locale } });
    // App component hot replacement reuses its stores; only the owner frees WASM.
    import.meta.hot?.dispose(() => {
      void unmount(app);
      viewport.dispose();
      session.dispose();
    });
    loading?.remove();
  } catch (error) {
    console.error(error);
    if (loading) {
      loading.setAttribute('role', 'alert');
      loading.textContent = m.main_load_failed();
    }
  }
}

void start();
