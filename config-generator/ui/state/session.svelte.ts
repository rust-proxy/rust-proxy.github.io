import { EngineSession, isSchema, listSchemas, type SchemaInfo } from '../bridge/engine';
import type { Action, ExportFile, FieldView, SectionView, Snapshot } from '../types';

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function sectionErrorCount(section: SectionView): number {
  let count = section.fields.filter(field => field.visible && field.error).length;
  for (const collection of section.collections) {
    if (!collection.visible) continue;
    for (const row of collection.rows) {
      if (row.visible) count += row.fields.filter(field => field.visible && field.error).length;
    }
    if (collection.selector?.visible && collection.selector.error) count += 1;
  }
  return count;
}

export interface SearchEntry { path: string; label: string; hint: string; section: string }

function collectSearchable(sections: SectionView[]): SearchEntry[] {
  const entries: SearchEntry[] = [];
  const push = (section: SectionView, field: FieldView) => {
    if (field.visible) entries.push({ path: field.path, label: field.label, hint: field.hint, section: section.label });
  };
  for (const section of sections) {
    if (!section.visible) continue;
    section.fields.forEach(field => push(section, field));
    for (const collection of section.collections) {
      if (!collection.visible) continue;
      for (const row of collection.rows) {
        if (row.visible) row.fields.forEach(field => push(section, field));
      }
      if (collection.selector) push(section, collection.selector);
    }
  }
  return entries;
}

const EMPTY_SNAPSHOT: Snapshot = {
  ui: { title: '', brand: '', mark: '', eyebrow: '', description: '', export_hint: '' },
  sections: [],
  mode: null,
  format: null,
  notices: [],
  errors: {},
  outputs: [],
  selected: '',
  filename: '',
  command: '',
  preview_lines: [],
  valid: false,
};

/// The single mutable view of a WASM session. Components never own configuration state.
export class SessionStore {
  readonly schemas: SchemaInfo[];
  reveal = $state(false);
  schema = $state('');
  snapshot = $state.raw<Snapshot>(EMPTY_SNAPSHOT);
  status = $state('');
  selected = '';
  #engine: EngineSession | null = null;

  ui = $derived(this.snapshot.ui);
  sections = $derived(this.snapshot.sections);
  visibleSections = $derived(this.sections.filter(section => section.visible));
  errors = $derived(Object.entries(this.snapshot.errors));
  valid = $derived(this.snapshot.valid);
  errorCounts = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const section of this.sections) counts.set(section.name, sectionErrorCount(section));
    return counts;
  });
  searchable = $derived.by(() => collectSearchable(this.sections));

  constructor(requested = '') {
    this.schemas = listSchemas();
    const initial = isSchema(requested, this.schemas) ? requested : (this.schemas[0]?.name ?? '');
    this.schema = initial;
    this.#open(initial);
  }

  errorCount(section: SectionView): number {
    return this.errorCounts.get(section.name) ?? 0;
  }

  #open(name: string): void {
    let engine: EngineSession;
    try { engine = new EngineSession(name); }
    catch (error) { this.status = message(error); return; }
    this.#engine?.dispose();
    this.#engine = engine;
    this.schema = name;
    this.selected = '';
    this.reveal = false;
    try { engine.initialize(); }
    catch (error) { this.status = message(error); }
    this.#refresh();
  }

  #refresh(): void {
    if (!this.#engine) return;
    try { this.snapshot = this.#engine.snapshot(this.selected, this.reveal); }
    catch (error) { this.status = message(error); }
  }

  dispatch = (action: Action) => {
    if (!this.#engine) return;
    try {
      this.#engine.dispatch(action);
      this.status = action.type.startsWith('generate') ? '已更新随机值。' : '';
      this.#refresh();
    } catch (error) { this.status = message(error); }
  };

  selectSchema = (name: string) => {
    if (name === this.schema || !isSchema(name, this.schemas)) return;
    this.status = '';
    this.#open(name);
  };

  selectOutput = (name: string) => {
    this.selected = name;
    this.#refresh();
  };

  setReveal = (reveal: boolean) => {
    this.reveal = reveal;
    this.#refresh();
  };

  export(): ExportFile {
    if (!this.#engine) throw new Error('配置会话不可用。');
    return this.#engine.export(this.selected);
  }

  dispose(): void {
    this.#engine?.dispose();
    this.#engine = null;
  }
}
