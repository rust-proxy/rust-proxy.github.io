import { mount, unmount } from 'svelte';
import init from '../pkg/engine';
import wasmUrl from '../pkg/engine_bg.wasm?url';
import App from './App.svelte';
import { SessionStore } from './state/session.svelte';
import { ThemeStore } from './state/theme.svelte';
import { ViewportStore } from './state/viewport.svelte';
import { readSchemaParam, writeSchemaParam } from './state/url';
import { WorkbenchStore } from './state/workbench.svelte';
import './styles/index.css';

async function start() {
  const loading = document.getElementById('ce-loading');
  try {
    await init({ module_or_path: wasmUrl });
    const target = document.getElementById('app');
    if (!target) throw new Error('找不到应用容器。');
    const request = readSchemaParam(window.location.search);
    const session = new SessionStore(request ?? '');
    if (request !== null && request !== session.schema) writeSchemaParam(session.schema);
    const viewport = new ViewportStore();
    const theme = new ThemeStore();
    const workbench = new WorkbenchStore(() => viewport.narrow);
    const app = mount(App, { target, props: { session, theme, workbench } });
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
      loading.textContent = '配置编辑器加载失败，请刷新页面并确认浏览器支持 WebAssembly。';
    }
  }
}

void start();
