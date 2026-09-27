import { describe, expect, it, vi } from 'vitest';
import { THEMES, ThemeStore } from './theme.svelte';

function stubMatchMedia(matches: boolean) {
  const mql = {
    matches,
    media: '',
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  };
  vi.stubGlobal('matchMedia', vi.fn(() => mql));
  return mql;
}

describe('ThemeStore', () => {
  it('starts from the system preference and keeps themes in memory', () => {
    stubMatchMedia(true);
    const dark = new ThemeStore();
    expect(dark.current).toBe('mocha');
    stubMatchMedia(false);
    const light = new ThemeStore();
    expect(light.current).toBe('latte');
  });

  it('selects a registered flavor and ignores unknown ids', () => {
    stubMatchMedia(false);
    const theme = new ThemeStore();
    expect(THEMES.map(option => option.id)).toContain('frappe');
    theme.select('frappe');
    expect(theme.current).toBe('frappe');
    theme.select('unknown');
    expect(theme.current).toBe('frappe');
  });
});
