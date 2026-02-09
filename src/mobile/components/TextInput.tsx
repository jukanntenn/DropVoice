import type React from "react";
import { useTranslation } from "react-i18next";
import { SettingsButton } from "./SettingsButton";

interface TextInputProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  disabled: boolean;
  onOpenSettings: () => void;
}

export function TextInput({
  value,
  onChange,
  onKeyDown,
  disabled,
  onOpenSettings,
}: TextInputProps) {
  const { t } = useTranslation();

  const charCount = value.length;
  const isOverLimit = charCount > 10000;
  const isWarning = charCount > 8000 && !isOverLimit;

  return (
    <div className="group relative">
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        placeholder={t("input.placeholder")}
        disabled={disabled}
        rows={6}
        maxLength={10000}
        className="min-h-[200px] w-full resize-y rounded-2xl border border-border/50 bg-white/50 px-4 pt-3 pb-8 text-[15px] leading-relaxed text-foreground placeholder:text-muted-foreground transition-all duration-200 focus:border-primary/50 focus:bg-white/70 focus:outline-none focus:ring-4 focus:ring-primary/10 disabled:cursor-not-allowed disabled:opacity-50 dark:border-white/10 dark:bg-slate-800/50 dark:text-white dark:placeholder:text-slate-400 dark:focus:bg-slate-800/70 dark:focus:ring-primary/20"
      />
      <div className="pointer-events-none absolute right-4 bottom-4 flex items-center gap-2">
        <span
          className={[
            "text-[11px] font-medium transition-colors duration-200",
            isOverLimit
              ? "text-red-400"
              : isWarning
                ? "text-amber-400"
                : "text-muted-foreground dark:text-slate-500 group-focus-within:text-primary dark:group-focus-within:text-primary",
          ].join(" ")}
        >
          {charCount.toLocaleString()}
        </span>
        <SettingsButton onClick={onOpenSettings} />
      </div>
    </div>
  );
}
