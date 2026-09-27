export type ThemeId = 'latte' | 'frappe' | 'macchiato' | 'mocha';

export interface ThemeOption {
  id: ThemeId;
  dark: boolean;
}

/// Theme registry: adding a flavor means one entry here, one label in the message catalog, and
/// one palette block in theme.css.
export const THEMES: readonly ThemeOption[] = [
  { id: 'latte', dark: false },
  { id: 'frappe', dark: true },
  { id: 'macchiato', dark: true },
  { id: 'mocha', dark: true },
];

const DARK_QUERY = '(prefers-color-scheme: dark)';

/// In-memory theme; deliberately not persisted so no preference leaves the page.
export class ThemeStore {
  current = $state<ThemeId>('latte');

  constructor() {
    if (typeof window !== 'undefined') {
      this.current = window.matchMedia(DARK_QUERY).matches ? 'mocha' : 'latte';
    }
  }

  select(id: string): void {
    if (THEMES.some(theme => theme.id === id)) this.current = id as ThemeId;
  }
}
