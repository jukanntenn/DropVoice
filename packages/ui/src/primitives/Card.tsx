/**
 * Built on native HTML + class-variance-authority; spec prefers @base-ui/react
 * but accepts fallback. Follows the shadcn/ui Card compound pattern.
 */
import type { HTMLAttributes, ReactNode } from 'react';

import { cn } from '../lib/cn';

export function Card({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'shadow-glass dark:shadow-glass-dark rounded-2xl border border-white/60 bg-white/80 text-neutral-900 backdrop-blur-md dark:border-white/10 dark:bg-slate-900/70 dark:text-slate-100',
        className
      )}
      {...props}
    />
  );
}

export function CardHeader({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('flex flex-col gap-1.5 p-6', className)} {...props} />;
}

export function CardTitle({ className, ...props }: HTMLAttributes<HTMLHeadingElement>) {
  return <h3 className={cn('text-lg font-semibold leading-none', className)} {...props} />;
}

export function CardDescription({ className, ...props }: HTMLAttributes<HTMLParagraphElement>) {
  return (
    <p className={cn('text-sm text-neutral-500 dark:text-neutral-400', className)} {...props} />
  );
}

export function CardContent({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('p-6 pt-0', className)} {...props} />;
}

export function CardFooter({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('flex items-center p-6 pt-0', className)} {...props} />;
}

export type CardProps = HTMLAttributes<HTMLDivElement>;
export interface CardCompoundProps {
  children?: ReactNode;
  className?: string;
}
