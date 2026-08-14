import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';

import { TextArea, cn } from '@dropvoice/ui';

interface TextInputPanelProps {
  value: string;
  onChange: (value: string) => void;
  onSend: () => void;
  /** 最大字符数，默认 10000。 */
  maxLength?: number;
  /** 是否允许发送（连接已就绪且文本非空）。 */
  canSend: boolean;
}

/**
 * 文本输入面板：玻璃 textarea + 字数计数。
 * - Enter 发送（默认行为），Shift+Enter 换行。
 * - 字数计数显示 `current/max`，近上限变琥珀、超限变红。
 */
export function TextInputPanel({
  value,
  onChange,
  onSend,
  maxLength = 10000,
  canSend,
}: TextInputPanelProps) {
  const { t } = useTranslation();

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) {
        event.preventDefault();
        if (canSend) onSend();
      }
    },
    [canSend, onSend]
  );

  const count = value.length;
  const nearLimit = count > maxLength * 0.8;
  const overLimit = count >= maxLength;

  return (
    <div className="relative">
      <TextArea
        className="min-h-[200px] pb-8"
        placeholder={t('common:app.description')}
        value={value}
        maxLength={maxLength}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        aria-label={t('common:app.description')}
      />
      <span
        className={cn(
          'pointer-events-none absolute bottom-3 right-4 text-[11px] font-medium',
          overLimit
            ? 'text-danger-500'
            : nearLimit
              ? 'text-warning-500'
              : 'text-neutral-400 dark:text-slate-400'
        )}
      >
        {count}/{maxLength}
      </span>
    </div>
  );
}
