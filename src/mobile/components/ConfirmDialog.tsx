import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmLabel,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const { t } = useTranslation();
  const confirmRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (isOpen) {
      confirmRef.current?.focus();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4 backdrop-blur-sm">
      <div className="w-full max-w-sm rounded-3xl bg-white p-6 shadow-xl dark:bg-slate-900">
        <div className="text-lg font-semibold text-slate-900 dark:text-slate-100">
          {title}
        </div>
        <p className="mt-2 text-sm text-slate-600 dark:text-slate-300">
          {message}
        </p>
        <div className="mt-5 flex gap-3">
          <button
            type="button"
            onClick={onCancel}
            className="flex-1 rounded-2xl bg-slate-200 py-3 font-medium text-slate-700 transition-colors hover:bg-slate-300 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
          >
            {t("devices.cancel")}
          </button>
          <button
            ref={confirmRef}
            type="button"
            onClick={onConfirm}
            className="flex-1 rounded-2xl bg-red-500 py-3 font-medium text-white transition-colors hover:bg-red-600"
          >
            {confirmLabel ?? t("devices.removeConfirmAction")}
          </button>
        </div>
      </div>
    </div>
  );
}
