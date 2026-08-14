import { useTranslation } from 'react-i18next';
import { Eraser, Send, Undo2 } from 'lucide-react';

import { cn } from '@dropvoice/ui';

interface ActionButtonsProps {
  onRestore: () => void;
  onSend: () => void;
  onClear: () => void;
  canSend: boolean;
  canRestore: boolean;
}

const secondaryClass =
  'flex h-12 w-12 items-center justify-center rounded-full border border-white/60 bg-white/50 text-neutral-600 shadow-sm backdrop-blur-md transition-all duration-200 hover:border-white/80 hover:bg-white/70 hover:text-neutral-900 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40 dark:border-white/10 dark:bg-slate-800/50 dark:text-slate-300 dark:hover:bg-slate-800/70 dark:hover:text-white';

/**
 * 操作按钮：三圆布局（原 DropVoice 移动端标志性形态）。
 * - Restore(48 玻璃) / Send(64 青绿渐变 CTA) / Clear(48 玻璃)
 */
export function ActionButtons({
  onRestore,
  onSend,
  onClear,
  canSend,
  canRestore,
}: ActionButtonsProps) {
  const { t } = useTranslation();

  return (
    <div className="mt-2 flex items-center justify-between px-1">
      <button
        type="button"
        onClick={onRestore}
        disabled={!canRestore}
        aria-label={t('common:actions.restore')}
        className={secondaryClass}
      >
        <Undo2 className="h-5 w-5" />
      </button>

      <button
        type="button"
        onClick={onSend}
        disabled={!canSend}
        aria-label={t('common:actions.send')}
        className={cn(
          'from-primary shadow-primary/30 flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-br to-teal-500 text-white shadow-lg transition-all duration-200',
          'hover:shadow-primary/40 hover:shadow-xl active:scale-95',
          'disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none disabled:saturate-0'
        )}
      >
        <Send className="h-7 w-7" />
      </button>

      <button
        type="button"
        onClick={onClear}
        aria-label={t('common:actions.clear')}
        className={secondaryClass}
      >
        <Eraser className="h-5 w-5" />
      </button>
    </div>
  );
}
