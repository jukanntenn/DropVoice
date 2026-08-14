import { initReactI18next } from 'react-i18next';
import i18n from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import enCommon from '../locales/en/common.json';
import enDevices from '../locales/en/devices.json';
import enErrors from '../locales/en/errors.json';
import enSettings from '../locales/en/settings.json';
import jaCommon from '../locales/ja/common.json';
import jaDevices from '../locales/ja/devices.json';
import jaErrors from '../locales/ja/errors.json';
import jaSettings from '../locales/ja/settings.json';
import zhCommon from '../locales/zh/common.json';
import zhDevices from '../locales/zh/devices.json';
import zhErrors from '../locales/zh/errors.json';
import zhSettings from '../locales/zh/settings.json';
import zhTwCommon from '../locales/zh-TW/common.json';
import zhTwDevices from '../locales/zh-TW/devices.json';
import zhTwErrors from '../locales/zh-TW/errors.json';
import zhTwSettings from '../locales/zh-TW/settings.json';

export const SUPPORTED_LANGUAGES = ['en', 'zh', 'zh-TW', 'ja'] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export const DEFAULT_LANGUAGE: SupportedLanguage = 'en';

export const resources = {
  en: {
    common: enCommon,
    devices: enDevices,
    errors: enErrors,
    settings: enSettings,
  },
  zh: {
    common: zhCommon,
    devices: zhDevices,
    errors: zhErrors,
    settings: zhSettings,
  },
  'zh-TW': {
    common: zhTwCommon,
    devices: zhTwDevices,
    errors: zhTwErrors,
    settings: zhTwSettings,
  },
  ja: {
    common: jaCommon,
    devices: jaDevices,
    errors: jaErrors,
    settings: jaSettings,
  },
} as const;

export const NAMESPACES = ['common', 'devices', 'errors', 'settings'] as const;
export type Namespace = (typeof NAMESPACES)[number];

export interface InitOptions {
  language?: SupportedLanguage;
  lookupLocalStorage?: string;
}

/**
 * Initialize i18next with React binding and language detector.
 *
 * Safe to call multiple times — subsequent calls are no-ops once i18next is
 * initialized. Resolves with the shared i18n instance once initialization
 * completes (or immediately if already initialized).
 */
export async function initI18n(options: InitOptions = {}): Promise<typeof i18n> {
  if (i18n.isInitialized) {
    return i18n;
  }

  await i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
      resources,
      fallbackLng: DEFAULT_LANGUAGE,
      supportedLngs: SUPPORTED_LANGUAGES as unknown as string[],
      ns: NAMESPACES as unknown as string[],
      defaultNS: 'common',
      lng: options.language,
      interpolation: {
        escapeValue: false,
      },
      detection: {
        order: ['querystring', 'localStorage', 'navigator'],
        lookupQuerystring: 'lang',
        lookupLocalStorage: options.lookupLocalStorage ?? 'dropvoice-lang',
        caches: ['localStorage'],
      },
      react: {
        useSuspense: false,
      },
    });

  return i18n;
}

/**
 * Change the active language at runtime.
 */
export function changeLanguage(language: SupportedLanguage): Promise<void> {
  return i18n.changeLanguage(language).then(() => undefined);
}

/**
 * Validate that a string is a supported language code.
 */
export function isSupportedLanguage(value: unknown): value is SupportedLanguage {
  return typeof value === 'string' && (SUPPORTED_LANGUAGES as readonly string[]).includes(value);
}
