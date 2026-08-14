import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { DEFAULT_LANGUAGE } from './config';
import { detectLanguage, resolveLanguage } from './detector';

describe('resolveLanguage', () => {
  it('returns the default language for null/undefined/empty input', () => {
    expect(resolveLanguage(null)).toBe(DEFAULT_LANGUAGE);
    expect(resolveLanguage(undefined)).toBe(DEFAULT_LANGUAGE);
    expect(resolveLanguage('')).toBe(DEFAULT_LANGUAGE);
  });

  it('returns the exact match when the candidate is a supported language', () => {
    expect(resolveLanguage('en')).toBe('en');
    expect(resolveLanguage('zh')).toBe('zh');
    expect(resolveLanguage('zh-TW')).toBe('zh-TW');
    expect(resolveLanguage('ja')).toBe('ja');
  });

  it('matches case-insensitively', () => {
    expect(resolveLanguage('EN')).toBe('en');
    expect(resolveLanguage('JA')).toBe('ja');
  });

  it('maps the base of a regional tag to its base language', () => {
    expect(resolveLanguage('en-US')).toBe('en');
    expect(resolveLanguage('ja-JP')).toBe('ja');
  });

  it('maps zh-TW / zh-Hant to the Traditional Chinese variant', () => {
    expect(resolveLanguage('zh-TW')).toBe('zh-TW');
    expect(resolveLanguage('zh-Hant')).toBe('zh-TW');
    expect(resolveLanguage('zh-tw')).toBe('zh-TW');
  });

  it('maps generic zh to Simplified Chinese', () => {
    expect(resolveLanguage('zh-CN')).toBe('zh');
    expect(resolveLanguage('zh-Hans')).toBe('zh');
  });

  it('falls back to the default language when nothing matches', () => {
    expect(resolveLanguage('fr')).toBe(DEFAULT_LANGUAGE);
    expect(resolveLanguage('de-DE')).toBe(DEFAULT_LANGUAGE);
    expect(resolveLanguage('klingon')).toBe(DEFAULT_LANGUAGE);
  });
});

describe('detectLanguage', () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.history.replaceState({}, '', '/');
  });

  afterEach(() => {
    window.localStorage.clear();
    window.history.replaceState({}, '', '/');
  });

  it('reads ?lang= from the query string first', () => {
    window.history.replaceState({}, '', '/?lang=ja');
    expect(detectLanguage()).toBe('ja');
  });

  it('falls back to localStorage when no query string is present', () => {
    window.localStorage.setItem('dropvoice-lang', 'zh-TW');
    expect(detectLanguage()).toBe('zh-TW');
  });

  it('falls back to navigator.language when neither query nor storage is set', () => {
    // jsdom's default navigator.language is 'en-US'.
    expect(['en', 'en-US']).toContain(detectLanguage());
  });
});
