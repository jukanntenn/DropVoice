import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "./ui/button";

export default function LanWarning() {
  const { t } = useTranslation();
  const [visible, setVisible] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const dismissed = await invoke<boolean>("get_lan_warning_dismissed");
        if (cancelled) return;
        setVisible(!dismissed);
      } catch {
        if (cancelled) return;
        setVisible(true);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    load();

    return () => {
      cancelled = true;
    };
  }, []);

  const dismiss = async () => {
    try {
      await invoke("set_lan_warning_dismissed");
    } catch {
    } finally {
      setVisible(false);
    }
  };

  if (loading || !visible) return null;

  return (
    <div className="animate-slide-up flex w-full max-w-sm items-center gap-3 rounded-2xl border border-amber-200/60 bg-amber-50/80 px-5 py-4 backdrop-blur-sm dark:border-amber-500/20 dark:bg-amber-950/40">
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-amber-100 dark:bg-amber-900/50">
        <AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-400" />
      </div>
      <span className="flex-1 text-sm leading-relaxed text-amber-800 dark:text-amber-200">
        {t("connection.warning")}
      </span>
      <Button
        variant="ghost"
        size="icon"
        className="h-8 w-8 shrink-0 rounded-lg text-amber-600 transition-colors hover:bg-amber-100 hover:text-amber-800 dark:text-amber-400 dark:hover:bg-amber-900/50 dark:hover:text-amber-200"
        onClick={dismiss}
        aria-label={t("common.dismiss")}
      >
        <X className="h-4 w-4" />
      </Button>
    </div>
  );
}
