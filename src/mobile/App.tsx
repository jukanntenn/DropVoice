import type React from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { AddDeviceModal } from "./components/AddDeviceModal";
import { ConfirmDialog } from "./components/ConfirmDialog";
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

  const [showAddDevice, setShowAddDevice] = useState(false);
  const [renameDeviceId, setRenameDeviceId] = useState<string | null>(null);
  const [showRename, setShowRename] = useState(false);
  const [removeDeviceId, setRemoveDeviceId] = useState<string | null>(null);

  const prevActiveDeviceIdRef = useRef<string | null>(null);
  const touchStartRef = useRef<{ x: number; y: number } | null>(null);

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
    retryDevice,
    sendToActive,
  } = useMultiWebSocket(
    managedDevices,
    setDevices,
    managedActiveDeviceId,
    setActiveDeviceId,
  );

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    const touch = e.touches[0];
    touchStartRef.current = { x: touch.clientX, y: touch.clientY };
  }, []);

  const handleTouchEnd = useCallback(
    (e: React.TouchEvent) => {
      if (!touchStartRef.current) return;
      const touch = e.changedTouches[0];
      const dx = touch.clientX - touchStartRef.current.x;
      const dy = touch.clientY - touchStartRef.current.y;
      touchStartRef.current = null;

      if (Math.abs(dx) < 80 || Math.abs(dy) > Math.abs(dx) * 0.5) return;

      const sorted = [...devices].sort((a, b) => b.lastConnected - a.lastConnected);
      const currentIdx = sorted.findIndex((d) => d.id === activeDeviceId);
      if (currentIdx < 0) return;

      if (dx < 0 && currentIdx < sorted.length - 1) {
        setActiveDevice(sorted[currentIdx + 1].id);
      } else if (dx > 0 && currentIdx > 0) {
        setActiveDevice(sorted[currentIdx - 1].id);
      }
    },
    [activeDeviceId, devices, setActiveDevice],
  );

  useEffect(() => {
    initMobileI18n().then(() => setI18nReady(true));
  }, []);

  // Save current draft for old device, load draft for new device on switch
  useEffect(() => {
    const prevId = prevActiveDeviceIdRef.current;
    const newId = activeDeviceId;

    if (prevId && prevId !== newId) {
      saveDraft(text, prevId);
      const switchedDevice = devices.find((d) => d.id === newId);
      if (switchedDevice) {
        showToast(t("devices.switchedTo", { name: switchedDevice.name }), "success", 1500);
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

  // Debounced draft save for active device
  useEffect(() => {
    if (!activeDeviceId) return;
    const timer = window.setTimeout(() => {
      saveDraft(text, activeDeviceId);
    }, 500);
    return () => window.clearTimeout(timer);
  }, [text, activeDeviceId]);

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
    (url: string, name?: string) => {
      return addDevice(url, name);
    },
    [addDevice],
  );

  const handleRemoveDevice = useCallback(
    (deviceId: string) => {
      setRemoveDeviceId(deviceId);
    },
    [],
  );

  const handleConfirmRemove = useCallback(() => {
    if (!removeDeviceId) return;
    removeDevice(removeDeviceId);
    if (renameDeviceId === removeDeviceId) {
      setRenameDeviceId(null);
      setShowRename(false);
    }
    setRemoveDeviceId(null);
  }, [removeDevice, removeDeviceId, renameDeviceId]);

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
          onRetry={retryDevice}
          onAdd={() => setShowAddDevice(true)}
        />

        {/* Main glass card */}
        <div className="animate-slide-up mt-3 rounded-3xl p-0">
          <TextInput
            value={text}
            onChange={setText}
            onKeyDown={handleKeyDown}
            disabled={
              devices.find((d) => d.id === activeDeviceId)?.status !== "connected"
            }
            onOpenSettings={handleOpenSettings}
          />

          <div
            className="mt-6 flex items-center justify-between px-1"
            onTouchStart={handleTouchStart}
            onTouchEnd={handleTouchEnd}
          >
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

      <DeviceNameEditor
        isOpen={showRename}
        currentName={devices.find((d) => d.id === renameDeviceId)?.name ?? ""}
        onSave={handleSaveRename}
        onClose={() => {
          setShowRename(false);
          setRenameDeviceId(null);
        }}
      />

      <ConfirmDialog
        isOpen={removeDeviceId !== null}
        title={t("devices.removeConfirmTitle")}
        message={t("devices.removeConfirm", {
          name: devices.find((d) => d.id === removeDeviceId)?.name ?? "",
        })}
        onConfirm={handleConfirmRemove}
        onCancel={() => setRemoveDeviceId(null)}
      />
    </div>
  );
}
