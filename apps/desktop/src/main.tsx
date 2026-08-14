import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { initI18n } from '@dropvoice/i18n';

import App from './App';
import './index.css';

async function bootstrap() {
  await initI18n();
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>
  );
}

bootstrap();
