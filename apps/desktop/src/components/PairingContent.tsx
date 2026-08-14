import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Check, Copy, Info } from 'lucide-react';

import { writeText } from '@tauri-apps/plugin-clipboard-manager';

import { Button, QRCode, Tooltip, useErrorHandler } from '@dropvoice/ui';

import type { ConnectionInfo } from '../lib/invoke';

/**
 * 配对内容（§1 从 PairingView 抽取）：QR 区 + 链接区 + 重连码区。
 *
 * 主视图首配（PairingView 全屏）与"添加设备"浮层（Dialog）共用本组件。
 * 自管 copied 复制态（§3 剪贴板权限修好后一键复制为唯一全量获取通路）与
 * 重连码 ℹ tooltip（§4 显式可发现性提示）。
 */
interface PairingContentProps {
  connectionInfo: ConnectionInfo | undefined;
}

export function PairingContent({ connectionInfo }: PairingContentProps) {
  const { t } = useTranslation();
  const { handleError } = useErrorHandler();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!connectionInfo?.qr_payload) return;
    try {
      await writeText(connectionInfo.qr_payload);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      handleError(error);
    }
  };

  const payload = connectionInfo?.qr_payload;

  return (
    <div className="mt-4 flex w-full max-w-sm flex-col items-center gap-8">
      {/* QR 区：标签 + 白卡 */}
      <div className="flex w-full flex-col items-center gap-4">
        <p className="text-sm font-medium text-neutral-600 dark:text-slate-300">
          {t('common:connection.scanQR')}
        </p>
        {payload && <QRCode value={payload} size={155} />}
      </div>

      {/* 链接区：玻璃药丸 + 复制（§3：一行截断保留，复制按钮为唯一全量获取通路） */}
      {payload && (
        <div className="flex w-full max-w-xs flex-col items-center gap-2">
          <p className="text-sm font-medium text-neutral-600 dark:text-slate-300">
            {t('common:connection.copyLink')}
          </p>
          <div className="shadow-glass hover:shadow-glow dark:shadow-glass-dark inline-flex w-full items-center gap-2 rounded-2xl border border-white/60 bg-white/80 py-0 pl-4 pr-2 backdrop-blur-md transition-all hover:border-white/80 dark:border-white/10 dark:bg-slate-900/70 dark:hover:border-white/20">
            <code className="flex-1 truncate text-sm font-medium text-neutral-900 dark:text-white">
              {payload}
            </code>
            <Tooltip content={t('common:actions.copy')}>
              <Button
                variant="ghost"
                size="icon"
                onClick={handleCopy}
                aria-label={t('common:actions.copy')}
                className="hover:bg-primary/10 hover:text-primary h-8 w-8 shrink-0 rounded-lg"
              >
                {copied ? (
                  <Check className="text-success-500 h-4 w-4" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
              </Button>
            </Tooltip>
          </div>
        </div>
      )}

      {/* 重连码（§4：显式 ℹ 图标作 tooltip 触发器，键盘可达） */}
      {connectionInfo?.pairing_code && (
        <div className="flex items-center gap-1.5 text-xs text-neutral-500 dark:text-neutral-400">
          <span>{t('common:connection.reconnectCode')}</span>
          <Tooltip content={t('common:connection.reconnectCodeHint')}>
            <button
              type="button"
              className="hover:text-primary cursor-help text-neutral-400"
              aria-label={t('common:connection.reconnectCodeHint')}
            >
              <Info className="h-3.5 w-3.5" />
            </button>
          </Tooltip>
          <span className="font-mono font-medium tracking-wider text-neutral-700 dark:text-neutral-300">
            {connectionInfo.pairing_code}
          </span>
        </div>
      )}
    </div>
  );
}
