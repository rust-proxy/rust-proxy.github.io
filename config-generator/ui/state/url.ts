const SCHEMA_PARAM = 'schema';

export function readSchemaParam(search: string): string | null {
  return new URLSearchParams(search).get(SCHEMA_PARAM);
}

export function writeSchemaParam(name: string): void {
  const url = new URL(window.location.href);
  url.searchParams.set(SCHEMA_PARAM, name);
  window.history.replaceState(window.history.state, '', url);
}
