import { ArrowLeft } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { useSettings, type Language, type Theme } from "../hooks/useSettings";

interface SettingsPageProps {
  onBack: () => void;
}

export function SettingsPage({ onBack }: SettingsPageProps) {
  const { t } = useTranslation();
  const { language, setLanguage, theme, setTheme } = useSettings();

  const languages: { value: Language; label: string }[] = [
    { value: "en", label: "English" },
    { value: "zh", label: "中文" },
  ];

  const themes: { value: Theme; labelKey: "themeSystem" | "themeLight" | "themeDark" }[] =
    [
      { value: "system", labelKey: "themeSystem" },
      { value: "light", labelKey: "themeLight" },
      { value: "dark", labelKey: "themeDark" },
    ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-teal-50 via-white to-cyan-50 px-5 pt-[calc(env(safe-area-inset-top)+1.25rem)] pb-[calc(env(safe-area-inset-bottom)+1.75rem)] dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
      <div className="pointer-events-none fixed inset-0 overflow-hidden">
        <div className="absolute -top-20 -right-20 h-64 w-64 rounded-full bg-primary/10 blur-3xl" />
        <div className="absolute -bottom-20 -left-20 h-64 w-64 rounded-full bg-cyan-400/10 blur-3xl dark:bg-cyan-500/5" />
      </div>

      <div className="relative mx-auto max-w-md">
        <div className="mb-6 flex items-center gap-3">
          <button
            type="button"
            onClick={onBack}
            className="flex h-10 w-10 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-black/5 hover:text-foreground dark:text-slate-400 dark:hover:bg-white/5 dark:hover:text-white"
            aria-label="Back"
          >
            <ArrowLeft className="h-5 w-5" />
          </button>
          <h1 className="text-xl font-semibold text-foreground dark:text-white">
            {t("settings.title")}
          </h1>
        </div>

        <div className="rounded-3xl border border-white/60 bg-white/70 p-4 shadow-glass backdrop-blur-xl dark:border-white/10 dark:bg-slate-900/70 dark:shadow-glass-dark">
          <div className="space-y-4">
            <div className="flex items-center justify-between gap-4 py-3">
              <label className="shrink-0 text-sm font-medium text-muted-foreground dark:text-slate-400">
                {t("settings.language")}
              </label>
              <Select
                value={language}
                onValueChange={(v) => setLanguage(v as Language)}
              >
                <SelectTrigger className="h-9 flex-1 bg-white/50 dark:bg-slate-800/50">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {languages.map((item) => (
                    <SelectItem key={item.value} value={item.value}>
                      {item.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between gap-4 py-3">
              <label className="shrink-0 text-sm font-medium text-muted-foreground dark:text-slate-400">
                {t("settings.theme")}
              </label>
              <Select value={theme} onValueChange={(v) => setTheme(v as Theme)}>
                <SelectTrigger className="h-9 flex-1 bg-white/50 dark:bg-slate-800/50">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {themes.map((item) => (
                    <SelectItem key={item.value} value={item.value}>
                      {t(`settings.${item.labelKey}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
