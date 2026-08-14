import { DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES, type SupportedLanguage } from './config';

/**
 * Resolve the best supported language from a raw candidate string.
 *
 * Falls back to {@link DEFAULT_LANGUAGE} when no match is found.
 */
export function resolveLanguage(candidate: string | null | undefined): SupportedLanguage {
  if (!candidate) {
    return DEFAULT_LANGUAGE;
  }

  const lower = candidate.toLowerCase();
  // Case-insensitive exact match against the supported languages. The
  // SUPPORTED_LANGUAGES array mixes lowercase codes ('en', 'zh') with a
  // mixed-case code ('zh-TW'), so compare against the lowercased set.
  const exact = (SUPPORTED_LANGUAGES as readonly string[]).find(
    (code) => code.toLowerCase() === lower
  );
  if (exact) {
    return exact as SupportedLanguage;
  }

  // Match base language (e.g. "en-US" → "en").
  const base = lower.split('-')[0]!;
  const baseMatch = (SUPPORTED_LANGUAGES as readonly string[]).find(
    (code) => code.toLowerCase() === base
  );
  if (baseMatch) {
    // For Chinese, fall through to the variant detection below so zh-TW-Hant
    // / zh-tw / zh-Hant all resolve to the Traditional variant.
    if (base !== 'zh') {
      return baseMatch as SupportedLanguage;
    }
  }

  // Map common Chinese variants: zh-TW / zh-Hant / zh-tw → Traditional.
  if (base === 'zh') {
    if (lower.includes('tw') || lower.includes('hant')) {
      return 'zh-TW';
    }
    return 'zh';
  }

  return DEFAULT_LANGUAGE;
}

/**
 * Detect the user's preferred language from the browser environment.
 *
 * Order: querystring → localStorage → navigator.
 */
export function detectLanguage(): SupportedLanguage {
  if (typeof window === 'undefined') {
    return DEFAULT_LANGUAGE;
  }

  const params = new URLSearchParams(window.location.search);
  const query = params.get('lang');
  if (query) {
    return resolveLanguage(query);
  }

  const stored = window.localStorage.getItem('dropvoice-lang');
  if (stored) {
    return resolveLanguage(stored);
  }

  if (typeof navigator !== 'undefined' && navigator.language) {
    return resolveLanguage(navigator.language);
  }

  return DEFAULT_LANGUAGE;
}
