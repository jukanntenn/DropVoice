import { Undo2 } from "lucide-react";

interface RestoreButtonProps {
  onClick: () => void;
  disabled: boolean;
}

export function RestoreButton({ onClick, disabled }: RestoreButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="flex h-12 w-12 items-center justify-center rounded-full border border-border/50 bg-white/50 text-muted-foreground shadow-sm backdrop-blur-sm transition-all duration-200 hover:border-border hover:bg-white/70 hover:text-foreground active:scale-95 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-white/50 disabled:hover:text-muted-foreground dark:border-white/10 dark:bg-slate-800/50 dark:text-slate-400 dark:hover:bg-slate-800/70 dark:hover:text-white dark:disabled:hover:bg-slate-800/50 dark:disabled:hover:text-slate-400"
      aria-label="Restore last message"
    >
      <Undo2 className="h-5 w-5" />
    </button>
  );
}
