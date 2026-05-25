import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { Contrast, Globe, Moon, Settings, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useTheme } from "./ThemeProvider";
import SettingsModal from "./SettingsModal";
import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

export default function Header() {
  const { t, i18n } = useTranslation();
  const { theme, toggleTheme } = useTheme();

  const [version, setVersion] = useState<string>("");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const currentLang = (() => {
    const lang = i18n.language?.toLowerCase() ?? "";
    if (lang.startsWith("zh-tw") || lang.startsWith("zh-hk") || lang.startsWith("zh-hant")) return "zh-TW";
    if (lang.startsWith("zh")) return "zh";
    if (lang.startsWith("ja")) return "ja";
    return "en";
  })();

  useEffect(() => {
    let isActive = true;

    getVersion()
      .then((v) => {
        if (isActive) setVersion(v || "?.?.?");
      })
      .catch(() => {
        if (isActive) setVersion("?.?.?");
      });

    return () => {
      isActive = false;
    };
  }, []);

  const setLanguage = async (next: string) => {
    try {
      await invoke("set_language", { lang: next });
      await i18n.changeLanguage(next);
    } catch {
      await i18n.changeLanguage(next);
    }
  };

  return (
    <>
      <header className="sticky top-0 z-50 flex h-14 items-center justify-between border-b border-transparent px-3 backdrop-blur-none dark:border-transparent dark:bg-transparent">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-teal-400 shadow-lg shadow-primary/25">
            <img
              src="/app-icon.png"
              alt="DropVoice"
              className="h-6 w-6 rounded-lg"
            />
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-lg font-semibold tracking-tight text-foreground dark:text-white">
              DropVoice
            </span>
            {version && (
              <span className="rounded-md bg-primary/15 px-1.5 py-0.5 text-[10px] font-medium text-primary dark:bg-primary/20 dark:text-teal-400">
                v{version}
              </span>
            )}
          </div>
        </div>

        <div className="flex items-center gap-1">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setIsSettingsOpen(true)}
                aria-label={t("settings.title")}
                className="h-9 w-9 rounded-xl transition-colors hover:bg-secondary"
              >
                <Settings className="h-4.5 w-4.5 text-muted-foreground dark:text-slate-400" />
              </Button>
            </TooltipTrigger>
            <TooltipContent className="rounded-lg bg-slate-900 px-3 py-1.5 text-xs text-white dark:bg-white dark:text-slate-900">
              <p>{t("settings.title")}</p>
            </TooltipContent>
          </Tooltip>

          <Select
            value={currentLang}
            onValueChange={(value) => setLanguage(value)}
          >
            <SelectTrigger className="h-9 w-auto gap-2 border-transparent bg-transparent px-3 text-foreground shadow-none transition-colors hover:bg-secondary focus:ring-0 focus:ring-offset-0 dark:text-white">
              <Globe className="h-4 w-4 text-muted-foreground dark:text-slate-400" />
              <SelectValue placeholder={t("language.switch")} />
            </SelectTrigger>
            <SelectContent className="rounded-xl border-border/50 bg-white/95 backdrop-blur-xl dark:bg-slate-900/95">
              <SelectItem value="en" className="rounded-lg">
                English
              </SelectItem>
              <SelectItem value="zh" className="rounded-lg">
                简体中文
              </SelectItem>
              <SelectItem value="zh-TW" className="rounded-lg">
                繁體中文
              </SelectItem>
              <SelectItem value="ja" className="rounded-lg">
                日本語
              </SelectItem>
            </SelectContent>
          </Select>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={toggleTheme}
                aria-label={t(
                  `theme.${
                    theme === "system"
                      ? "light"
                      : theme === "light"
                        ? "dark"
                        : "system"
                  }`,
                )}
                className="h-9 w-9 rounded-xl transition-colors hover:bg-secondary"
              >
                {theme === "system" ? (
                  <Contrast className="h-4.5 w-4.5 text-slate-500 dark:text-slate-400" />
                ) : theme === "light" ? (
                  <Sun className="h-4.5 w-4.5 text-amber-500" />
                ) : (
                  <Moon className="h-4.5 w-4.5 text-slate-600" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent className="rounded-lg bg-slate-900 px-3 py-1.5 text-xs text-white dark:bg-white dark:text-slate-900">
              <p>{t(`theme.${theme}`)}</p>
            </TooltipContent>
          </Tooltip>
        </div>
      </header>

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />
    </>
  );
}
