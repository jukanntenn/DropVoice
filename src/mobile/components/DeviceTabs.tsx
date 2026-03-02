import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { Device } from "../types";
import { DeviceCard } from "./DeviceCard";

interface DeviceTabsProps {
  devices: Device[];
  activeDeviceId: string | null;
  onSelect: (deviceId: string) => void;
  onRemove: (deviceId: string) => void;
  onRename: (deviceId: string) => void;
  onAdd: () => void;
}

export function DeviceTabs({
  devices,
  activeDeviceId,
  onSelect,
  onRemove,
  onRename,
  onAdd,
}: DeviceTabsProps) {
  const { t } = useTranslation();

  const emptyState = useMemo(() => devices.length === 0, [devices.length]);

  return (
    <div className="sticky top-0 z-10 -mx-5 bg-gradient-to-br from-teal-50/90 via-white/90 to-cyan-50/90 px-5 pb-3 pt-2 backdrop-blur-sm dark:from-slate-950/90 dark:via-slate-900/90 dark:to-slate-950/90">
      <div className="flex items-center gap-2 overflow-x-auto scrollbar-hide">
        {devices.map((device) => (
          <DeviceCard
            key={device.id}
            name={device.name}
            status={device.status}
            isActive={device.id === activeDeviceId}
            onSelect={() => onSelect(device.id)}
            onRemove={() => onRemove(device.id)}
            onRename={() => onRename(device.id)}
          />
        ))}
        <button
          type="button"
          onClick={onAdd}
          className={[
            "flex items-center gap-1 rounded-2xl border border-dashed px-3 py-2 text-sm font-medium",
            "border-slate-300/70 bg-white/40 text-slate-600 hover:border-primary/60 hover:bg-primary/10",
            "dark:border-slate-700/70 dark:bg-slate-900/40 dark:text-slate-300",
          ].join(" ")}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            className="h-5 w-5"
          >
            <path
              fillRule="evenodd"
              d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
              clipRule="evenodd"
            />
          </svg>
          <span>{t("devices.add")}</span>
        </button>
      </div>
      {emptyState ? (
        <div className="py-3 text-center text-sm text-muted-foreground dark:text-slate-400">
          {t("devices.emptyState")}
        </div>
      ) : null}
    </div>
  );
}
