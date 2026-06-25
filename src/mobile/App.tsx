import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { AddDeviceModal } from "./components/AddDeviceModal";
import { ConfirmDialog } from "./components/ConfirmDialog";
import { ConnectionPill } from "./components/ConnectionPill";
import { DeviceDots } from "./components/DeviceDots";
import { DeviceMenu } from "./components/DeviceMenu";
import { TextInput } from "./components/TextInput";
import { SendButton } from "./components/SendButton";
import { RestoreButton } from "./components/RestoreButton";
import { ClearButton } from "./components/ClearButton";
import { SettingsPage } from "./components/SettingsPage";
import { Toast, useToast } from "./components/Toast";
import { useDeviceManager } from "./hooks/useDeviceManager";
import { useMultiWebSocket } from "./hooks/useMultiWebSocket";
import { useSettings } from "./hooks/useSettings";
import {
  clearDraft,
  hasLastSent,
  loadDraft,
  loadLastSent,
  saveDraft,
  saveLastSent,
} from "./utils/storage";
import { initMobileI18n } from "./i18n";
import { storageAvailable } from "./utils/createStorage";
import { getDeviceHostLabel } from "./utils/deviceColor";

type Page = "main" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("main");
  const { resolvedTheme } = useSettings();

  const [i18nReady, setI18nReady] = useState(false);
  const [text, setText] = useState("");

  const [showAddDevice, setShowAddDevice] = useState(false);
  const [removeDeviceId, setRemoveDeviceId] = useState<string | null>(null);

  const prevActiveDeviceIdRef = useRef<string | null>(null);
  const textRef = useRef(text);
  textRef.current = text;

  const { t } = useTranslation();
  const { toasts, showToast, removeToast } = useToast();
  const {
    isInitialized,
    devices: managedDevices,
    setDevices,
    activeDeviceId: managedActiveDeviceId,
    setActiveDeviceId,
  } = useDeviceManager();

  const {
    devices,
    activeDeviceId,
    isSending,
    lastError,
    clearError,
    addDevice,
    removeDevice,
    setActiveDevice,
    retryDevice,
    sendToActive,
  } = useMultiWebSocket(
    managedDevices,
    setDevices,
    managedActiveDeviceId,
    setActiveDeviceId,
  );

  useEffect(() => {
    initMobileI18n().then(() => setI18nReady(true));
  }, []);

  useEffect(() => {
    if (i18nReady && !storageAvailable()) {
      showToast(t("notifications.storageUnavailable"), "error", 8000);
    }
  }, [i18nReady, showToast, t]);

  // Save current draft for old device, load draft for new device on switch
  useEffect(() => {
    const prevId = prevActiveDeviceIdRef.current;
    const newId = activeDeviceId;

    if (prevId && prevId !== newId) {
      saveDraft(text, prevId);
      const switchedDevice = devices.find((d) => d.id === newId);
      if (switchedDevice) {
        showToast(
          t("devices.switchedTo", { ip: getDeviceHostLabel(switchedDevice) }),
          "success",
          1500,
        );
      }
    }

    if (newId) {
      const draft = loadDraft(newId);
      setText(draft ?? "");
    } else {
      setText("");
    }

    prevActiveDeviceIdRef.current = newId;
  }, [activeDeviceId]);

  // Reflect the active device's host in the browser tab title.
  useEffect(() => {
    const active = devices.find((d) => d.id === activeDeviceId);
    document.title = active ? getDeviceHostLabel(active) : "DropVoice";
  }, [activeDeviceId, devices]);

  // Debounced draft save for active device
  useEffect(() => {
    if (!activeDeviceId) return;
    const timer = window.setTimeout(() => {
      saveDraft(text, activeDeviceId);
    }, 500);
    return () => window.clearTimeout(timer);
  }, [text, activeDeviceId]);

  // Immediate draft save on page hide / unload
  useEffect(() => {
    if (!activeDeviceId) return;
    const save = () => {
      if (textRef.current && activeDeviceId) {
        saveDraft(textRef.current, activeDeviceId);
      }
    };
    const onVisibilityChange = () => {
      if (document.visibilityState === "hidden") save();
    };
    document.addEventListener("visibilitychange", onVisibilityChange);
    window.addEventListener("beforeunload", save);
    return () => {
      document.removeEventListener("visibilitychange", onVisibilityChange);
      window.removeEventListener("beforeunload", save);
    };
  }, [activeDeviceId]);

  useEffect(() => {
    if (!lastError) return;
    if (lastError.startsWith("devices.")) {
      showToast(t(lastError), "error");
    } else {
      showToast(lastError, "error");
    }
    clearError();
  }, [clearError, lastError, showToast, t]);

  const canSend = useMemo(() => {
    const active = devices.find((d) => d.id === activeDeviceId);
    return (
      !!active &&
      active.status === "connected" &&
      !isSending &&
      text.trim().length > 0 &&
      text.length <= 10000
    );
  }, [activeDeviceId, devices, isSending, text]);

  const canRestore = useMemo(() => {
    return activeDeviceId ? hasLastSent(activeDeviceId) : false;
  }, [activeDeviceId, toasts]);

  const canClear = useMemo(() => {
    return text.trim().length > 0;
  }, [text]);

  const activeDevice = useMemo(
    () => devices.find((d) => d.id === activeDeviceId) ?? null,
    [activeDeviceId, devices],
  );
  const dark = resolvedTheme === "dark";

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed) return;
    if (!activeDeviceId) {
      showToast(t("devices.selectDevice"), "error");
      return;
    }
    const active = devices.find((d) => d.id === activeDeviceId);
    if (!active || active.status !== "connected" || isSending) {
      showToast(t("devices.deviceNotConnected"), "error");
      return;
    }

    if (trimmed.length > 10000) {
      showToast(t("input.textTooLong"), "error");
      return;
    }

    saveLastSent(trimmed, activeDeviceId);
    clearDraft(activeDeviceId);

    const ok = sendToActive(trimmed);
    if (!ok) {
      showToast(t("devices.sendFailed"), "error");
      return;
    }

    setText("");
  }, [activeDeviceId, devices, isSending, sendToActive, showToast, t, text]);

  const handleRestore = useCallback(() => {
    if (!activeDeviceId) return;
    const last = loadLastSent(activeDeviceId);
    if (!last) return;
    setText(last);
    showToast(t("notifications.lastMessageRestored"), "success");
  }, [activeDeviceId, showToast, t]);

  const handleClearInput = useCallback(() => {
    setText("");
    if (activeDeviceId) clearDraft(activeDeviceId);
  }, [activeDeviceId]);

  const handleOpenSettings = useCallback(() => {
    setPage("settings");
  }, []);

  const handleBackFromSettings = useCallback(() => {
    setPage("main");
  }, []);

  const handleAddDevice = useCallback(
    (url: string) => {
      return addDevice(url);
    },
    [addDevice],
  );

  const handleConfirmRemove = useCallback(() => {
    if (!removeDeviceId) return;
    removeDevice(removeDeviceId);
    setRemoveDeviceId(null);
  }, [removeDevice, removeDeviceId]);

  if (page === "settings") {
    return <SettingsPage onBack={handleBackFromSettings} />;
  }

  if (!i18nReady || !isInitialized) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-teal-50 via-white to-cyan-50 p-4 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground dark:text-slate-400">
            Loading...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-teal-50 via-white to-cyan-50 px-5 pt-[calc(env(safe-area-inset-top)+1.25rem)] pb-[calc(env(safe-area-inset-bottom)+1.75rem)] dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
      {/* Decorative background elements */}
      <div className="pointer-events-none fixed inset-0 overflow-hidden">
        <div className="absolute -top-20 -right-20 h-64 w-64 rounded-full bg-primary/10 blur-3xl" />
        <div className="absolute -bottom-20 -left-20 h-64 w-64 rounded-full bg-cyan-400/10 blur-3xl dark:bg-cyan-500/5" />
      </div>

      <div className="relative mx-auto max-w-md">
        <div className="flex min-h-[2.25rem] items-center justify-between">
          <ConnectionPill device={activeDevice} onRetry={() => activeDevice && retryDevice(activeDevice.id)} />
          <DeviceMenu
            canRemove={!!activeDeviceId}
            onAdd={() => setShowAddDevice(true)}
            onRemove={() => activeDeviceId && setRemoveDeviceId(activeDeviceId)}
          />
        </div>

        {/* Main glass card */}
        <div className="animate-slide-up mt-3 rounded-3xl p-0">
          <TextInput
            value={text}
            onChange={setText}
            disabled={
              devices.find((d) => d.id === activeDeviceId)?.status !== "connected"
            }
            onOpenSettings={handleOpenSettings}
          />

          {devices.length === 0 ? (
            <p className="mt-3 text-center text-sm text-muted-foreground dark:text-slate-400">
              {t("devices.emptyState")}
            </p>
          ) : (
            <DeviceDots
              devices={devices}
              activeDeviceId={activeDeviceId}
              dark={dark}
              onSelect={setActiveDevice}
            />
          )}

          <div className="mt-6 flex items-center justify-between px-1">
            <RestoreButton onClick={handleRestore} disabled={!canRestore} />
            <SendButton
              onClick={handleSend}
              disabled={!canSend}
              isSending={isSending}
            />
            <ClearButton onClick={handleClearInput} disabled={!canClear} />
          </div>
        </div>
      </div>

      {toasts.map((toast) => (
        <Toast
          key={toast.id}
          message={toast.message}
          type={toast.type}
          duration={toast.duration}
          onClose={() => removeToast(toast.id)}
        />
      ))}

      <AddDeviceModal
        isOpen={showAddDevice}
        onClose={() => setShowAddDevice(false)}
        onAdd={handleAddDevice}
      />

      <ConfirmDialog
        isOpen={removeDeviceId !== null}
        title={t("devices.removeConfirmTitle")}
        message={t("devices.removeConfirm", {
          ip: getDeviceHostLabel(
            devices.find((d) => d.id === removeDeviceId) ?? { id: "", name: "", url: "", status: "disconnected", lastConnected: 0 },
          ),
        })}
        onConfirm={handleConfirmRemove}
        onCancel={() => setRemoveDeviceId(null)}
      />
    </div>
  );
}
