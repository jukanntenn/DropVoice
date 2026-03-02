import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { QRScanner } from "./QRScanner";
import { normalizeToWebSocketUrl } from "../utils/deviceStorage";

type Tab = "scan" | "input";

interface AddDeviceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (url: string, name?: string) => boolean;
}

export function AddDeviceModal({ isOpen, onClose, onAdd }: AddDeviceModalProps) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("scan");
  const [isScanning, setIsScanning] = useState(false);
  const [urlInput, setUrlInput] = useState("");
  const [nameInput, setNameInput] = useState("");
  const [error, setError] = useState<string | null>(null);

  const reset = useCallback(() => {
    setIsScanning(false);
    setUrlInput("");
    setNameInput("");
    setError(null);
    setTab("scan");
  }, []);

  const close = useCallback(() => {
    reset();
    onClose();
  }, [onClose, reset]);

  const normalizedUrl = useMemo(() => normalizeToWebSocketUrl(urlInput), [urlInput]);

  const submitUrl = useCallback(() => {
    setError(null);
    const wsUrl = normalizedUrl;
    if (!wsUrl) {
      setError(t("devices.invalidUrl"));
      return;
    }
    const ok = onAdd(wsUrl, nameInput.trim() ? nameInput.trim() : undefined);
    if (ok) close();
  }, [close, nameInput, normalizedUrl, onAdd, t]);

  const onScan = useCallback(
    (text: string) => {
      const wsUrl = normalizeToWebSocketUrl(text);
      if (!wsUrl) {
        setError(t("devices.invalidUrl"));
        return;
      }
      const ok = onAdd(wsUrl);
      if (ok) close();
    },
    [close, onAdd, t],
  );

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4 backdrop-blur-sm">
      <div className="flex w-full max-w-md flex-col overflow-hidden rounded-3xl bg-white shadow-xl dark:bg-slate-900">
        <div className="flex items-center justify-between border-b border-slate-200 px-4 py-3 dark:border-slate-800">
          <div className="text-lg font-semibold text-slate-900 dark:text-slate-100">
            {t("devices.addDevice")}
          </div>
          <button
            type="button"
            onClick={close}
            className="text-slate-400 transition-colors hover:text-slate-600 dark:hover:text-slate-200"
            aria-label={t("devices.cancel")}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              className="h-6 w-6"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        <div className="flex border-b border-slate-200 dark:border-slate-800">
          <button
            type="button"
            onClick={() => {
              setTab("scan");
              setError(null);
            }}
            className={[
              "flex-1 py-3 text-sm font-medium transition-colors",
              tab === "scan"
                ? "border-b-2 border-primary text-primary"
                : "text-slate-500 dark:text-slate-400",
            ].join(" ")}
          >
            {t("devices.scanQR")}
          </button>
          <button
            type="button"
            onClick={() => {
              setTab("input");
              setIsScanning(false);
              setError(null);
            }}
            className={[
              "flex-1 py-3 text-sm font-medium transition-colors",
              tab === "input"
                ? "border-b-2 border-primary text-primary"
                : "text-slate-500 dark:text-slate-400",
            ].join(" ")}
          >
            {t("devices.inputUrl")}
          </button>
        </div>

        <div className="max-h-[75vh] overflow-y-auto p-4">
          {tab === "scan" ? (
            <div className="space-y-4">
              {isScanning ? (
                <QRScanner
                  onScan={onScan}
                  onError={(e) => {
                    setError(e);
                  }}
                />
              ) : (
                <div className="flex flex-col items-center justify-center rounded-3xl border border-slate-200 bg-slate-50 px-4 py-10 text-center dark:border-slate-800 dark:bg-slate-950/40">
                  <div className="text-sm text-slate-600 dark:text-slate-300">
                    {t("devices.scanHint")}
                  </div>
                </div>
              )}

              <button
                type="button"
                onClick={() => setIsScanning((v) => !v)}
                className={[
                  "w-full rounded-2xl py-3 font-medium transition-colors",
                  isScanning
                    ? "bg-slate-200 text-slate-700 hover:bg-slate-300 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
                    : "bg-primary text-white hover:bg-primary/90",
                ].join(" ")}
              >
                {isScanning ? t("devices.stopScan") : t("devices.startScan")}
              </button>
            </div>
          ) : (
            <div className="space-y-4">
              <div>
                <label className="mb-2 block text-sm font-medium text-slate-700 dark:text-slate-200">
                  {t("devices.connectionAddress")}
                </label>
                <input
                  type="text"
                  value={urlInput}
                  onChange={(e) => setUrlInput(e.target.value)}
                  placeholder={t("devices.urlPlaceholder")}
                  className="w-full rounded-2xl border border-slate-300 bg-white px-4 py-3 text-slate-900 outline-none focus:ring-2 focus:ring-primary/50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                />
              </div>
              <div>
                <label className="mb-2 block text-sm font-medium text-slate-700 dark:text-slate-200">
                  {t("devices.deviceNameOptional")}
                </label>
                <input
                  type="text"
                  value={nameInput}
                  onChange={(e) => setNameInput(e.target.value)}
                  placeholder={t("devices.namePlaceholder")}
                  className="w-full rounded-2xl border border-slate-300 bg-white px-4 py-3 text-slate-900 outline-none focus:ring-2 focus:ring-primary/50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                />
              </div>
              <button
                type="button"
                onClick={submitUrl}
                className="w-full rounded-2xl bg-primary py-3 font-medium text-white transition-colors hover:bg-primary/90"
              >
                {t("devices.add")}
              </button>
            </div>
          )}

          {error ? (
            <div className="mt-4 rounded-2xl bg-red-100 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-200">
              {error}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
