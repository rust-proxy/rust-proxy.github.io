export type ThemeId = 'latte' | 'frappe' | 'macchiato' | 'mocha';

export interface ThemeOption {
  id: ThemeId;
  label: string;
  dark: boolean;
}

/// Theme registry: adding a flavor means one entry here and one palette block in theme.css.
export const THEMES: readonly ThemeOption[] = [
  { id: 'latte', label: 'Latte（浅色）', dark: false },
  { id: 'frappe', label: 'Frappé', dark: true },
  { id: 'macchiato', label: 'Macchiato', dark: true },
  { id: 'mocha', label: 'Mocha', dark: true },
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
