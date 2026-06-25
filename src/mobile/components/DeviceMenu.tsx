import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { MoreVertical, Plus, Trash2 } from "lucide-react";

interface DeviceMenuProps {
  canRemove: boolean;
  onAdd: () => void;
  onRemove: () => void;
}

/** Kebab overflow menu with Add device and Remove (active) device actions. */
export function DeviceMenu({ canRemove, onAdd, onRemove }: DeviceMenuProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const choose = (fn: () => void) => {
    fn();
    setOpen(false);
  };

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-label={t("devices.menu")}
        aria-haspopup="menu"
        aria-expanded={open}
        className="flex h-9 w-9 items-center justify-center rounded-full text-slate-500 transition-colors hover:bg-black/5 hover:text-slate-700 dark:text-slate-400 dark:hover:bg-white/5 dark:hover:text-slate-200"
      >
        <MoreVertical className="h-5 w-5" />
      </button>
      {open && (
        <div
          role="menu"
          className="absolute right-0 top-full z-20 mt-1 w-44 overflow-hidden rounded-2xl border border-slate-200 bg-white py-1 shadow-lg dark:border-slate-700 dark:bg-slate-800"
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => choose(onAdd)}
            className="flex w-full items-center gap-2 px-4 py-2.5 text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-700"
          >
            <Plus className="h-4 w-4" /> {t("devices.addDevice")}
          </button>
          <button
            type="button"
            role="menuitem"
            disabled={!canRemove}
            onClick={() => canRemove && choose(onRemove)}
            className="flex w-full items-center gap-2 px-4 py-2.5 text-sm text-red-600 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-40 dark:text-red-400 dark:hover:bg-red-900/30"
          >
            <Trash2 className="h-4 w-4" /> {t("devices.removeDevice")}
          </button>
        </div>
      )}
    </div>
  );
}
