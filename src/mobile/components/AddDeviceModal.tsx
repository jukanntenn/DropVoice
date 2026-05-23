import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { QRScanner } from "./QRScanner";
import { normalizeToWebSocketUrl } from "../utils/deviceStorage";
import { checkCameraSupport } from "../utils/cameraSupport";

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
  const [addedCount, setAddedCount] = useState(0);
  const [lastAdded, setLastAdded] = useState<string | null>(null);
  const autoSwitchTimerRef = useRef<number | null>(null);
  const lastAddedTimerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (autoSwitchTimerRef.current !== null) {
        window.clearTimeout(autoSwitchTimerRef.current);
        autoSwitchTimerRef.current = null;
      }
      if (lastAddedTimerRef.current !== null) {
        window.clearTimeout(lastAddedTimerRef.current);
        lastAddedTimerRef.current = null;
      }
    };
  }, []);

  const reset = useCallback(() => {
    if (autoSwitchTimerRef.current !== null) {
      window.clearTimeout(autoSwitchTimerRef.current);
      autoSwitchTimerRef.current = null;
    }
    if (lastAddedTimerRef.current !== null) {
      window.clearTimeout(lastAddedTimerRef.current);
      lastAddedTimerRef.current = null;
    }
    setIsScanning(false);
    setUrlInput("");
    setNameInput("");
    setError(null);
    setTab("scan");
    setAddedCount(0);
    setLastAdded(null);
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
      if (ok) {
        setAddedCount((c) => c + 1);
        setLastAdded(wsUrl);
        setError(null);
        if (lastAddedTimerRef.current !== null) {
          window.clearTimeout(lastAddedTimerRef.current);
        }
        lastAddedTimerRef.current = window.setTimeout(() => {
          setLastAdded(null);
        }, 2000);
      }
    },
    [onAdd, t],
  );

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4 backdrop-blur-sm">
      <div className="flex w-full max-w-md flex-col overflow-hidden rounded-3xl bg-white shadow-xl dark:bg-slate-900">
        <div className="flex items-center justify-between border-b border-slate-200 px-4 py-3 dark:border-slate-800">
          <div className="flex items-center gap-2">
            <div className="text-lg font-semibold text-slate-900 dark:text-slate-100">
              {t("devices.addDevice")}
            </div>
            {addedCount > 0 && (
              <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                {t("devices.scanAddedCount", { count: addedCount })}
              </span>
            )}
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
              <div className="rounded-2xl bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:bg-amber-900/30 dark:text-amber-200">
                <div className="flex items-start gap-2">
                  <svg
                    className="mt-0.5 h-5 w-5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fillRule="evenodd"
                      d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                      clipRule="evenodd"
                    />
                  </svg>
                  <div>
                    <p className="font-medium">{t("devices.hintTitle")}</p>
                    <p className="mt-1">{t("devices.scanHintHttps")}</p>
                  </div>
                </div>
              </div>

              {isScanning ? (
                <div className="relative">
                  <QRScanner
                    onScan={onScan}
                    onError={(e) => {
                      setError(e);
                    }}
                    continuous
                  />
                  {lastAdded && (
                    <div className="absolute inset-0 z-20 flex items-center justify-center rounded-2xl bg-black/40">
                      <div className="flex items-center gap-2 rounded-xl bg-white/90 px-4 py-2 text-sm font-medium text-green-700 dark:bg-slate-800/90 dark:text-green-400">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-5 w-5">
                          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z" clipRule="evenodd" />
                        </svg>
                        {t("devices.added")}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center rounded-3xl border border-slate-200 bg-slate-50 px-4 py-10 text-center dark:border-slate-800 dark:bg-slate-950/40">
                  <div className="text-sm text-slate-600 dark:text-slate-300">
                    {t("devices.scanHint")}
                  </div>
                </div>
              )}

              <button
                type="button"
                onClick={async () => {
                  if (isScanning) {
                    setIsScanning(false);
                    return;
                  }

                  setError(null);
                  const result = await checkCameraSupport();
                  if (!result.supported) {
                    const message =
                      result.reason === "INSECURE_CONTEXT"
                        ? t("devices.cameraRequiresHttpsAutoSwitch")
                        : result.reason === "MEDIA_DEVICES_UNAVAILABLE"
                          ? t("devices.cameraNotSupported")
                          : result.reason === "PERMISSION_DENIED"
                            ? t("devices.cameraPermissionDenied")
                            : result.reason === "NO_CAMERA"
                              ? t("devices.cameraNotFound")
                              : t("devices.cameraInitFailed", {
                                  reason: result.detail ?? "",
                                });

                    setError(message);

                    if (autoSwitchTimerRef.current !== null) {
                      window.clearTimeout(autoSwitchTimerRef.current);
                    }
                    autoSwitchTimerRef.current = window.setTimeout(() => {
                      setTab("input");
                      setIsScanning(false);
                      setError(null);
                      autoSwitchTimerRef.current = null;
                    }, 2000);
                    return;
                  }

                  setIsScanning(true);
                }}
                className={[
                  "w-full rounded-2xl py-3 font-medium transition-colors",
                  isScanning
                    ? "bg-slate-200 text-slate-700 hover:bg-slate-300 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
                    : "bg-primary text-white hover:bg-primary/90",
                ].join(" ")}
              >
                {isScanning ? t("devices.done") : t("devices.startScan")}
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
                {normalizedUrl && normalizedUrl !== urlInput.trim() ? (
                  <div className="mt-1 px-1 text-xs text-slate-500 dark:text-slate-400">
                    {t("devices.willConnectTo", { url: normalizedUrl })}
                  </div>
                ) : null}
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

          {error && (
            <div className="mt-4 rounded-2xl bg-red-100 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-200">
              <div className="flex items-start gap-2">
                <svg
                  className="mt-0.5 h-5 w-5 flex-shrink-0"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                    clipRule="evenodd"
                  />
                </svg>
                <div>
                  <p className="font-medium">{t("devices.cameraUnavailableTitle")}</p>
                  <p className="mt-1">{error}</p>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
