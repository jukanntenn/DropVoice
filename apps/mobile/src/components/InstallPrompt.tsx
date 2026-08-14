import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Bookmark, Download, X } from 'lucide-react';

import { getPWAInstallStrategy } from '@dropvoice/core';
import { Button } from '@dropvoice/ui';

interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
}

/**
 * PWA 安装提示：
 * - 'pwa'：监听 beforeinstallprompt，显示"安装应用"按钮。
 * - 'bookmark'：iOS Safari 等不支持 PWA 安装的环境，显示"添加书签"提示。
 * - 'none'：不显示。
 */
export function InstallPrompt() {
  const { t } = useTranslation();
  const strategy = getPWAInstallStrategy();
  const [deferredPrompt, setDeferredPrompt] = useState<BeforeInstallPromptEvent | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    if (strategy !== 'pwa') return;
    const handler = (event: Event) => {
      event.preventDefault();
      setDeferredPrompt(event as BeforeInstallPromptEvent);
    };
    window.addEventListener('beforeinstallprompt', handler);
    return () => window.removeEventListener('beforeinstallprompt', handler);
  }, [strategy]);

  if (strategy === 'none' || dismissed) return null;

  const handleInstall = async () => {
    if (!deferredPrompt) {
      setDismissed(true);
      return;
    }
    try {
      await deferredPrompt.prompt();
      const choice = await deferredPrompt.userChoice;
      if (choice.outcome === 'accepted' || choice.outcome === 'dismissed') {
        setDeferredPrompt(null);
        setDismissed(true);
      }
    } catch {
      setDismissed(true);
    }
  };

  if (strategy === 'pwa' && !deferredPrompt) return null;

  return (
    <div className="border-primary/20 bg-primary/10 flex items-center justify-between gap-2 rounded-2xl border px-3 py-2 text-sm backdrop-blur-md">
      <div className="flex items-center gap-2">
        {strategy === 'pwa' ? (
          <Download className="text-primary-600 dark:text-primary-400 h-4 w-4" />
        ) : (
          <Bookmark className="text-primary-600 dark:text-primary-400 h-4 w-4" />
        )}
        <span className="text-primary-900 dark:text-primary-200">
          {strategy === 'pwa' ? t('common:app.name') : t('common:app.description')}
        </span>
      </div>
      <div className="flex items-center gap-1">
        {strategy === 'pwa' ? (
          <Button size="sm" variant="primary" onClick={handleInstall}>
            {t('common:actions.confirm')}
          </Button>
        ) : (
          <span className="text-primary-700 dark:text-primary-300 text-xs">
            {t('common:app.name')} · {strategy}
          </span>
        )}
        <Button
          size="icon"
          variant="ghost"
          aria-label={t('common:actions.close')}
          onClick={() => setDismissed(true)}
          className="h-7 w-7"
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
