import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => {
  const instance = {
    initialize: vi.fn(),
    dispatch: vi.fn(),
    snapshot: vi.fn(),
    export: vi.fn(),
    free: vi.fn(),
  };
  function EngineCtor() { return instance; }
  return { instance, Engine: vi.fn(EngineCtor), schemas: vi.fn() };
});

vi.mock('../../pkg/engine', () => ({ Engine: mocks.Engine, schemas: mocks.schemas }));

const { EngineSession, isSchema, listSchemas } = await import('./engine');

beforeEach(() => {
  mocks.instance.initialize.mockReset();
  mocks.instance.dispatch.mockReset();
  mocks.instance.snapshot.mockReset();
  mocks.instance.export.mockReset();
  mocks.instance.free.mockReset();
});

describe('listSchemas', () => {
  it('maps the registry entries', () => {
    mocks.schemas.mockReturnValue(JSON.stringify([['alpha', 'Alpha'], ['beta', 'Beta']]));
    expect(listSchemas()).toEqual([{ name: 'alpha', label: 'Alpha' }, { name: 'beta', label: 'Beta' }]);
  });

  it('reports a safe message on malformed registry data', () => {
    mocks.schemas.mockReturnValue('not json');
    expect(() => listSchemas()).toThrow('无法读取配置方案。');
  });

  it('rejects unknown schema identifiers', () => {
    const schemas = [{ name: 'alpha', label: 'Alpha' }];
    expect(isSchema('alpha', schemas)).toBe(true);
    expect(isSchema('other', schemas)).toBe(false);
  });
});

describe('EngineSession', () => {
  it('parses snapshots and exports at the boundary', () => {
    const session = new EngineSession('alpha');
    expect(mocks.Engine).toHaveBeenCalledWith('alpha');
    mocks.instance.snapshot.mockReturnValue('{"valid":true}');
    expect(session.snapshot('out', true)).toEqual({ valid: true });
    expect(mocks.instance.snapshot).toHaveBeenCalledWith('out', true);
    mocks.instance.export.mockReturnValue(JSON.stringify({ filename: 'a.json', text: '{}' }));
    expect(session.export('out')).toEqual({ filename: 'a.json', text: '{}' });
  });

  it('serializes actions and reports failures', () => {
    const session = new EngineSession('alpha');
    session.dispatch({ type: 'set', field: 'host', value: 'x' });
    expect(mocks.instance.dispatch).toHaveBeenCalledWith(JSON.stringify({ type: 'set', field: 'host', value: 'x' }));

    mocks.instance.snapshot.mockReturnValue('{');
    expect(() => session.snapshot('', false)).toThrow('无法读取界面状态。');

    mocks.instance.export.mockReturnValue('broken');
    expect(() => session.export('')).toThrow('无法导出配置。');
  });

  it('wraps thrown values from the WASM constructor', () => {
    mocks.Engine.mockImplementationOnce(() => { throw new Error('未知方案'); });
    expect(() => new EngineSession('bad')).toThrow('未知方案');
  });

  it('frees the WASM engine on dispose', () => {
    const session = new EngineSession('alpha');
    session.dispose();
    expect(mocks.instance.free).toHaveBeenCalled();
  });
});
