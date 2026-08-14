import { useTranslation } from 'react-i18next';

import type { Device, SendMode } from '@dropvoice/core';
import { cn } from '@dropvoice/ui';

interface SendModeSelectorProps {
  mode: SendMode;
  devices: Device[];
  selectedDevices: string[];
  onModeChange: (mode: SendMode) => void;
  onSelectionChange: (devices: string[]) => void;
}

const MODES: SendMode[] = ['active', 'all', 'selected'];

/**
 * 发送模式选择器（规范 10 §4.4）。
 *
 * 仅在设备数 > 1 时显示。selected 模式下展开多选列表。
 * 受控组件：父级持有 mode 与 selectedDevices 状态，通过回调驱动变更。
 */
export function SendModeSelector({
  mode,
  devices,
  selectedDevices,
  onModeChange,
  onSelectionChange,
}: SendModeSelectorProps) {
  const { t } = useTranslation();

  if (devices.length <= 1) return null;

  const toggleSelected = (deviceId: string) => {
    onSelectionChange(
      selectedDevices.includes(deviceId)
        ? selectedDevices.filter((id) => id !== deviceId)
        : [...selectedDevices, deviceId]
    );
  };

  return (
    <div className="flex flex-col gap-2 py-1">
      <div
        className="flex gap-1 rounded-full border border-white/60 bg-white/50 p-1 backdrop-blur-md dark:border-white/10 dark:bg-slate-800/50"
        role="radiogroup"
      >
        {MODES.map((m) => (
          <button
            key={m}
            type="button"
            role="radio"
            aria-checked={mode === m}
            className={cn(
              'flex-1 rounded-full px-3 py-1.5 text-sm font-medium transition-all',
              mode === m
                ? 'from-primary shadow-primary/30 bg-gradient-to-br to-teal-500 text-white shadow-sm'
                : 'text-neutral-600 hover:text-neutral-900 dark:text-slate-300 dark:hover:text-white'
            )}
            onClick={() => onModeChange(m)}
          >
            {t(`devices:sendMode.${m}`)}
          </button>
        ))}
      </div>
      {mode === 'selected' && (
        <div className="flex flex-col gap-1 rounded-2xl border border-white/60 bg-white/40 p-2 backdrop-blur-md dark:border-white/10 dark:bg-slate-800/40">
          {devices.map((device) => (
            <label
              key={device.id}
              className="flex items-center gap-2 rounded-lg px-2 py-1 text-sm hover:bg-white/50 dark:hover:bg-slate-800/50"
            >
              <input
                type="checkbox"
                checked={selectedDevices.includes(device.id)}
                onChange={() => toggleSelected(device.id)}
                className="accent-primary-600 h-4 w-4"
              />
              <span className="truncate">{device.name}</span>
            </label>
          ))}
        </div>
      )}
    </div>
  );
}
