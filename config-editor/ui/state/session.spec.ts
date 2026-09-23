import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { CollectionView, FieldView, SectionView, Snapshot } from '../types';

const mocks = vi.hoisted(() => ({
  engine: {
    schema: '',
    initialize: vi.fn(),
    dispatch: vi.fn(),
    snapshot: vi.fn(),
    export: vi.fn(),
    dispose: vi.fn(),
  },
  failConstruction: false,
}));

vi.mock('../bridge/engine', () => ({
  EngineSession: vi.fn(function (name: string) {
    if (mocks.failConstruction) throw new Error('会话创建失败');
    mocks.engine.schema = name;
    return mocks.engine;
  }),
  listSchemas: vi.fn(() => [{ name: 'alpha', label: 'Alpha' }, { name: 'beta', label: 'Beta' }]),
  isSchema: (name: string, schemas: { name: string }[]) => schemas.some(item => item.name === name),
}));

const { SessionStore } = await import('./session.svelte');

function field(key: string, overrides: Partial<FieldView> = {}): FieldView {
  return { key, path: key, label: key, hint: '', placeholder: '', kind: 'text', options: [], value: '', visible: true, error: '', generated: false, ...overrides };
}

function snapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
    ui: { title: 'T', brand: 'B', mark: 'M', eyebrow: 'E', description: 'D', export_hint: 'H' },
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
    ...overrides,
  };
}

beforeEach(() => {
  mocks.failConstruction = false;
  vi.clearAllMocks();
  mocks.engine.schema = '';
  mocks.engine.initialize.mockReset();
  mocks.engine.snapshot.mockReturnValue(snapshot());
  mocks.engine.export.mockReturnValue({ filename: 'out.json', text: '{}' });
});

describe('SessionStore', () => {
  it('registers schemas and falls back to the first one for unknown requests', () => {
    const store = new SessionStore('missing');
    expect(store.schemas.map(item => item.name)).toEqual(['alpha', 'beta']);
    expect(store.schema).toBe('alpha');
    expect(store.selected).toBe('');
    expect(mocks.engine.snapshot).toHaveBeenCalledWith('', false);
  });

  it('keeps the session but reports status when initialization fails', () => {
    mocks.engine.initialize.mockImplementationOnce(() => { throw new Error('无法安全生成随机值'); });
    const store = new SessionStore('alpha');
    expect(store.status).toBe('无法安全生成随机值');
    expect(store.snapshot).toBe(mocks.engine.snapshot.mock.results.at(-1)?.value);
  });

  it('dispatches edits, refreshes and reports generated updates', () => {
    const store = new SessionStore('alpha');
    const next = snapshot({ valid: true });
    mocks.engine.snapshot.mockReturnValue(next);
    store.dispatch({ type: 'set', field: 'x', value: '1' });
    expect(mocks.engine.dispatch).toHaveBeenCalledWith({ type: 'set', field: 'x', value: '1' });
    expect(store.snapshot).toBe(next);
    expect(store.status).toBe('');
    store.dispatch({ type: 'generate', field: 'x' });
    expect(store.status).toBe('已更新随机值。');
  });

  it('switches schema, resets reveal and selection, and frees the old session', () => {
    const store = new SessionStore('alpha');
    store.setReveal(true);
    store.selectOutput('selected');
    const previous = mocks.engine;
    mocks.engine.initialize.mockClear();
    store.selectSchema('beta');
    expect(store.schema).toBe('beta');
    expect(store.reveal).toBe(false);
    expect(store.selected).toBe('');
    expect(previous.dispose).toHaveBeenCalled();
    expect(previous.initialize).toHaveBeenCalled();
  });

  it('ignores unknown schemas and reports construction failures without dropping the session', () => {
    const store = new SessionStore('alpha');
    store.selectSchema('missing');
    expect(store.schema).toBe('alpha');
    mocks.failConstruction = true;
    store.selectSchema('beta');
    expect(store.schema).toBe('alpha');
    expect(store.status).toBe('会话创建失败');
  });

  it('delegates export to the engine', () => {
    const store = new SessionStore('alpha');
    expect(store.export()).toEqual({ filename: 'out.json', text: '{}' });
  });

  it('derives section error counts from visible fields, rows and selector', () => {
    const collection: CollectionView = {
      name: 'items', label: 'Items', hint: '', add_label: 'Add', generate_label: 'Gen', generated: false,
      visible: true, editable: true, removable: false,
      selector: field('items.choose', { error: '请选择', visible: true }),
      rows: [
        { id: '1', number: 1, visible: true, fields: [field('items.0.name', { error: '必填' }), field('items.0.hidden', { error: '忽略', visible: false })] },
        { id: '2', number: 2, visible: false, fields: [field('items.1.name', { error: '忽略' })] },
      ],
    };
    const section: SectionView = {
      name: 'general', label: 'General', detail: '', collapsed: false, visible: true,
      fields: [field('host', { error: '必填' }), field('port')],
      collections: [collection],
      notices: [],
    };
    const store = new SessionStore('alpha');
    mocks.engine.snapshot.mockReturnValue(snapshot({ sections: [section] }));
    store.dispatch({ type: 'set', field: 'host', value: 'x' });
    expect(store.errorCount(section)).toBe(3);
    expect(store.errors).toHaveLength(0);
  });

  it('indexes only visible searchable fields without values', () => {
    const collection: CollectionView = {
      name: 'items', label: 'Items', hint: '', add_label: 'Add', generate_label: 'Gen', generated: false,
      visible: true, editable: true, removable: false,
      selector: field('items.choose', { label: '选择项', visible: true }),
      rows: [{ id: '1', number: 1, visible: true, fields: [field('items.0.name', { label: '名称', hint: '提示' })] }],
    };
    const section: SectionView = {
      name: 'general', label: 'General', detail: '', collapsed: false, visible: true,
      fields: [field('host', { label: '主机' }), field('hidden', { visible: false })],
      collections: [collection],
      notices: [],
    };
    const store = new SessionStore('alpha');
    mocks.engine.snapshot.mockReturnValue(snapshot({ sections: [section] }));
    store.dispatch({ type: 'set', field: 'host', value: 'x' });
    expect(store.searchable.map(entry => entry.path)).toEqual(['host', 'items.0.name', 'items.choose']);
    expect(store.searchable.find(entry => entry.path === 'items.0.name')?.hint).toBe('提示');
  });
});
