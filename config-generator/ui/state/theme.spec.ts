import { describe, expect, it, vi } from 'vitest';
import { ThemeStore } from './theme.svelte';

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
  it('starts from the system preference and toggles in memory', () => {
    stubMatchMedia(true);
    const theme = new ThemeStore();
    expect(theme.dark).toBe(true);
    theme.toggle();
    expect(theme.dark).toBe(false);
    theme.toggle();
    expect(theme.dark).toBe(true);
  });
});
