const NARROW_QUERY = '(max-width: 959px)';
const MEDIUM_QUERY = '(max-width: 1279px)';

/// A single source of truth for the layout breakpoints used across the workbench.
export class ViewportStore {
  narrow = $state(false);
  medium = $state(false);
  #cleanup?: () => void;

  constructor() {
    if (typeof window === 'undefined') return;
    const narrow = window.matchMedia(NARROW_QUERY);
    const medium = window.matchMedia(MEDIUM_QUERY);
    const sync = () => {
      this.narrow = narrow.matches;
      this.medium = medium.matches;
    };
    sync();
    narrow.addEventListener('change', sync);
    medium.addEventListener('change', sync);
    this.#cleanup = () => {
      narrow.removeEventListener('change', sync);
      medium.removeEventListener('change', sync);
    };
  }

  dispose(): void {
    this.#cleanup?.();
    this.#cleanup = undefined;
  }
}
