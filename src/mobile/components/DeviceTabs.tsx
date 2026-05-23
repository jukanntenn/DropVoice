import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { Device } from "../types";
import { DeviceCard } from "./DeviceCard";

const RENAME_HINT_KEY = "dropvoice_rename_hint_dismissed";

function sortByRecency(devices: Device[]): Device[] {
  return [...devices].sort((a, b) => b.lastConnected - a.lastConnected);
}

interface DeviceTabsProps {
  devices: Device[];
  activeDeviceId: string | null;
  onSelect: (deviceId: string) => void;
  onRemove: (deviceId: string) => void;
  onRename: (deviceId: string) => void;
  onRetry: (deviceId: string) => void;
  onAdd: () => void;
}

export function DeviceTabs({
  devices,
  activeDeviceId,
  onSelect,
  onRemove,
  onRename,
  onRetry,
  onAdd,
}: DeviceTabsProps) {
  const { t } = useTranslation();

  const sortedDevices = useMemo(() => sortByRecency(devices), [devices]);
  const emptyState = useMemo(() => devices.length === 0, [devices.length]);

  const [showRenameHint, setShowRenameHint] = useState(false);

  useEffect(() => {
    if (sortedDevices.length === 1 && /^PC-\d/.test(sortedDevices[0].name)) {
      try {
        const dismissed = localStorage.getItem(RENAME_HINT_KEY);
        if (!dismissed) setShowRenameHint(true);
      } catch {}
    } else {
      setShowRenameHint(false);
    }
  }, [sortedDevices]);

  const dismissRenameHint = () => {
    try {
      localStorage.setItem(RENAME_HINT_KEY, "1");
    } catch {}
    setShowRenameHint(false);
  };

  // Auto-dismiss after a rename happens (when device name no longer starts with PC-)
  useEffect(() => {
    if (showRenameHint && sortedDevices.length === 1 && !/^PC-\d/.test(sortedDevices[0].name)) {
      dismissRenameHint();
    }
  }, [showRenameHint, sortedDevices]);

  return (
    <div className="sticky top-0 z-10 -mx-5 bg-gradient-to-br from-teal-50/90 via-white/90 to-cyan-50/90 px-5 pb-3 pt-2 backdrop-blur-sm dark:from-slate-950/90 dark:via-slate-900/90 dark:to-slate-950/90">
      <div className="flex items-center gap-2 overflow-x-auto scrollbar-hide">
        {sortedDevices.map((device) => (
          <DeviceCard
            key={device.id}
            name={device.name}
            status={device.status}
            isActive={device.id === activeDeviceId}
            hasExhaustedRetries={device.hasExhaustedRetries}
            errorType={device.errorType}
            onSelect={() => onSelect(device.id)}
            onRemove={() => onRemove(device.id)}
            onRename={() => onRename(device.id)}
            onRetry={() => onRetry(device.id)}
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
      {showRenameHint && (
        <div className="mt-2 flex items-center justify-center gap-2 text-xs text-slate-500 dark:text-slate-400">
          <span>{t("devices.renameHint")}</span>
          <button
            type="button"
            onClick={dismissRenameHint}
            className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
            aria-label="Dismiss"
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" className="h-3.5 w-3.5">
              <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
            </svg>
          </button>
        </div>
      )}
    </div>
  );
}
