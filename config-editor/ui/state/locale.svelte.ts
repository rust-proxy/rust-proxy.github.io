import { m } from '../paraglide/messages';
import { baseLocale, isLocale, locales, overwriteGetLocale } from '../paraglide/runtime';
import { readLangParam, writeLangParam } from './url';

export type Locale = (typeof locales)[number];

/// The base locale stays the default so the docs language and existing links are unchanged;
/// other locales are opt-in through the explicit `lang` parameter.
export function resolveLocale(requested: string | null): Locale {
  if (requested && isLocale(requested)) return requested;
  return baseLocale;
}

/// Locale is fixed per document load; switching writes the URL and reloads, so the generated
/// message functions need no reactive bridge. Kept in sync with the WASM session locale.
export class LocaleStore {
  readonly current: Locale;

  constructor() {
    this.current = resolveLocale(readLangParam(window.location.search));
    overwriteGetLocale(() => this.current);
    document.documentElement.lang = this.current;
  }

  select(code: string): void {
    if (!isLocale(code) || code === this.current) return;
    writeLangParam(code);
    window.location.reload();
  }
}

export function localeName(code: string): string {
  return code === 'en' ? m.locale_en() : m.locale_zh_cn();
}
