import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "./ui/button";
import { Switch } from "./ui/switch";
import { Slider } from "./ui/slider";
import { Label } from "./ui/label";

type SettingsState = {
  autoStartEnabled: boolean;
  minimizeToTrayEnabled: boolean;
  minimizeToTrayVisible: boolean;
  inputDelay: number;
};

export default function SettingsModal({
  isOpen,
  onClose,
}: {
  isOpen: boolean;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<SettingsState>({
    autoStartEnabled: true,
    minimizeToTrayEnabled: true,
    minimizeToTrayVisible: false,
    inputDelay: 10,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [isSavingDelay, setIsSavingDelay] = useState(false);

  useEffect(() => {
    if (!isOpen) return;

    let isActive = true;
    setIsLoading(true);
    Promise.all([
      invoke<boolean>("get_minimize_to_tray"),
      invoke<boolean>("get_minimize_to_tray_visible"),
      isEnabled(),
      invoke<number>("get_input_delay"),
    ])
      .then(([minimizeToTrayEnabled, minimizeToTrayVisible, autoStartEnabled, inputDelay]) => {
        if (!isActive) return;
        setSettings({ autoStartEnabled, minimizeToTrayEnabled, minimizeToTrayVisible, inputDelay });
      })
      .catch((error) => {
        console.error("Failed to load settings:", error);
      })
      .finally(() => {
        if (!isActive) return;
        setIsLoading(false);
      });

    return () => {
      isActive = false;
    };
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [isOpen, onClose]);

  const handleAutoStartChange = async (nextEnabled: boolean) => {
    setSettings((prev) => ({ ...prev, autoStartEnabled: nextEnabled }));
    try {
      if (nextEnabled) {
        await enable();
      } else {
        await disable();
      }
    } catch (error) {
      console.error("Failed to set auto-start:", error);
      setSettings((prev) => ({ ...prev, autoStartEnabled: !nextEnabled }));
    }
  };

  const handleMinimizeToTrayChange = async (nextEnabled: boolean) => {
    setSettings((prev) => ({ ...prev, minimizeToTrayEnabled: nextEnabled }));
    try {
      await invoke("set_minimize_to_tray", { enabled: nextEnabled });
    } catch (error) {
      console.error("Failed to set minimize to tray:", error);
      setSettings((prev) => ({ ...prev, minimizeToTrayEnabled: !nextEnabled }));
    }
  };

  const handleInputDelayChange = (value: number[]) => {
    const newDelay = value[0];
    setSettings((prev) => ({ ...prev, inputDelay: newDelay }));
  };

  const handleInputDelayChangeCommit = async (value: number[]) => {
    const newDelay = value[0];
    setIsSavingDelay(true);
    try {
      await invoke("set_input_delay", { delayMs: newDelay });
    } catch (error) {
      console.error("Failed to set input delay:", error);
      // Revert on error
      setSettings((prev) => ({ ...prev, inputDelay: settings.inputDelay }));
    } finally {
      setIsSavingDelay(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="w-[360px] rounded-2xl border border-white/60 bg-white/80 p-6 shadow-glass backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/90"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-6 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-foreground dark:text-white">
            {t("settings.title")}
          </h2>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            aria-label={t("common.dismiss")}
            className="h-8 w-8 rounded-lg"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          </div>
        ) : (
          <div className="space-y-6">
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1">
                <p className="text-sm font-medium text-foreground dark:text-white">
                  {t("settings.autoStart.label")}
                </p>
                <p className="mt-1.5 text-[11px] text-muted-foreground/80 dark:text-slate-400">
                  {t("settings.autoStart.description")}
                </p>
              </div>
              <Switch
                checked={settings.autoStartEnabled}
                onCheckedChange={handleAutoStartChange}
              />
            </div>

            {settings.minimizeToTrayVisible && (
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1">
                  <p className="text-sm font-medium text-foreground dark:text-white">
                    {t("settings.minimizeToTray.label")}
                  </p>
                  <p className="mt-1.5 text-[11px] text-muted-foreground/80 dark:text-slate-400">
                    {t("settings.minimizeToTray.description")}
                  </p>
                </div>
                <Switch
                  checked={settings.minimizeToTrayEnabled}
                  onCheckedChange={handleMinimizeToTrayChange}
                />
              </div>
            )}

            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <Label htmlFor="input-delay" className="text-sm font-medium text-foreground dark:text-white">
                  {t("settings.inputDelay.label")}
                </Label>
                <span className="text-sm font-mono text-muted-foreground dark:text-slate-400">
                  {settings.inputDelay}ms
                </span>
              </div>
              <p className="text-[11px] text-muted-foreground/80 dark:text-slate-400">
                {t("settings.inputDelay.description")}
              </p>
              <Slider
                id="input-delay"
                min={1}
                max={100}
                step={1}
                value={[settings.inputDelay]}
                onValueChange={handleInputDelayChange}
                onValueCommit={handleInputDelayChangeCommit}
                disabled={isSavingDelay}
                className="cursor-pointer"
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
