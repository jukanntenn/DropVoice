import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { initI18n } from '@dropvoice/i18n';
import { isSecureContext, isServiceWorkerSupported } from '@dropvoice/core';

import App from './App';
import './index.css';

async function bootstrap() {
  await initI18n();
  // 仅在安全上下文（HTTPS / localhost）注册 Service Worker。
  if (isServiceWorkerSupported() && isSecureContext()) {
    try {
      await navigator.serviceWorker.register('/sw.js');
    } catch (err) {
      console.warn('SW registration failed:', err);
    }
  }
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>
  );
}

bootstrap();
