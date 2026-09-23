const DARK_QUERY = '(prefers-color-scheme: dark)';

/// In-memory theme; deliberately not persisted so no preference leaves the page.
export class ThemeStore {
  dark = $state(false);

  constructor() {
    if (typeof window !== 'undefined') this.dark = window.matchMedia(DARK_QUERY).matches;
  }

  toggle(): void {
    this.dark = !this.dark;
  }
}
