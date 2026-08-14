import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AlertTriangle, X } from 'lucide-react';

const DISMISS_KEY = 'dropvoice:lan-warning-dismissed';

export function LanWarningBanner() {
  const { t } = useTranslation();
  const [dismissed, setDismissed] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.localStorage.getItem(DISMISS_KEY) === 'true';
  });

  if (dismissed) return null;

  const handleDismiss = () => {
    window.localStorage.setItem(DISMISS_KEY, 'true');
    setDismissed(true);
  };

  return (
    <div className="animate-slide-up flex w-full max-w-sm items-center gap-3 rounded-2xl border border-amber-200/60 bg-amber-50/80 px-5 py-4 backdrop-blur-sm dark:border-amber-500/20 dark:bg-amber-950/40">
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-amber-100 dark:bg-amber-900/50">
        <AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-400" aria-hidden="true" />
      </div>
      <p className="flex-1 text-sm leading-relaxed text-amber-800 dark:text-amber-200">
        {t('common:lan.warningText')}
      </p>
      <button
        type="button"
        onClick={handleDismiss}
        aria-label={t('common:lan.warningDismiss')}
        className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-amber-600 transition-colors hover:bg-amber-100 hover:text-amber-800 dark:text-amber-400 dark:hover:bg-amber-900/50 dark:hover:text-amber-200"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}
