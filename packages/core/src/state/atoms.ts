import { atom } from 'jotai';

import type { SupportedLanguage } from '@dropvoice/i18n';

export type Theme = 'light' | 'dark' | 'system';

/** spec 02 §2.2: default language is 'zh'. */
export const languageAtom = atom<SupportedLanguage>('zh');

export const themeAtom = atom<Theme>('system');

export const inputDelayAtom = atom<number>(10);

export const sendModeAtom = atom<'active' | 'all' | 'selected'>('active');

export const selectedDeviceIdsAtom = atom<string[]>([]);
