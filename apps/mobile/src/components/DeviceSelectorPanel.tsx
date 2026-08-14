import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { MoreVertical, Plus, Trash2 } from 'lucide-react';

import type { Device } from '@dropvoice/core';
import { Button, DeviceSelector } from '@dropvoice/ui';

interface DeviceSelectorPanelProps {
  devices: Device[];
  activeDeviceId: string | null;
  onSelect: (deviceId: string) => void;
  onAddDevice: () => void;
  onRemoveDevice: (deviceId: string) => void;
}

/**
 * 设备选择器面板：渲染 chip 列表 + Add 按钮 + Manage 菜单。
 *
 * Manage 菜单展开后列出所有设备并允许删除（替代长按手势，跨平台可用）。
 * 即使 devices 为空也始终渲染 Add 按钮，确保首次使用时用户能添加设备。
 */
export function DeviceSelectorPanel({
  devices,
  activeDeviceId,
  onSelect,
  onAddDevice,
  onRemoveDevice,
}: DeviceSelectorPanelProps) {
  const { t } = useTranslation();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="flex flex-wrap items-center gap-2 py-2">
      {devices.length > 0 && (
        <DeviceSelector devices={devices} activeDeviceId={activeDeviceId} onSelect={onSelect} />
      )}
      <Button variant="outline" size="sm" onClick={onAddDevice} aria-label={t('devices:addDevice')}>
        <Plus className="h-4 w-4" />
        {t('common:actions.add')}
      </Button>
      {devices.length > 0 && (
        <div className="relative">
          <Button
            variant="ghost"
            size="icon"
            aria-label={t('devices:title')}
            onClick={() => setMenuOpen((open) => !open)}
          >
            <MoreVertical className="h-4 w-4" />
          </Button>
          {menuOpen && (
            <div
              className="shadow-glass dark:shadow-glass-dark absolute right-0 top-full z-20 mt-1 min-w-48 rounded-xl border border-white/60 bg-white/90 py-1 backdrop-blur-xl dark:border-white/10 dark:bg-slate-900/90"
              role="menu"
            >
              {devices.map((device) => (
                <div
                  key={device.id}
                  className="flex items-center justify-between px-3 py-1.5 text-sm"
                >
                  <button
                    type="button"
                    role="menuitem"
                    className="flex-1 rounded-lg px-2 py-1 text-left text-neutral-700 hover:bg-white/60 dark:text-slate-200 dark:hover:bg-slate-800/60"
                    onClick={() => {
                      onSelect(device.id);
                      setMenuOpen(false);
                    }}
                  >
                    <span className="truncate">{device.name}</span>
                  </button>
                  <button
                    type="button"
                    role="menuitem"
                    aria-label={t('devices:removeDevice')}
                    className="text-danger-600 hover:bg-danger-500/10 dark:text-danger-400 ml-2 rounded-lg p-1"
                    onClick={() => {
                      onRemoveDevice(device.id);
                      setMenuOpen(false);
                    }}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
