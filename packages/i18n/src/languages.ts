import type { SupportedLanguage } from './config';

/**
 * A selectable language entry surfaced in the desktop/mobile settings UI.
 *
 * Structurally compatible with `HeaderLanguageOption` in `@dropvoice/ui` —
 * apps can pass `LANGUAGE_OPTIONS` straight to the `<Header languages=...>`
 * prop without an adapter.
 */
export interface LanguageOption {
  value: SupportedLanguage;
  label: string;
}

/**
 * The four supported languages (spec 15 §2.1) with their display labels.
 *
 * Kept here so the desktop and mobile apps share a single source of truth
 * instead of each maintaining a copy.
 */
export const LANGUAGE_OPTIONS: LanguageOption[] = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: '简体中文' },
  { value: 'zh-TW', label: '繁體中文' },
  { value: 'ja', label: '日本語' },
];
