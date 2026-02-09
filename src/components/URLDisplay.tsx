import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Check, Copy } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";

export interface URLDisplayProps {
  url: string;
}

export default function URLDisplay({ url }: URLDisplayProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) window.clearTimeout(timerRef.current);
    };
  }, []);

  const copy = async () => {
    try {
      await writeText(url);
      setCopied(true);
      if (timerRef.current) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div className="flex w-full max-w-xs flex-col items-center gap-4">
      <p className="text-sm font-medium text-muted-foreground dark:text-slate-300">
        {t("connection.enterUrl")}
      </p>
      <div className="inline-flex items-center gap-2 rounded-2xl border border-slate-200/80 bg-white/80 py-0 pl-4 pr-2 shadow-lg backdrop-blur-md transition-all hover:border-slate-300 hover:shadow-xl dark:border-white/10 dark:bg-slate-900/70 dark:shadow-glass-dark dark:hover:border-white/20 dark:hover:bg-slate-900/80">
        <code className="truncate text-sm font-medium text-foreground dark:text-white">
          {url}
        </code>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 shrink-0 rounded-lg transition-all hover:bg-primary/10 hover:text-primary"
              onClick={copy}
              aria-label={copied ? t("copy.copied") : t("copy.button")}
            >
              {copied ? (
                <Check className="h-4 w-4 text-emerald-500" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </Button>
          </TooltipTrigger>
          <TooltipContent className="rounded-lg bg-slate-900 px-3 py-1.5 text-xs text-white dark:bg-white dark:text-slate-900">
            <p>{copied ? t("copy.copied") : t("copy.button")}</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
}
