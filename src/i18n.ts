// src/i18n.ts
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import enTranslation from './locales/en.json';
import zhTranslation from './locales/zh.json';
import zhTWTranslation from './locales/zh-TW.json';
import jaTranslation from './locales/ja.json';

export const SUPPORTED_LANGUAGES = ['en', 'zh', 'zh-TW', 'ja'] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export function detectLanguage(locale: string): SupportedLanguage {
  const lower = locale.toLowerCase();
  if (lower.startsWith('zh-tw') || lower.startsWith('zh-hk') || lower.startsWith('zh-mo') || lower.startsWith('zh-hant')) {
    return 'zh-TW';
  }
  if (lower.startsWith('zh')) {
    return 'zh';
  }
  if (lower.startsWith('ja')) {
    return 'ja';
  }
  return 'en';
}

export const initI18n = async () => {
  // Get saved or system language from backend
  let userLang: SupportedLanguage = 'en';
  try {
    userLang = await invoke<SupportedLanguage>('get_language');
  } catch (error) {
    console.warn('Could not get language from backend, defaulting to en:', error);
  }

  const resources = {
    en: { translation: enTranslation },
    zh: { translation: zhTranslation },
    'zh-TW': { translation: zhTWTranslation },
    ja: { translation: jaTranslation },
  };

  await i18n
    .use(initReactI18next)
    .init({
      resources,
      lng: userLang,
      fallbackLng: 'en',
      interpolation: {
        escapeValue: false
      },
      react: {
        useSuspense: false
      }
    });
};

export default i18n;
