import type { SupportedLanguage } from '@dropvoice/i18n';

/**
 * Hero 活体演示的轮换台词——每个 locale 一套真实听写场景，
 * 末句向开发者眨眼。演示即 i18n 展示（设计定稿）。
 */
export const DEMO_PHRASES: Record<SupportedLanguage, readonly string[]> = {
  zh: ['你好，世界', '这段话是说出来的，不是打出来的', 'fix: 修复了登录后白屏的问题'],
  'zh-TW': ['你好，世界', '這段話是說出來的，不是打出來的', 'fix: 修復了登入後白畫面的問題'],
  en: [
    'Hello, world',
    'This sentence was spoken, not typed',
    'fix: resolve blank screen after login',
  ],
  ja: [
    'こんにちは、世界',
    'この文章は話したもので、打ったものではない',
    'fix: ログイン後の白画面を修正',
  ],
};

export function phrasesFor(language: string | undefined): readonly string[] {
  if (language === 'zh' || language === 'zh-TW' || language === 'ja') {
    return DEMO_PHRASES[language];
  }
  // en 及任何未知语言（含 zh-* 区域变体）回退英文。
  return DEMO_PHRASES.en;
}
