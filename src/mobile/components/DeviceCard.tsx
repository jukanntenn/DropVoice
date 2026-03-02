import { useRef } from "react";
import type { DeviceStatus } from "../types";

interface DeviceCardProps {
  name: string;
  status: DeviceStatus;
  isActive: boolean;
  onSelect: () => void;
  onRemove: () => void;
  onRename: () => void;
}

const statusIcon: Record<DeviceStatus, string> = {
  connected: "●",
  disconnected: "○",
  connecting: "⟳",
};

export function DeviceCard({
  name,
  status,
  isActive,
  onSelect,
  onRemove,
  onRename,
}: DeviceCardProps) {
  const timerRef = useRef<number | null>(null);
  const longPressRef = useRef(false);

  const startLongPress = () => {
    longPressRef.current = false;
    timerRef.current = window.setTimeout(() => {
      longPressRef.current = true;
      onRename();
    }, 500);
  };

  const cancelLongPress = () => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const handleClick = () => {
    if (!longPressRef.current) onSelect();
  };

  const statusClass =
    status === "connected"
      ? "text-green-600 dark:text-green-400"
      : status === "connecting"
        ? "text-amber-600 dark:text-amber-400"
        : "text-slate-400 dark:text-slate-500";

  const activeClass = isActive
    ? "border-primary bg-primary/10"
    : "border-slate-200/70 bg-white/50 dark:border-slate-700/70 dark:bg-slate-900/40";

  return (
    <div
      role="button"
      tabIndex={0}
      className={[
        "relative flex items-center gap-2 rounded-2xl border px-3 py-2",
        "select-none transition-colors",
        activeClass,
      ].join(" ")}
      onClick={handleClick}
      onPointerDown={startLongPress}
      onPointerUp={cancelLongPress}
      onPointerCancel={cancelLongPress}
      onPointerLeave={cancelLongPress}
    >
      <span
        className={[
          "text-sm leading-none",
          statusClass,
          status === "connecting" ? "animate-spin" : "",
        ].join(" ")}
      >
        {statusIcon[status]}
      </span>
      <span className="max-w-[7.5rem] truncate text-sm font-medium text-slate-900 dark:text-slate-100">
        {name}
      </span>
      <button
        type="button"
        className="ml-1 text-slate-400 transition-colors hover:text-red-500 dark:text-slate-500 dark:hover:text-red-400"
        onPointerDown={(e) => {
          e.stopPropagation();
        }}
        onClick={(e) => {
          e.stopPropagation();
          cancelLongPress();
          onRemove();
        }}
        aria-label="Remove"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          className="h-4 w-4"
        >
          <path
            fillRule="evenodd"
            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
            clipRule="evenodd"
          />
        </svg>
      </button>
    </div>
  );
}
