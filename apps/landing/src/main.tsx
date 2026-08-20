import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { getDefaultStore } from 'jotai';

import { initI18n } from '@dropvoice/i18n';
import { themeAtom, type Theme } from '@dropvoice/core';

import App from './App';
import './index.css';

const THEME_KEY = 'dropvoice-landing-theme';

function readTheme(): Theme {
  try {
    const stored = localStorage.getItem(THEME_KEY);
    if (stored === 'light' || stored === 'dark') {
      return stored;
    }
  } catch {
    // localStorage 不可用时按默认亮色处理。
  }
  // 设计定稿：landing 首访默认亮色（清晨主调），不跟随系统。
  return 'light';
}

function bootstrap() {
  getDefaultStore().set(themeAtom, readTheme());
  getDefaultStore().sub(themeAtom, () => {
    const next = getDefaultStore().get(themeAtom);
    if (next === 'light' || next === 'dark') {
      try {
        localStorage.setItem(THEME_KEY, next);
      } catch {
        // 忽略持久化失败。
      }
    }
  });

  void initI18n().then(() => {
    createRoot(document.getElementById('root')!).render(
      <StrictMode>
        <App />
      </StrictMode>
    );
  });
}

bootstrap();
