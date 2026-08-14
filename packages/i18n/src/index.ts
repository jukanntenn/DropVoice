export {
  changeLanguage,
  DEFAULT_LANGUAGE,
  initI18n,
  isSupportedLanguage,
  NAMESPACES,
  type Namespace,
  type SupportedLanguage,
  SUPPORTED_LANGUAGES,
  resources,
} from './config';
export { LANGUAGE_OPTIONS, type LanguageOption } from './languages';
export { detectLanguage, resolveLanguage } from './detector';

export { default as enBundle } from './en';
export type { TranslationKey } from './en';
export type { FlattenKeys, TranslationBundle } from './types';
