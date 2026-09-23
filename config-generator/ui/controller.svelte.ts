import { Engine, schemas as readSchemas } from '../pkg/engine';
import type { Action, ExportFile, Snapshot } from './types';

export class Controller {
  readonly schemas: { name: string; label: string }[];
  private engine: Engine;
  private selected = '';
  reveal = $state(false);
  schema = $state('');
  view: Snapshot = $state.raw({} as Snapshot);
  status = $state('');

  constructor(requested = '') {
    this.schemas = (JSON.parse(readSchemas()) as [string, string][]).map(([name, label]) => ({ name, label }));
    this.schema = this.schemas.some(item => item.name === requested) ? requested : (this.schemas[0]?.name ?? '');
    this.engine = new Engine(this.schema);
    try { this.engine.initialize(); }
    catch (error) { this.status = String(error); }
    this.refresh();
  }

  private read(): Snapshot {
    return JSON.parse(this.engine.snapshot(this.selected, this.reveal));
  }

  private refresh() { this.view = this.read(); }

  dispatch = (action: Action) => {
    try {
      this.engine.dispatch(JSON.stringify(action));
      this.status = action.type.startsWith('generate') ? '已更新随机值。' : '';
      this.refresh();
    } catch (error) { this.status = String(error); }
  };

  selectSchema = (name: string) => {
    if (name === this.schema || !this.schemas.some(item => item.name === name)) return;
    const engine = new Engine(name);
    this.status = '';
    try { engine.initialize(); }
    catch (error) { this.status = String(error); }
    this.engine.free();
    this.engine = engine;
    this.schema = name;
    this.selected = '';
    this.reveal = false;
    this.refresh();
  };

  select = (name: string) => { this.selected = name; this.refresh(); };
  showSecrets = (reveal: boolean) => { this.reveal = reveal; this.refresh(); };
  export(): ExportFile { return JSON.parse(this.engine.export(this.selected)); }
  dispose() { this.engine.free(); }
}
