/**
 * Toast — sonner-based toaster + re-exported `toast` helper (spec 03 section 2.5).
 *
 * Apps render `<Toaster />` once near the root and call `toast.error(...)`
 * (or use the `useErrorHandler` hook) to fire notifications.
 */
import { Toaster as SonnerToaster, toast } from 'sonner';

export { toast };

export interface ToasterProps {
  position?:
    'top-left' | 'top-center' | 'top-right' | 'bottom-left' | 'bottom-center' | 'bottom-right';
  richColors?: boolean;
  closeButton?: boolean;
  /** Follow the system color scheme ("light" | "dark" | "system"). */
  theme?: 'light' | 'dark' | 'system';
  className?: string;
}

export function Toaster({
  position = 'bottom-right',
  richColors = true,
  closeButton = true,
  theme = 'system',
  className,
}: ToasterProps) {
  return (
    <SonnerToaster
      position={position}
      richColors={richColors}
      closeButton={closeButton}
      theme={theme}
      toastOptions={{
        classNames: {
          toast: className,
        },
      }}
    />
  );
}
