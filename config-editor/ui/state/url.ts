const SCHEMA_PARAM = 'schema';
const LANG_PARAM = 'lang';

function readParam(search: string, key: string): string | null {
  return new URLSearchParams(search).get(key);
}

function writeParam(key: string, value: string): void {
  const url = new URL(window.location.href);
  url.searchParams.set(key, value);
  window.history.replaceState(window.history.state, '', url);
}

export function readSchemaParam(search: string): string | null {
  return readParam(search, SCHEMA_PARAM);
}

export function writeSchemaParam(name: string): void {
  writeParam(SCHEMA_PARAM, name);
}

export function readLangParam(search: string): string | null {
  return readParam(search, LANG_PARAM);
}

export function writeLangParam(locale: string): void {
  writeParam(LANG_PARAM, locale);
}
