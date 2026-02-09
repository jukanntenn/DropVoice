import type React from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { StatusIndicator } from "./components/StatusIndicator";
import { TextInput } from "./components/TextInput";
import { SendButton } from "./components/SendButton";
import { RestoreButton } from "./components/RestoreButton";
import { ClearButton } from "./components/ClearButton";
import { SettingsPage } from "./components/SettingsPage";
import { Toast, useToast } from "./components/Toast";
import { useWebSocket } from "./hooks/useWebSocket";
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

  const { t } = useTranslation();
  const { toasts, showToast, removeToast } = useToast();
  const { status, isSending, lastError, send, reconnectAttempts } =
    useWebSocket();

  useEffect(() => {
    initMobileI18n().then(() => setI18nReady(true));
  }, []);

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
    showToast(lastError, "error");
  }, [lastError, showToast]);

  const canSend = useMemo(() => {
    return (
      status === "connected" &&
      !isSending &&
      text.trim().length > 0 &&
      text.length <= 10000
    );
  }, [isSending, status, text]);

  const canRestore = useMemo(() => {
    return hasLastSent();
  }, [showRestore]);

  const canClear = useMemo(() => {
    return text.trim().length > 0;
  }, [text]);

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed) return;
    if (status !== "connected" || isSending) return;

    if (trimmed.length > 10000) {
      showToast(t("input.textTooLong"), "error");
      return;
    }

    saveLastSent(trimmed);
    clearDraft();

    const ok = send(trimmed);
    if (!ok) {
      showToast(t("notifications.sendFailed"), "error");
      return;
    }

    setText("");
    setShowRestore(true);
    showToast(t("notifications.sent"), "success");
  }, [isSending, send, showToast, status, t, text]);

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

  if (page === "settings") {
    return <SettingsPage onBack={handleBackFromSettings} />;
  }

  if (!i18nReady) {
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
        <StatusIndicator
          status={status}
          reconnectAttempts={reconnectAttempts}
        />

        {/* Main glass card */}
        <div className="animate-slide-up mt-0 p-0 rounded-3xl">
          <TextInput
            value={text}
            onChange={setText}
            onKeyDown={handleKeyDown}
            disabled={status !== "connected"}
            onOpenSettings={handleOpenSettings}
          />

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
          onClose={() => removeToast(toast.id)}
        />
      ))}
    </div>
  );
}
