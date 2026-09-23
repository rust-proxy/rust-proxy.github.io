import { tick } from 'svelte';
import { focusField } from '../lib/dom';

export type Pane = 'editor' | 'preview';

/// Owns the editor/preview split, its scroll memory, and cross-pane navigation.
export class WorkbenchStore {
  pane = $state<Pane>('editor');
  #positions: Record<Pane, number> = { editor: 0, preview: 0 };
  #narrow: () => boolean;

  constructor(narrow: () => boolean) {
    this.#narrow = narrow;
  }

  async switchPane(next: Pane): Promise<void> {
    if (next === this.pane) return;
    this.#positions[this.pane] = window.scrollY;
    this.pane = next;
    await tick();
    window.scrollTo(0, this.#positions[next]);
  }

  async navigateToField(path: string): Promise<void> {
    if (this.#narrow()) await this.switchPane('editor');
    await tick();
    focusField(path);
  }

  async showPreview(): Promise<void> {
    if (this.#narrow()) await this.switchPane('preview');
    const output = document.getElementById('cg-preview');
    output?.focus();
    output?.scrollIntoView({ block: 'start' });
  }

  async skipToEditor(): Promise<void> {
    if (this.#narrow()) await this.switchPane('editor');
    document.getElementById('cg-search')?.focus();
  }

  reset(): void {
    this.pane = 'editor';
    this.#positions = { editor: 0, preview: 0 };
  }
}
