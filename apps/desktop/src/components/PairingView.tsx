import { useTranslation } from 'react-i18next';
import { Loader2 } from 'lucide-react';

import { Button } from '@dropvoice/ui';

import type { ConnectionInfo } from '../lib/invoke';
import { PairingContent } from './PairingContent';

interface PairingViewProps {
  connectionInfo: ConnectionInfo | undefined;
  onStart: () => void;
  isStarting: boolean;
}

/**
 * 首配全屏视图（§1 重构后）：
 * - `!running` → 启动态（Loader + Retry）。
 * - 否则 → 复用 {@link PairingContent}（QR/链接/重连码）。
 *
 * "添加设备"（已连接时再添一台）改由 App.tsx 的 `<Dialog>` 浮层承载，浮层内同样
 * 复用 PairingContent，故本组件不再包含 latch 状态。
 */
export function PairingView({ connectionInfo, onStart, isStarting }: PairingViewProps) {
  const { t } = useTranslation();

  // 服务尚未运行：显示启动态（自动启动中或失败重试）。
  if (!connectionInfo?.running) {
    return (
      <div className="flex w-full max-w-sm flex-col items-center gap-4 pt-8">
        <Loader2 className="text-primary h-8 w-8 animate-spin" aria-hidden="true" />
        <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('common:app.loading')}</p>
        <Button variant="outline" onClick={onStart} isLoading={isStarting}>
          {t('common:actions.retry')}
        </Button>
      </div>
    );
  }

  return <PairingContent connectionInfo={connectionInfo} />;
}
