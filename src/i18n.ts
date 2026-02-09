// src/i18n.ts
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import enTranslation from './locales/en.json';
import zhTranslation from './locales/zh.json';

export const initI18n = async () => {
  // Get saved or system language from backend
  let userLang = 'en';
  try {
    userLang = await invoke<string>('get_language');
  } catch (error) {
    console.warn('Could not get language from backend, defaulting to en:', error);
  }

  const resources = {
    en: { translation: enTranslation },
    zh: { translation: zhTranslation }
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
