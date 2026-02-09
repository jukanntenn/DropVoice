import { Send, Loader2 } from "lucide-react";

interface SendButtonProps {
  onClick: () => void;
  disabled: boolean;
  isSending: boolean;
}

export function SendButton({ onClick, disabled, isSending }: SendButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled || isSending}
      className="flex h-16 w-16 items-center justify-center rounded-full bg-gradient-to-br from-primary to-teal-500 text-white shadow-lg shadow-primary/30 transition-all duration-200 hover:shadow-xl hover:shadow-primary/40 active:scale-95 disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none disabled:saturate-0"
      aria-label="Send to PC"
    >
      {isSending ? (
        <Loader2 className="h-7 w-7 animate-spin" />
      ) : (
        <Send className="h-7 w-7" />
      )}
    </button>
  );
}
