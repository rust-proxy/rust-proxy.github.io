import { beforeEach, describe, expect, it } from 'vitest';
import { readLangParam, readSchemaParam, writeLangParam, writeSchemaParam } from './url';

beforeEach(() => {
  window.history.replaceState({}, '', '/config-editor/?retained=yes#query-state');
});

describe('URL synchronization', () => {
  it('reads the schema parameter and reports its absence', () => {
    expect(readSchemaParam('?retained=yes&schema=beta')).toBe('beta');
    expect(readSchemaParam('?retained=yes')).toBeNull();
  });

  it('writes the schema without dropping unrelated parameters or the fragment', () => {
    writeSchemaParam('beta');
    const url = new URL(window.location.href);
    expect(url.searchParams.get('schema')).toBe('beta');
    expect(url.searchParams.get('retained')).toBe('yes');
    expect(url.hash).toBe('#query-state');
  });

  it('reads and writes the language parameter alongside other state', () => {
    expect(readLangParam('?lang=en')).toBe('en');
    expect(readLangParam('?retained=yes')).toBeNull();
    writeLangParam('en');
    const url = new URL(window.location.href);
    expect(url.searchParams.get('lang')).toBe('en');
    expect(url.searchParams.get('retained')).toBe('yes');
    expect(url.hash).toBe('#query-state');
  });
});
