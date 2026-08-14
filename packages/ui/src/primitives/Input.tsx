/**
 * Built on native HTML + class-variance-authority; spec prefers @base-ui/react
 * but accepts fallback.
 */
import { cva } from 'class-variance-authority';
import type { InputHTMLAttributes, Ref } from 'react';

import { cn } from '../lib/cn';

const inputVariants = cva(
  'flex h-10 w-full rounded-lg border border-white/60 bg-white/50 px-3 py-2 text-sm text-neutral-900 transition-all duration-200 placeholder:text-neutral-400 focus-visible:border-primary/50 focus-visible:bg-white/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/30 disabled:cursor-not-allowed disabled:opacity-50 dark:border-white/10 dark:bg-slate-800/50 dark:text-slate-100 dark:placeholder:text-slate-400 dark:focus-visible:bg-slate-800/70'
);

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  className?: string;
  ref?: Ref<HTMLInputElement>;
}

export function Input({ className, ref, type = 'text', ...props }: InputProps) {
  return <input ref={ref} type={type} className={cn(inputVariants(), className)} {...props} />;
}
