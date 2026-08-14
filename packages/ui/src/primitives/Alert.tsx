/**
 * Built on native HTML + class-variance-authority; spec prefers @base-ui/react
 * but accepts fallback.
 */
import { cva, type VariantProps } from 'class-variance-authority';
import { AlertCircle, CheckCircle2, Info, XCircle } from 'lucide-react';
import { type ReactNode } from 'react';

import { cn } from '../lib/cn';

const alertVariants = cva('relative flex w-full gap-3 rounded-md border p-4 text-sm', {
  variants: {
    variant: {
      info: 'border-info-500/30 bg-info-500/10 text-info-600 dark:text-info-500',
      success: 'border-success-500/30 bg-success-500/10 text-success-600 dark:text-success-500',
      warning: 'border-warning-500/30 bg-warning-500/10 text-warning-600 dark:text-warning-500',
      danger: 'border-danger-500/30 bg-danger-500/10 text-danger-600 dark:text-danger-500',
    },
  },
  defaultVariants: {
    variant: 'info',
  },
});

const alertIcons = {
  info: Info,
  success: CheckCircle2,
  warning: AlertCircle,
  danger: XCircle,
} as const;

export interface AlertProps extends VariantProps<typeof alertVariants> {
  title?: ReactNode;
  children?: ReactNode;
  className?: string;
}

export function Alert({ variant = 'info', title, children, className }: AlertProps) {
  const Icon = alertIcons[variant ?? 'info'];
  return (
    <div role="alert" className={cn(alertVariants({ variant }), className)}>
      <Icon className="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
      <div className="flex flex-col gap-1">
        {title && <span className="font-medium">{title}</span>}
        {children && <div className="text-neutral-700 dark:text-neutral-300">{children}</div>}
      </div>
    </div>
  );
}

export type AlertVariants = VariantProps<typeof alertVariants>;
