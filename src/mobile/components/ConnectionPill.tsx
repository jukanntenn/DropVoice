import { useTranslation } from "react-i18next";
import type { Device } from "../types";

interface ConnectionPillProps {
  device: Device | null;
  onRetry: () => void;
}

/** Shows the active device's error label + Retry. Renders nothing unless the device is disconnected. */
export function ConnectionPill({ device, onRetry }: ConnectionPillProps) {
  const { t } = useTranslation();
  if (!device || device.status !== "disconnected") return null;

  const errorLabel = device.errorType
    ? t(
        `devices.error${device.errorType.charAt(0).toUpperCase()}${device.errorType.slice(1)}`,
      )
    : t("devices.errorUnknown");

  return (
    <div className="flex items-center gap-2 rounded-full bg-red-50 px-3 py-1.5 text-xs font-medium text-red-600 dark:bg-red-900/30 dark:text-red-300">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 20 20"
        fill="currentColor"
        className="h-3.5 w-3.5 flex-shrink-0"
      >
        <path
          fillRule="evenodd"
          d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
          clipRule="evenodd"
        />
      </svg>
      <span>{errorLabel}</span>
      <button
        type="button"
        onClick={onRetry}
        className="font-semibold text-red-700 hover:underline dark:text-red-200"
      >
        {t("devices.retry")}
      </button>
    </div>
  );
}
