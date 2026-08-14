/**
 * DeviceSelector — chip-style device picker (spec 07 section 4.4).
 *
 * Uses deterministic per-device colors from `@dropvoice/core/utils` so each
 * device keeps a stable hue across renders.
 */
import type { Device } from '@dropvoice/core/types';
import { getDeviceColor, getDeviceColorSoft } from '@dropvoice/core/utils';
import { Check } from 'lucide-react';

import { cn } from '../lib/cn';

export interface DeviceSelectorProps {
  devices: Device[];
  activeDeviceId: string | null;
  onSelect: (deviceId: string) => void;
}

function prefersDark(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
}

export function DeviceSelector({ devices, activeDeviceId, onSelect }: DeviceSelectorProps) {
  const dark = prefersDark();

  if (devices.length === 0) {
    return null;
  }

  return (
    <div role="radiogroup" aria-label="Select device" className="flex flex-wrap gap-2">
      {devices.map((device) => {
        const isActive = device.id === activeDeviceId;
        const solid = getDeviceColor(device, dark);
        const soft = getDeviceColorSoft(device, dark);
        return (
          <button
            key={device.id}
            type="button"
            role="radio"
            aria-checked={isActive}
            onClick={() => onSelect(device.id)}
            style={isActive ? { backgroundColor: soft, borderColor: solid } : undefined}
            className={cn(
              'inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm backdrop-blur-md transition-all active:scale-95',
              isActive
                ? 'border-white/60 bg-white/70 font-medium text-neutral-900 shadow-sm dark:border-white/10 dark:bg-slate-900/70 dark:text-slate-100'
                : 'border-white/40 bg-white/40 text-neutral-600 hover:bg-white/60 dark:border-white/10 dark:bg-slate-800/40 dark:text-slate-300 dark:hover:bg-slate-800/60'
            )}
          >
            <span className="h-2 w-2 rounded-full" style={{ backgroundColor: solid }} />
            <span className="max-w-[12rem] truncate">{device.name}</span>
            {isActive && <Check className="h-3 w-3" aria-hidden="true" />}
          </button>
        );
      })}
    </div>
  );
}
