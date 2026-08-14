/**
 * ConnectionStatus — connected/disconnected/connecting pill (spec 07 section 4.3).
 *
 * Renders a status dot, a localized label, optional latency readout, plus a
 * retry button when an `onRetry` handler is supplied. The `connecting` state
 * uses framer-motion for a pulsing dot.
 *
 * §9.6：删除 `mode` 徽标链路（ConnectionModeBadge），仅保留连接状态丸。
 */
import { motion } from 'framer-motion';
import { Loader2, RefreshCw, Wifi, WifiOff } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { cn } from '../lib/cn';

export interface ConnectionStatusProps {
  status: 'connected' | 'disconnected' | 'connecting';
  latency?: number;
  onRetry?: () => void;
}

const statusConfig = {
  connected: {
    labelKey: 'common:status.connected',
    dotClass: 'bg-success-500',
    Icon: Wifi,
  },
  disconnected: {
    labelKey: 'common:status.disconnected',
    dotClass: 'bg-danger-500',
    Icon: WifiOff,
  },
  connecting: {
    labelKey: 'common:status.connecting',
    dotClass: 'bg-warning-500',
    Icon: Loader2,
  },
} as const;

export function ConnectionStatus({ status, latency, onRetry }: ConnectionStatusProps) {
  const { t } = useTranslation();
  const config = statusConfig[status];
  const Icon = config.Icon;
  const showRetry = onRetry && (status === 'disconnected' || status === 'connecting');

  return (
    <div className="inline-flex items-center gap-2 rounded-full border border-white/60 bg-white/70 px-3 py-1 text-sm shadow-sm backdrop-blur-md dark:border-white/10 dark:bg-slate-900/70">
      {status === 'connecting' ? (
        <motion.span
          className={cn('h-2 w-2 rounded-full', config.dotClass)}
          animate={{ opacity: [1, 0.3, 1] }}
          transition={{ duration: 1, repeat: Infinity, ease: 'easeInOut' }}
          aria-hidden="true"
        />
      ) : (
        <span className={cn('h-2 w-2 rounded-full', config.dotClass)} aria-hidden="true" />
      )}
      <Icon
        className={cn('h-3.5 w-3.5 text-neutral-500', status === 'connecting' && 'animate-spin')}
        aria-hidden="true"
      />
      <span className="font-medium text-neutral-700 dark:text-neutral-300">
        {t(config.labelKey)}
      </span>
      {status === 'connected' && typeof latency === 'number' && (
        <span className="text-xs text-neutral-500 dark:text-neutral-400">{latency}ms</span>
      )}
      {showRetry && (
        <button
          type="button"
          onClick={onRetry}
          className="text-primary-600 hover:bg-primary-50 dark:text-primary-400 dark:hover:bg-primary-950 ml-1 inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs transition-colors"
        >
          <RefreshCw className="h-3 w-3" aria-hidden="true" />
          {t('common:actions.retry')}
        </button>
      )}
    </div>
  );
}
