import { Settings } from "lucide-react";

interface SettingsButtonProps {
  onClick: () => void;
}

export function SettingsButton({ onClick }: SettingsButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="pointer-events-auto flex h-7 w-7 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground dark:text-slate-500 dark:hover:text-slate-300"
      aria-label="Settings"
    >
      <Settings className="h-4 w-4" />
    </button>
  );
}

