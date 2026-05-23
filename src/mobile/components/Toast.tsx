import { useEffect, useMemo, useState } from "react";
import { CheckCircle, XCircle } from "lucide-react";

interface ToastProps {
  message: string;
  type: "success" | "error";
  onClose: () => void;
  duration?: number;
}

export function Toast({ message, type, onClose, duration = 3000 }: ToastProps) {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setIsVisible(false);
      window.setTimeout(onClose, 200);
    }, duration);
    return () => window.clearTimeout(timer);
  }, [duration, onClose]);

  const isSuccess = type === "success";

  return (
    <div
      className={[
        "fixed bottom-[calc(env(safe-area-inset-bottom)+1.5rem)] left-1/2 z-50 flex w-[calc(100%-2rem)] max-w-sm -translate-x-1/2 items-center gap-3 rounded-2xl px-3 py-2.5 shadow-lg backdrop-blur-md transition-all duration-200",
        isSuccess
          ? "border border-primary/20 bg-primary/90 text-white"
          : "border border-red-500/20 bg-red-500/90 text-white",
        isVisible ? "translate-y-0 opacity-100" : "translate-y-2 opacity-0",
      ].join(" ")}
    >
      {isSuccess ? (
        <CheckCircle className="h-5 w-5 shrink-0" />
      ) : (
        <XCircle className="h-5 w-5 shrink-0" />
      )}
      <span className="text-sm font-medium">{message}</span>
    </div>
  );
}

interface ToastData {
  id: number;
  message: string;
  type: "success" | "error";
  duration?: number;
}

let nextToastId = 0;

export function useToast() {
  const [toasts, setToasts] = useState<ToastData[]>([]);

  const showToast = (message: string, type: ToastData["type"] = "success", duration?: number) => {
    const id = (nextToastId += 1);
    setToasts((prev) => [...prev, { id, message, type, duration }]);
  };

  const removeToast = (id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  return useMemo(() => ({ toasts, showToast, removeToast }), [toasts]);
}
