import { describe, expect, it, vi } from 'vitest';
import { ViewportStore } from './viewport.svelte';

function stubMatchMedia() {
  const narrowListeners = new Set<() => void>();
  const mediumListeners = new Set<() => void>();
  const make = (matches: boolean, listeners: Set<() => void>) => ({
    matches,
    media: '',
    onchange: null,
    addEventListener: vi.fn((_type: string, callback: () => void) => { listeners.add(callback); }),
    removeEventListener: vi.fn((_type: string, callback: () => void) => { listeners.delete(callback); }),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  });
  const narrow = make(false, narrowListeners);
  const medium = make(true, mediumListeners);
  vi.stubGlobal('matchMedia', vi.fn((query: string) => (query.includes('959') ? narrow : medium)));
  return { narrow, medium };
}

describe('ViewportStore', () => {
  it('reads both breakpoints from a single source', () => {
    stubMatchMedia();
    const viewport = new ViewportStore();
    expect(viewport.narrow).toBe(false);
    expect(viewport.medium).toBe(true);
    viewport.dispose();
  });

  it('detaches media query listeners on dispose', () => {
    const { narrow, medium } = stubMatchMedia();
    const viewport = new ViewportStore();
    viewport.dispose();
    expect(narrow.removeEventListener).toHaveBeenCalledTimes(1);
    expect(medium.removeEventListener).toHaveBeenCalledTimes(1);
  });
});
