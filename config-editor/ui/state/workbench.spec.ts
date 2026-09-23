import { beforeEach, describe, expect, it, vi } from 'vitest';
import { WorkbenchStore } from './workbench.svelte';

beforeEach(() => {
  document.body.innerHTML = '';
  Element.prototype.scrollIntoView = vi.fn();
  window.scrollTo = vi.fn() as unknown as typeof window.scrollTo;
});

describe('WorkbenchStore', () => {
  it('resets to the editor pane', () => {
    const workbench = new WorkbenchStore(() => true);
    workbench.reset();
    expect(workbench.pane).toBe('editor');
  });

  it('focuses the target field when navigating without switching panes', async () => {
    document.body.innerHTML = '<input id="ce-host">';
    const workbench = new WorkbenchStore(() => false);
    await workbench.navigateToField('host');
    expect(workbench.pane).toBe('editor');
    expect(document.activeElement).toBe(document.getElementById('ce-host'));
  });

  it('switches panes and repositions the viewport', async () => {
    const workbench = new WorkbenchStore(() => true);
    await workbench.switchPane('preview');
    expect(workbench.pane).toBe('preview');
    expect(window.scrollTo).toHaveBeenCalledWith(0, 0);
  });
});
