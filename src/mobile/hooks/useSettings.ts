import { useCallback, useEffect, useState } from "react";
import { setMobileLanguage } from "../i18n";

export type Theme = "light" | "dark" | "system";
export type Language = "en" | "zh";

const THEME_KEY = "dropvoice-mobile-theme";
const LANGUAGE_KEY = "dropvoice-mobile-lang";

function getSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function detectLanguage(): Language {
  const browserLang = navigator.language || (navigator as any).userLanguage;
  if (typeof browserLang === "string" && browserLang.toLowerCase().startsWith("zh")) {
    return "zh";
  }
  return "en";
}

function loadTheme(): Theme {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system") return saved;
  } catch {}
  return "system";
}

function loadLanguage(): Language {
  try {
    const saved = localStorage.getItem(LANGUAGE_KEY);
    if (saved === "en" || saved === "zh") return saved;
  } catch {}
  return detectLanguage();
}

function saveTheme(theme: Theme): void {
  try {
    localStorage.setItem(THEME_KEY, theme);
  } catch {}
}

function saveLanguage(lang: Language): void {
  try {
    localStorage.setItem(LANGUAGE_KEY, lang);
  } catch {}
}

export function useSettings() {
  const [theme, setThemeState] = useState<Theme>(() => loadTheme());
  const [language, setLanguageState] = useState<Language>(() => loadLanguage());
  const [resolvedTheme, setResolvedTheme] = useState<"light" | "dark">(() => {
    const initialTheme = loadTheme();
    return initialTheme === "system" ? getSystemTheme() : initialTheme;
  });

  useEffect(() => {
    const sync = () => {
      setThemeState(loadTheme());
      setLanguageState(loadLanguage());
    };

    window.addEventListener("storage", sync);
    window.addEventListener("dropvoice-mobile-settings", sync as EventListener);
    return () => {
      window.removeEventListener("storage", sync);
      window.removeEventListener("dropvoice-mobile-settings", sync as EventListener);
    };
  }, []);

  useEffect(() => {
    setResolvedTheme(theme === "system" ? getSystemTheme() : theme);
  }, [theme]);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", resolvedTheme === "dark");
  }, [resolvedTheme]);

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  useEffect(() => {
    if (theme !== "system") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => {
      setResolvedTheme(mediaQuery.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [theme]);

  const setTheme = useCallback((next: Theme) => {
    setThemeState(next);
    saveTheme(next);
    window.dispatchEvent(new Event("dropvoice-mobile-settings"));
  }, []);

  const setLanguage = useCallback((next: Language) => {
    setLanguageState(next);
    saveLanguage(next);
    setMobileLanguage(next);
    window.dispatchEvent(new Event("dropvoice-mobile-settings"));
  }, []);

  return {
    language,
    setLanguage,
    theme,
    setTheme,
    resolvedTheme,
  };
}
