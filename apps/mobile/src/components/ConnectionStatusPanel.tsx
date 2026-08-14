import { useTranslation } from 'react-i18next';

import type { ConnectionState, Device } from '@dropvoice/core';
import { ConnectionStatus } from '@dropvoice/ui';

interface ConnectionStatusPanelProps {
  activeDevice: Device | null;
  /** §9.6：完整连接状态（含 offline 原因），用于区分 idle 与 offline。 */
  status: ConnectionState;
  onRetry: () => void;
}

/**
 * 连接状态面板：根据活跃设备的连接状态渲染 pill（§9.6 重写）。
 *
 * 把细粒度 ConnectionState 映射为 ConnectionStatus composite 接受的粗态
 * （connected / connecting / disconnected）。offline 也映射为 'disconnected'
 * 从而自动显示重试按钮；并在 offline 时展示 `devices:desktopOffline` 文案。
 */
export function ConnectionStatusPanel({
  activeDevice,
  status,
  onRetry,
}: ConnectionStatusPanelProps) {
  const { t } = useTranslation();
  const displayStatus: 'connected' | 'disconnected' | 'connecting' =
    status.status === 'connected'
      ? 'connected'
      : status.status === 'connecting' || status.status === 'reconnecting'
        ? 'connecting'
        : 'disconnected';

  return (
    <div className="flex flex-col items-center gap-1 py-2">
      <ConnectionStatus status={displayStatus} onRetry={activeDevice ? onRetry : undefined} />
      {activeDevice && status.status === 'offline' && (
        <span className="text-xs text-neutral-500 dark:text-neutral-400">
          {t('devices:desktopOffline')}
        </span>
      )}
      {!activeDevice && (
        <span className="text-sm text-neutral-500 dark:text-neutral-400">
          {t('devices:emptyState')}
        </span>
      )}
    </div>
  );
}
