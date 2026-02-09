import { useTranslation } from "react-i18next";
import type { ConnectionStatus } from "../hooks/useWebSocket";

interface StatusIndicatorProps {
  status: ConnectionStatus;
  reconnectAttempts: number;
}

export function StatusIndicator({ status, reconnectAttempts }: StatusIndicatorProps) {
  const { t } = useTranslation();

  // Don't show indicator when connected (normal state)
  if (status === "connected") {
    return null;
  }

  const config =
    status === "connecting"
      ? {
          dotClass: "bg-amber-500",
          text:
            reconnectAttempts > 0
              ? `${t("status.reconnecting")} (${reconnectAttempts}/5)`
              : t("status.connecting"),
          pulse: true,
        }
      : {
          dotClass: "bg-red-500",
          text:
            reconnectAttempts >= 5
              ? t("status.connectionLost")
              : t("status.disconnected"),
          pulse: false,
        };

  return (
    <div className="animate-slide-up flex items-center gap-2 rounded-2xl border border-amber-200/50 bg-amber-50/70 px-3 py-2.5 backdrop-blur-sm dark:border-amber-500/20 dark:bg-amber-950/40">
      <span
        className={[
          "h-2.5 w-2.5 rounded-full",
          config.dotClass,
          config.pulse ? "animate-pulse" : "",
        ].join(" ")}
      />
      <span className="text-sm font-medium text-amber-700 dark:text-amber-300">
        {config.text}
      </span>
    </div>
  );
}
