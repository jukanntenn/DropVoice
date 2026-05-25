import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import enTranslation from "./locales/en.json";
import zhTranslation from "./locales/zh.json";
import zhTWTranslation from "./locales/zh-TW.json";
import jaTranslation from "./locales/ja.json";

export const SUPPORTED_LANGUAGES = ["en", "zh", "zh-TW", "ja"] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export function detectLanguage(locale: string): SupportedLanguage {
  const lower = locale.toLowerCase();
  if (lower.startsWith("zh-tw") || lower.startsWith("zh-hk") || lower.startsWith("zh-mo") || lower.startsWith("zh-hant")) {
    return "zh-TW";
  }
  if (lower.startsWith("zh")) {
    return "zh";
  }
  if (lower.startsWith("ja")) {
    return "ja";
  }
  return "en";
}

function getInitialLanguage(): SupportedLanguage {
  const saved = localStorage.getItem("dropvoice-mobile-lang");
  if (saved && SUPPORTED_LANGUAGES.includes(saved as SupportedLanguage)) {
    return saved as SupportedLanguage;
  }
  return detectLanguage(navigator.language || "");
}

export async function initMobileI18n() {
  const resources = {
    en: { translation: enTranslation },
    zh: { translation: zhTranslation },
    "zh-TW": { translation: zhTWTranslation },
    ja: { translation: jaTranslation },
  };

  await i18n.use(initReactI18next).init({
    resources: resources as any,
    lng: getInitialLanguage(),
    fallbackLng: "en",
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });
}

export function setMobileLanguage(lang: SupportedLanguage) {
  localStorage.setItem("dropvoice-mobile-lang", lang);
  i18n.changeLanguage(lang);
}

export default i18n;
