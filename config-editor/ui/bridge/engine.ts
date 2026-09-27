import { Engine, schemas as readSchemas } from '../../pkg/engine';
import { m } from '../paraglide/messages';
import type { Action, ExportFile, Snapshot } from '../types';

export interface SchemaInfo {
  name: string;
  labels: [string, string][];
  locales: string[];
  default_locale: string;
}

/// Selects the schema selector label for the active locale, falling back to the schema default.
export function schemaLabel(schema: SchemaInfo, locale: string): string {
  return (
    schema.labels.find(([tag]) => tag === locale)?.[1] ??
    schema.labels.find(([tag]) => tag === schema.default_locale)?.[1] ??
    schema.name
  );
}

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
    return JSON.parse(readSchemas()) as SchemaInfo[];
  } catch {
    throw new Error(m.bridge_schema_read_failed());
  }
}

export function isSchema(name: string, schemas: SchemaInfo[]): boolean {
  return schemas.some(item => item.name === name);
}

// Owns one WASM `Engine` and confines JSON (de)serialization to the boundary.
export class EngineSession {
  readonly schema: string;
  #engine: Engine;

  constructor(schema: string, locale: string) {
    this.schema = schema;
    try { this.#engine = new Engine(schema, locale); }
    catch (error) { throw new Error(toMessage(error, m.bridge_session_create_failed())); }
  }

  initialize(): void {
    try { this.#engine.initialize(); }
    catch (error) { throw new Error(toMessage(error, m.bridge_init_failed())); }
  }

  dispatch(action: Action): void {
    try { this.#engine.dispatch(JSON.stringify(action)); }
    catch (error) { throw new Error(toMessage(error, m.bridge_action_invalid())); }
  }

  snapshot(selected: string, reveal: boolean): Snapshot {
    try { return parse<Snapshot>(this.#engine.snapshot(selected, reveal), m.bridge_snapshot_failed()); }
    catch (error) { throw new Error(toMessage(error, m.bridge_snapshot_failed())); }
  }

  export(selected: string): ExportFile {
    try { return parse<ExportFile>(this.#engine.export(selected), m.bridge_export_failed()); }
    catch (error) { throw new Error(toMessage(error, m.bridge_export_failed())); }
  }

  dispose(): void { this.#engine.free(); }
}
