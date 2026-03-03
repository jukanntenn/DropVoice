import type React from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { AddDeviceModal } from "./components/AddDeviceModal";
import { DeviceNameEditor } from "./components/DeviceNameEditor";
import { DeviceTabs } from "./components/DeviceTabs";
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

type Page = "main" | "settings";

export default function App() {
  const [page, setPage] = useState<Page>("main");
  useSettings();

  const [i18nReady, setI18nReady] = useState(false);
  const [text, setText] = useState("");
  const [showRestore, setShowRestore] = useState(false);

  const [showAddDevice, setShowAddDevice] = useState(false);
  const [renameDeviceId, setRenameDeviceId] = useState<string | null>(null);
  const [showRename, setShowRename] = useState(false);
  const [isStabilizing, setIsStabilizing] = useState(true);

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
    renameDevice,
    setActiveDevice,
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
    if (!isInitialized) {
      setIsStabilizing(true);
      return;
    }

    setIsStabilizing(true);
    const timer = window.setTimeout(() => {
      setIsStabilizing(false);
    }, 2000);

    return () => window.clearTimeout(timer);
  }, [isInitialized]);

  useEffect(() => {
    const draft = loadDraft();
    if (draft) setText(draft);
    setShowRestore(hasLastSent());
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      saveDraft(text);
    }, 500);
    return () => window.clearTimeout(timer);
  }, [text]);

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
    if (isStabilizing) return false;
    const active = devices.find((d) => d.id === activeDeviceId);
    return (
      !!active &&
      active.status === "connected" &&
      !isSending &&
      text.trim().length > 0 &&
      text.length <= 10000
    );
  }, [activeDeviceId, devices, isSending, isStabilizing, text]);

  const canRestore = useMemo(() => {
    return hasLastSent();
  }, [showRestore]);

  const canClear = useMemo(() => {
    return text.trim().length > 0;
  }, [text]);

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed) return;
    if (isStabilizing) return;
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

    saveLastSent(trimmed);
    clearDraft();

    const ok = sendToActive(trimmed);
    if (!ok) {
      showToast(t("devices.sendFailed"), "error");
      return;
    }

    setText("");
    setShowRestore(true);
  }, [activeDeviceId, devices, isSending, isStabilizing, sendToActive, showToast, t, text]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.ctrlKey && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  const handleRestore = useCallback(() => {
    const last = loadLastSent();
    if (!last) return;
    setText(last);
    showToast(t("notifications.lastMessageRestored"), "success");
  }, [showToast, t]);

  const handleClearInput = useCallback(() => {
    setText("");
    clearDraft();
  }, []);

  const handleOpenSettings = useCallback(() => {
    setPage("settings");
  }, []);

  const handleBackFromSettings = useCallback(() => {
    setPage("main");
  }, []);

  const handleAddDevice = useCallback(
    (url: string, name?: string) => {
      return addDevice(url, name);
    },
    [addDevice],
  );

  const handleRemoveDevice = useCallback(
    (deviceId: string) => {
      removeDevice(deviceId);
      if (renameDeviceId === deviceId) {
        setRenameDeviceId(null);
        setShowRename(false);
      }
    },
    [removeDevice, renameDeviceId],
  );

  const handleRenameDevice = useCallback((deviceId: string) => {
    setRenameDeviceId(deviceId);
    setShowRename(true);
  }, []);

  const handleSaveRename = useCallback(
    (newName: string) => {
      if (!renameDeviceId) return;
      renameDevice(renameDeviceId, newName);
    },
    [renameDevice, renameDeviceId],
  );

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
        <DeviceTabs
          devices={devices}
          activeDeviceId={activeDeviceId}
          onSelect={setActiveDevice}
          onRemove={handleRemoveDevice}
          onRename={handleRenameDevice}
          onAdd={() => setShowAddDevice(true)}
        />

        {/* Main glass card */}
        <div className="animate-slide-up mt-3 rounded-3xl p-0">
          <TextInput
            value={text}
            onChange={setText}
            onKeyDown={handleKeyDown}
            disabled={
              isStabilizing ||
              devices.find((d) => d.id === activeDeviceId)?.status !== "connected"
            }
            onOpenSettings={handleOpenSettings}
          />

          <div className="mt-6 flex items-center justify-between px-1">
            <RestoreButton onClick={handleRestore} disabled={!canRestore} />
            <SendButton
              onClick={handleSend}
              disabled={!canSend}
              isSending={isSending || isStabilizing}
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
          onClose={() => removeToast(toast.id)}
        />
      ))}

      <AddDeviceModal
        isOpen={showAddDevice}
        onClose={() => setShowAddDevice(false)}
        onAdd={handleAddDevice}
      />

      <DeviceNameEditor
        isOpen={showRename}
        currentName={devices.find((d) => d.id === renameDeviceId)?.name ?? ""}
        onSave={handleSaveRename}
        onClose={() => {
          setShowRename(false);
          setRenameDeviceId(null);
        }}
      />
    </div>
  );
}
