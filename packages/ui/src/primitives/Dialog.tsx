/**
 * Dialog primitive built on @base-ui/react (spec 07 §2.2).
 *
 * Wraps Base UI's Dialog with cva-driven styling. Provides modal semantics
 * (focus trap, scroll lock, Escape-to-close, backdrop click) out of the box.
 * Public API stays compatible with the previous framer-motion version:
 * `{ open, onClose, title?, description?, children?, footer?, className? }`.
 */
import { Dialog as BaseDialog } from '@base-ui/react/dialog';
import { X } from 'lucide-react';
import { type ReactNode } from 'react';

import { cn } from '../lib/cn';

export interface DialogProps {
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  description?: ReactNode;
  children?: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export function Dialog({
  open,
  onClose,
  title,
  description,
  children,
  footer,
  className,
}: DialogProps) {
  return (
    <BaseDialog.Root
      open={open}
      onOpenChange={(next) => {
        if (!next) onClose();
      }}
    >
      <BaseDialog.Portal>
        <BaseDialog.Backdrop
          className={cn(
            'fixed inset-0 z-30 bg-slate-950/50 backdrop-blur-sm',
            'transition-opacity duration-150 data-[ending-style]:opacity-0 data-[starting-style]:opacity-0'
          )}
        />
        <BaseDialog.Popup
          className={cn(
            'shadow-glass dark:shadow-glass-dark fixed left-1/2 top-1/2 z-40 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-white/60 bg-white/80 p-6 text-neutral-900 backdrop-blur-xl dark:border-slate-700/50 dark:bg-slate-900/90 dark:text-slate-100',
            'data-[starting-style]:scale-95 data-[starting-style]:opacity-0',
            'data-[ending-style]:scale-95 data-[ending-style]:opacity-0',
            'transition-all duration-150 ease-out',
            className
          )}
        >
          {(title || description) && (
            <div className="mb-4">
              {title && (
                <BaseDialog.Title className="dark:text-neutral-0 text-lg font-semibold text-neutral-900">
                  {title}
                </BaseDialog.Title>
              )}
              {description && (
                <BaseDialog.Description className="mt-1 text-sm text-neutral-500 dark:text-neutral-400">
                  {description}
                </BaseDialog.Description>
              )}
            </div>
          )}
          {children && (
            <div className="text-sm text-neutral-700 dark:text-neutral-300">{children}</div>
          )}
          {footer && <div className="mt-6 flex justify-end gap-2">{footer}</div>}
          <BaseDialog.Close
            render={
              <button
                type="button"
                aria-label="Close"
                className="absolute right-4 top-4 text-neutral-400 transition-colors hover:text-neutral-700 dark:hover:text-neutral-200"
              />
            }
          >
            <X className="h-4 w-4" aria-hidden="true" />
          </BaseDialog.Close>
        </BaseDialog.Popup>
      </BaseDialog.Portal>
    </BaseDialog.Root>
  );
}
