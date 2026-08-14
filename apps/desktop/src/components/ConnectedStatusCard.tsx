import { useTranslation } from 'react-i18next';
import { CheckCircle2, Plus, Zap } from 'lucide-react';

import type { ClientInfo } from '@dropvoice/core';
import { Button } from '@dropvoice/ui';

interface ConnectedStatusCardProps {
  activeConnections: number;
  /** Connected client details (unused in the calm view — kept for future surfacing). */
  clients?: ClientInfo[];
  /** Current injection queue depth; shown only when > 0. */
  queueDepth?: number;
  onAddDevice: () => void;
}

export function ConnectedStatusCard({
  activeConnections,
  queueDepth,
  onAddDevice,
}: ConnectedStatusCardProps) {
  const { t } = useTranslation();
  const busy = (queueDepth ?? 0) > 0;

  return (
    <div className="animate-slide-up shadow-glass dark:shadow-glass-dark mt-4 w-full max-w-sm rounded-3xl border border-white/60 bg-white/70 p-6 backdrop-blur-xl dark:border-white/10 dark:bg-slate-900/70">
      <div className="flex flex-col items-center gap-4 text-center">
        <div className="flex items-center gap-2 text-neutral-900 dark:text-slate-100">
          <CheckCircle2 className="text-success-500 h-5 w-5" aria-hidden="true" />
          <span className="text-base font-semibold">{t('common:connection.ready')}</span>
        </div>

        <div className="flex items-center gap-2 text-sm text-neutral-600 dark:text-slate-300">
          <span className="relative flex h-2.5 w-2.5">
            <span className="bg-success-500 absolute inline-flex h-full w-full animate-ping rounded-full opacity-60" />
            <span className="bg-success-500 relative inline-flex h-2.5 w-2.5 rounded-full" />
          </span>
          {t('common:connection.activeConnections', { count: activeConnections })}
        </div>

        {busy && (
          <div className="border-primary/30 bg-primary/10 text-primary-700 dark:text-primary-300 inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-medium">
            <Zap className="h-3.5 w-3.5" aria-hidden="true" />
            {t('common:queue.typing', { count: queueDepth })}
          </div>
        )}

        <Button variant="ghost" size="sm" onClick={onAddDevice} className="mt-2">
          <Plus className="h-4 w-4" />
          {t('common:connection.addDevice')}
        </Button>
      </div>
    </div>
  );
}
