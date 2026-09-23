import { Engine, schemas as readSchemas } from '../../pkg/engine';
import type { Action, ExportFile, Snapshot } from '../types';

export interface SchemaInfo { name: string; label: string }

function toMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message || fallback;
  if (typeof error === 'string' && error) return error;
  return fallback;
}

function parse<T>(value: string, fallback: string): T {
  try { return JSON.parse(value) as T; }
  catch { throw new Error(fallback); }
}

export function listSchemas(): SchemaInfo[] {
  try {
    return (JSON.parse(readSchemas()) as [string, string][]).map(([name, label]) => ({ name, label }));
  } catch {
    throw new Error('无法读取配置方案。');
  }
}

export function isSchema(name: string, schemas: SchemaInfo[]): boolean {
  return schemas.some(item => item.name === name);
}

// Owns one WASM `Engine` and confines JSON (de)serialization to the boundary.
export class EngineSession {
  readonly schema: string;
  #engine: Engine;

  constructor(schema: string) {
    this.schema = schema;
    try { this.#engine = new Engine(schema); }
    catch (error) { throw new Error(toMessage(error, '无法创建配置会话。')); }
  }

  initialize(): void {
    try { this.#engine.initialize(); }
    catch (error) { throw new Error(toMessage(error, '初始化配置失败。')); }
  }

  dispatch(action: Action): void {
    try { this.#engine.dispatch(JSON.stringify(action)); }
    catch (error) { throw new Error(toMessage(error, '无效的编辑操作。')); }
  }

  snapshot(selected: string, reveal: boolean): Snapshot {
    try { return parse<Snapshot>(this.#engine.snapshot(selected, reveal), '无法读取界面状态。'); }
    catch (error) { throw new Error(toMessage(error, '无法读取界面状态。')); }
  }

  export(selected: string): ExportFile {
    try { return parse<ExportFile>(this.#engine.export(selected), '无法导出配置。'); }
    catch (error) { throw new Error(toMessage(error, '无法导出配置。')); }
  }

  dispose(): void { this.#engine.free(); }
}
