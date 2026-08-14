/**
 * Built on native HTML + class-variance-authority; spec prefers @base-ui/react
 * but accepts fallback.
 */
import { cva, type VariantProps } from 'class-variance-authority';
import type { HTMLAttributes, ReactNode } from 'react';

import { cn } from '../lib/cn';

const badgeVariants = cva(
  'inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium transition-colors',
  {
    variants: {
      variant: {
        default: 'bg-neutral-200 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300',
        primary: 'bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-200',
        success: 'bg-success-500/15 text-success-600 dark:text-success-500',
        warning: 'bg-warning-500/15 text-warning-600 dark:text-warning-500',
        danger: 'bg-danger-500/15 text-danger-600 dark:text-danger-500',
        info: 'bg-info-500/15 text-info-600 dark:text-info-500',
        outline:
          'border border-neutral-300 text-neutral-700 dark:border-neutral-700 dark:text-neutral-300',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
);

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: VariantProps<typeof badgeVariants>['variant'];
  children?: ReactNode;
}

export function Badge({ variant, className, children, ...props }: BadgeProps) {
  return (
    <span className={cn(badgeVariants({ variant }), className)} {...props}>
      {children}
    </span>
  );
}

export type BadgeVariants = VariantProps<typeof badgeVariants>;
