import type React from "react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

interface DeviceNameEditorProps {
  isOpen: boolean;
  currentName: string;
  onSave: (newName: string) => void;
  onClose: () => void;
}

export function DeviceNameEditor({
  isOpen,
  currentName,
  onSave,
  onClose,
}: DeviceNameEditorProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(currentName);

  useEffect(() => {
    setName(currentName);
  }, [currentName]);

  if (!isOpen) return null;

  const save = () => {
    const trimmed = name.trim();
    if (!trimmed) return;
    onSave(trimmed);
    onClose();
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") save();
    if (e.key === "Escape") onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4 backdrop-blur-sm">
      <div className="w-full max-w-sm rounded-3xl bg-white p-6 shadow-xl dark:bg-slate-900">
        <div className="text-lg font-semibold text-slate-900 dark:text-slate-100">
          {t("devices.renameDevice")}
        </div>
        <div className="mt-4">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={onKeyDown}
            autoFocus
            className="w-full rounded-2xl border border-slate-300 bg-white px-4 py-3 text-slate-900 outline-none focus:ring-2 focus:ring-primary/50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
          />
        </div>
        <div className="mt-5 flex gap-3">
          <button
            type="button"
            onClick={onClose}
            className="flex-1 rounded-2xl bg-slate-200 py-3 font-medium text-slate-700 transition-colors hover:bg-slate-300 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
          >
            {t("devices.cancel")}
          </button>
          <button
            type="button"
            onClick={save}
            disabled={!name.trim()}
            className="flex-1 rounded-2xl bg-primary py-3 font-medium text-white transition-colors hover:bg-primary/90 disabled:opacity-50"
          >
            {t("devices.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
